"""Complete foreign-library Saga body, residual closure and source provenance."""
import copy, hashlib, json, os, subprocess, unittest
from pathlib import Path
import test_sagas as sagas
g = sagas.g

class OwnershipCompilerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.card = next(c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name']=='Korvold and the Noble Thief')

    implemented = sagas.SagaCompilerTests.implemented

    def test_full_body_and_renamed_identity(self):
        self.assertTrue(self.implemented(self.card))
        renamed = dict(self.card, name='Independent Foreign Library Saga')
        self.assertTrue(self.implemented(renamed))
        self.assertEqual(g.saga_chapter_program("Exile the top three cards of target opponent's library. You may play those cards this turn."), ('Opponent', ['ExileTargetLibraryPlay { amount: 3 }']))

    def test_each_chapter_rejects_unknown_parenthetical_nested_and_executable_residuals(self):
        for i, line in enumerate(self.card['oracle_text'].splitlines()):
            for extra in [' Discover 4.', ' (Discover 4.)', ' (Outer (inner) clause.)', ' (You win the game.)']:
                lines=self.card['oracle_text'].splitlines();lines[i]=line+extra
                self.assertFalse(self.implemented(dict(self.card,oracle_text='\n'.join(lines))), (i,extra))
        for old,new in [('Add one mana of any color.', 'Add three mana of any color.'), ('this turn.', 'for as long as they remain exiled.'), ('You may play those cards', 'You may cast those cards'), ('target opponent', 'target player'), ('three cards', '4294967296 cards')]:
            changed=self.card['oracle_text'].replace(old,new)
            self.assertNotEqual(changed,self.card['oracle_text'])
            self.assertFalse(self.implemented(dict(self.card,oracle_text=changed)), (old,new))

    def test_original_foreign_sources_match_checkout_and_pin(self):
        root=os.environ.get('RUSTMAGE_XMAGE_SOURCE')
        if not root:self.skipTest('Set RUSTMAGE_XMAGE_SOURCE for pinned Java provenance')
        m=json.loads((g.ROOT/'docs/saga-admissions.json').read_text())
        row=next(c for c in m['cards'] if c['name']==self.card['name'])
        self.assertEqual(row['recommended_batch'],'ownership_control_265')
        for source in row['sources']:
            path=source['java_source'];expected=source['java_sha256']
            self.assertEqual(hashlib.sha256((Path(root)/path).read_bytes()).hexdigest(),expected,path)
            blob=subprocess.check_output(['git','-C',root,'show',m['java_revision']+':'+path])
            self.assertEqual(hashlib.sha256(blob).hexdigest(),expected,path)

if __name__=='__main__':unittest.main()
