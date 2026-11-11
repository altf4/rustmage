#!/usr/bin/env python3
"""Format Cargo workspace targets in batches that fit Windows command lines."""
from collections import defaultdict
import json
from pathlib import Path
import subprocess
import sys

from build_platform import argument_batches

ROOT = Path(__file__).resolve().parents[1]

if __name__ == '__main__':
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--no-deps', '--format-version', '1'],
        cwd=ROOT,
    ))
    members = set(metadata['workspace_members'])
    by_edition = defaultdict(set)
    for package in metadata['packages']:
        if package['id'] not in members:
            continue
        for target in package['targets']:
            by_edition[target['edition']].add(Path(target['src_path']))
    for edition, paths in sorted(by_edition.items()):
        for batch in argument_batches(sorted(paths)):
            result = subprocess.run(
                ['rustfmt', '--edition', edition, *sys.argv[1:], *batch],
                cwd=ROOT,
            )
            if result.returncode:
                sys.exit(result.returncode)
