#!/usr/bin/env python3
"""Build native helpers and stage application-owned binaries for Tauri packaging."""
import argparse
import os
import pathlib
import platform
import shutil
import subprocess
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def run(*args):
    subprocess.run(args, cwd=ROOT, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', default=os.environ.get('ELARYDIL_KVM_TARGET'))
    parser.add_argument('--skip-build', action='store_true')
    parser.add_argument('--output', type=pathlib.Path, default=ROOT / 'dist')
    args = parser.parse_args()
    target = args.target
    host = platform.system()
    windows = 'windows' in target if target else host == 'Windows'
    macos = 'apple-darwin' in target if target else host == 'Darwin'
    if macos and host != 'Darwin':
        raise SystemExit('A macOS host and Apple SDK are required to build the bundled DDC helper.')
    if macos and ((target and not target.startswith('aarch64')) or
                  (not target and platform.machine().lower() not in ('arm64', 'aarch64'))):
        raise SystemExit('Intel macOS is unsupported. Cross-build with --target aarch64-apple-darwin on a Mac.')
    if macos:
        os.environ.setdefault('MACOSX_DEPLOYMENT_TARGET', '13.0')
    command = ['cargo', 'build', '--release', '--locked']
    if target:
        command += ['--target', target]
    if not args.skip_build:
        run(*command)
    directory = ROOT / 'target' / target / 'release' if target else ROOT / 'target/release'
    stage = ROOT / 'apps/desktop/src-tauri/bin'
    stage.mkdir(exist_ok=True)
    # This is an ignored application-owned build staging directory.
    for entry in stage.iterdir():
        if entry.is_file() and entry.name != '.gitkeep':
            entry.unlink()
    extension = '.exe' if windows else ''
    for name in ['elarydil-kvm', 'elarydil-kvm-agent', 'elarydil-kvm-ddc', 'elarydil-kvm-command']:
        shutil.copy2(directory / (name + extension), stage / (name + extension))
    if macos:
        run('make', '-C', 'vendor/m1ddc',
            'CFLAGS=-Wall -Werror -Wextra -fmodules -arch arm64 -DMAX_DISPLAYS=32')
        shutil.copy2(ROOT / 'vendor/m1ddc/m1ddc', stage / 'm1ddc')
        identity = os.environ.get('APPLE_SIGNING_IDENTITY') or '-'
        if identity:
            for binary in stage.iterdir():
                if binary.name != '.gitkeep':
                    options = ['--timestamp=none'] if identity == '-' else ['--options', 'runtime', '--timestamp']
                    run('codesign', '--force', *options, '--sign', identity, str(binary))
    args.output.mkdir(parents=True, exist_ok=True)
    label = target or f'{host.lower()}-{platform.machine()}'
    archive = args.output / f'elarydil-kvm-cli-{label}.zip'
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, strict_timestamps=False) as bundle:
        for file in stage.iterdir():
            if file.name != '.gitkeep':
                bundle.write(file, 'elarydil-kvm/' + file.name)
        for name in ['LICENSE', 'README.md', 'IMPLEMENTATION_STATUS.md']:
            bundle.write(ROOT / name, 'elarydil-kvm/' + name)
        for file in (ROOT / 'docs').rglob('*'):
            if file.is_file():
                bundle.write(file, 'elarydil-kvm/' + str(file.relative_to(ROOT)))
    print(archive)


if __name__ == '__main__':
    main()
