"""Full-set membership and admission remain distinct from rules acceptance."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from scripts import set_status

if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT = Path(__file__).resolve().parents[1]

class SetStatusTests(unittest.TestCase):
    def test_full_foundations_freeze_matches_all_pinned_printings(self):
        manifest = json.loads((ROOT / 'data/sets/fdn.json').read_text())
        fresh = set_status.freeze(ROOT, xmage_source(), 'FDN')
        self.assertEqual(fresh, manifest)
        self.assertEqual(len(manifest['cards']), 517)
        self.assertEqual(sum(len(c['printings']) for c in manifest['cards']), 771)
        report = set_status.status(ROOT, manifest)
        self.assertEqual(report['total_cards'], report['admitted_cards'] + report['gated_cards'])
        self.assertEqual(report['unresolved'], [])
        self.assertTrue(report['acceptance'].startswith('pending:'))

    def test_stale_facts_and_duplicate_reprints_cannot_inflate_admission(self):
        manifest = json.loads((ROOT / 'data/sets/fdn.json').read_text())
        duplicate = copy.deepcopy(manifest)
        duplicate['cards'].append(duplicate['cards'][0])
        duplicate['card_count'] += 1
        with self.assertRaisesRegex(ValueError, 'duplicate set Oracle identity'):
            set_status.status(ROOT, duplicate)
        incomplete = copy.deepcopy(manifest)
        incomplete['cards'].pop()
        with self.assertRaisesRegex(ValueError, 'incomplete set manifest'):
            set_status.status(ROOT, incomplete)
        stale = copy.deepcopy(manifest)
        stale['cards'][0]['facts_sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'set facts changed'):
            set_status.status(ROOT, stale)

    def test_missing_source_entries_do_not_erase_oracle_members(self):
        card = dict(name='Starter Reprint', oracle_id='starter', reference_printing=dict(set='fdn'))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'data/standard').mkdir(parents=True)
            raw = b'// intentionally missing from the XMage table\n'
            (root / 'set.java').write_bytes(raw)
            import hashlib
            inventory = dict(xmage_source_commit='pin', set_tables=dict(FDN=dict(source='set.java', sha256=hashlib.sha256(raw).hexdigest(), printing_count=0)))
            (root / 'data/standard/xmage-inventory.json').write_text(json.dumps(inventory))
            (root / 'data/standard/cards.json').write_text(json.dumps(dict(cards=[card])))
            manifest = set_status.freeze(root, root, 'FDN')
            self.assertEqual([c['oracle_id'] for c in manifest['cards']], ['starter'])
            self.assertEqual(manifest['cards'][0]['printings'], [])

    def test_multiface_membership_uses_one_physical_identity(self):
        card=dict(name='Front // Back', oracle_id='both', card_faces=[dict(name='Front'),dict(name='Back')])
        manifest=dict(set='TEST', card_count=1, unresolved=[], cards=[dict(name=card['name'],oracle_id='both',facts_sha256=set_status.digest(card))])
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            (root/'data/standard').mkdir(parents=True)
            (root/'data/standard/cards.json').write_text(json.dumps(dict(cards=[card])))
            (root/'data/standard/coverage.json').write_text(json.dumps(dict(cards=[dict(name='Front',implemented=True)])))
            report=set_status.status(root,manifest)
            self.assertEqual((report['total_cards'],report['admitted_cards']),(1,1))

if __name__ == '__main__':
    unittest.main()
