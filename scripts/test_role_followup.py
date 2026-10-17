"""Whole Role follow-ups: source binding, separate reflexive targets and raw closure."""
import copy,json,re,subprocess,unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
NAMES=['Become Brutes','Cursed Courtier','Cut In','Diminisher Witch',"Eriette's Whisper",'Merry Bards','Protective Parents','Shatter the Oath','Spellbook Vendor',"The Witch's Vanity",'Unassuming Sage',"Witch's Mark"]
class RoleFollowupTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def compile_one(self,c):
  text,coverage=metadata.LegacyMetadataTests.compile([c]);row=next(r for r in coverage['cards'] if r['name']==c['name']);return text,row
 def test_all_twelve_whole_bodies_are_name_independent(self):
  for name in NAMES:
   with self.subTest(name=name):
    old=self.cards[name];text,row=self.compile_one(old);self.assertTrue(row['implemented'])
    new=copy.deepcopy(old);new['name']='Independent Whole Role';new['oracle_text']=new['oracle_text'].replace(name,new['name']);other,row=self.compile_one(new);self.assertTrue(row['implemented'])
    norm=lambda s:sorted(l for l in s.splitlines() if not l.startswith('pub const CATALOGUE_VERSION:'))
    self.assertEqual(norm(text.replace(json.dumps(name),json.dumps(new['name']))),norm(other))
 def test_source_and_payment_and_group_programs_preserve_exact_scope(self):
  texts={n:self.compile_one(self.cards[n])[0] for n in NAMES}
  self.assertIn('subject: Subject::Source',texts['Cursed Courtier']);self.assertIn('TargetSpec::NONE',texts['Cursed Courtier'])
  self.assertIn('MayPayMana',texts['Unassuming Sage']);self.assertIn('subject: Subject::Source',texts['Unassuming Sage']);self.assertNotIn('Instruction::Reflexive',texts['Unassuming Sage'])
  for name in ['Merry Bards','Spellbook Vendor']:
   self.assertIn('Instruction::MayPayMana',texts[name]);self.assertIn('Instruction::Reflexive { ability: 1 }',texts[name]);self.assertIn('condition: TriggerCondition::Reflexive',texts[name])
  self.assertIn('condition: TriggerCondition::SelfBargained',texts['Diminisher Witch'])
  for name in ['Cut In',"Eriette's Whisper",'Shatter the Oath']:
   self.assertIn('TargetPlan::Groups',texts[name]);self.assertIn('Instruction::SelectGroup(1)',texts[name]);self.assertIn('min: 0, max: 1',texts[name])
  self.assertIn('min: 1, max: 2',texts['Become Brutes']);self.assertIn('Instruction::GrantKeywords',texts['Become Brutes'])
  self.assertIn('max_mana: 2',texts["The Witch's Vanity"])
  self.assertEqual(len(re.findall(r'condition: TriggerCondition::Chapter',texts["The Witch's Vanity"])),3)
  mark=texts["Witch's Mark"];self.assertRegex(mark,r'ChooseObjects \{[^\n]+optional_skip: Some\(1\)');self.assertIn('Draw(PlayerSet::Controller, 2), Instruction::CreateRole',mark)
  self.assertIsNone(g.source_role_program('create a Royal Role token attached to it.','CreatureEntersYou'))
  self.assertIsNone(g.source_role_program('create a Royal Role token attached to it.','SelfDies'))
 def test_unknown_raw_residuals_and_infix_reminders_never_admit(self):
  for name in NAMES:
   base=self.cards[name];text=base['oracle_text'];variants=[text+'\nDiscover 4.',text+' ((Draw seven cards.))']
   for m in re.finditer(r'\bRole token\b',text):
    for bad in ['(Draw seven cards.)',' ((Draw seven cards.)) ','\t(Draw seven cards.)\t']:
     variants.append(text[:m.start()+4]+bad+text[m.start()+4:])
   for variant in variants:
    with self.subTest(name=name,text=variant):
     _,row=self.compile_one(dict(base,oracle_text=variant));self.assertFalse(row['implemented'])
  base=self.cards['Diminisher Witch']
  for variant in ['Bargain (You may sacrifice an artifact, enchantment, or token as you cast this spell.)\nWhen this creature enters, draw a card. (Draw seven cards.)','Bargain (Draw seven cards.)\nWhen this creature enters, draw a card.']:
   self.assertFalse(self.compile_one(dict(base,oracle_text=variant))[1]['implemented'])
 def test_exact_twelve_admissions_preserve_every_prior_program_and_recipe(self):
  old=subprocess.check_output(['git','-C',str(g.ROOT),'show','ce9a90a1867ce741387d3ed0a1aac0aa1592ca08:src/generated/catalogue.rs'],text=True);new=(g.ROOT/'src/generated/catalogue.rs').read_text()
  defs=lambda t:[l for l in t.splitlines() if l.lstrip().startswith('Definition {')]
  before,after=defs(old),defs(new);self.assertEqual(len(before),5328);self.assertEqual(len(after),5328)
  changed=[(a,b) for a,b in zip(before,after) if a!=b];self.assertEqual(len(changed),12)
  self.assertEqual({json.loads(re.search(r'name: ("(?:[^"\\]|\\.)*")',b)[1]) for a,b in changed},set(NAMES))
  for a,b in changed:self.assertIn('effect: Effect::Unsupported',a)
  index=next(i for i,l in enumerate(after) if 'name: "Diminisher Witch",' in l)
  def strip(text,is_new=False):
   rows=[];function=None;omitted=0
   for l in text.splitlines():
    if l.startswith('pub(super) fn '):function=l
    if is_new and function and function.startswith('pub(super) fn bargain_reduction(') and l==f'        {index} => Some(0),':omitted+=1;continue
    if not l.lstrip().startswith('Definition {') and not l.startswith(('pub const CATALOGUE_VERSION:','    pub const ALL:')):rows.append(l)
   self.assertEqual(omitted,1 if is_new else 0)
   return rows
  self.assertEqual(strip(old),strip(new,True))
if __name__=='__main__':unittest.main()
