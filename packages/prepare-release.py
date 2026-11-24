#!/usr/bin/env python3
"""Collect the native build artifacts into stable, versioned release assets."""
import argparse, hashlib, pathlib, shutil, sys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from version import VERSION
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--artifacts',type=pathlib.Path,required=True)
p.add_argument('--output',type=pathlib.Path,required=True)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
def copy(pattern,name):
 files=list(a.artifacts.rglob(pattern))
 if len(files)!=1:raise SystemExit(f'Expected one {pattern}; got {files}')
 shutil.copy2(files[0],a.output/name)
copy('*.dmg',f'Elarydil-KVM-{VERSION}-macos-arm64.dmg')
copy('*-setup.exe',f'Elarydil-KVM-{VERSION}-windows-x64-setup.exe')
copy('*_amd64.deb',f'Elarydil-KVM-{VERSION}-linux-amd64.deb')
copy('*.x86_64.rpm',f'Elarydil-KVM-{VERSION}-linux-x86_64.rpm')
copy('*cli-aarch64-apple-darwin.zip','elarydil-kvm-cli-macos-arm64.zip')
copy('*cli-windows-*.zip','elarydil-kvm-cli-windows-x64.zip')
copy('*cli-linux-*.zip','elarydil-kvm-cli-linux-x64.zip')
shutil.copy2(pathlib.Path(__file__).resolve().parents[1]/'docs/MACOS_FIRST_LAUNCH.md',a.output/'MACOS-FIRST-LAUNCH.md')
manifest=[]
for file in sorted(a.output.iterdir()):
 if file.is_file() and file.name!='SHA256SUMS.txt':manifest.append(f'{hashlib.sha256(file.read_bytes()).hexdigest()}  {file.name}')
(a.output/'SHA256SUMS.txt').write_text('\n'.join(manifest)+'\n')
print('\n'.join(manifest))
