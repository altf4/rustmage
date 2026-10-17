"""Balanced original-raw modal authentication, independent of card identities."""
import copy,json,subprocess,unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata
from scripts.catalogue_preservation import canonical,assert_union
PARENT='a9380ec478646c42308c7284de197fa466db24d2'
class ModalRawTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
 def card(self,body,name='Independent Modal Subject',spell=False):
  return dict(metadata.LegacyMetadataTests.creature(name),oracle_text=body,**({'type_line':'Instant'}if spell else{}))
 def compile(self,card):
  rust,cov=metadata.LegacyMetadataTests.compile([card]);return rust,next(x['implemented']for x in cov['cards']if x['name']==card['name'])
 def test_detection_requires_a_complete_modal_header(self):
  for raw in ['Tiered (Choose one additional cost.)', 'Choose one additional cost.', 'Choose one card from your hand.']:
   self.assertFalse(g.modal_route(raw),raw)
   self.assertEqual(g.modal_reminder_text(dict(oracle_text=raw)),raw)
  for raw in ['Choose one —','Choose one or both —','Cho(unknown)ose one —','Choose one (unknown) —','Choose one or (unknown)both —']:
   self.assertTrue(g.modal_route(raw),raw)
 def test_all_generic_hosts_and_reserved_names_reject_unknown_raw(self):
  basic='• Draw a card.\n• You gain 1 life.'
  hosts=[('When this creature enters, choose one —\n'+basic,False),('Whenever this creature attacks, choose one —\n'+basic,False),('{2}: Choose one —\n'+basic,False),('Choose one —\n'+basic,True),('Choose one or both —\n'+basic,True),('When this creature enters, choose one —\n'+basic+'\n{1}: You gain 1 life.',False),('{1}: Choose one —\n'+basic+'\nWhenever this creature attacks, draw a card.',False)]
  for name in ['Independent Modal Subject',*[v[0]for v in g.LEGACY]]:
   for body,spell in hosts:
    c=self.card(body,name,spell);self.assertTrue(self.compile(c)[1],(name,body))
    for bad in [body+' (You win the game.)',body+'\n(You win the game.)',body.replace('Draw a card.','Draw a card. ((You win the game.))'),body.replace('Draw a card.','Dr(You win the game.)aw a card.'),body.replace('choose one','cho(You win the game.)ose one').replace('Choose one','Cho(You win the game.)ose one'),body+' (',body+' )']:
     self.assertFalse(self.compile(dict(c,oracle_text=bad))[1],(name,bad))
 def test_printed_modal_cards_keep_complete_programs_and_legitimate_reminders(self):
  cov=json.loads((g.ROOT/'data/standard/coverage.json').read_text());names=[c['name']for c in cov['cards']if c['implemented']and c['name']in self.cards and g.modal_route(self.cards[c['name']].get('oracle_text'))]
  self.assertGreater(len(names),50)
  for name in names:
   c=self.cards[name];self.assertTrue(self.compile(c)[1],name)
   renamed='Independent Printed Modal';raw=c['oracle_text'].replace(name,renamed)
   if c['type_line'].startswith('Legendary '):raw=raw.replace(name.split(',')[0],renamed)
   self.assertTrue(self.compile(dict(c,name=renamed,oracle_text=raw))[1],name)
 def test_matching_semantic_counts_subjects_and_recipes(self):
  cases=[('Scry 3.','Look at the top three cards of your library, then put any number of them on the bottom and the rest on top in any order.'),('Surveil 2, then draw two cards.','To surveil 2, look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.'),('Mill three cards, then draw a card.','To mill three cards, put the top three cards of your library into your graveyard.'),('Create a Food token.','It\'s an artifact with "{2}, {T}, Sacrifice this token: You gain 3 life."'),('Target creature gains indestructible until end of turn.','Damage and effects that say "destroy" don\'t destroy it.'),('Creatures you control gain hexproof until end of turn.',"They can't be the targets of spells or abilities your opponents control."),('Target creature you control fights target creature you don\'t control.','Each deals damage equal to its power to the other.')]
  for effect,reminder in cases:
   raw='When this creature enters, choose one —\n• '+effect+' ('+reminder+')\n• Draw a card.';c=self.card(raw)
   self.assertTrue(self.compile(c)[1],raw)
   clean=self.card(raw.replace(' ('+reminder+')',''));self.assertEqual(g.modal_reminder_text(c),clean['oracle_text'])
   for wrong in [reminder+' You win the game.',reminder.replace('three','two').replace('surveil 2','surveil 3').replace('{2}','{3}').replace('indestructible','flying').replace('They','It')]:
    if wrong==reminder:continue
    self.assertFalse(self.compile(dict(c,oracle_text=raw.replace(reminder,wrong)))[1],wrong)
   stolen='When this creature enters, choose one —\n• Draw a card. ('+reminder+')\n• You gain 1 life.'
   self.assertFalse(self.compile(self.card(stolen))[1],stolen)
 def test_valid_nested_roles_require_matching_inner_and_outer_semantics(self):
  ward='Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.'
  for host,spell in [('When this creature enters, choose one —',False),('{2}: Choose one —',False),('Choose one —',True)]:
   raw=host+'\n• Create a Royal Role token attached to target creature you control. (Enchanted creature gets +1/+1 and has ward {1}. ('+ward+'))\n• Draw a card.';c=self.card(raw,spell=spell)
   self.assertTrue(self.compile(c)[1],raw)
   clean=raw[:raw.index(' (Enchanted')]+raw[raw.index('\n• Draw'):]
   self.assertEqual(g.modal_reminder_text(c),clean)
   for bad in [raw.replace('pays {1}','pays {2}'),raw.replace('ward {1}','ward {2}'),raw.replace(ward,'You win the game.'),raw.replace('Enchanted creature gets +1/+1 and has ward {1}.','You win the game.'),raw.replace('('+ward+')','(('+ward+'))'),raw.replace('Royal Role','Monster Role')]:
    self.assertFalse(self.compile(dict(c,oracle_text=bad))[1],bad)
 def test_unknown_outer_unbalanced_and_pathological_nesting_fail_closed(self):
  base='When this creature enters, choose one —\n• Draw a card.\n• You gain 1 life.'
  for text in [base+' ('*300+'You win the game.'+')'*300,'(Flying (This creature can\'t be blocked except by creatures with flying or reach.))\n'+base,'Flying ((This creature can\'t be blocked except by creatures with flying or reach.))\n'+base]:
   self.assertFalse(self.compile(self.card(text))[1])
  good="Flying (This creature can't be blocked except by creatures with flying or reach.)\n"+base
  self.assertTrue(self.compile(self.card(good))[1])
 def test_names_flavor_labels_and_quotes_cannot_supply_a_mechanic(self):
  scry='Look at the top three cards of your library, then put any number of them on the bottom and the rest on top in any order.'
  food='It\'s an artifact with "{2}, {T}, Sacrifice this token: You gain 3 life."'
  role='Enchanted creature gets +1/+1 and has ward {1}.'
  for label,reminder in [('Scry 3',scry),('Create a Food token',food),('Create a Royal Role token attached to target creature',role)]:
   raw='When this creature enters, choose one —\n• '+label+' — Draw a card. ('+reminder+')\n• You gain 1 life.'
   self.assertFalse(self.compile(self.card(raw))[1],raw)
  name='Scry 3.'
  raw='Choose one —\n• '+name+' deals 1 damage to target creature. ('+scry+')\n• Draw a card.'
  self.assertFalse(self.compile(self.card(raw,name,True))[1],raw)
 def test_singular_and_nested_keyword_reminders_preserve_actual_programs(self):
  pairs=[('Scry 1.','Look at the top card of your library. You may put that card on the bottom of your library.'),('Surveil 1.','To surveil 1, look at the top card of your library. You may put it into your graveyard.'),('Mill a card.','To mill a card, put the top card of your library into your graveyard.')]
  for effect,reminder in pairs:
   raw='When this creature enters, choose one —\n• '+effect+' ('+reminder+')\n• Draw a card.'
   self.assertTrue(self.compile(self.card(raw))[1],raw)
  reminder='Look at the top card of your library. You may put that card on the bottom of your library.'
  raw='When this creature enters, choose one —\n• Create a Sorcerer Role token attached to target creature you control. (Enchanted creature gets +1/+1 and has "Whenever this creature attacks, scry 1. ('+reminder+')")\n• Draw a card.'
  self.assertTrue(self.compile(self.card(raw))[1],raw)
 def test_no_actual_program_helper_factory_color_or_alias_changes(self):
  physical={c['oracle_id']:c['name']for c in self.cards.values()};before=canonical(subprocess.check_output(['git','-C',str(g.ROOT),'show',PARENT+':src/generated/catalogue.rs'],text=True),physical);after=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical);proof=assert_union(before,after,changed=set(),added=set(),helpers=set(),aliases={});self.assertEqual(proof['preserved_definitions'],5332);self.assertEqual(proof['prior_helpers'],911)
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed=set(),added=set(),helpers=set(),aliases={})
if __name__=='__main__':unittest.main()
