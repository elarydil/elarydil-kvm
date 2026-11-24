#!/usr/bin/env python3
"""Verify packaged Windows payloads are x64 and need no extra C/C++ runtime."""
import argparse
import pathlib
import struct

ROOT = pathlib.Path(__file__).resolve().parents[1]


def imports(path):
    data = path.read_bytes()
    assert data[:2] == b'MZ', f'{path.name}: not a PE executable'
    pe, = struct.unpack_from('<I', data, 0x3c)
    assert data[pe:pe+4] == b'PE\0\0', f'{path.name}: invalid PE signature'
    machine, sections = struct.unpack_from('<HH', data, pe+4)
    optional_size, = struct.unpack_from('<H', data, pe+20)
    optional = pe+24
    magic, = struct.unpack_from('<H', data, optional)
    assert machine == 0x8664 and magic == 0x20b, f'{path.name}: expected x64 PE32+'
    table = optional+optional_size

    def offset(rva):
        for i in range(sections):
            size, start, raw_size, raw = struct.unpack_from('<IIII', data, table+i*40+8)
            if start <= rva < start+max(size, raw_size):
                return raw+rva-start
        raise ValueError(f'{path.name}: invalid section RVA {rva:x}')

    rva, = struct.unpack_from('<I', data, optional+112+8)
    if not rva:
        return []
    pos = offset(rva)
    result = []
    while any(data[pos:pos+20]):
        name_rva, = struct.unpack_from('<I', data, pos+12)
        name = offset(name_rva)
        result.append(data[name:data.index(0, name)].decode('ascii'))
        pos += 20
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=pathlib.Path, default=ROOT/'apps/desktop/src-tauri/bin')
    parser.add_argument('--gui', type=pathlib.Path)
    args = parser.parse_args()
    paths = [args.directory/(name+'.exe') for name in
             ['elarydil-kvm', 'elarydil-kvm-agent', 'elarydil-kvm-ddc', 'elarydil-kvm-command']]
    if args.gui:
        paths.append(args.gui)
    for path in paths:
        dependencies = imports(path)
        extra = [name for name in dependencies if name.lower().startswith(
            ('vcruntime', 'msvcp', 'libgcc', 'libstdc++'))]
        assert not extra, f'{path.name}: unbundled runtime dependencies: {extra}'
        print(f'{path.name}: x64, no external C/C++ runtime dependency')


if __name__ == '__main__':
    main()
