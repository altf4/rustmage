"""Pinned full-source provenance; no rules text is compiled."""
import hashlib,json,os,re,subprocess,unittest
from pathlib import Path
if __package__:
    from .source_paths import xmage_source
else:
    from source_paths import xmage_source

ROOT=Path(__file__).resolve().parents[1]

NAMES = ["Archenemy's Charm", 'Auroral Procession', 'Barkform Harvester', 'Coati Scavenger', 'Dundoolin Weaver', 'Elvish Regrower', 'Fateshaper Aspirant', 'Fight On!', "Hazel's Nocturne", 'Macabre Waltz', "Morcant's Loyalist", 'Oltec Archaeologists', 'Pull from the Grave', 'Restore with Empathy', "Rydia's Return", 'Tomb Trawler', 'Tombstone, Career Criminal', 'True Ancestry', 'White Widow, Free Agent']

class GraveyardRecoveryTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((ROOT / 'data/standard/cards.json').read_text())['cards']}

    def test_pinned_java_complete_card_and_direct_dependency_provenance(self):
        import hashlib
        import subprocess
        from pathlib import Path
        manifest = json.loads((ROOT / 'docs/graveyard-recovery-admissions.json').read_text())
        java = xmage_source()
        self.assertEqual({r['name'] for r in manifest['cards']}, set(NAMES))
        checked = set()
        for row in manifest['cards']:
            for key in ['oracle_id', 'mana_cost', 'type_line', 'oracle_text']:
                self.assertEqual(row[key], self.cards[row['name']][key])
            for source in row['sources']:
                name = source['java_source']
                if name in checked:
                    continue
                raw = (java / name).read_bytes()
                pinned = subprocess.check_output(['git', '-C', str(java), 'show', manifest['java_revision'] + ':' + name])
                self.assertEqual(raw, pinned, name)
                self.assertEqual(hashlib.sha256(raw).hexdigest(), source['java_sha256'], name)
                checked.add(name)
        self.assertEqual(len(checked), 69)
