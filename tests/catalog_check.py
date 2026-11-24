"""Own application strings must resolve through the shared catalog."""
import json,pathlib,re,unittest
ROOT=pathlib.Path(__file__).resolve().parents[1];CAT=json.loads((ROOT/'locales/en-US.json').read_text(encoding='utf-8'))
class Catalog(unittest.TestCase):
 def test_static_frontend_keys(self):
  for path in (ROOT/'apps/desktop/src').glob('*.tsx'):
   for key in re.findall(r"\bt\(['\"]([^'\"]+)['\"]",path.read_text(encoding='utf-8')):self.assertIn(key,CAT,(path.name,key))
 def test_error_keys(self):
  for path in (ROOT/'crates/elarydil-kvm/src').rglob('*.rs'):
   for key in re.findall(r'Error::new\("([a-z_]+)"\)',path.read_text(encoding='utf-8')):self.assertIn('error.'+key,CAT,(path.name,key))
 def test_placeholders(self):
  for key,value in CAT.items():
   self.assertIsInstance(value,str);self.assertTrue(value.strip(),key)
   self.assertEqual(value.count('{'),value.count('}'),key)
 def test_navigation_and_sources(self):
  for page in ['dashboard','devices','monitors','profiles','events','settings']:
   for field in ['nav','title','subtitle','eyebrow']:self.assertIn(f'ui.{field}.{page}',CAT)
  for value in [15,16,17,18]:self.assertIn(f'input.{value}',CAT)
if __name__=='__main__':unittest.main(verbosity=2)
