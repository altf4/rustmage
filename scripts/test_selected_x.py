"""Full selected-X Omen grammar and closed subtype-scaled effects."""
import copy,unittest
from scripts import test_omen as omen
class SelectedXCompilerTests(unittest.TestCase):
    name="Scavenger Regent // Exude Toxin"
    @classmethod
    def setUpClass(cls):omen.OmenCompilerTests.setUpClass.__func__(cls)
    def compile(self,c):return omen.OmenCompilerTests.compile(self,c)
    def admitted(self,c):return omen.OmenCompilerTests.admitted(self,c)
    def test_complete_pair_and_renamed_general_cost_subtype_variants(self):
        for subtype in ["Dragon","Wizard","Bird"]:
            c=copy.deepcopy(self.cards[self.name]);c['name']='Test Regent // Test Toxin';c['card_faces'][0]['name']='Test Regent';c['card_faces'][1]['name']='Test Toxin'
            c['card_faces'][1]['oracle_text']=c['card_faces'][1]['oracle_text'].replace('non-Dragon','non-'+subtype)
            rust,coverage=self.compile(c);self.assertTrue(self.admitted(c));self.assertEqual(coverage['omen_definitions'],1)
            self.assertIn('GrowCreaturesScaled',rust);self.assertIn('excluding_subtype: Some("'+subtype+'")',rust)
    def test_unknown_residuals_on_both_faces_and_malformed_x_remain_closed(self):
        for face in [0,1]:
            for extra in [' Discover 2.',' (Draw nineteen cards.)','\n(You win the game.)']:
                c=copy.deepcopy(self.cards[self.name]);c['card_faces'][face]['oracle_text']+=extra;self.assertFalse(self.admitted(c))
        for cost in ['{X}{X}{B}','{Y}{B}','{X}{B/P}','{X}{4294967296}','{B}{B}','']:
            c=copy.deepcopy(self.cards[self.name]);c['card_faces'][1]['mana_cost']=cost;c['mana_cost']=' // '.join(f['mana_cost'] for f in c['card_faces'] if f['mana_cost']);self.assertFalse(self.admitted(c))
    def test_plain_scalar_route_does_not_erase_any_unknown_parenthetical(self):
        c=copy.deepcopy(self.cards[self.name]['card_faces'][1]);c.update(layout='normal',type_line='Sorcery',colors=['B'],keywords=[],oracle_id=self.cards[self.name]['oracle_id'])
        c['oracle_text']=c['oracle_text'].split(' (Then shuffle')[0];self.assertTrue(self.admitted(c))
        for text in [c['oracle_text']+' (Draw a card.)',c['oracle_text'].replace('non-Dragon','non-Dra(Draw a card.)gon')]:
            bad=copy.deepcopy(c);bad['oracle_text']=text;self.assertFalse(self.admitted(bad))
        for subtype in ['UnknownType','noncreature']:
            bad=copy.deepcopy(c);bad['oracle_text']=bad['oracle_text'].replace('non-Dragon','non-'+subtype);self.assertFalse(self.admitted(bad))
    def test_mass_x_legacy_aliases_use_actual_types_and_closed_costs(self):
        c=copy.deepcopy(self.cards[self.name]['card_faces'][1]);c.update(layout='normal',type_line='Sorcery',colors=['B'],keywords=[],oracle_id=self.cards[self.name]['oracle_id'])
        c['oracle_text']=c['oracle_text'].split(' (Then shuffle')[0]
        for name in ['Grizzly Bears','Llanowar Elves','Shock','Quick Study']:
            good=dict(c,name=name);rust,_=self.compile(good)
            row=next(line for line in rust.splitlines() if 'Definition {' in line and 'name: "'+name+'"' in line)
            self.assertIn('kind: 4',row);self.assertIn('effect: Effect::Spell',row)
            self.assertIn('GrowCreaturesScaled',rust)
            for cost in ['{X}{X}{B}','{Y}{B}','{X}{B/P}','{X}{4294967296}','{B}{B}','']:
                self.assertFalse(self.admitted(dict(good,mana_cost=cost)),(name,cost))
            for text in [good['oracle_text']+' Discover 4.',good['oracle_text'].replace('Dragon','UnknownType'),good['oracle_text'].replace('Each','Each (Unknown (nested) effect.)'), '('+good['oracle_text'].replace('non-Dragon','non-Dra(Unknown.)gon')+')']:
                self.assertFalse(self.admitted(dict(good,oracle_text=text)),(name,text))
if __name__=='__main__':unittest.main()
