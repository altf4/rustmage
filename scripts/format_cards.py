#!/usr/bin/env python3
"""Format authored cards in batches that fit native Windows command lines."""
from pathlib import Path
import subprocess
import sys

from build_platform import argument_batches

ROOT = Path(__file__).resolve().parents[1]

if __name__ == '__main__':
    for batch in argument_batches(sorted((ROOT/'src/card_definitions').rglob('*.rs'))):
        result = subprocess.run(['rustfmt', '--edition', '2024', *sys.argv[1:], *batch])
        if result.returncode:
            sys.exit(result.returncode)
