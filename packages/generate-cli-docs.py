#!/usr/bin/env python3
"""Regenerate command reference from the actual compiled CLI help screens."""
import argparse, os, pathlib, re, subprocess
ROOT=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary',type=pathlib.Path,default=ROOT/'target/debug'/('elarydil-kvm.exe' if os.name=='nt' else 'elarydil-kvm'))
a=p.parse_args()
source=(ROOT/'crates/elarydil-kvm/src/cli.rs').read_text(encoding='utf-8').split('const VALUES:')[0]
ops=re.findall(r'    "([a-z.-]+)",',source)
parts=['# Elarydil KVM — complete CLI reference\n\nGenerated from the compiled CLI by `python packages/generate-cli-docs.py`.\nSee [setup guide](CLI.md), [headless agent](HEADLESS.md), and [custom commands](COMMAND_HOOKS.md).\n']
for op in ['',*ops]:
 command=op.split('.') if op else []
 help=subprocess.check_output([str(a.binary.resolve()),*command,'--help'],encoding='utf-8',env={**os.environ,'NO_COLOR':'1'})
 parts.append('## '+(' '.join(command) or 'elarydil-kvm')+'\n\n```text\n'+help.rstrip()+'\n```\n')
(ROOT/'docs/CLI_REFERENCE.md').write_text('\n'.join(parts),encoding='utf-8')
print(f'Documented {len(ops)} commands.')
