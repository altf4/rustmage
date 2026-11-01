"""A newly appended identity must not invalidate unrelated set/engine sources."""
import copy
import json
from pathlib import Path
import unittest
import link_catalogue


class WorkspaceCatalogueTests(unittest.TestCase):
    def test_new_physical_identity_after_tokens_keeps_engine_and_old_sets_stable(self):
        metadata, records, version = link_catalogue.load()
        before = link_catalogue.render(metadata, records, version)
        metadata = copy.deepcopy(metadata)
        records = dict(records)
        card = dict(metadata['cards'][4])
        index = len(metadata['cards'])
        card.update(index=index, name='Workspace Registry Probe', oracle_id=None,
                    module=f'card_{index}', path='cache_probe/registry_probe.rs', set='cache_probe')
        metadata['cards'].append(card)
        metadata['colors'].append(metadata['colors'][4])
        metadata['physical_count'] += 1
        metadata['aliases'] = sorted(metadata['aliases'] + [['Workspace Registry Probe', index]])
        records[index] = dict(complete=True, relative='src/card_definitions/cache_probe/registry_probe.rs')
        after = link_catalogue.render(metadata, records, 'changed-catalogue')
        root = link_catalogue.ROOT
        for path, original in before.items():
            if path.is_relative_to(root/'crates/engine') or path.is_relative_to(root/'crates/cards'):
                self.assertEqual(after[path], original, str(path))
        report = json.loads(after[root/'data/standard/coverage.json'])
        self.assertTrue(any(c['name'] == card['name'] and c['implemented'] for c in report['cards']))
        self.assertIn(f'Card::from_catalogue_id({index}),', after[root/'src/generated/catalogue.rs'])
        self.assertIn(root/'crates/cards/cache_probe/Cargo.toml', after)


if __name__ == '__main__':
    unittest.main()
