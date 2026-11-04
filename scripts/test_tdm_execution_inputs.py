#!/usr/bin/env python3
"""Small adversarial contracts for TDM's reusable-corpus source identity."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from tdm_execution_inputs import REQUIRED, encoded, payload


class ExecutionInputs(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for name in REQUIRED:
            self.put(name, 'required '+name)
        self.put('crates/engine/src/policy.rs', 'old policy')
        self.put('src/card_definitions/tdm/example.rs', 'typed rules')
        self.put('src/card_definitions/tdm/example.json', '{"complete":true}')
        self.put('src/generated/catalogue.rs', 'catalogue unchanged')
        self.put('crates/cards/tdm/Cargo.toml', 'set package')
        self.put('examples/support/tdm_games.rs', 'self hashed helper')

    def put(self, name, text):
        p = self.root/name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)

    def test_engine_only_change_invalidates_identity_with_same_catalogue(self):
        before = encoded(self.root)
        catalogue = (self.root/'src/generated/catalogue.rs').read_bytes()
        self.put('crates/engine/src/policy.rs', 'new policy')
        self.assertNotEqual(before, encoded(self.root))
        self.assertEqual(catalogue, (self.root/'src/generated/catalogue.rs').read_bytes())

    def test_pair_provenance_and_build_profile_are_bound(self):
        before = encoded(self.root)
        self.put('src/card_definitions/tdm/example.json', '{"complete":false}')
        changed = encoded(self.root)
        self.assertNotEqual(before, changed)
        self.put('Cargo.toml', '[profile.test]\nopt-level=1\n')
        self.assertNotEqual(changed, encoded(self.root))

    def test_entrypoint_and_new_compiled_module_are_bound(self):
        before = encoded(self.root)
        self.put('tests/tdm_complete_games.rs', 'new test wiring')
        changed = encoded(self.root)
        self.assertNotEqual(before, changed)
        self.put('crates/engine/src/new_module.rs', 'new rule')
        self.assertNotEqual(changed, encoded(self.root))

    def test_evidence_updates_and_self_hash_do_not_form_a_cycle(self):
        before = encoded(self.root)
        self.put('README.md', 'new accepted counts')
        self.put('docs/tdm-set-acceptance.json', 'new evidence')
        self.put('examples/support/tdm_execution_inputs.json', 'previous manifest')
        self.put('examples/support/tdm_games.rs', 'new separately self-hashed helper')
        self.assertEqual(before, encoded(self.root))

    def test_digests_are_exact_relative_and_canonical(self):
        value = payload(self.root)
        self.assertEqual(value['source_count'], len(value['files']))
        self.assertIn('crates/engine/src/policy.rs', value['files'])
        for name, digest in value['files'].items():
            self.assertFalse(Path(name).is_absolute())
            self.assertNotIn('..', Path(name).parts)
            self.assertEqual(digest, hashlib.sha256((self.root/name).read_bytes()).hexdigest())
        self.assertEqual(json.loads(encoded(self.root)), value)

    def test_missing_or_symlinked_input_fails_closed(self):
        p = self.root/'Cargo.lock'
        p.unlink()
        with self.assertRaisesRegex(ValueError, 'missing or nonregular'):
            payload(self.root)
        p.symlink_to('Cargo.toml')
        with self.assertRaisesRegex(ValueError, 'missing or nonregular'):
            payload(self.root)


if __name__ == '__main__':
    unittest.main()
