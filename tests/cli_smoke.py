"""Exercise compiled CLI + independent native agent, including Unix IPC auth."""
import unittest, subprocess, tempfile, pathlib, json, os, socket, time, concurrent.futures, shutil
ROOT=pathlib.Path(__file__).resolve().parents[1]
BINARY=ROOT/'target/debug'/('elarydil-kvm.exe' if os.name=='nt' else 'elarydil-kvm')
class Headless(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory(prefix='elarydil-kvm-smoke-');self.home=pathlib.Path(self.temp.name);self.call('agent','start')
 def tearDown(self):
  self.call('agent','stop',check=False);self.temp.cleanup()
 def call(self,*args,check=True):
  p=subprocess.run([str(BINARY),'--home',str(self.home),'--json',*args],text=True,encoding="utf-8",capture_output=True,timeout=35)
  value=json.loads(p.stdout)
  if check:self.assertEqual(p.returncode,0,(p.stdout,p.stderr))
  return value
 def test_configuration_cli_and_restart(self):
  profile=self.call('profiles','create','--name','Work desk')['result'];self.assertTrue(profile['id'].startswith('profile_'))
  self.call('config','set','app.theme','"dark"')
  self.assertEqual(self.call('config','get','app.theme')['result'],'dark')
  self.call('agent','restart');self.assertEqual(self.call('config','get','app.theme')['result'],'dark')
  self.call('agent','stop');self.assertEqual(self.call('config','show')['result']['app']['theme'],'dark')
 def test_permissions_version_and_auth(self):
  if os.name=='nt':self.skipTest('Unix modes only')
  self.assertEqual(self.home.stat().st_mode&0o777,0o700)
  self.assertEqual((self.home/'agent.sock').stat().st_mode&0o777,0o600)
  self.assertEqual((self.home/'token').stat().st_mode&0o777,0o600)
  for version,token,code in [(2,(self.home/'token').read_text(),'protocol_version'),(1,'wrong','authentication')]:
   with socket.socket(socket.AF_UNIX)as s:
    s.settimeout(3);s.connect(str(self.home/'agent.sock'));s.sendall(json.dumps(dict(version=version,id='test',token=token,command='status',payload={})).encode()+b'\n');response=json.loads(s.makefile().readline());self.assertEqual(response['error']['code'],code)
 def test_concurrent_clients_and_validation(self):
  for batch in range(3):
   with concurrent.futures.ThreadPoolExecutor(max_workers=8)as pool:list(pool.map(lambda i:self.call('profiles','create','--name',f'Profile {i}'),range(batch*8,(batch+1)*8)))
   self.assertEqual(len(self.call('profiles','list')['result']),(batch+1)*8)
  error=self.call('config','set','safety.globalMonitorCooldownMs','120001',check=False);self.assertEqual(error['error']['code'],'invalid_config')
  self.assertEqual(self.call('config','get','safety.globalMonitorCooldownMs')['result'],100)
 def test_headless_capabilities_and_simulation_rejection(self):
  status=self.call('status')['result'];self.assertTrue(status['guiIndependent']);self.assertIn(status['agent'],['running','degraded'])
  self.assertNotIn('hardwareTests',self.call('doctor')['result'])
  self.assertEqual(self.call('rules','simulate','unknown',check=False)['error']['code'],'not_found')
  self.assertEqual(self.call('config','reset',check=False)['error']['code'],'confirmation_required')
 def test_custom_commands_headless_configuration_and_exit_codes(self):
  c=self.call('config','show')['result']
  c['usbDevices']=[dict(id='device_test',alias='Test keyboard',enabled=True,selectors=[dict(id='selector_test',strategy='vid-pid',vid=1,pid=1,serial=None,path=None,allowAmbiguous=False)])]
  c['monitors']=[dict(id='monitor_test',alias='Test display',fingerprint='test',connector=None,inputOptions=[dict(value=18,vcpCode=96,label='HDMI',verified=False)])]
  c['profiles']=[dict(id='profile_test',name='Test profile',enabled=True,rules=[dict(id='rule_test',name='Test rule',enabled=True,trigger='disconnect',deviceIds=['device_test'],mode='all',minimumCount=1,coincidenceWindowMs=2000,debounceMs=400,rearmMs=500,cooldownMs=5000,dryRun=True,actions=[dict(id='action_test',monitorId='monitor_test',vcpCode=96,inputValue=18)])])]
  path=self.home/'import.json';path.write_text(json.dumps(c),encoding='utf-8')
  self.call('config','import',str(path))
  command="[IO.File]::WriteAllText('command.txt', $env:ELARYDIL_KVM_EVENT); exit 7" if os.name=='nt' else 'printf "%s" "$ELARYDIL_KVM_EVENT" > command.txt; exit 7'
  self.call('rules','commands','set','rule_test','--on-connect',command,'--on-disconnect','echo away','--working-dir',str(self.home),'--timeout-ms','10000')
  self.call('rules','commands','enable','rule_test')
  saved=self.call('rules','commands','get','rule_test')['result'];self.assertTrue(saved['enabled']);self.assertEqual(saved['onConnect'],command)
  self.assertEqual(self.call('rules','commands','run','rule_test','--event','connect',check=False)['error']['code'],'confirmation_required')
  result=subprocess.run([str(BINARY),'--home',str(self.home),'--json','rules','commands','run','rule_test','--event','connect','--confirm','--wait'],text=True,encoding='utf-8',capture_output=True,timeout=20)
  self.assertEqual(result.returncode,7,(result.stdout,result.stderr))
  self.assertEqual(json.loads(result.stdout)['result']['eventType'],'command.failed')
  self.assertEqual((self.home/'command.txt').read_text(),'connect')
  report=self.call('diagnostics','export')['result'];self.assertEqual(report['config']['profiles'][0]['rules'][0]['commands']['onConnect'],'<redacted>')
  self.call('agent','stop')
  self.assertEqual(self.call('rules','commands','get','rule_test')['result']['onConnect'],command)
  self.assertEqual(self.call('rules','commands','run','rule_test','--event','connect','--confirm',check=False)['error']['code'],'agent_not_running')
 def test_startup_failure_preserves_stderr(self):
  if os.name=='nt':self.skipTest('Unix socket path limit only')
  deep=self.home/('x'*110);deep.mkdir()
  result=subprocess.run([str(BINARY),'--home',str(deep),'--json','agent','start'],text=True,encoding="utf-8",capture_output=True,timeout=10)
  value=json.loads(result.stdout)
  self.assertNotEqual(result.returncode,0)
  self.assertEqual(value['error']['code'],'agent_not_running')
  self.assertTrue(value['error'].get('detail'))
  self.assertTrue((deep/'agent.stderr.log').read_text().strip())
 def test_gui_recovery_from_installed_helper_directory(self):
  if os.name=='nt':self.skipTest('Unix executable fixture; Windows layout is exercised by native packaging')
  app=self.home/'installed-app';helpers=app/'bin';helpers.mkdir(parents=True)
  cli=helpers/'elarydil-kvm';shutil.copy2(BINARY,cli)
  gui=app/'elarydil-kvm-gui';gui.write_text('#!/bin/sh\nprintf ready > "$ELARYDIL_KVM_HOME/gui-marker"\nwhile [ ! -f "$ELARYDIL_KVM_HOME/gui-stop" ]; do sleep .02; done\nprintf stopped > "$ELARYDIL_KVM_HOME/gui-exited"\n');gui.chmod(0o755)
  try:result=subprocess.run([str(cli),'--home',str(self.home),'--json','gui','open'],text=True,encoding="utf-8",capture_output=True,timeout=5)
  finally:
   (self.home/'gui-stop').touch()
   for _ in range(50):
    if (self.home/'gui-exited').exists():break
    time.sleep(.02)
  self.assertEqual(result.returncode,0,(result.stdout,result.stderr))
  for _ in range(50):
   if (self.home/'gui-marker').exists():break
   time.sleep(.02)
  self.assertEqual((self.home/'gui-marker').read_text(),'ready')
  self.assertEqual((self.home/'gui-exited').read_text(),'stopped')
  self.assertIn(self.call('status')['result']['agent'],['running','degraded'])
class Help(unittest.TestCase):
 def test_every_public_command_has_specific_help_and_examples(self):
  import re
  source=(ROOT/'crates/elarydil-kvm/src/cli.rs').read_text(encoding='utf-8').split('const VALUES:')[0]
  for op in re.findall(r'    "([a-z.-]+)",',source):
   p=subprocess.run([str(BINARY),*op.split('.'),'--help'],text=True,encoding='utf-8',capture_output=True,timeout=5)
   self.assertEqual(p.returncode,0,(op,p.stderr));self.assertIn('Examples:',p.stdout,op);self.assertNotIn('See docs/CLI.md.',p.stdout)
  status=subprocess.run([str(BINARY),'status','--help'],text=True,encoding='utf-8',capture_output=True).stdout
  self.assertNotIn('--monitor',status)
if __name__=='__main__':unittest.main(verbosity=2)
