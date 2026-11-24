#!/usr/bin/env python3
"""Change the release version everywhere it is written, or check that it matches.

    python3 packages/bump-version.py 0.1.3     # bump and create release notes
    python3 packages/bump-version.py --check   # verify every file has one version

The version in apps/desktop/src-tauri/tauri.conf.json is the source of truth
(see version.py). A push to main with a version that has no GitHub release yet
builds the installers and publishes the `v<version>` release.
"""
import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'packages'))
from version import VERSION  # noqa: E402

# (file, pattern with {v} for the escaped version, expected number of matches)
TARGETS = [
    ('apps/desktop/src-tauri/tauri.conf.json', r'("version": "){v}(")', 1),
    ('apps/desktop/package.json', r'^(  "version": "){v}(")', 1),
    ('apps/desktop/package-lock.json', r'("name": "elarydil-kvm",\s*"version": "){v}(")', 2),
    ('crates/elarydil-kvm/Cargo.toml', r'^(version = "){v}(")', 1),
    ('apps/desktop/src-tauri/Cargo.toml', r'^(version = "){v}(")', 1),
    ('Cargo.lock', r'(name = "elarydil-kvm"\nversion = "){v}(")', 1),
    ('apps/desktop/src-tauri/Cargo.lock', r'(name = "elarydil-kvm(?:-gui)?"\nversion = "){v}(")', 2),
    ('locales/en-US.json', r'("ui.version": "){v}(")', 1),
    ('docs/DEPENDENCIES.json', r'("name": "elarydil-kvm(?:-gui)?",\s*"version": "){v}(")', 2),
    ('README.md', r'(\*\*Version ){v}(\*\*)', 1),
    ('README.md', r'(releases/){v}(\.md)', 1),
    ('README.md', r'(Elarydil-KVM-){v}(-)', 4),
    ('docs/README.md', r'(Release notes for ){v}(\])', 1),
    ('docs/README.md', r'(releases/){v}(\.md)', 1),
]
NOTES_MARKER = 'TODO'


def regex(pattern, version):
    return re.compile(pattern.replace('{v}', re.escape(version)), re.M)


def check(version):
    problems = []
    for path, pattern, count in TARGETS:
        found = len(regex(pattern, version).findall((ROOT / path).read_text(encoding='utf-8')))
        if found != count:
            problems.append(f'{path}: expected {count} x {version}, found {found}')
    return problems


def parse(version):
    if not re.fullmatch(r'\d+\.\d+\.\d+', version):
        raise SystemExit(f'Version must look like 1.2.3, got {version!r}')
    return tuple(int(part) for part in version.split('.'))


def bump(new):
    if parse(new) <= parse(VERSION):
        raise SystemExit(f'{new} is not newer than the current {VERSION}')
    problems = check(VERSION)
    if problems:
        raise SystemExit('Version is already inconsistent; fix this first:\n' + '\n'.join(problems))
    for path, pattern, count in TARGETS:
        file = ROOT / path
        text, replaced = regex(pattern, VERSION).subn(lambda m: m.group(1) + new + m.group(2), file.read_text(encoding='utf-8'))
        assert replaced == count, (path, replaced)
        file.write_text(text, encoding='utf-8')

    notes = ROOT / f'docs/releases/{new}.md'
    if not notes.exists():
        notes.write_text(
            f'Elarydil KVM {new} {NOTES_MARKER}: one sentence about this release.\n\n'
            f'- {NOTES_MARKER}: what changed.\n\n'
            '**Downloads**: Windows 11 x64 installer, Apple Silicon macOS DMG, Linux .deb/.rpm, or CLI-only bundles. '
            'Verify with SHA256SUMS.txt.\n\n'
            '**macOS**: copy the app from the DMG to Applications and follow MACOS-FIRST-LAUNCH.md '
            'for the first launch.\n',
            encoding='utf-8')
    print(f'{VERSION} -> {new}. Write the release notes in {notes.relative_to(ROOT)} '
          f'(remove every "{NOTES_MARKER}"), then commit and merge into main.')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('version', nargs='?', help='new version, for example 0.1.3')
    group.add_argument('--check', action='store_true', help='verify the current version is consistent')
    args = parser.parse_args()
    if args.check:
        problems = check(VERSION)
        notes = ROOT / f'docs/releases/{VERSION}.md'
        if not notes.exists():
            problems.append(f'{notes.relative_to(ROOT)} is missing')
        elif NOTES_MARKER in notes.read_text(encoding='utf-8'):
            problems.append(f'{notes.relative_to(ROOT)} still contains {NOTES_MARKER}')
        if problems:
            raise SystemExit('\n'.join(problems))
        print(f'{VERSION}: consistent')
    else:
        bump(args.version)


if __name__ == '__main__':
    main()
