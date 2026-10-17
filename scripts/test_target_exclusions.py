import copy,hashlib,json,os,re,subprocess,unittest
from pathlib import Path
from scripts import generate_catalogue as g
NAMES={'Amazing Acrobatics','Coeurl','Decoy Ploy','Guerrilla Gorilla','Overwhelming Surge','Rock Soldiers','Sterling Keykeeper'}
class ExclusionTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.cards={c['name']:c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name'] in NAMES}
 def compile(self,c):
  payload=json.dumps([c],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode();rust,cov=g.compile_catalogue(dict(cards=[c],card_count=1,cards_sha256=hashlib.sha256(payload).hexdigest()),dict(banned_card_names=[]))
  return rust,next(row['implemented'] for row in cov['cards'] if row['name']==c['name'])
 def definition(self,rust,name):
  rows=[line for line in rust.splitlines() if line.lstrip().startswith('Definition {')]
  names=[json.loads(re.search(r'name: ("(?:[^"\\]|\\.)*")',row)[1]) for row in rows]
  row=next(row for row in rows if 'name: '+json.dumps(name)+',' in row)
  return re.sub(r'Card\((\d+)\)',lambda m:'Card('+json.dumps(names[int(m[1])])+')',row)
 def test_complete_all_seven_and_reserved_names(self):
  for name,c in self.cards.items():
   original=self.definition(self.compile(c)[0],name)
   for alias in [name,'Fresh Filter',*[v[0] for v in g.LEGACY]]:
    text=c['oracle_text'].replace(name,alias);rust,ok=self.compile(dict(c,name=alias,oracle_text=text));self.assertTrue(ok,(name,alias));self.assertEqual(self.definition(rust,alias),original.replace(name,alias),(name,alias))
    if name in {'Amazing Acrobatics','Decoy Ploy','Overwhelming Surge'}:self.assertIn('text: "Choose both"',rust)
 def test_every_raw_new_route_fails_closed(self):
  for name,c in self.cards.items():
   for i,line in enumerate(c['oracle_text'].splitlines()):
    for suffix in [' Discover 4.',' (You win the game.)',' ((You win the game.))']:
     lines=c['oracle_text'].splitlines();lines[i]+=suffix
     self.assertFalse(self.compile(dict(c,oracle_text='\n'.join(lines)))[1],(name,i,suffix))
   for token in ['noncreature','nonenchantment','non-Mount','one or both']:
    if token not in c['oracle_text']:continue
    for injected in ['(You win the game.)',' ((You win the game.)) ']:
     text=c['oracle_text'].replace(token,token[:3]+injected+token[3:])
     self.assertFalse(self.compile(dict(c,oracle_text=text))[1],(name,token,injected))
 def test_intrinsic_reminder_is_line_bound_on_noncreature_hosts(self):
  reminder='Damage and effects that say "destroy" don\'t destroy it.'
  for host in ['Artifact','Enchantment']:
   body='{2}, {T}: Tap target nonenchantment creature.'
   c=dict(self.cards['Coeurl'],name='Independent exclusion host',type_line=host,mana_cost='{2}',oracle_text='Indestructible\n'+body)
   clean,ok=self.compile(c);self.assertTrue(ok)
   shown,ok=self.compile(dict(c,oracle_text='Indestructible ('+reminder+')\n'+body));self.assertTrue(ok);self.assertEqual(self.definition(clean,c['name']),self.definition(shown,c['name']))
   for raw in ['Indestructible\n'+body+' ('+reminder+')','Indestructible ('+reminder+') (You win the game.)\n'+body]:self.assertFalse(self.compile(dict(c,oracle_text=raw))[1])
 def test_three_legal_combinations_only_and_ordered_groups(self):
  for name in ['Amazing Acrobatics','Decoy Ploy','Overwhelming Surge']:
   c=self.cards[name];modes=g.modal_spell_template(c);self.assertEqual(len(modes),3);self.assertEqual(len(modes[2][1]),2);self.assertIn('SelectGroup(1)',modes[2][2])
   self.assertIsNone(g.modal_spell_template(dict(c,oracle_text=c['oracle_text']+'\n• Draw a card.')))
 def test_historical_seven_union_and_current_programs_remain_exact(self):
  from scripts.catalogue_preservation import canonical,assert_union
  old=subprocess.check_output(['git','-C',str(g.ROOT),'show','ce4b0c104a8e0dbc0a76753a03608eb7bc4618be:src/generated/catalogue.rs'],text=True);physical={c['oracle_id']:c['name'] for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}
  merged=subprocess.check_output(['git','-C',str(g.ROOT),'show','8108a6f3366f8d5553f4059b9c648481769eb420:src/generated/catalogue.rs'],text=True)
  before=canonical(old,physical);after=canonical(merged,physical);changed={n+'#0' for n in NAMES};report=assert_union(before,after,changed=changed,added=set(),helpers=set(),aliases={});self.assertEqual(report['preserved_definitions'],5324)
  current=canonical((g.ROOT/'src/generated/catalogue.rs').read_text(),physical)
  for key in changed:self.assertEqual(current['definitions'][key],after['definitions'][key])
  mutant=copy.deepcopy(after);mutant['definitions']['Plains#0']='unrelated mutation'
  with self.assertRaises(AssertionError):assert_union(before,mutant,changed=changed,added=set(),helpers=set(),aliases={})
 def test_all_full_records_and_explicit_direct_imports_are_pinned(self):
  m=json.loads((g.ROOT/'docs/target-exclusion-admissions.json').read_text());root=Path(os.environ.get('RUSTMAGE_XMAGE_SOURCE','/home/altf4/Code/mage'));self.assertEqual({r['name'] for r in m['cards']},NAMES)
  for row in m['cards']:
   for k in ['name','oracle_id','mana_cost','type_line','oracle_text']:self.assertEqual(row[k],self.cards[row['name']][k])
   for rec in row['sources']:
    blob=subprocess.check_output(['git','-C',str(root),'show',m['java_revision']+':'+rec['java_source']]);self.assertEqual((root/rec['java_source']).read_bytes(),blob);self.assertEqual(hashlib.sha256(blob).hexdigest(),rec['java_sha256'])
if __name__=='__main__':unittest.main()
