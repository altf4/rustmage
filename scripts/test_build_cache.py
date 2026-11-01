"""Observable cache invalidation contracts; no compiler or allocation probes."""
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import build_cache


class CacheInputTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)/'checkout'
        self.root.mkdir()
        self.target = Path(self.temporary.name)/'cache'
        self.source = self.root/'engine.rs'
        self.source.write_text('first engine\n')
        os.utime(self.source, ns=(10_000, 10_000))
        self.paths = [('engine.rs', self.source)]
        self.context = {'compiler': 'fixed'}
        build_cache.save(self.root, self.target, self.paths, self.context)

    def test_checkout_timestamp_is_restored_only_for_identical_bytes(self):
        os.utime(self.source, ns=(20_000, 20_000))
        result = build_cache.restore(self.root, self.target, self.paths, self.context)
        self.assertEqual(result, {'restored': 1, 'changed': 0})
        self.assertEqual(self.source.stat().st_mtime_ns, 10_000)
        self.source.write_text('changed engine\n')
        os.utime(self.source, ns=(10_000, 10_000))
        result = build_cache.restore(self.root, self.target, self.paths, self.context)
        self.assertEqual(result['restored'], 0)
        self.assertEqual(self.source.read_text(), 'changed engine\n')
        self.assertGreater(self.source.stat().st_mtime_ns, 20_000)

    def test_changed_toolchain_does_not_restore_timestamps(self):
        os.utime(self.source, ns=(20_000, 20_000))
        result = build_cache.restore(self.root, self.target, self.paths, {'compiler': 'new'})
        self.assertEqual(result['restored'], 0)
        self.assertEqual(self.source.stat().st_mtime_ns, 20_000)

    def test_new_file_is_kept_dirty_and_outputs_are_untouched(self):
        new = self.root/'new.rs'
        new.write_text('new input\n')
        output = self.target/'engine.rlib'
        output.write_bytes(b'cached artifact')
        result = build_cache.restore(self.root, self.target,
                                     self.paths+[('new.rs', new)], self.context)
        self.assertEqual(result['changed'], 1)
        self.assertEqual(new.read_text(), 'new input\n')
        self.assertEqual(output.read_bytes(), b'cached artifact')

    def test_fresh_checkout_restores_tsv_and_arbitrary_embedded_assets(self):
        names = ['engine.rs', 'data/outside-card-names.tsv',
                 'data/interchangeable-card-names.tsv', 'data/embedded.opaque']
        for name in names[1:]:
            path = self.root/name
            path.parent.mkdir(exist_ok=True)
            path.write_text('unchanged embedded input\n')
        for name in names:
            os.utime(self.root/name, ns=(10_000, 10_000))
        listing = ('\0'.join(names)+'\0').encode()
        with patch.object(build_cache.subprocess, 'check_output', return_value=listing):
            build_cache.save(self.root, self.target, build_context=self.context)
            for name in names:
                os.utime(self.root/name, ns=(20_000, 20_000))
            result = build_cache.restore(self.root, self.target, build_context=self.context)
            self.assertEqual(result, {'restored': len(names), 'changed': 0})
            self.assertTrue(all((self.root/name).stat().st_mtime_ns == 10_000 for name in names))
            changed = self.root/names[1]
            changed.write_text('changed embedded input\n')
            os.utime(changed, ns=(10_000, 10_000))
            result = build_cache.restore(self.root, self.target, build_context=self.context)
            self.assertEqual(result['changed'], 1)
            self.assertGreater(changed.stat().st_mtime_ns, 20_000)


if __name__ == '__main__':
    unittest.main()
