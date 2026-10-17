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
 def test_new_generic_productions_authenticate_raw_without_role_clauses(self):
  fixtures=[
   (self.cards['Shatter the Oath'],'Destroy target creature an opponent controls with mana value 2 or less.','max_mana: 2'),
   (self.cards['Shatter the Oath'],'Destroy target creature or enchantment.','Destroy'),
   (self.cards['Diminisher Witch'],'When this creature enters, if it was bargained, draw a card.','TriggerCondition::SelfBargained'),
  ]
  for base,text,semantic in fixtures:
   for name in ['Independent generic production',*[row[0] for row in g.LEGACY]]:
    c=dict(base,name=name,oracle_text=text)
    rust,row=self.compile_one(c);self.assertTrue(row['implemented'],(name,text));self.assertIn(semantic,rust)
    for cost in ['{Y}','{4294967296}']:
     self.assertFalse(self.compile_one(dict(c,mana_cost=cost))[1]['implemented'])
    self.assertFalse(self.compile_one(dict(c,type_line='Malformed'))[1]['implemented'])
   variants=[text+' (You win the game.)',text+' ((You win the game.))',text+'\n(Draw seven cards.)',text+' (This creature can attack and {T} as soon as it comes under your control.)']
   for index in range(1,len(text)):
    if text[index-1].isalpha() and text[index].isalpha():
     for residual in ['(Draw seven cards.)','((Draw seven cards.))']:
      variants.append(text[:index]+residual+text[index:])
   for index in [m.end() for m in re.finditer(r'\s+',text)]:
    variants.append(text[:index]+'(You win the game.) '+text[index:])
   for variant in variants:
    for name in ['Independent generic production','Shock']:
     self.assertFalse(self.compile_one(dict(base,name=name,oracle_text=variant))[1]['implemented'],(name,variant))
 def test_generic_bargained_event_preserves_authenticated_intrinsic_reminder(self):
  base=self.cards['Diminisher Witch']
  for noun,typ,keyword,reminder in [('creature','Creature','Flying',"This creature can't be blocked except by creatures with flying or reach."),('artifact','Artifact','Indestructible','Damage and effects that say "destroy" don\'t destroy it.'),('enchantment','Enchantment','Indestructible','Damage and effects that say "destroy" don\'t destroy it.')]:
   body='When this '+noun+' enters, if it was bargained, draw a card.'
   clean=dict(base,name='Independent keyword and bargained event',type_line=typ,oracle_text=keyword+'\n'+body)
   text=keyword+' ('+reminder+')\n'+body
   expected,row=self.compile_one(clean);self.assertTrue(row['implemented'])
   actual,row=self.compile_one(dict(clean,oracle_text=text));self.assertTrue(row['implemented'])
   # The complete semantics, including keyword, cost and type, remain identical.
   drop=lambda s:[line for line in s.splitlines() if not line.startswith('pub const CATALOGUE_VERSION:')]
   self.assertEqual(drop(expected),drop(actual))
   for variant in [keyword+' (You win the game.)\n'+body,body+' ('+reminder+')']:
    self.assertFalse(self.compile_one(dict(clean,oracle_text=variant))[1]['implemented'])
 def test_generic_bargained_raw_authentication_survives_every_source_alias(self):
  base=self.cards['Shatter the Oath']
  forms=[('creature','Creature','{3}',''),('artifact','Artifact','{3}',''),('enchantment','Enchantment','{3}',''),('land','Land','',''),('Aura','Enchantment — Aura','{3}','Enchant creature\n'),('Equipment','Artifact — Equipment','{3}','Equip {1}\n')]
  cards=[]
  for noun,typ,cost,prefix in forms:
   cards.append(dict(base,name='Fresh Binding',type_line=typ,mana_cost=cost,power='2',toughness='2',oracle_text=prefix+'When this '+noun+' enters, if it was bargained, draw a card.'))
  for name,subject,typ in [('Fresh Witch','Fresh Witch','Creature'),('Fresh Witch, Watcher','Fresh Witch','Legendary Creature'),('Fresh Witch, Watcher','Fresh Witch, Watcher','Legendary Creature')]:
   cards.append(dict(base,name=name,type_line=typ,power='2',toughness='2',oracle_text='When '+subject+' enters, if it was bargained, draw a card.'))
  for card in cards:
   rust,row=self.compile_one(card);self.assertTrue(row['implemented'],card);self.assertIn('TriggerCondition::SelfBargained',rust)
   text=card['oracle_text']
   variants=[text+' (You win the game.)',text+' ((You win the game.))',text+'\n(Draw seven cards.)']
   for index in range(1,len(text)):
    if text[index-1].isalpha() and text[index].isalpha():
     variants.append(text[:index]+'(Draw seven cards.)'+text[index:])
   for index in [m.end() for m in re.finditer(r'\s+',text)]:
    variants.append(text[:index]+'((You win the game.)) '+text[index:])
   for variant in variants:
    self.assertFalse(self.compile_one(dict(card,oracle_text=variant))[1]['implemented'],(card['name'],variant))
 def test_exact_twelve_admissions_preserve_every_prior_program_and_recipe(self):
  old=subprocess.check_output(['git','-C',str(g.ROOT),'show','c41c060c024626843b17cf292aab838d36fd6eb1:src/generated/catalogue.rs'],text=True);new=(g.ROOT/'src/generated/catalogue.rs').read_text()
  defs=lambda t:[l for l in t.splitlines() if l.lstrip().startswith('Definition {')]
  before,after=defs(old),defs(new);self.assertEqual(len(before),5331);self.assertEqual(len(after),5331)
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
