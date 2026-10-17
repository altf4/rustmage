"""Source discovery works after relocation and never records personal home roots."""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest
from unittest.mock import patch
if __package__:
    from .source_paths import ROOT, xmage_source
else:
    from source_paths import ROOT, xmage_source


class SourcePathsTests(unittest.TestCase):
    def test_explicit_override_and_sibling_checkout(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            repo = base / 'a renamed checkout'
            repo.mkdir()
            sibling = base / 'mage'
            sibling.mkdir()
            override = base / 'elsewhere' / 'oracle engine'
            with patch.dict(os.environ, {'RUSTMAGE_XMAGE_SOURCE': str(override)}):
                self.assertEqual(xmage_source(repo), override.resolve())
            with patch.dict(os.environ, {'RUSTMAGE_XMAGE_SOURCE': ''}):
                self.assertEqual(xmage_source(repo), sibling.resolve())

    def test_linked_worktree_uses_primary_checkouts_sibling(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            primary = base / 'projects' / 'renamed repo'
            primary.mkdir(parents=True)
            sibling = primary.parent / 'mage'
            sibling.mkdir()
            subprocess.run(['git', 'init', '-q', str(primary)], check=True)
            subprocess.run(['git', '-C', str(primary), '-c', 'user.name=Test',
                            '-c', 'user.email=test@example.invalid', 'commit', '-qm',
                            'Initial', '--allow-empty'], check=True)
            worktree = base / 'worktrees' / 'isolated'
            subprocess.run(['git', '-C', str(primary), 'worktree', 'add', '-q',
                            '--detach', str(worktree)], check=True)
            with patch.dict(os.environ, {'RUSTMAGE_XMAGE_SOURCE': ''}):
                self.assertEqual(xmage_source(worktree), sibling.resolve())

    def test_tracked_files_do_not_embed_personal_home_paths(self):
        pattern = re.compile(r'/(?:home|Users)/[A-Za-z0-9_.-]+/')
        paths = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '-z']).split(b'\0')
        offenders = []
        for raw in paths:
            if not raw:
                continue
            path = ROOT / os.fsdecode(raw)
            if path.is_file() and pattern.search(path.read_text(errors='ignore')):
                offenders.append(os.fsdecode(raw))
        self.assertEqual(offenders, [], 'Use portable locations instead of personal home roots')


if __name__ == '__main__':
    unittest.main()
