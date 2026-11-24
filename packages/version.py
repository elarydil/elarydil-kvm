#!/usr/bin/env python3
"""Single source of the release version: apps/desktop/src-tauri/tauri.conf.json.

Change it with bump-version.py. Run directly to print it
(`python3 packages/version.py` -> 1.0.0), or import `VERSION` and `TAG` (v1.0.0).
"""
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
VERSION = json.loads((ROOT / 'apps/desktop/src-tauri/tauri.conf.json').read_text(encoding='utf-8'))['version']
TAG = f'v{VERSION}'

if __name__ == '__main__':
    print(VERSION)
