"""Locate external source checkouts without depending on a developer's home path."""
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def xmage_source(repo=ROOT):
    """Use the explicit override, a sibling checkout, or the primary repo's sibling."""
    configured = os.environ.get('RUSTMAGE_XMAGE_SOURCE')
    if configured:
        return Path(configured).expanduser().resolve()
    repo = Path(repo).resolve()
    sibling = repo.parent / 'mage'
    if sibling.is_dir():
        return sibling
    result = subprocess.run(
        ['git', '-C', str(repo), 'rev-parse', '--path-format=absolute', '--git-common-dir'],
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, check=False,
    )
    if result.returncode == 0:
        # A linked worktree shares the primary checkout's .git directory.
        return Path(result.stdout.strip()).resolve().parent.parent / 'mage'
    return sibling
