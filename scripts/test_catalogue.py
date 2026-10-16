import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('generate', Path(__file__).with_name('generate_catalogue.py'))
generate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generate)


class CatalogueCompilerTests(unittest.TestCase):
    def test_opponent_entry_tapped_extraction_is_exact_and_keeps_unknown_labels_and_residuals(self):
        for subject,mask in [('Creatures',1),('Artifacts',16),('Artifacts and creatures',17)]:
            for battlefield in ['', ' the battlefield']:
                for label in ['', 'Venom Blast — ']:
                    clause=label+subject+' your opponents control enter'+battlefield+' tapped.'
                    card=self.card(oracle_text='Flying\n'+clause+'\nWhen this creature enters, draw a card.')
                    residual,value=generate.extract_opponent_entry_tapped(card)
                    self.assertEqual(value,mask);self.assertEqual(residual['oracle_text'],'Flying\nWhen this creature enters, draw a card.');self.assertIsNotNone(generate.trigger_creature_template(residual))
        for clause in ['Creatures you control enter tapped.','Creatures enter tapped.','Creatures an opponent controls enter tapped.','Creature tokens your opponents control enter tapped.','Creatures your opponents control enter tapped unless you control an artifact.','Creatures your opponents control enter tapped. Draw a card.','Creatures your opponents control enter tapped this turn.','Unknown — Creatures your opponents control enter tapped.','Venom Blast - Creatures your opponents control enter tapped.','Venom Blast — Whenever you attack, discover 3.']:
            card=self.card(oracle_text=clause);self.assertEqual(generate.extract_opponent_entry_tapped(card),(card,0),clause)
        for changes in [dict(layout='transform'),dict(type_line='Instant'),dict(type_line='Sorcery'),dict(type_line='Land')]:
            card=self.card(oracle_text='Creatures your opponents control enter tapped.',**changes);self.assertEqual(generate.extract_opponent_entry_tapped(card),(card,0))
        card=self.card(oracle_text='Creatures your opponents control enter tapped.\nVenom Blast — Draw a card.')
        residual,mask=generate.extract_opponent_entry_tapped(card);self.assertEqual(mask,1);self.assertEqual(residual['oracle_text'],'Venom Blast — Draw a card.');self.assertIsNone(generate.creature_template(residual));self.assertIsNone(generate.trigger_creature_template(residual))

    def test_opponent_creature_entry_prefix_preserves_relation_guards_limits_and_owned_context(self):
        condition='CreatureEnters { relation: CastRelation::Opponent, other: false }'
        result=generate.trigger_creature_template(self.card(oracle_text='Whenever a creature an opponent controls enters, you gain 1 life.'))
        self.assertEqual(result[2],[ (condition,['GainLife(PlayerSet::Controller, 1)'],'None') ])
        wrapped=generate.trigger_creature_template(self.card(oracle_text='Whenever a creature an opponent controls enters, if you gained life this turn, put a +1/+1 counter on that creature. This ability triggers only once each turn.'))
        self.assertIn('InterveningIf',wrapped[2][0][0]);self.assertIn('OncePerTurn',wrapped[2][0][0]);self.assertIn(condition,wrapped[2][0][0]);self.assertIn('Subject::Entering',wrapped[2][0][1][0]);self.assertEqual(wrapped[2][0][2],'None')
        for prefix in ['Whenever a creature enters, ','Whenever another creature an opponent controls enters, ','Whenever one or more creatures an opponent controls enter, ','Whenever a creature your opponents control enters, ']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=prefix+'you gain 1 life.')),prefix)
        for tail in ['put a +1/+1 counter on it.','it gets +1/+1 until end of turn.','you gain 1 life. Then discover 3.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Whenever a creature an opponent controls enters, '+tail)),tail)

    def test_opponent_entry_tapped_two_complete_admissions_emit_only_fully_supported_masks(self):
        import hashlib,json,re
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/opponent-entry-tapped-admissions.json').read_text())
        source,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),2)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(card['oracle_text'],row['oracle_text']);residual,mask=generate.extract_opponent_entry_tapped(card);self.assertEqual(mask,1 if row['name']=='Authority of the Consuls' else 17)
            self.assertTrue(generate.trigger_creature_template(residual) or generate.creature_template(residual) or generate.permanent_program_template(residual))
        helper=source.split('fn opponent_entry_tapped(card: Card) -> u16 {',1)[1].split('\n}',1)[0]
        for name,mask in [('Authority of the Consuls',1),('Spider-Woman, Stunning Savior',17)]:
            index=re.search(r'\('+re.escape(json.dumps(name))+r', ([0-9]+)\)',source)[1];self.assertIn(index+' => '+str(mask)+',',helper)
        dismantler=next(c for c in rows['cards'] if c['name']=='Dauntless Dismantler');residual,mask=generate.extract_opponent_entry_tapped(dismantler);self.assertEqual(mask,16);self.assertIn('{X}{X}{W}',residual['oracle_text']);self.assertNotIn('Dauntless Dismantler',supported)
        index=re.search(r'\("Dauntless Dismantler", ([0-9]+)\)',source)[1];self.assertNotIn(index+' => ',helper)
        names={row['name'] for row in manifest['cards']};altered=dict(rows,cards=[dict(c,oracle_text=c['oracle_text']+'\nWhenever you attack, discover 3.') if c['name'] in names else c for c in rows['cards']]);altered['cards_sha256']=hashlib.sha256(json.dumps(altered['cards'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        gated_source,gated=generate.compile_catalogue(altered,target);self.assertTrue(all(not c['implemented'] for c in gated['cards'] if c['name'] in names));gated_helper=gated_source.split('fn opponent_entry_tapped(card: Card) -> u16 {',1)[1].split('\n}',1)[0];self.assertNotRegex(gated_helper,r'\n        [0-9]+ => ')

    def test_entering_object_effects_bind_only_complete_explicit_entry_clauses(self):
        for count,n in [('a',1),('two',2),('six',6)]:
            for haste in [False,True]:
                text='put '+count+' +1/+1 '+('counter' if n==1 else 'counters')+' on that creature'+(' and it gains haste until end of turn' if haste else '')+'.'
                result=generate.entering_object_program(text,'OtherCreatureEntersYou');self.assertIsNotNone(result);self.assertEqual(result[0],'None');self.assertIn('Subject::Entering',result[1][0]);self.assertIn('amount: '+str(n),result[1][0]);self.assertEqual(len(result[1]),2 if haste else 1)
                self.assertIsNone(generate.statement_program(text));self.assertIsNone(generate.targeted_program(text))
                for event in ['SelfDies','SelfAttacks','SelfEntersOrAttacks','SelfEntersOrDies','SpellCast {','AtStep {']:
                    self.assertIsNone(generate.entering_object_program(text,event),(event,text))
        for text in ['put a +1/+1 counter on it.','put a +1/+1 counter on target creature.','put a -1/-1 counter on that creature.','put a +1/+1 counter on that creature. It gains haste until end of turn.','put a +1/+1 counter on that creature and you gain 1 life.','put a +1/+1 counter on that creature and it gains flying until end of turn.','put a +1/+1 counter on that creature. Then discover 3.']:
            self.assertIsNone(generate.entering_object_program(text,'OtherCreatureEntersYou'),text)
        guarded=generate.trigger_creature_template(self.card(oracle_text='Whenever another creature you control enters, if you gained life this turn, put a +1/+1 counter on that creature. This ability triggers only once each turn.'));self.assertIsNotNone(guarded);self.assertIn('OncePerTurn',guarded[2][0][0]);self.assertIn('InterveningIf',guarded[2][0][0]);self.assertIn('Subject::Entering',guarded[2][0][1][0])

    def test_entering_object_admissions_require_the_three_complete_bodies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/entering-object-admissions.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),3)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(card['oracle_text'],row['oracle_text']);result=generate.trigger_creature_template(card);self.assertIsNotNone(result);self.assertIn('Subject::Entering',' '.join(result[2][0][1]));self.assertEqual(result[2][0][2],'None')
            for tail in ['Whenever you attack, discover 3.','Creatures you control lose flying.','As this creature enters, choose a color.']:
                bad=dict(card,oracle_text=card['oracle_text']+'\n'+tail)
                for compiler in [generate.creature_template,generate.trigger_creature_template,generate.permanent_program_template,generate.static_permanent_template]:self.assertIsNone(compiler(bad),(row['name'],tail))

    def test_entry_predicate_prefixes_preserve_entry_time_thresholds_and_singular_unions(self):
        cases=[('another creature you control with power 2 or less','CreaturePowerAtMost(2)'),('another outlaw you control','SubtypeAny(&["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"])'),('another Villain and/or artifact you control','KindOrSubtype { kinds: 16, subtype: "Villain" }'),('another Frog, Rabbit, Raccoon, or Squirrel you control','SubtypeAny(&["Frog", "Rabbit", "Raccoon", "Squirrel"])'),('another Frog or Rabbit you control','SubtypeAny(&["Frog", "Rabbit"])')]
        for subject,predicate in cases:
            result=generate.trigger_creature_template(self.card(oracle_text='Whenever '+subject+' enters, draw a card.'));self.assertIsNotNone(result);self.assertEqual(result[2][0][0],'OtherFilteredEntersYou(EntryPredicate::'+predicate+')')
        plain=generate.trigger_creature_template(self.card(oracle_text='Whenever a creature you control enters, draw a card.'));self.assertEqual(plain[2][0][0],'CreatureEntersYou')
        wrapped=generate.trigger_creature_template(self.card(oracle_text='Whenever another creature you control with power 2 or less enters, if you gained life this turn, draw a card. This ability triggers only once each turn.'));self.assertIsNotNone(wrapped);self.assertIn('OncePerTurn',wrapped[2][0][0]);self.assertIn('InterveningIf',wrapped[2][0][0])

    def test_entry_predicate_unknown_relations_multiplicity_bounds_and_antecedents_stay_gated(self):
        for subject in ['another creature you control with power 32768 or less','another creature you control with power -1 or less','another creature you control with toughness 2 or less','another Frog, Rabbit or Squirrel you control','another Frog or Unknown you control','another Frog or Frog you control','another Villain or artifact you control','one or more outlaws you control','a creature an opponent controls with flying','another outlaw with flying you control']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Whenever '+subject+' enters, draw a card.')),subject)
        for effect in ['put a +1/+1 counter on that creature. Then discover 3.','it gets +1/+1 until end of turn.','draw a card. Then discover 3.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Whenever another creature you control with power 2 or less enters, '+effect)))

    def test_entry_predicate_nine_admissions_are_whole_cards_with_unknown_tails_rejected(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/entry-predicate-admissions.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),9)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(row['oracle_text'],card['oracle_text']);bad=dict(card,oracle_text=card['oracle_text']+'\nWhenever you attack, discover 3.')
            for compiler in [generate.creature_template,generate.trigger_creature_template,generate.permanent_program_template,generate.static_permanent_template]:self.assertIsNone(compiler(bad),row['name'])
        for name in ['Terror of the Peaks']:self.assertNotIn(name,supported)

    def test_entry_suppression_extraction_is_closed_and_preserves_residual_rules(self):
        for subject,mask in [('Creatures',1),('Artifacts and creatures',17)]:
            for battlefield in ['', ' the battlefield']:
                text=subject+" entering"+battlefield+" don't cause abilities to trigger."
                card=self.card(oracle_text='Flash\nFlying\n'+text+'\nWhen this creature enters, draw a card.')
                residual,value=generate.extract_entry_trigger_suppression(card)
                self.assertEqual(value,mask);self.assertEqual(residual['oracle_text'],'Flash\nFlying\nWhen this creature enters, draw a card.')
                self.assertIsNotNone(generate.trigger_creature_template(residual))
        for text in ["Creatures entering or dying don't cause abilities to trigger.","Artifacts entering don't cause abilities to trigger.","Creatures entering don't cause abilities of opponents' permanents to trigger.","Creatures entering don't cause abilities to trigger. Draw a card.","Creature tokens entering don't cause abilities to trigger."]:
            card=self.card(oracle_text=text);self.assertEqual(generate.extract_entry_trigger_suppression(card),(card,0))
        for typ in ['Instant','Sorcery','Land']:
            card=self.card(type_line=typ,oracle_text="Creatures entering don't cause abilities to trigger.")
            self.assertEqual(generate.extract_entry_trigger_suppression(card),(card,0))
        card=self.card(layout='transform',oracle_text="Creatures entering don't cause abilities to trigger.")
        self.assertEqual(generate.extract_entry_trigger_suppression(card),(card,0))

    def test_entry_suppression_admissions_require_complete_bodies_and_preserve_identity(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/entry-suppression-admissions.json').read_text())
        source,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        self.assertEqual(len(manifest['cards']),3)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(row['oracle_text'],card['oracle_text'])
            for tail in ['Whenever you attack, discover 3.','Creatures you control lose flying.','As this creature enters, choose a color.']:
                proxy=dict(card,oracle_text=card['oracle_text']+'\n'+tail);residual,mask=generate.extract_entry_trigger_suppression(proxy);self.assertNotEqual(mask,0)
                for compiler in [generate.creature_template,generate.trigger_creature_template,generate.permanent_program_template,generate.static_permanent_template]:self.assertIsNone(compiler(residual),(row['name'],tail))
                altered=dict(rows,cards=[proxy if c['name']==card['name'] else c for c in rows['cards']]);import hashlib
                canonical=json.dumps(altered['cards'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode();altered['cards_sha256']=hashlib.sha256(canonical).hexdigest()
                _,gated=generate.compile_catalogue(altered,target)
                self.assertFalse(next(c['implemented'] for c in gated['cards'] if c['name']==card['name']))
        self.assertIn('fn entry_trigger_suppression',source)

    def test_fixed_damage_subjects_and_gain_sequence_keep_exact_targets_and_bounds(self):
        subjects={'any target':'Any','target creature':'Creature','target creature or planeswalker':'CreaturePlaneswalker','target opponent':'Opponent','target player':'Player','target player or planeswalker':'PlayerPlaneswalker','target tapped creature':'CreatureTapped','target attacking or blocking creature':'CreatureAttackingBlocking'}
        for subject,rule in subjects.items():
            for suffix,program in [('',[]),(' and you gain 2 life',['GainLife(PlayerSet::Controller, 2)'])]:
                self.assertEqual(generate.targeted_program('this spell deals 3 damage to '+subject+suffix+'.'),(rule,['Damage(3)']+program))
        self.assertIn('TargetRule::Union',generate.TARGET_RULES['PlayerPlaneswalker']);self.assertIn('kinds: 64',generate.TARGET_RULES['PlayerPlaneswalker']);self.assertIn('TargetRule::Player',generate.TARGET_RULES['PlayerPlaneswalker'])
        self.assertEqual(generate.targeted_program('this spell deals 32767 damage to any target and you gain 65535 life.'),('Any',['Damage(32767)','GainLife(PlayerSet::Controller, 65535)']))
        for body in ['this spell deals 32768 damage to any target.','this spell deals 3 damage to any target and you gain 65536 life.','this spell deals -1 damage to any target.','this spell deals 3 damage to target untapped creature.','this spell deals 3 damage to target creature with flying.','this spell deals 3 damage to any target and that player gains 2 life.','this spell deals 3 damage to any target and gain 2 life.']:
            self.assertIsNone(generate.targeted_program(body),body)

    def test_fixed_effect_tails_preserve_existing_damage_and_private_effect_order(self):
        self.assertEqual(generate.targeted_program('this spell deals 1 damage to any target. Draw a card.'),('Any',['Damage(1)','Draw(PlayerSet::Controller, 1)']))
        self.assertEqual(generate.targeted_program('this spell deals 3 damage to target attacking or blocking creature. You gain 1 life.'),('CreatureAttackingBlocking',['Damage(3)','GainLife(PlayerSet::Controller, 1)']))
        self.assertEqual(generate.targeted_program('this spell deals 3 damage to any target and you gain 2 life. Draw a card.'),('Any',['Damage(3)','GainLife(PlayerSet::Controller, 2)','Draw(PlayerSet::Controller, 1)']))
        self.assertEqual(generate.statement_program('Discard a card, then draw two cards.'),['Discard(PlayerSet::Controller, 1)','Draw(PlayerSet::Controller, 2)'])
        self.assertEqual(generate.statement_program('Draw two cards. Each player loses 2 life.'),['Draw(PlayerSet::Controller, 2)','LoseLife(PlayerSet::Each, 2)'])
        self.assertEqual(generate.statement_program('this creature deals 1 damage to you. Create a Clue token.')[0],'DamagePlayers(PlayerSet::Controller, 1)')
        self.assertEqual(generate.statement_program('this creature deals 2 damage to you and you draw a card.'),['DamagePlayers(PlayerSet::Controller, 2)','Draw(PlayerSet::Controller, 1)'])
        for body in ['Discard a card, then draw five cards.','Discard a card, then you draw two cards.','Each player loses 65536 life.','this creature deals 32768 damage to you.','this creature deals 1 damage to you. Discover 3.']:
            self.assertIsNone(generate.statement_program(body),body)
        self.assertIsNone(generate.targeted_program('this spell deals 3 damage to any target and you gain 2 life. Then discover 3.'))

    def test_fixed_effect_admissions_cover_twelve_whole_bodies_without_old_damage_regressions(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/fixed-effect-admissions.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),12)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(card['oracle_text'],row['oracle_text']);bad=dict(card,oracle_text=card['oracle_text']+'\nWhenever you attack, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template,generate.static_permanent_template,generate.spell_program_template,generate.modal_spell_template]:self.assertIsNone(compiler(bad),row['name'])
        for name in ['Playful Shove','Tweeze','Take Out the Trash','Unsubtle Mockery','Broadside Barrage','Incinerating Blast','Vibrant Outburst',"Winter's Intervention",'Iron-Fist Pulverizer']:self.assertIn(name,supported)
        for name in ['Boulder Dash','Twin Bolt','Strategic Betrayal','Vision of Love']:self.assertNotIn(name,supported)

    def test_filtered_entry_conditions_preserve_self_other_nontoken_kind_and_subtype(self):
        cases=[('another nontoken creature',1,True,None,False),('another nontoken Bird',0,True,'Bird',False),('this creature or another nontoken Cat',0,True,'Cat',True),('this creature or another Mutant',0,False,'Mutant',True),('this creature or another artifact',16,False,None,True)]
        for subject,kinds,nontoken,subtype,include in cases:
            result=generate.trigger_creature_template(self.card(oracle_text='Whenever '+subject+' you control enters, draw a card.'));self.assertIsNotNone(result)
            condition,program,targets=result[2][0];self.assertIn('PermanentEntersYou {',condition);self.assertIn('kinds: '+str(kinds),condition);self.assertIn('nontoken: '+str(nontoken).lower(),condition);self.assertIn('include_source: '+str(include).lower(),condition);self.assertIn('subtype: '+('Some("'+subtype+'")' if subtype else 'None'),condition);self.assertEqual(program,['Draw(PlayerSet::Controller, 1)']);self.assertEqual(targets,'None')
        old=generate.trigger_creature_template(self.card(oracle_text='Whenever this creature or another creature you control enters, draw a card.'));self.assertEqual(old[2][0][0],'CreatureEntersYou')
        guarded=generate.trigger_creature_template(self.card(oracle_text='Whenever another nontoken Bird you control enters, if you gained life this turn, draw a card. This ability triggers only once each turn.'));self.assertIsNotNone(guarded);self.assertIn('OncePerTurn',guarded[2][0][0]);self.assertIn('InterveningIf',guarded[2][0][0]);self.assertIn('PermanentEntersYou',guarded[2][0][0])

    def test_filtered_entry_unknown_predicates_multiplicity_and_pronouns_remain_gated(self):
        for subject in ['another nontoken Wizard with flying','one or more nontoken Birds','another nontoken Bird or Dragon','another nontoken noncreature artifact','another nontoken Robot Villain','another nontoken Unknown','another creature with power 2 or less']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Whenever '+subject+' you control enters, draw a card.')),subject)
        for effect in ['put a +1/+1 counter on it.','it gets +1/+1 until end of turn.','discover 3.','draw a card. Then discover 3.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Whenever another nontoken Bird you control enters, '+effect)),effect)
        good=self.card(name='Arni, Humble Scribe',type_line='Legendary Creature — Human Wizard',oracle_text='Whenever another nontoken creature you control enters, untap Arni.\n{T}: Draw a card, then discard a card.')
        self.assertIsNotNone(generate.permanent_program_template(good))
        for wrong in ['Another Arni','it','the entering creature']:
            self.assertIsNone(generate.permanent_program_template(dict(good,oracle_text=good['oracle_text'].replace('untap Arni.','untap '+wrong+'.'))))
        self.assertIsNone(generate.permanent_program_template(dict(good,type_line='Creature — Human Wizard')))

    def test_filtered_entry_admissions_are_eight_complete_bodies_and_reject_unknown_residuals(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/filtered-entry-admissions.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),8)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(card['oracle_text'],row['oracle_text']);bad=dict(card,oracle_text=card['oracle_text']+'\nWhenever you attack, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template,generate.static_permanent_template]:self.assertIsNone(compiler(bad),row['name'])
        for name in ['Ultron, Artificial Malevolence','Harvestrite Host','Giott, King of the Dwarves']:
            self.assertNotIn(name,supported)

    def test_alliance_is_cosmetic_and_preserves_exact_full_event_and_program(self):
        text='Whenever another creature you control enters, put a +1/+1 counter on this creature.'
        expected=generate.trigger_creature_template(self.card(oracle_text=text))
        self.assertEqual(generate.trigger_creature_template(self.card(oracle_text='Alliance — '+text)),expected)
        self.assertEqual(expected[2][0][0],'OtherCreatureEntersYou')
        for prefix in ['Alliance - ','Alliance: ','Alliances — ','Unknown — ']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=prefix+text)))
        for body in ['Whenever another creature an opponent controls enters, draw a card.','Whenever another creature you control enters, discover 3.','Whenever one or more other creatures you control enter, draw a card.','Whenever another creature you control enters, put a +1/+1 counter on it.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Alliance — '+body)),body)
        # Cosmetic labels cannot rescue unsupported event subjects or effects.
        self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Alliance — Whenever you unlock a Room, draw a card.')))

    def test_alliance_admissions_are_six_complete_bodies_with_no_unknown_residual(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/alliance-admissions.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),6)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(card['oracle_text'],row['oracle_text'])
            plain=dict(card,oracle_text=card['oracle_text'].replace('Alliance — ',''))
            self.assertEqual(generate.trigger_creature_template(card),generate.trigger_creature_template(plain))
            bad=dict(card,oracle_text=card['oracle_text']+'\nWhenever you attack, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template,generate.static_permanent_template]:self.assertIsNone(compiler(bad),row['name'])
        for name in ['Harvestrite Host']:
            self.assertNotIn(name,supported)

    def test_modal_trigger_metadata_preserves_every_mode_event_and_target_plan(self):
        card=self.card(oracle_text='Flying\nWhen this creature enters, choose one —\n• Tap target creature an opponent controls.\n• Draw a card.')
        result=generate.trigger_creature_template(card);self.assertIsNotNone(result);self.assertEqual(result[1],1);condition,program,target=result[2][0];self.assertEqual((condition,target),('SelfEnters','None'));self.assertEqual(len(program),1);self.assertTrue(program[0].startswith('Modal(&['));self.assertIn('TargetRule::Object',program[0]);self.assertIn('relation: 1',program[0]);self.assertIn('Instruction::Draw(PlayerSet::Controller, 1)',program[0])
        card=self.card(oracle_text='Morbid — At the beginning of your end step, if a creature died this turn, choose one —\n• You gain 2 life.\n• Draw a card.')
        result=generate.trigger_creature_template(card);self.assertIsNotNone(result);self.assertIn('CreatureDied',result[2][0][0])
        card=self.card(oracle_text='When this creature enters, choose one —\n• Tap target creature.\n• Untap target creature.\nWhenever this creature attacks, draw a card.')
        result=generate.trigger_creature_template(card);self.assertEqual(len(result[2]),2);self.assertEqual(result[2][1],('SelfAttacks',['Draw(PlayerSet::Controller, 1)'],'None'))

    def test_modal_trigger_complete_bullet_scope_bounds_and_unknown_modes_gate_the_body(self):
        header='When this creature enters, choose one —\n'
        for count in [2,64]:self.assertIsNotNone(generate.trigger_creature_template(self.card(oracle_text=header+'\n'.join(['• Draw a card.']*count))))
        for count in [0,1,65]:self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=header+'\n'.join(['• Draw a card.']*count))))
        for body in [header+'• Draw a card.\n• Discover 3.',header+'• Draw a card.\nDraw a card.',header+'• Draw a card.\n• Untap target creature.\nWhenever you attack, discover 3.',header.replace('choose one —','choose one or both —')+'• Draw a card.\n• You gain 2 life.',header.replace('choose one —','choose up to one —')+'• Draw a card.\n• You gain 2 life.',header.replace('When this creature enters, ','Whenever you commit a crime, ')+'• Draw a card.\n• You gain 2 life.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=body)),body)
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='{2}: Choose one —\n• Draw a card.\n• You gain 2 life.')))

    def test_modal_trigger_admissions_require_all_twelve_complete_bodies_and_each_residual(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());manifest=json.loads((generate.ROOT/'docs/modal-trigger-admissions.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),12)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(card['oracle_text'],row['oracle_text']);bad=dict(card,oracle_text=card['oracle_text']+'\nWhenever you attack, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template,generate.static_permanent_template]:self.assertIsNone(compiler(bad),row['name'])
        for name in ['Archive Arbiter','Flash Thompson, Spider-Fan','Monument to Endurance','Three Bowls of Porridge']:
            self.assertNotIn(name,supported)

    def test_targeted_object_actions_keep_exact_subject_and_optional_cardinality(self):
        subjects={'target creature an opponent controls':'CreatureOpponent','target creature you control':'CreatureYou','other target creature':'OtherCreature','other target permanent you control':'OtherPermanentYou','target tapped creature':'CreatureTapped','target creature or Vehicle':'CreatureVehicle','target nonland permanent':'Nonland','target artifact or creature':'ArtifactCreature','target land':'Land'}
        for subject,rule in subjects.items():
            for optional in ['', 'up to one ']:
                spec=(rule,0,1) if optional else rule
                self.assertEqual(generate.targeted_program('Return '+optional+subject+" to its owner's hand."),(spec,['ReturnToHand']))
                for verb in ['Tap','Untap']:self.assertEqual(generate.targeted_program(verb+' '+optional+subject+'.'),(spec,[verb]))
        for text in ["Return target creature an opponent controls to your hand.","Return up to two target tapped creatures to their owners' hands.",'Tap target untapped creature.','Tap target creature you do not control.','Untap target permanent with mana value 3 or less.']:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_targeted_stun_programs_preserve_tail_focus_and_unknown_residual_gates(self):
        for word,amount in [('a',1),('one',1),('two',2),('three',3)]:
            line='Tap target creature an opponent controls and put '+word+' stun counters on it.';base=['Tap',generate.counter_instruction('Targets',word,'stun')];self.assertEqual(generate.targeted_program(line),('CreatureOpponent',base));self.assertEqual(generate.targeted_program(line+' Scry 2.'),('CreatureOpponent',base+['Scry(2)']))
        for line in ['Tap target creature an opponent controls and put four stun counters on it.','Tap target creature an opponent controls and put a stun counter on each of them.','Tap target creature an opponent controls. Put a stun counter on it. Then discover 3.',"Return up to one target nonland permanent to its owner's hand. If it was a token, draw a card."]:
            self.assertIsNone(generate.targeted_program(line),line)
        self.assertEqual(generate.targeted_program('Tap target artifact or creature. Draw a card.'),('ArtifactCreature',['Tap','Draw(PlayerSet::Controller, 1)']))

    def test_targeted_bounce_tap_admits_eighteen_complete_bodies_and_rejects_unknown_tails(self):
        import json
        manifest=json.loads((generate.ROOT/'docs/targeted-bounce-tap-admissions.json').read_text());rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};self.assertEqual(len(manifest['cards']),18)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(row['oracle_text'],card['oracle_text']);bad=dict(card,oracle_text=card['oracle_text']+'\nWhenever you attack, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template,generate.static_permanent_template]:self.assertIsNone(compiler(bad),row['name'])
        for name in ['Brackish Blunder','Hotshot Investigators','Mirkwood Nurturer','Nobody','Sunpearl Kirin','Floodpits Drowner','Snaremaster Sprite']:
            self.assertNotIn(name,supported)

    def test_source_public_predicates_are_exact_and_reject_untracked_history(self):
        expected={"it's attacking":'SourceAttacking',"it's modified":'SourceModified',"you haven't cast a spell this turn":'Not(&StatValue::SpellsCastThisTurn)',"you've drawn two or more cards this turn":'CardsDrawnThisTurn','you control a token':'Tokens(',"there's a Lesson card in your graveyard":'subtype: Some("Lesson")','there are two or more instant and/or sorcery cards in your graveyard':'kind: 12','you have 25 or more life':'ControllerLife'}
        for clause,value in expected.items():
            self.assertIn(value,generate.stat_threshold_condition(clause),clause)
            result=generate.static_permanent_template(self.card(oracle_text='This creature has flying as long as '+clause+'.'));self.assertIsNotNone(result,clause)
        for clause in ["it's attacking a player","it's been modified","you've drawn two cards this game","you haven't cast a noncreature spell this turn",'you have 65536 or more life',"there's a Lesson in any graveyard",'you control a noncreature token',"it hasn't dealt damage","you committed a crime this turn"]:
            self.assertIsNone(generate.stat_threshold_condition(clause),clause)
        for clause in expected:
            self.assertIsNone(generate.static_permanent_template(self.card(oracle_text='This creature has flash as long as '+clause+'.')))

    def test_conditional_source_block_flags_and_additional_bonus_preserve_exact_alias_scope(self):
        for restriction,mask in [("can't be blocked",16384),("can't be blocked by more than one creature",65536)]:
            line='As long as there are eight or more permanent cards in your graveyard, Test gets an additional +2/+2 and '+restriction+'.'
            card=self.card(name='Test, Legend',type_line='Legendary Creature — Fungus',oracle_text=line)
            proxy,_,scales=generate.extract_stat_rules(card);self.assertEqual(len(scales),1);self.assertIn('power: 2, toughness: 2',scales[0]);result=generate.static_permanent_template(proxy);self.assertIsNotNone(result);self.assertIn('keywords: Keywords('+str(mask)+')',result[6][0])
            for prefix in ['', 'During your turn, ']:
                card=self.card(oracle_text=prefix+'This creature '+restriction+' as long as you control an artifact as long as you gained life this turn.');result=generate.static_permanent_template(card);self.assertIsNotNone(result);self.assertIn('requires_life_gain: true',result[6][0]);self.assertIn('own_turn: '+str(bool(prefix)).lower(),result[6][0])
        for line in ["This creature can't be blocked by more than two creatures as long as you control an artifact.","Creatures you control can't be blocked as long as you control an artifact.","This creature can't be blocked as long as you control an artifact. Then discover 3.","As long as you control an artifact, this creature gets +1/+1 and can't be blocked by creatures with flying."]:
            proxy,_,_=generate.extract_stat_rules(self.card(oracle_text=line));self.assertIsNone(generate.static_permanent_template(proxy),line)

    def test_source_predicates_admit_thirteen_entire_bodies_and_keep_all_unknown_residuals_gated(self):
        import json
        manifest=json.loads((generate.ROOT/'docs/source-state-predicate-admissions.json').read_text());rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        self.assertEqual(len(manifest['cards']),13)
        for row in manifest['cards']:
            self.assertIn(row['name'],supported);card=next(c for c in rows['cards'] if c['name']==row['name']);self.assertEqual(row['oracle_text'],card['oracle_text']);proxy,_,_=generate.extract_stat_rules(dict(card,oracle_text=card['oracle_text']+'\nWhenever you attack, discover 3.'))
            for compiler in [generate.static_permanent_template,generate.trigger_creature_template,generate.permanent_program_template]:self.assertIsNone(compiler(proxy),row['name'])
        for name in ['Ruric Thar, Magecrusher','Karakyk Guardian','Colossal Rattlewurm','Undercover Skrull']:
            self.assertNotIn(name,supported)

    def test_conditional_source_count_composition_preserves_life_gain_and_own_turn_requirements(self):
        for prefix,own_turn in [('',False),('During your turn, ',True)]:
            for line in ['As long as you control an artifact, this creature has flying as long as you gained life this turn.','This creature has flying as long as you control an artifact as long as you gained life this turn.']:
                result=generate.static_permanent_template(self.card(oracle_text=prefix+line));self.assertIsNotNone(result);grant=result[6][0];self.assertIn('condition: Some(StatValue::AtLeast',grant);self.assertIn('requires_life_gain: true',grant);self.assertIn('own_turn: '+str(own_turn).lower(),grant)
        for line in ['As long as you control an artifact, this creature has flying as long as an opponent gained life this turn.','As long as you control an artifact, this creature has flying as long as you gained life last turn.','As long as you control an artifact, this creature has flying as long as you gained life this turn. Then discover 3.']:
            self.assertIsNone(generate.static_permanent_template(self.card(oracle_text=line)))

    def test_conditional_source_keywords_use_exact_count_predicates_and_reject_out_of_zone_flash(self):
        for text in ['As long as you control an artifact, this creature has haste.','This creature has flying as long as you control a Dragon.','This creature has lifelink as long as you control another Faerie.','Descend 4 — This creature has flying as long as there are four or more permanent cards in your graveyard.']:
            result=generate.static_permanent_template(self.card(oracle_text=text));self.assertIsNotNone(result,text);self.assertEqual(len(result[6]),1);self.assertIn('condition: Some(StatValue::AtLeast',result[6][0]);self.assertIn('source_only: true',result[6][0])
        for text in ['This creature has flash as long as you control a Desert.','This creature has changeling as long as you control an artifact.','Creatures you control have flying as long as you control an artifact.','This creature has flying as long as you control a creature with flying.','This creature has flying as long as it is attacking.','This creature has flying as long as you control an artifact. Then discover 3.']:
            self.assertIsNone(generate.static_permanent_template(self.card(oracle_text=text)),text)

    def test_combined_source_count_stats_leave_exact_keyword_clause_for_whole_body_compilation(self):
        for line in ['As long as you control an artifact, this creature gets +1/+0 and has haste.','This creature gets +1/+0 and has haste as long as you control an artifact.']:
            proxy,_,scales=generate.extract_stat_rules(self.card(oracle_text=line));self.assertEqual(len(scales),1);self.assertEqual(proxy['oracle_text'],'This creature has haste as long as you control an artifact.');result=generate.static_permanent_template(proxy);self.assertIsNotNone(result);self.assertIn('keywords: Keywords(4)',result[6][0])
        for line in ['As long as you control an artifact, this creature gets +32768/+0 and has haste.','As long as you control an artifact, this creature gets +1/+0 and has flash.','As long as you control an artifact, this creature gets +1/+0 and has trample and discover 3.']:
            proxy,_,scales=generate.extract_stat_rules(self.card(oracle_text=line));self.assertEqual(scales,[]);self.assertIsNone(generate.static_permanent_template(proxy))

    def test_conditional_source_keyword_admissions_preserve_all_nine_residual_bodies_and_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Barrow Naughty','Didact Echo','Dreadwing Scavenger','Echo of Dusk','Goblin Tomb Raider','Gravblade Heavy','Kargan Dragonrider','Spineseeker Centipede','Wildfire Wickerfolk']:
            self.assertIn(name,supported);c=next(c for c in rows['cards'] if c['name']==name);proxy,_,_=generate.extract_stat_rules(dict(c,oracle_text=c['oracle_text']+'\nWhenever you attack, discover 3.'));self.assertIsNone(generate.static_permanent_template(proxy));self.assertIsNone(generate.trigger_creature_template(proxy));self.assertIsNone(generate.permanent_program_template(proxy))
        for name in ["Bolg's Company",'Colossal Rattlewurm','Undercover Skrull']:
            self.assertNotIn(name,supported)

    def test_source_stat_threshold_counts_have_exact_boolean_bounds_and_scopes(self):
        for condition,minimum in [('you control an artifact',1),('you control another creature',1),('you control seven or more lands',7),('there are four or more permanent cards in your graveyard',4),('there are seven or more cards in your graveyard',7),("you've cast two or more spells this turn",2),('you control 65535 or more lands',65535)]:
            value=generate.stat_threshold_condition(condition);self.assertIn('StatValue::AtLeast',value);self.assertIn('minimum: '+str(minimum),value)
        for condition in ['you control 65536 or more lands','you control a creature with power 4 or greater',"you control artifacts and it's your turn",'you control a noncreature token',"there are seven or more cards in an opponent's graveyard",'you control one or fewer cards',"you've cast two spells this game"]:
            self.assertIsNone(generate.stat_threshold_condition(condition),condition)

    def test_source_stat_threshold_extraction_retains_entire_residual_and_bounded_signed_bonus(self):
        for line in ['As long as you control an artifact, this creature gets +1/+0.','This creature gets +1/+0 as long as you control an artifact.']:
            proxy,base,scales=generate.extract_stat_rules(self.card(oracle_text='Flying\n'+line));self.assertEqual(proxy['oracle_text'],'Flying');self.assertIsNone(base);self.assertEqual(len(scales),1);self.assertIn('minimum: 1',scales[0]);self.assertIn('power: 1, toughness: 0',scales[0])
        proxy,_,scales=generate.extract_stat_rules(self.card(oracle_text='Threshold — This creature gets -32767/+32767 as long as there are seven or more cards in your graveyard.'));self.assertEqual(len(scales),1)
        for text in ['This creature gets +32768/+0 as long as you control an artifact.','Enchanted creature gets +1/+0 as long as you control an artifact.','This creature gets +1/+0 as long as you control an artifact. Then discover 3.','As long as this creature is attacking, this creature gets +1/+0.']:
            proxy,_,scales=generate.extract_stat_rules(self.card(oracle_text=text));self.assertEqual(scales,[]);self.assertEqual(proxy['oracle_text'],text)
        proxy,_,scales=generate.extract_stat_rules(self.card(oracle_text='This creature gets +1/+0 as long as you control an artifact.\nThis creature gets +1/+0 as long as you control an artifact.\nWhenever you attack, discover 3.'));self.assertEqual(len(scales),2);self.assertIn('discover 3',proxy['oracle_text'])

    def test_source_stat_threshold_admits_nine_full_bodies_and_keeps_unknown_keyword_history_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        names=['Basking Capybara','Billowing Shriekmass','Brightspear Zealot','Cloudsculpt Technician','Frilled Cave-Wurm','Gaelicat','Gigantoad','Mind Drill Assailant','Scorpion Sentinel']
        for name in names:
            self.assertIn(name,supported);c=next(c for c in rows['cards'] if c['name']==name);proxy,_,_=generate.extract_stat_rules(dict(c,oracle_text=c['oracle_text']+'\nWhenever you attack, discover 3.'));self.assertIsNone(generate.permanent_program_template(proxy));self.assertIsNone(generate.trigger_creature_template(proxy))
        for name in ['Armory Mice','Colossal Rattlewurm']:
            self.assertNotIn(name,supported)

    def test_firebending_exact_triggers_preserve_multiplicity_and_scaled_count(self):
        for n in [0,1,2,65535]:
            result=generate.trigger_creature_template(self.card(oracle_text='Firebending '+str(n)))
            self.assertEqual(result[2],[('SelfAttacks',[f'AddCombatRed({n})'],'None')])
        result=generate.trigger_creature_template(self.card(oracle_text='Firebending 1\nFirebending 2'))
        self.assertEqual(len(result[2]),2)
        scaled=generate.trigger_creature_template(self.card(oracle_text='Firebending X, where X is the number of creatures you control.'))
        self.assertEqual(scaled[2][0][1],['AddCombatRedScaled('+generate.amount_value('the number of creatures you control')+')'])
        for text in ['Firebending 65536','Firebending -1',"Firebending X, where X is this creature's power.",'Firebending 2. Draw a card.','Firebending 2 This ability triggers only once each turn.','Firebending X, where X is the number of creatures an opponent controls.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_lesson_reduction_and_life_predefined_token_composition_are_exact(self):
        self.assertEqual(generate.reduction_filter('Lesson'),'CastFilter::Subtype("Lesson")')
        for token in ['Clue','Treasure']:
            self.assertEqual(generate.statement_program('You lose 1 life and create a '+token+' token.'),['LoseLife(PlayerSet::Controller, 1)',*generate.predefined_program('create a '+token+' token.')])
        for text in ['You lose 1 life and create a Map token.','You lose 1 life and create a Treasure token. Then discover 3.','You lose 1 life and create a token copy of target creature.']:
            self.assertIsNone(generate.statement_program(text))
        self.assertIsNone(generate.reduction_filter('non-Lesson'))

    def test_firebending_migration_admits_eight_complete_bodies_and_gates_power_copy_flashback(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Azula, On the Hunt','Fire Sages','Namazu Trader','Rough Rhino Cavalry','Sun Warriors','Uncle Iroh','Vindictive Warden','Zhao, Ruthless Admiral']:
            self.assertIn(name,supported);c=next(c for c in rows['cards'] if c['name']==name);bad=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template]:self.assertIsNone(compiler(bad),name)
        for name in ['Firebending Student','Fire Lord Azula','Fire Lord Zuko','Iroh, Grand Lotus','Zuko, Exiled Prince','Tundra Tank','Jeong Jeong, the Deserter']:
            self.assertNotIn(name,supported)

    def test_poison_effects_bind_only_explicit_target_or_damage_recipient(self):
        self.assertEqual(generate.targeted_program('Target opponent gets two poison counters.'),('Opponent',['AddPoison(PlayerSet::Targeted, 2)']))
        self.assertEqual(generate.damage_event_program('that player gets two poison counters.'),('None',['AddPoison(PlayerSet::DamagedPlayer, 2)']))
        for text in ['That player gets two poison counters.','Target player gets two poison counters.','Target opponent gets three poison counters.','Target opponent gets two poison counters unless they pay {1}.','Target opponent gets two poison counters. Then discover 3.']:
            self.assertIsNone(generate.targeted_program(text),text)
        self.assertIsNone(generate.statement_program('That player gets two poison counters.'))
        self.assertEqual(generate.nonmana_ward_program('Ward—Get five poison counters.'),['MayOpponent { skip: 2 }','AddPoison(PlayerSet::Opponent, 5)','Skip(1)','CounterRelated'])
        for text in ['Ward—Get five poison counters and pay {1}.','Ward—Get four poison counters.','Ward—Get five poison counters. Draw a card.']:
            self.assertIsNone(generate.nonmana_ward_program(text))

    def test_poison_damage_and_death_filters_are_exact_and_not_aggregated(self):
        line='Whenever a nontoken artifact creature you control deals combat damage to a player, that player gets two poison counters.'
        result=generate.trigger_creature_template(self.card(oracle_text=line));self.assertIsNotNone(result);condition=result[2][0][0];self.assertIn('tokens: Some(false)',condition);self.assertIn('artifact: true',condition);self.assertIn('combat_only: true',condition);self.assertIn('one_or_more: false',condition)
        death=generate.trigger_creature_template(self.card(oracle_text='Whenever another creature you control with deathtouch dies, each opponent sacrifices a nontoken creature of their choice.'))
        self.assertIsNotNone(death);self.assertTrue(death[2][0][0].startswith('CreatureDiesFiltered {'));self.assertIn('keyword: Keywords(128)',death[2][0][0]);self.assertIn('other: true',death[2][0][0])
        for text in ['Whenever one or more nontoken artifact creatures you control deal combat damage to a player, that player gets three poison counters.', 'Whenever another creature you control with toxic dies, draw a card.', 'Whenever a nontoken artifact creature you control deals damage to a player, that player gets two poison counters and you draw a card.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)))

    def test_poison_admits_all_four_whole_bodies_with_residual_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Fynn, the Fangbearer','Persuasive Interrogators','The Serpent Society','Virulent Silencer']:
            self.assertIn(name,supported);c=next(c for c in rows['cards'] if c['name']==name);bad=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template]:self.assertIsNone(compiler(bad),name)

    def test_draw_and_token_composition_keeps_entire_sentence_and_order(self):
        for word,n in [('a card',1),('two cards',2),('three cards',3),('four cards',4)]:
            result=generate.statement_program('Draw '+word+' and create a Treasure token.')
            self.assertEqual(result,[f'Draw(PlayerSet::Controller, {n})',*generate.predefined_program('create a Treasure token.')])
        robot='create a 2/2 colorless Robot artifact creature token.'
        self.assertEqual(generate.statement_program('Draw two cards and '+robot),['Draw(PlayerSet::Controller, 2)',*generate.token_program(robot)])
        for text in ['Draw a card and create a Map token.', 'Draw a card and create a Treasure token. It becomes a creature.', 'Draw a card and create a Treasure token unless an opponent pays {1}.', 'Draw a card and create a token copy of target creature.', 'Draw a card or create a Treasure token.', 'Draw five cards and create a Treasure token.']:
            self.assertIsNone(generate.statement_program(text),text)
        optional=generate.optional_payment_program('you may sacrifice another creature or artifact. If you do, draw a card and create a Treasure token.',False)
        self.assertIn('optional_skip: Some(2)',optional[0]);self.assertEqual(optional[1:],[*generate.statement_program('Draw a card and create a Treasure token.')])

    def test_permanent_lines_normalize_reminder_whitespace_without_swallowing_residuals(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())['cards']
        for name in ['Lunatic Pandora','Popular Egotist']:
            c=next(c for c in rows if c['name']==name);normal=generate.permanent_program_template(c);self.assertIsNotNone(normal)
            variant=dict(c,oracle_text=' \n'.join('  '+line+'  ' for line in c['oracle_text'].split('\n')))
            self.assertEqual(generate.permanent_program_template(variant),normal)
            self.assertIsNone(generate.permanent_program_template(dict(c,oracle_text=c['oracle_text']+'\nDiscover 3.')))
        c=self.card(name='The Exact Names',type_line='Legendary Creature — Human',oracle_text='When The Exact Names enter, draw a card.')
        self.assertEqual(generate.trigger_creature_template(c)[2][0][0],'SelfEnters')
        self.assertIsNone(generate.trigger_creature_template(dict(c,oracle_text='When Other Names enter, draw a card.')))

    def test_draw_token_body_migration_admits_exact_complete_seven_cards(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Biomechan Engineer','Lunatic Pandora','Popular Egotist','Reckless Lackey','Redrock Sentinel','Seize the Spoils','The Sackville-Bagginses']:
            self.assertIn(name,supported)
            c=next(c for c in rows['cards'] if c['name']==name);bad=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template,generate.spell_program_template]:self.assertIsNone(compiler(bad),name)
        for name in ['Fanatical Offering','Dyadrine, Synthesis Amalgam','Esoteric Duplicator','Camellia, the Seedmiser']:
            self.assertNotIn(name,supported)

    def test_sacrifice_triggers_compile_exact_filters_and_triggered_mana(self):
        for phrase,kinds,other,subtype,turn in [('an artifact',16,False,None,False),('a Food',16,False,'Food',False),('another creature or artifact',17,True,None,False),('another permanent',0,True,None,False),('a permanent during your turn',0,False,None,True),('a token',0,False,None,False)]:
            result=generate.trigger_creature_template(self.card(oracle_text='Whenever you sacrifice '+phrase+', draw a card.'))
            self.assertIsNotNone(result);condition=result[2][0][0]
            self.assertTrue(condition.startswith('Sacrificed {'));self.assertIn('kinds: '+str(kinds),condition);self.assertIn('other: '+str(other).lower(),condition);self.assertIn('own_turn: '+str(turn).lower(),condition)
            if subtype:self.assertIn('subtype: Some("'+subtype+'")',condition)
        result=generate.trigger_creature_template(self.card(oracle_text='Whenever you sacrifice an artifact, put a +1/+1 counter on this creature and add {R}.'))
        self.assertEqual(result[2][0][1],[generate.counter_instruction('Source','a','+1/+1'),'AddMana(ManaProduction::Fixed([0, 0, 0, 1, 0, 0]))'])
        for text in ['Whenever you sacrifice one or more artifacts, draw a card.', 'Whenever an opponent sacrifices an artifact, draw a card.', 'When you sacrifice a Clue, return this card from your graveyard to your hand.', 'Whenever you sacrifice an artifact, return it to the battlefield.', 'Whenever you sacrifice an artifact, draw cards equal to its power.', 'Whenever you sacrifice an artifact for mana, draw a card.', 'Whenever you sacrifice an artifact, draw a card. Then discover 3.', 'Whenever you sacrifice an artifact, put a +1/+1 counter on this creature and add {R}. This mana lasts until end of combat.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_sacrifice_events_admit_seven_complete_bodies_and_gate_residual_rules(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Biotech Specialist','Crime Novelist','Experimental Confectioner','Gleaming Geardrake','Lightless Evangel','Pirate Peddlers','Tolls of War']:
            self.assertIn(name,supported)
            c=next(c for c in rows['cards'] if c['name']==name);bad=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template]:self.assertIsNone(compiler(bad),name)
        for name in ['Curious Cadaver','Esoteric Duplicator','Magnetic Snuffler','Camellia, the Seedmiser']:
            self.assertNotIn(name,supported)

    def test_permanent_departures_compile_exact_sources_and_kind_scoped_exclusions(self):
        for noun in ['artifact','enchantment','Aura','Equipment']:
            for clause,condition in [('is put into a graveyard from the battlefield','SelfDies'),('enters or is put into a graveyard from the battlefield','SelfEntersOrDies')]:
                result=generate.trigger_creature_template(self.card(oracle_text=f'When this {noun} {clause}, draw a card.'))
                self.assertIsNotNone(result);self.assertEqual(result[2][0][0],condition)
        for text,kinds,relation,excluded in [('an enchantment you control',32,'You',0),('an artifact',16,'Any',0),('another artifact you control',16,'You',16),('another artifact or creature',17,'Any',17),('another creature you control or a land you control',3,'You',1)]:
            result=generate.trigger_creature_template(self.card(oracle_text=f'Whenever {text} is put into a graveyard from the battlefield, draw a card.'))
            self.assertIsNotNone(result);self.assertEqual(result[2][0][0],f'PermanentToGraveyard {{ kinds: {kinds}, relation: CastRelation::{relation}, exclude_source_kinds: {excluded} }}')
        for text in ['When this artifact is put into a graveyard from anywhere, draw a card.',
                     'Whenever an enchantment you control leaves the battlefield, draw a card.',
                     'Whenever one or more enchantments you control are put into a graveyard from the battlefield, draw a card.',
                     'Whenever a nontoken artifact you control is put into a graveyard from the battlefield, draw a card.',
                     'Whenever an enchantment you control is put into a graveyard from the battlefield, return it to the battlefield.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_permanent_departures_admit_ten_whole_bodies_and_keep_residuals_gated(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ["Ashiok's Reaper",'Hopeful Vigil','Knight of Doves','Long Feng, Grand Secretariat','Mephitic Draught','Nutrient Block','Reach for the Sky','Savior of the Sleeping','Warehouse Tabby','Wicked Visitor']:
            self.assertIn(name,supported)
            c=next(c for c in rows['cards'] if c['name']==name);bad=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            for compiler in [generate.permanent_program_template,generate.trigger_creature_template,generate.aura_template]:self.assertIsNone(compiler(bad),name)
        for name in ['Bitter Chill','Demonic Ruckus','Krenko, Baron of Tin Street',"Tarrian's Soulcleaver",'Teysa, Opulent Oligarch','Ygra, Eater of All','Zoetic Glyph']:
            self.assertNotIn(name,supported)

    def test_temporary_blocking_prohibition_has_exact_target_and_duration(self):
        instruction=['GrantKeywords { subject: Subject::Targets, keywords: Keywords(2048) }']
        for phrase,rule in [('Target creature','Creature'),('Target creature you control','CreatureYou'),
                            ('Another target creature you control','OtherCreatureYou'),('Target creature an opponent controls','CreatureOpponent')]:
            self.assertEqual(generate.targeted_program(phrase+" can't block this turn."),(rule,instruction))
        for text in ["Target creature can't block.","Target creature can't attack this turn.",
                     "Target creature can't block next turn.","Target creature can't block this turn unless its controller pays {1}.",
                     "Target creatures can't block this turn.","Target creature can't block this turn. It becomes a Frog.",
                     "That creature can't block this turn.","Target land can't block this turn."]:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_temporary_blocking_admits_only_complete_six_bodies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Brambleback Brute','Sandstorm Verge','Sower of Chaos','Summit Intimidator','War Squeak','Zog, Triceraton Castaway']:
            self.assertIn(name,supported)
            c=next(c for c in rows['cards'] if c['name']==name);bad=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            for compiler in [generate.permanent_program_template,generate.aura_template,generate.land_template,generate.trigger_creature_template]:
                self.assertIsNone(compiler(bad),name)
        for name in ['Duel Tactics','Shriek, Treblemaker','Ty Lee, Artful Acrobat','Untimely Malfunction']:
            self.assertNotIn(name,supported)

    def test_source_returns_compile_only_exact_activated_zone_bindings(self):
        cases=[('Return this card from your graveyard to your hand.','Graveyard','ReturnSourceToHand(TargetZone::Graveyard)'),
               ('Return this card from your graveyard to the battlefield tapped.','Graveyard','ReturnSourceToBattlefield { tapped: true }'),
               ("Return this creature to its owner's hand.",'Battlefield','ReturnSourceToHand(TargetZone::Battlefield)')]
        for text,origin,instruction in cases:
            self.assertEqual(generate.source_return_program(text),(origin,[instruction]))
            compiled=generate.permanent_program_template(self.card(oracle_text='{2}: '+text));self.assertEqual(compiled[3][0][0]['origin'],origin);self.assertEqual(compiled[3][0][2],[instruction])
            self.assertIsNone(generate.statement_program(text));self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='When this creature dies, '+text[0].lower()+text[1:])))
        named=self.card(name='Arcanis the Omnipotent',type_line='Legendary Creature — Wizard',oracle_text="{2}{U}{U}: Return Arcanis to its owner's hand.")
        self.assertIsNotNone(generate.permanent_program_template(named));self.assertIsNone(generate.permanent_program_template(dict(named,name='Unrelated')))
        self.assertIsNone(generate.source_return_program("Return it to its owner's hand."))
        for text in ['Return this card from exile to your hand.','Return another card from your graveyard to your hand.',
                     'Return this card from your graveyard to the battlefield tapped with a finality counter on it.',
                     'Return this card from your graveyard to your hand. Draw a card.',
                     'Return this card from your graveyard to the battlefield under your control.']:
            self.assertIsNone(generate.source_return_program(text),text)
        for text in ['{T}: Return this card from your graveyard to your hand.',
                     'Sacrifice this creature: Return this card from your graveyard to your hand.',
                     'Remove a +1/+1 counter from this creature: Return this card from your graveyard to your hand.']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=text)),text)

    def test_eleven_source_return_bodies_keep_finality_and_trigger_residuals_gated(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text());_,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']};rows=json.loads((generate.ROOT/'docs/source-return-admissions.json').read_text())['cards'];self.assertEqual(len(rows),11)
        for row in rows:
            c=next(c for c in snapshot['cards'] if c['name']==row['name']);self.assertEqual(c['oracle_text'],row['oracle_text']);self.assertIn(c['name'],supported);changed=dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.');self.assertIsNone(generate.permanent_program_template(changed));self.assertIsNone(generate.aura_template(changed))
        for name in ['Balustrade Wurm','Resurrected Cultist','Uchbenbak, the Great Mistake','Grim Repriser','Proctor of Potential','Dutiful Griffin','Eagle\'s Rescue','Gallia, Tragic Host','Postmortem Professor','Angelic Destiny']:
            self.assertNotIn(name,supported)

    def test_graveyard_self_exile_costs_origins_and_renew_are_exact_and_mixed_costs_gated(self):
        for mana in ['{G}','{3}{U}','{1}{B/G}']:
            cost=generate.activation_cost(mana+', Exile this card from your graveyard');self.assertTrue(cost['exile_self']);self.assertEqual(cost['origin'],'Graveyard');self.assertFalse(cost['sacrifice'])
        for cost in ['Exile another card from your graveyard','Exile this card from a graveyard','Exile this card from your hand',
                     '{T}, Exile this card from your graveyard','Pay 1 life, Exile this card from your graveyard',
                     'Exile this card from your graveyard, Sacrifice a creature','Exile this card from your graveyard, Discard a card',
                     'Exile this card from your graveyard, Exile this card from your graveyard']:
            self.assertIsNone(generate.activation_cost(cost),cost)
        body='Renew — {G}, Exile this card from your graveyard: Put a +1/+1 counter on target creature. Activate only as a sorcery.'
        result=generate.permanent_program_template(self.card(oracle_text=body));self.assertIsNotNone(result);self.assertTrue(result[3][0][0]['exile_self']);self.assertTrue(result[3][0][3])
        for text in [body.replace('Exile this card from your graveyard','Discard a card'),body.replace(' Activate only as a sorcery.',''),body+'\nDiscover 4.',
                     '{1}, Exile this card from your graveyard: Add {G}.']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=text)),text)
        self.assertIn('other: true',generate.TARGET_RULES['OtherCreatureGraveYou']);self.assertIn('relation: 0',generate.TARGET_RULES['OtherCreatureGraveYou'])

    def test_sixteen_complete_graveyard_bodies_preserve_unknown_residual_and_keyword_counters_gates(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        rows=json.loads((generate.ROOT/'docs/graveyard-activation-admissions.json').read_text())['cards'];self.assertEqual(len(rows),16)
        for row in rows:
            c=next(c for c in snapshot['cards'] if c['name']==row['name']);self.assertEqual(c['oracle_text'],row['oracle_text']);self.assertIn(c['name'],supported)
            self.assertIsNone(generate.permanent_program_template(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')))
        for name in ["Alchemist's Assistant",'Champion of Dusan','Kheru Goldkeeper','Qarsi Revenant','Sagu Pummeler','Naga Fleshcrafter','Rot-Curse Rakshasa','Buried Treasure','Glitch Ghost Surveyor','Seasoned Cryomancer','Symbiote Spider-Man','Campus Crier']:
            self.assertNotIn(name,supported)

    def test_uncounterable_extraction_is_exact_and_preserves_whole_residuals(self):
        for reminder in ['', ' (This includes by the ward ability.)']:
            c=self.card(oracle_text="This spell can't be countered."+reminder+'\nFlying')
            rest,flag=generate.extract_uncounterable(c);self.assertTrue(flag);self.assertEqual(rest['oracle_text'],'Flying')
        for text in ["Creature spells you control can't be countered.","This spell can't be countered if you control a Bear.",
                     "This spell can't be countered. Draw a card.","This spell can't be countered.\nThis spell can't be countered."]:
            c=self.card(oracle_text=text);self.assertEqual(generate.extract_uncounterable(c),(c,False))
        c=self.card(oracle_text="This spell can't be countered.\nDiscover 4.");rest,flag=generate.extract_uncounterable(c)
        self.assertTrue(flag);self.assertIsNone(generate.creature_template(rest));self.assertIsNone(generate.permanent_program_template(rest))
        self.assertIn('max_mana: 3',generate.TARGET_RULES['CreaturePlaneswalkerMax3']);self.assertIn('kinds: 65',generate.TARGET_RULES['CreaturePlaneswalkerMax3'])

    def test_four_uncounterable_admissions_keep_unsupported_residual_bodies_gated(self):
        import copy,hashlib,json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        rows=json.loads((generate.ROOT/'docs/uncounterable-admissions.json').read_text())['cards'];self.assertEqual(len(rows),4)
        for row in rows:
            c=next(c for c in snapshot['cards'] if c['name']==row['name']);self.assertEqual(c['oracle_text'],row['oracle_text']);self.assertIn(c['name'],supported)
        for name in ['Inevitable Defeat','Slice from the Shadows','Koma, World-Eater','Frenzied Baloth','Hexing Squelcher']:
            self.assertNotIn(name,supported)
        changed=copy.deepcopy(snapshot)
        for c in changed['cards']:
            if c['name'] in {r['name'] for r in rows}:c['oracle_text']+='\nDiscover 4.'
        changed['cards_sha256']=hashlib.sha256(json.dumps(changed['cards'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        rust,bad=generate.compile_catalogue(changed,target)
        self.assertFalse(any(c['implemented'] for c in bad['cards'] if c['name'] in {r['name'] for r in rows}))
        self.assertEqual(rust.split('fn uncounterable(card: Card) -> bool {')[1].strip(), 'false\n}')

    def test_external_sacrifice_activation_cost_filters_are_exact_and_mana_remains_gated(self):
        for subject,kind,other,token in [('a creature',1,False,'None'),('another creature or artifact',17,True,'None'),
                                       ('an artifact or land',18,False,'None'),('another artifact or land',18,True,'None'),
                                       ('a land',2,False,'None'),('a token',0,False,'Some(true)'),('a Food',16,False,'None')]:
            cost=generate.activation_cost('{2}, {T}, Sacrifice '+subject)
            self.assertTrue(cost['tap']);self.assertFalse(cost['sacrifice']);self.assertIn('EffectChoiceKind::Sacrifice',cost['additional'])
            self.assertIn(f'kinds: {kind}',cost['additional']);self.assertIn('other: '+str(other).lower(),cost['additional']);self.assertIn('tokens: '+token,cost['additional'])
        for text in ['Sacrifice two creatures','Sacrifice thirteen creatures','Sacrifice a creature or token','Sacrifice a suspected creature',
                     'Sacrifice another creature, Sacrifice this creature','Sacrifice this creature, Sacrifice another creature',
                     'Sacrifice a creature, Pay 1 life','Sacrifice a creature, Remove a counter from this creature','Sacrifice a creature, Discard a card']:
            self.assertIsNone(generate.activation_cost(text),text)
        self.assertTrue(generate.activation_cost('Sacrifice this creature')['sacrifice'])
        self.assertIsNone(generate.activation_cost('Sacrifice this creature')['additional'])
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='Sacrifice another creature: Add {G}.')))
        self.assertIsNotNone(generate.permanent_program_template(self.card(oracle_text='{T}, Sacrifice a creature: Draw a card.')))

    def test_nineteen_external_sacrifice_bodies_match_pinned_text_and_reject_residuals(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        rows=json.loads((generate.ROOT/'docs/activation-sacrifice-admissions.json').read_text())['cards'];self.assertEqual(len(rows),19)
        for a in rows:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            c=dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')
            self.assertIsNone(generate.permanent_program_template(c));self.assertIsNone(generate.land_template(c))
        for name in ['Ice Cream Kitty','Agency Coroner','Intruding Soulrager',"Bolg's Company",'Jolene, Plundering Pugilist','Technodrome','Memorial Vault','Quina, Qu Gourmet','Diversion Specialist','Sidisi, Regent of the Mire','Metalhead','Alquist Proft, Master Sleuth','Boneyard Desecrator','Rune-Brand Juggler']:
            self.assertNotIn(name,supported)

    def test_creature_tapped_entry_extraction_is_exact_and_retains_counter_clause(self):
        cases=[('This creature enters tapped.','Always',''),("This creature enters tapped if it's not your turn.",'OpponentsTurn',''),
               ('This creature enters tapped with three stun counters on it. (Reminder.)','Always','This creature enters with three stun counters on it.')]
        for text,rule,remaining in cases:
            c,got=generate.extract_creature_entry_tapped(self.card(oracle_text=text));self.assertEqual(got,rule);self.assertEqual(c['oracle_text'],remaining)
        for text in ['This creature enters tapped unless you control an Island.','This creature enters tapped with X stun counters on it.',
                     'Target creature enters tapped.','This creature enters tapped. Draw a card.',
                     'This creature enters tapped.\nThis creature enters tapped.',
                     "This creature enters tapped.\nThis creature enters tapped if it's not your turn."]:
            c=self.card(oracle_text=text);self.assertEqual(generate.extract_creature_entry_tapped(c),(c,'Never'),text)
        for changes in [dict(type_line='Sorcery'),dict(layout='transform'),dict(type_line='Artifact')]:
            c=self.card(oracle_text='This creature enters tapped.',**changes);self.assertEqual(generate.extract_creature_entry_tapped(c),(c,'Never'))

    def test_five_tapped_entry_bodies_match_pinned_text_and_reject_residuals(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        rows=json.loads((generate.ROOT/'docs/tapped-entry-admissions.json').read_text())['cards'];self.assertEqual(len(rows),5)
        for a in rows:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            c,_=generate.extract_creature_entry_tapped(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.'))
            self.assertIsNone(generate.creature_template(c));self.assertIsNone(generate.trigger_creature_template(c));self.assertIsNone(generate.permanent_program_template(c))
        self.assertIn('entry: EntryTapped::OpponentsTurn',rust)
        for name in ['Alley Assailant','Gas Guzzler',"Shambling Cie'th",'Tonberry','Arc Reactor','Elixir','White Lotus Tile']:
            self.assertNotIn(name,supported)

    def test_nonmana_ward_costs_use_exact_optional_cost_branches(self):
        for clause,kind in [('Discard a card','Discard'),('Sacrifice a creature','Sacrifice')]:
            program=generate.nonmana_ward_program('Ward—'+clause+'.')
            self.assertIn('EffectChoiceKind::'+kind,program[0])
            self.assertIn('PlayerSet::Opponent',program[0]);self.assertIn('optional_skip: Some(1)',program[0])
            self.assertEqual(program[1:],['Skip(1)','CounterRelated'])
        for amount in [0,7,65535]:
            p=generate.nonmana_ward_program(f'Ward—Pay {amount} life.')
            self.assertEqual(p,[f'MayPayOpponentLife {{ amount: {amount}, skip: 2 }}',f'LoseLife(PlayerSet::Opponent, {amount})','Skip(1)','CounterRelated'])
        for clause in ['Ward—Pay 65536 life.','Ward—Pay X life.','Ward—Discard two cards.','Ward—Sacrifice an artifact.',
                       'Ward—Discard a card and pay 2 life.','Ward—Discard a card. Draw a card.',
                       'Ward—Pay 2 life, discard a card.','Flying, Ward—Pay 2 life.']:
            self.assertIsNone(generate.nonmana_ward_program(clause),clause)
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=clause)),clause)
        repeated=generate.trigger_creature_template(self.card(oracle_text='Ward—Discard a card.\nWard—Discard a card.'))
        self.assertEqual(len(repeated[2]),2)
        self.assertEqual(generate.targeted_program('Put a +1/+1 counter on target creature an opponent controls.')[0],'CreatureOpponent')
        self.assertEqual(generate.targeted_program('Target opponent loses 2 life and you gain 2 life.'),('Opponent',['LoseLife(PlayerSet::Targeted, 2)','GainLife(PlayerSet::Controller, 2)']))

    def test_four_nonmana_ward_bodies_match_pinned_text_and_remain_whole(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        rows=json.loads((generate.ROOT/'docs/nonmana-ward-admissions.json').read_text())['cards'];self.assertEqual(len(rows),4)
        for a in rows:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            altered=dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')
            self.assertIsNone(generate.trigger_creature_template(altered))
            self.assertIsNone(generate.permanent_program_template(altered))
        self.assertIn('Instruction::CounterRelated',rust)
        self.assertNotIn('Tragedy Feaster',supported)

    def card(self, **overrides):
        row = {'layout': 'normal', 'type_line': 'Creature — Cat', 'power': '2', 'toughness': '1',
               'mana_cost': '{W}', 'oracle_text': '', 'colors': ['W']}
        row.update(overrides)
        return row

    def test_exhaust_abilities_keep_independent_lifetime_flags_and_exact_source_proofs(self):
        card=self.card(oracle_text='Exhaust — {G}: Put a +1/+1 counter on this creature.\nExhaust — {2}{G}: Draw a card.\n{G}: Draw a card. Activate only once each turn.')
        result=generate.permanent_program_template(card);self.assertIsNotNone(result);abilities=result[3]
        self.assertEqual([a[0]['exhaust'] for a in abilities],[True,True,False]);self.assertEqual([a[4] for a in abilities],[False,False,True])
        self.assertEqual(generate.rust_activations(abilities).count('exhaust: true'),2)
        future=generate.permanent_program_template(self.card(oracle_text='Exhaust — {1}: Add two mana of any one color.'))
        self.assertIsNotNone(future);self.assertTrue(future[3][0][0]['exhaust'])
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='{1}: Add two mana of any one color.')))
        for text in ['Exhaust — {X}{G}: Draw a card.','Exhaust — {G}: Draw a card. Activate only once during each game.','Exhaust — {G}: Return this card from your graveyard to the battlefield.']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=text)),text)
        loot=self.card(name='Loot, the Pathfinder',type_line='Legendary Creature — Beast Noble',oracle_text='Exhaust — {R}: Loot deals 3 damage to any target.')
        self.assertIsNotNone(generate.permanent_program_template(loot));self.assertIsNone(generate.permanent_program_template(dict(loot,name='Unrelated')))
        targeted=generate.permanent_program_template(self.card(oracle_text='Exhaust — {R}: Put a +1/+1 counter on target creature. It gains double strike until end of turn.'))
        self.assertEqual(targeted[3][0][1],'Creature');self.assertTrue(all('Subject::Targets' in p for p in targeted[3][0][2]))
        # A complete source-counter sentence proves the subsequent source keyword;
        # later target/card pronouns and unknown clauses remain unsupported.
        self.assertIsNotNone(generate.permanent_program_template(self.card(oracle_text='Exhaust — {R}: Put a +1/+1 counter on this creature. It gains double strike until end of turn.')))

    def test_exhaust_loyalty_combinations_remain_gated(self):
        for cost in ['+1', '0', '−1']:
            walker=self.card(type_line='Planeswalker — Example',loyalty='5',
                             oracle_text=f'Exhaust — {cost}: Draw a card.\n0: Draw a card.')
            self.assertIsNone(generate.permanent_program_template(walker),cost)
            ordinary=dict(walker,oracle_text=walker['oracle_text'].replace('Exhaust — ',''))
            self.assertIsNotNone(generate.permanent_program_template(ordinary),cost)

    def test_ten_exhaust_bodies_match_pinned_text_and_gate_reset_exceptions_and_observers(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        admitted=json.loads((generate.ROOT/'docs/exhaust-admissions.json').read_text())['cards'];self.assertEqual(len(admitted),10)
        for a in admitted:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            self.assertIsNone(generate.permanent_program_template(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')))
        self.assertIn('pub(super) fn exhaust_mask(card: Card) -> u64',rust)
        for name in ['Afterburner Expert','Elvish Refueler','Boom Scholar','Greasewrench Goblin','Hog-Monkey','Mai, Jaded Edge','Mindspring Merfolk','Rangers\' Refueler','Skyserpent Seeker','Sita Varma, Masked Racer','Winter, Cursed Rider']:
            self.assertNotIn(name,supported)
        for count in [64,65]:
            c=self.card(oracle_text='\n'.join(['Exhaust — {G}: Draw a card.']*count));got=generate.permanent_program_template(c)
            if count==64:self.assertEqual(len(got[3]),64)
            else:self.assertIsNone(got)

    def test_life_gain_history_guards_preserve_effect_timing_targets_and_branching(self):
        self.assertEqual(generate.step_guard('you gained life this turn'),'StepGuard::GainedLife')
        self.assertIsNone(generate.step_guard('an opponent gained life this turn'))
        self.assertEqual(generate.statement_program('Draw a card if you gained life this turn.'),['If { guard: StepGuard::GainedLife, skip: 1 }','Draw(PlayerSet::Controller, 1)'])
        got=generate.targeted_program('Target creature an opponent controls gets -4/-4 until end of turn if you gained life this turn.')
        self.assertEqual(got,('CreatureOpponent',['If { guard: StepGuard::GainedLife, skip: 1 }','Grow { subject: Subject::Targets, power: -4, toughness: -4 }']))
        prefix='Infusion — When this creature enters, '
        body=generate.trigger_creature_template(self.card(oracle_text=prefix+'put two +1/+1 counters on it if you gained life this turn.'))
        self.assertEqual(body[2][0][0],'SelfEnters');self.assertIn('If { guard: StepGuard::GainedLife',body[2][0][1][0])
        self.assertIn('GainedLife',generate.trigger_creature_template(self.card(oracle_text='At the beginning of your end step, if you gained life this turn, draw a card.'))[2][0][0])
        branch=generate.statement_program('All creatures get -2/-2 until end of turn. Infusion — If you gained life this turn, destroy all creatures instead.')
        self.assertEqual(len(branch),4);self.assertIn('skip: 2',branch[0]);self.assertTrue(branch[1].startswith('MoveBattlefield'));self.assertEqual(branch[2],'Skip(1)');self.assertTrue(branch[3].startswith('GrowCreatures'))
        for text in ['Draw a card if an opponent gained life this turn.','Draw a card if you gained life last turn.','Draw a card. Scry 1. If you gained life this turn, destroy all creatures instead.']:
            self.assertIsNone(generate.statement_program(text),text)
        for text in ['Draw a card. If you gained life this turn, you gain 2 life. Draw two cards instead.',
                     'Draw a card. If you gained life this turn, you gain 2 life. Scry 1 instead.',
                     'All creatures get -2/-2 until end of turn. If you gained life this turn, draw a card. Destroy all creatures instead.']:
            self.assertIsNone(generate.statement_program(text),text)
            self.assertIsNone(generate.spell_program_template(self.card(type_line='Sorcery',oracle_text=text)),text)
        self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Whenever another creature enters, put two +1/+1 counters on it if you gained life this turn.')))

    def test_thirteen_life_gain_and_opponent_debuff_bodies_remain_complete(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        admitted=json.loads((generate.ROOT/'docs/life-gain-condition-admissions.json').read_text())['cards'];self.assertEqual(len(admitted),13)
        for a in admitted:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            changed=dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')
            self.assertIsNone(generate.permanent_program_template(changed));self.assertIsNone(generate.trigger_creature_template(changed));self.assertIsNone(generate.static_permanent_template(changed))
            proxy,base,scales=generate.extract_stat_rules(changed);self.assertIsNone(generate.trigger_creature_template(proxy));self.assertIsNone(generate.permanent_program_template(proxy))
        own=generate.extract_stat_rules(self.card(oracle_text='Infusion — This creature gets +2/+0 as long as you gained life this turn.'))
        self.assertIn('LifeGainOccurredThisTurn',own[2][0])
        global_=generate.static_permanent_template(self.card(oracle_text='Infusion — Creatures you control get +1/+0 and have trample as long as you gained life this turn.'))
        self.assertIn('requires_life_gain: true',global_[5][0]);self.assertIn('requires_life_gain: true',global_[6][0])
        for text in ['This creature gets +2/+0 as long as an opponent gained life this turn.','Creatures you control get +1/+0 as long as you gained life last turn.']:
            self.assertIsNone(generate.static_permanent_template(self.card(oracle_text=text)))
            self.assertEqual(generate.extract_stat_rules(self.card(oracle_text=text))[2],[])
        for name in ['Lumaret\'s Favor','Enlightened Confidant','Follow the Lumarets','Foolish Fate','Lunar Convocation','Moseo, Vein\'s New Dean','Ragost, Deft Gastronaut','Tragedy Feaster']:
            self.assertNotIn(name,supported)

    def test_self_target_events_keep_controller_kind_first_event_and_pronouns_exact(self):
        for noun,spells_only in [('a spell or ability',False),('a spell',True)]:
            for who,relation in [(' you control','You'),(' an opponent controls','Opponent'),('','Any')]:
                body=f'Whenever this creature becomes the target of {noun}{who}, draw a card.'
                got=generate.trigger_creature_template(self.card(oracle_text=body))
                self.assertEqual(got[2][0],(f'SelfTargeted {{ relation: CastRelation::{relation}, spells_only: {str(spells_only).lower()} }}',['Draw(PlayerSet::Controller, 1)'],'None'))
        prefix='Valiant — Whenever this creature becomes the target of a spell or ability you control for the first time each turn, '
        got=generate.trigger_creature_template(self.card(oracle_text=prefix+'put a +1/+1 counter on it.'))
        self.assertIn('OncePerTurn',got[2][0][0]);self.assertIn('Subject::Source',got[2][0][1][0])
        for body in ['Whenever another creature becomes the target of a spell, draw a card.',
                     'Whenever this creature becomes the target of an ability you control, draw a card.',
                     prefix+'if you control a Desert, draw a card.',
                     prefix+'put a +1/+1 counter on it. Scry 1.',
                     prefix+'tap target creature. Put a +1/+1 counter on it.',
                     prefix.replace('you control','an opponent controls')+'draw a card.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=body)),body)
        ruric=self.card(name='Ruric Thar, Biomagus',type_line='Legendary Creature — Ogre Crab Wizard',oracle_text='Prowess, prowess\nWhenever Ruric Thar becomes the target of a spell or ability an opponent controls, draw a card.')
        self.assertEqual(len(generate.trigger_creature_template(ruric)[2]),3)
        self.assertIsNone(generate.trigger_creature_template(dict(ruric,name='Unrelated')))

    def test_eight_self_target_bodies_match_pinned_text_and_gate_residuals(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        admitted=json.loads((generate.ROOT/'docs/self-target-admissions.json').read_text())['cards'];self.assertEqual(len(admitted),8)
        for a in admitted:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            changed=dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')
            self.assertIsNone(generate.permanent_program_template(changed));self.assertIsNone(generate.trigger_creature_template(changed));self.assertIsNone(generate.creature_template(changed))
        cactus=next(c for c in snapshot['cards'] if c['name']=='Cactarantula');_,rules=generate.extract_cost_reductions(cactus)
        self.assertEqual(len(rules),1);self.assertIn('subtype: Some("Desert")',rules[0]);self.assertIn('source_only: true',rules[0])
        for name in ['Altanak, the Thrice-Called','Pawpatch Recruit','Surrak, Elusive Hunter','Emberheart Challenger','Flowerfoot Swordmaster','Whiskervale Forerunner']:
            self.assertNotIn(name,supported)

    def test_power_blocking_flags_and_temporary_grants_keep_skulk_distinct(self):
        self.assertEqual(generate.KEYWORDS['skulk'],1048576)
        self.assertEqual(generate.creature_template(self.card(oracle_text='Skulk')),('Creature(2, 1)',1048576))
        self.assertEqual(generate.TEXT_FLAGS["this creature can't be blocked by creatures with greater power."],8388608)
        for text,mask in [("This creature can't be blocked by creatures with power 2 or less this turn.",2097152),("This creature can't be blocked by creatures with power 3 or greater this turn.",4194304),("This creature can't be blocked by creatures with greater power this turn.",8388608)]:
            self.assertEqual(generate.statement_program(text),[f'GrantKeywords {{ subject: Subject::Source, keywords: Keywords({mask}) }}'])
        for text in ["This creature can't be blocked by creatures with power 4 or greater this turn.","This creature can't be blocked by creatures with power 2 or less until your next turn.","This creature can't be blocked by creatures with greater toughness this turn."]:
            self.assertIsNone(generate.statement_program(text),text)
        self.assertEqual(generate.targeted_program("Return another target creature you control to its owner's hand."),('OtherCreatureYou',['ReturnToHand']))

    def test_four_power_blocking_bodies_reject_unsupported_residuals_and_names(self):
        import json
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        admitted=json.loads((generate.ROOT/'docs/power-blocking-admissions.json').read_text())['cards'];self.assertEqual(len(admitted),4)
        for a in admitted:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            changed=dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')
            self.assertIsNone(generate.permanent_program_template(changed));self.assertIsNone(generate.trigger_creature_template(changed));self.assertIsNone(generate.creature_template(changed))
        april=next(c for c in snapshot['cards'] if c['name']=="April O'Neil, Kunoichi Trainee")
        self.assertIsNone(generate.trigger_creature_template(dict(april,name='Unrelated',type_line='Legendary Creature — Human Ninja')))
        self.assertIsNone(generate.trigger_creature_template(self.card(name='Unrelated',type_line='Legendary Creature — Human Ninja',oracle_text="When this creature enters, scry 2.\nApril O'Neil can't be blocked by creatures with power 3 or greater.")))
        for name in ['Ingenious Prodigy','Gate Colossus','Azure Beastbinder','Delney, Streetwise Lookout','Bristlebane Outrider','Foggy Swamp Vinebender','Rust-Shield Rampager','Sandman, Shifting Scoundrel']:
            self.assertNotIn(name,supported)

    def test_own_turn_guard_and_authenticated_sacrifice_cost_names(self):
        for alias in ['Professor Zei','Professor Zei, Anthropologist']:
            card=self.card(name='Professor Zei, Anthropologist',type_line='Legendary Creature — Human Advisor Ally',oracle_text='{1}, {T}, Sacrifice '+alias+': Draw a card. Activate only during your turn.')
            result=generate.permanent_program_template(card);self.assertIsNotNone(result)
            cost,_,_,sorcery,_=result[3][0];self.assertEqual(cost['guard'],'OwnTurn');self.assertFalse(sorcery);self.assertTrue(cost['tap']);self.assertTrue(cost['sacrifice'])
        for clause in ['Sacrifice Professor Z','Sacrifice another Professor Zei','Sacrifice Professor Zei Jr.']:
            self.assertIsNone(generate.permanent_program_template(self.card(name='Professor Zei, Anthropologist',oracle_text=clause+': Draw a card.')))
        self.assertIsNone(generate.permanent_program_template(self.card(name='Unrelated',oracle_text='Sacrifice Professor Zei: Draw a card.')))
        for text in ['{T}: Draw a card. Activate only during an opponent’s turn.',
                     '{T}: Draw a card. Activate only during your turn and only once.',
                     '{T}: Draw a card. Activate only if you control a Dragon. Activate only during your turn.']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=text)),text)

    def test_four_own_turn_and_recovery_bodies_keep_exact_targets_and_residuals(self):
        import json
        for text,rule in [("Return another target permanent you control to its owner's hand.",'OtherPermanentYou'),('Return target instant or sorcery card from your graveyard to your hand.','InstantSorceryGraveYou')]:
            self.assertEqual(generate.targeted_program(text),(rule,['ReturnToHand']))
        self.assertIn('kinds: 12',generate.TARGET_RULES['InstantSorceryGraveYou']);self.assertIn('relation: 0',generate.TARGET_RULES['InstantSorceryGraveYou']);self.assertIn('other: true',generate.TARGET_RULES['OtherPermanentYou']);self.assertIn('nonland: false',generate.TARGET_RULES['OtherPermanentYou'])
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        admitted=json.loads((generate.ROOT/'docs/own-turn-admissions.json').read_text())['cards'];self.assertEqual(len(admitted),4)
        for a in admitted:
            c=next(c for c in snapshot['cards'] if c['name']==a['name']);self.assertEqual(c['oracle_text'],a['oracle_text']);self.assertIn(c['name'],supported)
            self.assertIsNone(generate.permanent_program_template(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')))
            self.assertIsNone(generate.trigger_creature_template(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')))
        for name in ['Wishclaw Talisman','Path to Redemption','M.O.D.O.K.','Scour for Scrap','Oltec Archaeologists']:
            self.assertNotIn(name,supported)

    def test_discard_history_amount_and_two_complete_bodies(self):
        import json
        self.assertEqual(generate.amount_value("the number of cards you've discarded this turn"),'AmountValue::CardsDiscardedThisTurn')
        self.assertEqual(generate.statement_program("Draw a card for each card you've discarded this turn."),['DrawScaled(PlayerSet::Controller, AmountValue::CardsDiscardedThisTurn)'])
        self.assertEqual(generate.statement_program("Discard a card. Then draw a card for each card you've discarded this turn."),['Discard(PlayerSet::Controller, 1)','DrawScaled(PlayerSet::Controller, AmountValue::CardsDiscardedThisTurn)'])
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(snapshot,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Green Goblin, Revenant','Misty Knight, Hero for Hire']:
            c=next(c for c in snapshot['cards'] if c['name']==name);self.assertIn(name,supported)
            self.assertIsNone(generate.permanent_program_template(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')))
        self.assertNotIn('Jiang Yanggu, Alone',supported)
        admitted=json.loads((generate.ROOT/'docs/discard-history-admissions.json').read_text())['cards']
        self.assertEqual({c['name'] for c in admitted},{'Green Goblin, Revenant','Misty Knight, Hero for Hire'})
        for a in admitted:self.assertEqual(a['oracle_text'],next(c['oracle_text'] for c in snapshot['cards'] if c['name']==a['name']))
        goblin=next(c for c in snapshot['cards'] if c['name']=='Green Goblin, Revenant')
        self.assertIsNone(generate.trigger_creature_template(dict(goblin,name='An Unrelated Creature')))
        for text in ["Draw a card for each card an opponent discarded this turn.","Draw a card for each card you've discarded this turn. Discover 4.","Discard two cards. Then draw a card for each card you've discarded this turn."]:
            self.assertIsNone(generate.statement_program(text),text)

    def test_activation_discard_costs_are_one_owned_choice_with_complete_gates(self):
        cost=generate.activation_cost('{1}{R}, {T}, Discard a card, Sacrifice this creature')
        self.assertEqual(cost['generic'],1);self.assertEqual(cost['colored'][3],1)
        self.assertTrue(cost['tap']);self.assertTrue(cost['sacrifice']);self.assertIn('EffectChoiceKind::Discard',cost['additional'])
        self.assertFalse(cost.get('discard',False)) # cycling's self-discard remains separate
        for text in ['Discard two cards','Discard a land card','Discard a card, Discard a card','Discard a card, Sacrifice another creature','Discard a card, Pay 1 life','Discard a card, Remove a counter from this creature']:
            self.assertIsNone(generate.activation_cost(text),text)
        for text in ['Discard a card: Add {R}.','{T}, Discard a card: Add {R}.',
                     'Discard a card: It deals 1 damage to any target.',
                     'Discard a card: Draw a card. Activate only once.',
                     'Discard a card: Draw a card. Unknown residual.',
                     'Whenever you discard a card, you gain 1 life.\nDiscard a card, Sacrifice this creature: Draw a card.']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=text)),text)
        result=generate.permanent_program_template(self.card(oracle_text='Discard a card: Draw a card. Activate only as a sorcery and only once each turn.'))
        self.assertEqual(len(result[3]),1);self.assertTrue(result[3][0][3]);self.assertTrue(result[3][0][4])
        result=generate.permanent_program_template(self.card(oracle_text='Discard a card: This creature gains indestructible until end of turn. Tap it.'))
        self.assertEqual(result[3][0][2],['GrantKeywords { subject: Subject::Source, keywords: Keywords(8192) }','TapSource'])

    def test_seven_discard_activation_bodies_admit_only_full_oracle(self):
        import json
        cards=json.loads((generate.ROOT/'data/standard/cards.json').read_text())['cards']
        names={'Charging Strifeknight','Flamecache Gecko','Inner Demons Gangsters','Iron-Shield Elf','Masked Meower','Murmuring Volume','Thunderhead Gunner'}
        for c in cards:
            if c['name'] not in names:continue
            result=generate.permanent_program_template(c);self.assertIsNotNone(result,c['name'])
            self.assertTrue(any(a[0]['additional'] is not None for a in result[3]),c['name'])
            self.assertIsNone(generate.permanent_program_template(dict(c,oracle_text=c['oracle_text']+'\nDiscover 4.')),c['name'])
        self.assertEqual(sum(c['name'] in names for c in cards),7)
        admitted=json.loads((generate.ROOT/'docs/activation-discard-admissions.json').read_text())['cards']
        self.assertEqual({c['name'] for c in admitted},names)
        for a in admitted:self.assertEqual(a['oracle_text'],next(c['oracle_text'] for c in cards if c['name']==a['name']))
        _,coverage=generate.compile_catalogue(json.loads((generate.ROOT/'data/standard/cards.json').read_text()),json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text()))
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        self.assertTrue(names<=supported)
        for name in ['Hardened Academic','Liliana the Faultless','Possessed Goat','Avishkar Raceway','Fomori Vault']:
            self.assertNotIn(name,supported)

    def test_reductions_preserve_exact_scope_filters_timing_and_unsupported_clauses(self):
        for text,needles in [
            ('Dragon spells you cast cost {1} less to cast.',['source_only: false','Subtype("Dragon")','SpellReductionAmount::Fixed(1)']),
            ('Kithkin spells and Soldier spells you cast cost {1} less to cast.',['AnyOf','Subtype("Kithkin")','Subtype("Soldier")']),
            ('During turns other than yours, spells you cast cost {1} less to cast.',['CostTurn::Other']),
            ('The second spell you cast each turn costs {2} less to cast.',['StepValue::SpellsCast','minimum: Some(1)','maximum: Some(1)']),
            ('This spell costs {3} less to cast if creatures you control have total toughness 10 or greater.',['source_only: true','TotalCreatureToughness']),
            ("This spell costs {2} less to cast if you've cast another spell this turn.",['minimum: Some(1)','maximum: None']),
            ('This spell costs {1} less to cast during your turn.',['CostTurn::Own']),
        ]:
            proxy,rules=generate.extract_cost_reductions(self.card(oracle_text=text+'\nFlying'))
            self.assertEqual(proxy['oracle_text'],'Flying');self.assertEqual(len(rules),1)
            for needle in needles:self.assertIn(needle,rules[0],text)
        for text in ['This spell costs {2} less to cast if it targets a tapped creature.',
                     'This spell costs {1}{U} less to cast if it targets an instant or sorcery spell.',
                     'This spell costs {X} less to cast, where X is the number of cards in your hand.',
                     'The first creature spell you cast each turn costs {2} less to cast.',
                     'Noncreature spells your opponents cast cost {1} more to cast.',
                     'Dragon spells you cast cost {65536} less to cast.',
                     'Artifact spells you cast cost {1} less to cast. Activate only once each turn.']:
            proxy,rules=generate.extract_cost_reductions(self.card(oracle_text=text))
            self.assertEqual(proxy['oracle_text'],text);self.assertEqual(rules,[],text)

    def test_activation_hybrid_and_two_brid_costs_preserve_one_ability_identity(self):
        for cost,n in [('{1}{B/R}',2),('{R/W}{R/W}',3),('{2/W}',2),('{2/W}{2/U}',4)]:
            parsed=generate.activation_cost(cost+', {T}, Pay 2 life')
            self.assertIsNotNone(parsed);self.assertEqual(len(parsed['alternatives'])+1,n);self.assertTrue(parsed['tap']);self.assertEqual(parsed['life'],2)
        row=self.card(oracle_text='{G/W}: This creature gains vigilance until end of turn. Activate only once each turn.')
        result=generate.permanent_program_template(row);self.assertEqual(len(result[3]),1);self.assertTrue(result[3][0][4]);self.assertEqual(len(result[3][0][0]['alternatives']),1)
        for text in ['{B/P}','{W/U/P}','{X}{R/W}','{3/W}','{W/U}, {R/W}']:
            self.assertIsNone(generate.activation_cost(text))

    def test_two_brid_mana_profit_gate_uses_the_cheapest_equivalent(self):
        # Canonical generic-side cost2 alone must not hide the one-mana side.
        for text in ['{2/W}: Add {W}{W}.','{2/W}{2/U}: Add {W}{W}{W}.']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=text)))
        self.assertIsNotNone(generate.permanent_program_template(self.card(oracle_text='{2/W}: Add {U}.')))
        self.assertIsNotNone(generate.permanent_program_template(self.card(oracle_text='{2/W}, {T}: Add {W}{W}.')))

    def test_three_hybrid_activation_bodies_keep_full_residual_gates(self):
        import json
        admissions=json.loads((generate.ROOT/'docs/hybrid-activation-admissions.json').read_text())['cards'];self.assertEqual(len(admissions),3)
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in report['cards'] if c['implemented']}
        for admitted in admissions:
            card=next(c for c in snapshot['cards'] if c['name']==admitted['name']);self.assertIn(admitted['name'],supported);self.assertEqual(card['oracle_text'],admitted['oracle_text'])
            proxy,_=generate.extract_entry_counters(dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.'))
            self.assertIsNone(generate.creature_template(proxy));self.assertIsNone(generate.trigger_creature_template(proxy));self.assertIsNone(generate.permanent_program_template(proxy));self.assertIsNone(generate.static_permanent_template(proxy))
        for name in ['Hired Claw','Gev, Scaled Scorch','Azra Oddsmaker']:
            self.assertNotIn(name,supported)

    def test_opponent_life_loss_predicate_and_vampire_target_scope_are_exact(self):
        self.assertEqual(generate.step_guard('an opponent lost life this turn'),'StepGuard::OpponentLostLife')
        proxy,rules=generate.extract_entry_counters(self.card(oracle_text='This creature enters with a +1/+1 counter on it if an opponent lost life this turn.'))
        self.assertEqual(proxy['oracle_text'],'');self.assertIn('OpponentLostLife',rules[0])
        result=generate.targeted_program('Put a +1/+1 counter on target Vampire you control.')
        self.assertEqual(result[0],'VampireYou');rule=generate.TARGET_RULES[result[0]]
        self.assertIn('kinds: 0',rule);self.assertIn('relation: 0',rule);self.assertIn('Subtype("Vampire")',rule)
        for text in ['you lost life this turn','an opponent was dealt damage this turn','an opponent lost life last turn']:
            self.assertIsNone(generate.step_guard(text))

    def test_three_life_loss_bodies_and_unsupported_residual_gates(self):
        import json
        admissions=json.loads((generate.ROOT/'docs/life-loss-admissions.json').read_text())['cards'];self.assertEqual(len(admissions),3)
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in report['cards'] if c['implemented']}
        for admitted in admissions:
            card=next(c for c in snapshot['cards'] if c['name']==admitted['name']);self.assertEqual(card['oracle_text'],admitted['oracle_text']);self.assertIn(admitted['name'],supported)
            proxy,_=generate.extract_entry_counters(dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.'))
            self.assertIsNone(generate.creature_template(proxy));self.assertIsNone(generate.trigger_creature_template(proxy));self.assertIsNone(generate.permanent_program_template(proxy));self.assertIsNone(generate.static_permanent_template(proxy))
        for name in ['Hired Claw','Fireglass Mentor','Gev, Scaled Scorch']:
            self.assertNotIn(name,supported)

    def test_morbid_predicate_is_shared_by_entry_cost_and_intervening_step_rules(self):
        guard='StepGuard::CreatureDied'
        self.assertEqual(generate.step_guard('a creature died this turn'),guard)
        proxy,rules=generate.extract_entry_counters(self.card(oracle_text='This creature enters with a +1/+1 counter on it if a creature died this turn.'))
        self.assertEqual(proxy['oracle_text'],'');self.assertIn(guard,rules[0])
        proxy,rules=generate.extract_cost_reductions(self.card(oracle_text='This spell costs {3} less to cast if a creature died this turn.\nFlying'))
        self.assertEqual(proxy['oracle_text'],'Flying');self.assertIn(guard,rules[0]);self.assertIn('Fixed(3)',rules[0])
        result=generate.trigger_creature_template(self.card(oracle_text='Morbid — At the beginning of your end step, if a creature died this turn, put a +1/+1 counter on this creature.'))
        self.assertIsNotNone(result);self.assertIn(guard,result[2][0][0]);self.assertIn('TriggerStep::End',result[2][0][0]);self.assertIn('CastRelation::You',result[2][0][0])
        for text in ['a creature you control died this turn','a nontoken creature died this turn','a creature left the battlefield this turn']:
            self.assertIsNone(generate.step_guard(text))

    def test_five_morbid_bodies_and_residual_gates(self):
        import json
        admissions=json.loads((generate.ROOT/'docs/death-history-admissions.json').read_text())['cards'];self.assertEqual(len(admissions),5)
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(snapshot,target);supported={c['name'] for c in report['cards'] if c['implemented']}
        for admitted in admissions:
            original=next(c for c in snapshot['cards'] if c['name']==admitted['name']);self.assertIn(admitted['name'],supported)
            self.assertEqual(admitted['oracle_text'],original['oracle_text'])
            proxy,_=generate.extract_cost_reductions(dict(original,oracle_text=original['oracle_text']+'\nWhenever you cast a spell, discover 3.'))
            proxy,_=generate.extract_entry_counters(proxy)
            self.assertIsNone(generate.creature_template(proxy));self.assertIsNone(generate.trigger_creature_template(proxy));self.assertIsNone(generate.permanent_program_template(proxy));self.assertIsNone(generate.static_permanent_template(proxy))
        for name in ['Blacksnag Buzzard','Rictus Robber','Old Flitterfang','Tragic Banshee']:
            self.assertNotIn(name,supported)

    def test_entry_counter_replacements_use_exact_pre_entry_predicates(self):
        texts = [
            ('Raid — This creature enters with a +1/+1 counter on it if you attacked this turn.', ['StepGuard::Attacked', 'Fixed(1)']),
            ("This creature enters with two +1/+1 counters on it if you've cast two or more spells this turn.", ['StepValue::SpellsCast', 'minimum: Some(2)', 'Fixed(2)']),
            ('This creature enters with a +1/+1 counter on it for each other creature and/or artifact you control.', ['kind: 17', 'relation: 0', 'other: true', 'cards_only: false']),
        ]
        for text, needles in texts:
            proxy, rules = generate.extract_entry_counters(self.card(oracle_text=text+'\nFlying'))
            self.assertEqual(proxy['oracle_text'], 'Flying');self.assertEqual(len(rules),1)
            for needle in needles:self.assertIn(needle,rules[0])
        for text in [
            'Raid — This creature enters with a +1/+1 counter on it if an opponent attacked this turn.',
            'This creature enters with two +1/+1 counters on it if a creature died this turn.',
            'This creature enters with a +1/+1 counter on it for each creature and/or artifact that entered this turn.',
            'This creature enters with a divinity counter on it if you cast it from your hand.',
            texts[0][0]+' Draw a card.',
        ]:
            proxy,rules=generate.extract_entry_counters(self.card(oracle_text=text))
            self.assertEqual(proxy['oracle_text'],text);self.assertEqual(rules,[])

    def test_three_entry_counter_bodies_keep_unsupported_residuals_gated(self):
        import json
        admissions=json.loads((generate.ROOT/'docs/entry-counter-admissions.json').read_text())['cards']
        self.assertEqual(len(admissions),3)
        snapshot=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(snapshot,target)
        supported={c['name'] for c in report['cards'] if c['implemented']}
        for admitted in admissions:
            original=next(c for c in snapshot['cards'] if c['name']==admitted['name'])
            self.assertEqual(admitted['oracle_text'],original['oracle_text'])
            self.assertIn(admitted['name'],supported)
            altered=dict(original,oracle_text=original['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            proxy,rules=generate.extract_entry_counters(altered);self.assertTrue(rules)
            self.assertIsNone(generate.creature_template(proxy));self.assertIsNone(generate.trigger_creature_template(proxy))
            self.assertIsNone(generate.permanent_program_template(proxy));self.assertIsNone(generate.static_permanent_template(proxy))
        for name in ['Cruel Administrator','Blacksnag Buzzard','Patched Plaything','Putrid Pals']:
            self.assertNotIn(name,supported)

    def test_attack_intervening_guards_require_the_complete_tapped_declaration(self):
        result=generate.trigger_creature_template(self.card(oracle_text='Whenever this creature attacks, if you control two or more tapped creatures, draw a card.'))
        self.assertIsNotNone(result)
        self.assertIn('InterveningIf',result[2][0][0]);self.assertIn('TappedCreatures',result[2][0][0]);self.assertIn('minimum: Some(2)',result[2][0][0]);self.assertIn('SelfAttacks',result[2][0][0])
        self.assertEqual(result[2][0][1],['Draw(PlayerSet::Controller, 1)'])

    def test_attack_history_costs_and_raid_entry_guards_are_exact(self):
        text='This spell costs {1} less to cast for each creature that attacked this turn.'
        proxy,rules=generate.extract_cost_reductions(self.card(oracle_text=text+'\nFlying'))
        self.assertEqual(proxy['oracle_text'],'Flying');self.assertIn('SpellReductionAmount::Attackers { factor: 1 }',rules[0]);self.assertIn('source_only: true',rules[0])
        for clause in ['creature that died this turn','creature you control that attacked this turn','creature put onto the battlefield attacking this turn']:
            original='This spell costs {1} less to cast for each '+clause+'.'
            proxy,rules=generate.extract_cost_reductions(self.card(oracle_text=original));self.assertEqual(proxy['oracle_text'],original);self.assertEqual(rules,[])
        entry=generate.trigger_creature_template(self.card(oracle_text='Raid — When this creature enters, if you attacked this turn, draw a card.'))
        self.assertIn('InterveningIf',entry[2][0][0]);self.assertIn('StepGuard::Attacked',entry[2][0][0]);self.assertEqual(entry[2][0][1],['Draw(PlayerSet::Controller, 1)'])
        program=generate.statement_program('Draw two cards. Then discard a card unless you attacked this turn.')
        self.assertEqual(program,['Draw(PlayerSet::Controller, 2)','If { guard: StepGuard::Attacked, skip: 1 }','Skip(1)','Discard(PlayerSet::Controller, 1)'])
        self.assertIsNone(generate.statement_program('Draw two cards. Then discard a card unless a creature entered this turn.'))

    def test_seven_attack_history_raid_bodies_require_complete_text(self):
        import json
        admissions=json.loads((generate.ROOT/'docs/attack-history-admissions.json').read_text())['cards'];self.assertEqual(len(admissions),7)
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        for admitted in admissions:
            name=admitted['name'];self.assertIn(name,supported)
            card=next(c for c in rows['cards'] if c['name']==name);self.assertEqual(card['oracle_text'],admitted['oracle_text'])
            proxy,_=generate.extract_cost_reductions(dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.'))
            self.assertIsNone(generate.spell_program_template(proxy),name);self.assertIsNone(generate.permanent_program_template(proxy),name)
            self.assertIsNone(generate.static_permanent_template(proxy),name);self.assertIsNone(generate.modal_spell_template(proxy),name)
        for name in ['Cruel Administrator','Avengers Assemble!','Full Throttle','Hexhaven Dueling Arena','Strongbox Raider']:
            self.assertNotIn(name,supported)

    def test_derived_cost_quantities_keep_live_stats_out_of_characteristic_layers(self):
        for text,needles in [
            ('This spell costs {X} less to cast, where X is the total power of creatures you control.',['CreaturePower','greatest: false','relation: 0']),
            ('This spell costs {X} less to cast, where X is the greatest power among creatures you control.',['CreaturePower','greatest: true']),
            ('This spell costs {X} less to cast, where X is the total power of creatures you control with flying.',['CreaturePower','keyword: Keywords(1)']),
            ('This spell costs {X} less to cast, where X is the number of differently named lands you control.',['DistinctNames','kind: 2']),
            ('Affinity for outlaws (This spell costs {1} less to cast for each Assassin, Mercenary, Pirate, Rogue, and/or Warlock you control.)',['CountUnion','"Assassin"','"Mercenary"','"Pirate"','"Rogue"','"Warlock"']),
            ("Instant and sorcery spells you cast with mana value 4 or greater cost {X} less to cast, where X is The Scarlet Witch's power.",['ManaValueMinimum','minimum: 4','CastFilter::InstantSorcery','SourcePower','source_only: false']),
        ]:
            proxy,rules=generate.extract_cost_reductions(self.card(name='The Scarlet Witch',oracle_text=text+'\nFlying'))
            self.assertEqual(proxy['oracle_text'],'Flying');self.assertEqual(len(rules),1)
            for needle in needles:self.assertIn(needle,rules[0],text)
        self.assertIsNone(generate.stat_value('total power of creatures you control'))
        self.assertIsNone(generate.stat_value('greatest power among creatures you control'))
        for text in [
            'This spell costs {X} less to cast, where X is the power of target creature.',
            'This spell costs {X} less to cast, where X is the total power of creatures that attacked this turn.',
            'This spell costs {X} less to cast, where X is the number of differently named cards in your hand.',
            "Instant and sorcery spells you cast with mana value 18446744073709551616 or greater cost {X} less to cast, where X is The Scarlet Witch's power.",
            "Instant and sorcery spells you cast with mana value 4 or greater cost {X} less to cast, where X is Another Creature's power.",
            'Affinity for outlaws (This spell costs {1} less to cast for each Assassin and Rogue you control.)',
        ]:
            proxy,rules=generate.extract_cost_reductions(self.card(name='The Scarlet Witch',oracle_text=text))
            self.assertEqual(proxy['oracle_text'],text);self.assertEqual(rules,[],text)

    def test_seven_derived_reduction_admissions_require_every_body_clause(self):
        import json
        admissions=json.loads((generate.ROOT/'docs/derived-reduction-admissions.json').read_text())['cards'];self.assertEqual(len(admissions),7)
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        for admitted in admissions:
            name=admitted['name'];self.assertIn(name,supported)
            card=next(c for c in rows['cards'] if c['name']==name);self.assertEqual(card['oracle_text'],admitted['oracle_text'])
            proxy,_=generate.extract_cost_reductions(dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.'))
            self.assertIsNone(generate.spell_program_template(proxy),name);self.assertIsNone(generate.permanent_program_template(proxy),name)
            self.assertIsNone(generate.static_permanent_template(proxy),name);self.assertIsNone(generate.modal_spell_template(proxy),name)
        for name in ['Ghalta the Immovable','The Pride of Hull Clade','The Skullspore Nexus','Full Throttle','Sunderflock']:
            self.assertNotIn(name,supported)

    def test_scaled_reductions_bind_exact_count_zones_unions_and_affinity(self):
        for text,needles in [
            ('Affinity for artifacts (This spell costs {1} less to cast for each artifact you control.)',['CountZone::Battlefield','kind: 16','source_only: true']),
            ('Affinity for Slivers',['Subtype','source_only: true']),
            ('This spell costs {1} less to cast for each artifact and/or creature card in your graveyard.',['CountZone::Graveyard','kind: 17','cards_only: true']),
            ('This spell costs {1} less to cast for each creature card you own in exile and in your graveyard.',['StatValue::Sum','CountZone::Exile','CountZone::Graveyard']),
            ('This spell costs {1} less to cast for each Cave you control and each Cave card in your graveyard.',['StatValue::Sum','"Cave"','CountZone::Battlefield','CountZone::Graveyard']),
            ('Vivid — This spell costs {1} less to cast for each color among permanents you control.',['StatValue::Colors','kind: 0']),
            ('Instant and sorcery spells you cast have affinity for creatures.',['CastFilter::InstantSorcery','source_only: false','kind: 1']),
            ('Spells you cast have affinity for artifacts. (They cost {1} less to cast for each artifact you control.)',['CastFilter::Any','kind: 16','source_only: false']),
            ('This spell costs {X} less to cast, where X is the greatest mana value among Goblins you control.',['GreatestManaValue','"Goblin"']),
        ]:
            proxy,rules=generate.extract_cost_reductions(self.card(oracle_text=text+'\nFlying'))
            self.assertEqual(proxy['oracle_text'],'Flying',text);self.assertEqual(len(rules),1,text)
            for needle in needles:self.assertIn(needle.replace('Subtype','subtype: Some'),rules[0],text)
            self.assertIn('SpellReductionAmount::Stat',rules[0])
        for text in [
            'Affinity for artifacts (This spell costs {1} less to cast for each creature you control.)',
            'Spells you cast have affinity for artifacts. (They cost {1} less to cast for each creature you control.)',
            'This spell costs {1} less to cast for each creature that died this turn.',
            'This spell costs {1} less to cast for each card in your hand.',
            'This spell costs {1} less to cast for each permanent sacrificed this way.',
            'This spell costs {1} less to cast for each creature you control with power 4 or greater.',
            'This spell costs {65536} less to cast for each artifact you control.',
        ]:
            proxy,rules=generate.extract_cost_reductions(self.card(oracle_text=text))
            self.assertEqual(proxy['oracle_text'],text);self.assertEqual(rules,[],text)

    def test_fourteen_scaled_reduction_bodies_and_provenance_are_complete(self):
        import json
        admissions=json.loads((generate.ROOT/'docs/scaled-reduction-admissions.json').read_text())['cards']
        self.assertEqual(len(admissions),14)
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        for admitted in admissions:
            name=admitted['name'];self.assertIn(name,supported)
            card=next(c for c in rows['cards'] if c['name']==name)
            self.assertEqual(card['oracle_text'],admitted['oracle_text'])
            proxy,rules=generate.extract_cost_reductions(dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.'))
            self.assertTrue(rules,name)
            self.assertIsNone(generate.spell_program_template(proxy),name);self.assertIsNone(generate.permanent_program_template(proxy),name)
            self.assertIsNone(generate.static_permanent_template(proxy),name);self.assertIsNone(generate.modal_spell_template(proxy),name)
        for name in ['Cruel Administrator','Don & Raph, Hard Science','Spectral Denial',
                     'The Dawning Archaic','Tomik, Wielder of Law','Rottenmouth Viper','Sunderflock','Sapling Nursery']:
            self.assertNotIn(name,supported)

    def test_hand_count_reductions_remain_gated_until_cast_zone_is_modeled(self):
        import hashlib
        import json
        clause='This spell costs {1} less to cast if you have fewer than 3 cards in hand.'
        # The predicate is valid for resolution/step programs, but cost queries
        # need the proposed casting state (three cards becomes two in hand).
        self.assertIn('StepValue::Hand',generate.step_guard('you have fewer than 3 cards in hand'))
        card=self.card(name='Hand Reduction Regression',oracle_id='hand-reduction-regression',
                       type_line='Sorcery',mana_cost='{1}{U}',
                       colors=['U'],oracle_text=clause+'\nDraw two cards.')
        proxy,rules=generate.extract_cost_reductions(card)
        self.assertEqual(proxy,card);self.assertEqual(rules,[])
        cards=[card]
        canonical=json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
        snapshot=dict(cards=cards,card_count=1,cards_sha256=hashlib.sha256(canonical).hexdigest())
        _,report=generate.compile_catalogue(snapshot,dict(banned_card_names=[]))
        row=next(row for row in report['cards'] if row['name']==card['name'])
        self.assertFalse(row['implemented'])

    def test_twenty_reduction_admissions_require_every_residual_body_clause(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        names=['Arcane Epiphany','Archmage of Runes','Ballyrush Banneret',"Dragonlord's Servant",'Focus the Mind',
               'Geist of Saint Thalia','Geyser Drake','Gigastorm Titan','Highspire Bell-Ringer','Honest Rutstein',
               'Mistmeadow Council','Mocking Sprite','Orysa, Tide Choreographer','Pearl of Wisdom','Saved by the Shell',
               'Stormcatch Mentor','Uthros Psionicist',"Venom's Hunger",'Visions of Villainy','Voyager Quickwelder']
        for name in names:
            self.assertIn(name,supported)
            card=next(c for c in rows['cards'] if c['name']==name)
            proxy,rules=generate.extract_cost_reductions(dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.'))
            self.assertTrue(rules,name)
            self.assertIsNone(generate.spell_program_template(proxy),name);self.assertIsNone(generate.permanent_program_template(proxy),name)
            self.assertIsNone(generate.static_permanent_template(proxy),name);self.assertIsNone(generate.modal_spell_template(proxy),name)
        for name in ['Thalia, the Survivor','Out of Air',"Dragon's Prey",'Quicksand Whirlpool','Spectral Denial',
                     'Eluge, the Shoreless Sea','Inquisitive Glimmer','Traxos, Academy Guardian']:
            self.assertNotIn(name,supported)

    def test_restricted_counter_targets_compile_exact_unions_colors_and_mana_ranges(self):
        for phrase,rule in [('artifact or enchantment spell','SpellArtifactEnchantment'),
                            ('artifact or creature spell','SpellArtifactCreature'),
                            ('red or green spell','SpellRedGreen'),('white or black spell','SpellWhiteBlack'),
                            ('spell with mana value 2','SpellMana2'),('spell with mana value 4 or greater','SpellManaMin4')]:
            self.assertEqual(generate.targeted_program('Counter target '+phrase+'.'),(rule,['Counter']))
            result=generate.targeted_program('Counter target '+phrase+' unless its controller pays {4}.')
            self.assertEqual(result[0],rule);self.assertIn('CounterUnlessPay',result[1][0])
            self.assertIn('generic: 4',result[1][0])
        for text in ['Counter target blue or red spell.','Counter target spell with mana value X.',
                     'Counter target spell unless its controller pays {X}.',
                     'Counter target artifact or enchantment spell you control.',
                     'Counter target spell with mana value 2 or less.']:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_seven_restricted_counter_admissions_require_complete_bodies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target)
        supported={r['name'] for r in report['cards'] if r['implemented']}
        for name in ['Annul','Flashfreeze','Precise Redaction','Spell Snare','Disdainful Stroke','Spectral Interference','Voracious Greatshark']:
            self.assertIn(name,supported)
            card=next(c for c in rows['cards'] if c['name']==name)
            extra=dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            self.assertIsNone(generate.spell_program_template(extra),name)
            self.assertIsNone(generate.permanent_program_template(extra),name)
        for name in ['Dazzling Denial','Spell Stutter','Syncopate','An Offer You Can\'t Refuse','Unravel']:
            self.assertNotIn(name,supported)

    def test_guarded_blocks_keep_only_the_first_sentence_conditional_and_else_offsets_exact(self):
        p=generate.statement_program('If you control an Elf, draw a card. You gain 2 life.')
        self.assertEqual(len(p),3);self.assertIn('If {',p[0]);self.assertIn('skip: 1',p[0]);self.assertEqual(p[1:],[ 'Draw(PlayerSet::Controller, 1)','GainLife(PlayerSet::Controller, 2)'])
        p=generate.statement_program('If you control an Elf, draw a card. Otherwise, you gain 2 life. Scry 1.')
        self.assertIn('skip: 2',p[0]);self.assertEqual(p[2],'Skip(1)');self.assertEqual(p[-1],'Scry(1)')
        p=generate.statement_program('If you control a Raccoon, you may discard a card. If you do, draw a card.')
        self.assertIn('skip: 2',p[0]);self.assertIn('ChooseObjects',p[1]);self.assertEqual(p[-1],'Draw(PlayerSet::Controller, 1)')
        self.assertIsNone(generate.statement_program('If you control an Elf, add {G}.'))
        self.assertIsNone(generate.statement_program('If you control an Elf, you may discard a card. If you do, add {G}.'))

    def test_intervening_if_and_effect_if_have_different_typed_timing(self):
        prefix='When this creature enters, '
        entry=generate.trigger_creature_template(self.card(oracle_text=prefix+'if you control an artifact, draw a card.'))
        self.assertIn('InterveningIf',entry[2][0][0]);self.assertEqual(entry[2][0][1],['Draw(PlayerSet::Controller, 1)'])
        effect=generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card. If you control another Hero, you gain 2 life.'))
        self.assertEqual(effect[2][0][0],'SelfEnters');self.assertEqual(effect[2][0][1][0],'Draw(PlayerSet::Controller, 1)');self.assertIn('If {',effect[2][0][1][1])
        limited=generate.trigger_creature_template(self.card(oracle_text=prefix+'if you control an artifact, draw a card. This ability triggers only once each turn.'))
        self.assertTrue(limited[2][0][0].startswith('OncePerTurn(&TriggerCondition::InterveningIf'))
        for prefix in ['When this creature dies, ','Whenever another creature you control dies, ','When you discard this card, ']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=prefix+'if you control an Elf, draw a card.')))

    def test_condition_predicates_bind_live_counts_and_keep_unknown_history_gated(self):
        for text,needle in [('you control another Elf','other: true'),('you control an enchantment','kind: 32'),
                            ('there are seven or more cards in your graveyard','CountZone::Graveyard'),
                            ('there are four or more permanent cards in your graveyard','kind: 371'),
                            ('there are four or more card types among cards in your graveyard','CardTypes'),
                            ('you control another creature with power 4 or greater','GreatestOtherCreaturePower')]:
            self.assertIn(needle,generate.step_guard(text),text)
        for text in ['you controlled a Mount as you cast this spell','you controlled that permanent','a nontoken creature died this turn',
                     'you control another Elf that attacked this turn','there are four or more cards exiled with this creature',
                     'you control 65536 or more creatures','you control a tapped Elf']:
            self.assertIsNone(generate.step_guard(text),text)

    def test_focused_creature_tails_preserve_target_ownership_and_replace_instead_of_stack(self):
        result=generate.targeted_program('Target creature gets +2/+2 until end of turn. If you control three or more creatures, it gets +4/+4 until end of turn instead.')
        self.assertEqual(result[0],'Creature');self.assertEqual(len(result[1]),4)
        self.assertIn('If {',result[1][0]);self.assertIn('power: 4',result[1][1]);self.assertEqual(result[1][2],'Skip(1)');self.assertIn('power: 2',result[1][3])
        result=generate.targeted_program('Put a +1/+1 counter on target creature you control. It gains hexproof until end of turn.')
        self.assertEqual(result[0],'CreatureYou');self.assertIn('PutCounters',result[1][0]);self.assertIn('GrantKeywords',result[1][1])
        for text in ['Target player draws a card. It gains hexproof until end of turn.',
                     'Target creature gets +1/+1 until end of turn. If a creature left the battlefield this turn, it gains flying until end of turn.',
                     'Return target creature to its owner\'s hand. If you controlled that creature, draw a card.',
                     'Target creature gets +2/+2 until end of turn. If you control three or more creatures, it gets +4/+4 until end of turn instead. Draw a card.']:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_creature_pronouns_fail_closed_after_competing_or_unproven_antecedents(self):
        start='Target creature gets +1/+1 until end of turn. '
        token='create a 1/1 red Goblin creature token.'
        for pronoun in ['It','That creature']:
            for middle in [token[0].upper()+token[1:],
                           'If you control a Rabbit, '+token,
                           'If you control a Rabbit, draw a card. Otherwise, '+token,
                           'If you control a Rabbit, '+token+' You gain 2 life.',
                           'Draw a card.']:
                text=start+middle+' '+pronoun+' gains flying until end of turn.'
                self.assertIsNone(generate.targeted_program(text),text)
                self.assertIsNone(generate.spell_program_template(self.card(type_line='Instant',oracle_text=text)),text)
            for prefix in ["Return target creature to its owner's hand. ", 'Destroy target creature. ']:
                text=prefix+pronoun+' gains flying until end of turn.'
                self.assertIsNone(generate.targeted_program(text),text)
            text=start+'If you control a Rabbit, it gets +2/+2 until end of turn. '+pronoun+' gains flying until end of turn.'
            self.assertIsNotNone(generate.targeted_program(text),text)
        # Introducing a token is still supported when no later pronoun binds it.
        self.assertIsNotNone(generate.targeted_program(start+'If you control a Rabbit, '+token))

    def test_all_nineteen_conditional_and_focused_tail_admissions_require_complete_bodies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        for name in ['Crypt Feaster','Donatello, Turtle Techie',"Dwynen's Elite",'Failed Fording','Hero in Training','Malamet Veteran','Moment of Valor','Nasty Little Rabbit','Nessian Hornbeetle','Rabbit Response','Rosa, Resolute White Mage','Snakeskin Veil','Spectacular Tactics','Stinging Cave Crawler','Take Out the Trash','Taken by Nightmares','Violent Urge','Warg Tactics',"You're Not Alone"]:
            self.assertIn(name,supported)
            c=next(c for c in rows['cards'] if c['name']==name);c=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            self.assertIsNone(generate.permanent_program_template(c),name);self.assertIsNone(generate.spell_program_template(c),name);self.assertIsNone(generate.modal_spell_template(c),name)
        for name in ['Boomerang Basics','Steer Clear','Keen-Eyed Curator','Starving Revenant','Raucous Audience','Take the Fall','Dazzling Denial']:
            self.assertNotIn(name,supported)

    def test_distinct_characteristic_expressions_preserve_owner_zone_and_card_requirements(self):
        colors=generate.amount_value('the number of colors among permanents you control')
        self.assertIn('StatValue::Colors',colors);self.assertIn('CountZone::Battlefield',colors)
        types=generate.amount_value('the number of card types among cards in your graveyard')
        self.assertIn('StatValue::CardTypes',types);self.assertIn('CountZone::Graveyard',types);self.assertIn('cards_only: true',types)
        for text in ['colors of mana spent to cast this spell','colors in your color identity','colors among permanents your opponents control',
                     'card types among cards exiled with this creature','colors among cards in your hand']:
            self.assertIsNone(generate.amount_value(text),text)

    def test_scaled_draw_growth_and_drain_are_complete_defined_amounts(self):
        draw=generate.statement_program('draw X cards, where X is the number of card types among cards in your graveyard.')
        self.assertIn('DrawScaled',draw[0]);self.assertIn('StatValue::CardTypes',draw[0])
        draw=generate.statement_program('draw cards equal to the number of colors among permanents you control.')
        self.assertIn('StatValue::Colors',draw[0])
        drain=generate.statement_program('each opponent loses X life and you gain X life, where X is the number of colors among permanents you control.')
        self.assertEqual(len(drain),1);self.assertIn('DrainLifeScaled',drain[0])
        growth=generate.targeted_program('another target creature you control gets +X/+X until end of turn, where X is the number of colors among permanents you control.')
        self.assertEqual(growth[0],'OtherCreatureYou');self.assertIn('GrowScaled',growth[1][0])
        self.assertEqual(generate.statement_program('discard your hand.'),['DiscardHand(PlayerSet::Controller)'])
        for text in ['draw X cards.','draw cards equal to the number of cards in your hand.',
                     'target creature gets +X/+X until end of turn.',
                     'target creature gets +X/+X until end of turn, where X is the sacrificed creature\'s power.',
                     'each opponent loses X life and you gain that much life.',
                     'draw X cards, where X is the number of colors of mana spent to cast this spell.']:
            self.assertIsNone(generate.statement_program(text),text);self.assertIsNone(generate.targeted_program(text),text)

    def test_distinct_amount_eight_admissions_require_complete_card_bodies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        for name in ['Explosive Prodigy','Frontline Rush','Glister Bairn','Luminollusk','Peer Past the Veil','Shimmercreep','Shinestriker','Squawkroaster']:
            self.assertIn(name,supported)
            c=next(c for c in rows['cards'] if c['name']==name)
            c=dict(c,oracle_text=c['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            proxy,_,_=generate.extract_stat_rules(c)
            self.assertIsNone(generate.permanent_program_template(proxy),name)
            self.assertIsNone(generate.spell_program_template(proxy),name)
            self.assertIsNone(generate.modal_spell_template(proxy),name)
        for name in ['Aurora Awakener','Bloom Tender','Fblthp, Knows the Way','Kithkeeper','Prismabasher','Rime Chill','Prismatic Undercurrents','Puca\'s Eye','Savanti Romero, Time\'s Exile']:
            self.assertNotIn(name,supported)

    def test_scaled_damage_exact_amounts_targets_and_word_orders(self):
        for text,target,value in [
            ('this spell deals damage to target creature equal to the number of lands you control.','Creature','kind: 2'),
            ('this creature deals damage equal to the number of Swamps you control to any target.','Any','Some("Swamp")'),
            ('this creature deals damage to target creature an opponent controls equal to the number of Goblins you control.','CreatureOpponent','Some("Goblin")'),
            ('this spell deals damage equal to the number of creatures you control plus the number of Equipment you control to target creature.','Creature','StatValue::Sum'),
            ('this spell deals X damage to target player, where X is your life total.','Player','ControllerLife'),
        ]:
            result=generate.targeted_program(text);self.assertIsNotNone(result,text)
            self.assertEqual(result[0],target);self.assertEqual(len(result[1]),1)
            self.assertIn('DamageScaled',result[1][0]);self.assertIn(value,result[1][0])
        for noun,subtype in [('Gates','Gate'),('Treasures','Treasure'),('Islands','Island'),('Swamps','Swamp'),('Mountains','Mountain'),('Forests','Forest')]:
            self.assertIn('Some("'+subtype+'")',generate.amount_value(noun+' you control'))

    def test_scaled_life_and_unknown_amounts_are_not_inferred(self):
        p=generate.statement_program('you gain 2 life for each Gate you control.')
        self.assertIn('GainLifeScaled',p[0]);self.assertIn('factor: 2',p[0]);self.assertIn('Some("Gate")',p[0])
        self.assertIn('ControllerLife',generate.statement_program('you gain X life, where X is your life total.')[0])
        for text in [
            'this creature deals damage equal to its power to any target.',
            'this creature deals damage equal to the number of lands you control to each opponent.',
            'this creature deals X damage to any target.',
            'this creature deals damage equal to the number of Goblins that died this turn to any target.',
            'this spell deals damage equal to the number of creatures you control plus the damage dealt this turn to target creature.',
            'this creature deals damage equal to the number of Swamps you control to any other target.',
            'this spell deals damage to target creature equal to the number of lands you control. Exile it if it dies this turn.',
            'you gain life equal to the sacrificed creature\'s toughness.',
            'you gain 65536 life for each Gate you control.',
            'you gain 0 life for each Gate you control.',
        ]:
            self.assertIsNone(generate.targeted_program(text),text)
            self.assertIsNone(generate.statement_program(text),text)

    def test_scaled_effects_admit_all_eight_whole_bodies_and_reject_added_dependencies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target)
        supported={r['name'] for r in report['cards'] if r['implemented']}
        for name in ['Archway Angel','Cat-Gator','Coordinated Maneuver','Jet, Freedom Fighter','Rumbling Rockslide','Slash of Light','Smaug the Magnificent','Volley Veteran']:
            self.assertIn(name,supported)
            card=next(c for c in rows['cards'] if c['name']==name)
            card=dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            self.assertIsNone(generate.permanent_program_template(card),name)
            self.assertIsNone(generate.spell_program_template(card),name)
            self.assertIsNone(generate.modal_spell_template(card),name)
        for name in ['Smaug, Wicked Worm','Stadium Headliner','Barret Wallace','Razor Rings']:
            self.assertNotIn(name,supported)

    def test_damage_pronouns_require_self_event_and_exact_source_alias(self):
        card=self.card(name='Example the Great',type_line='Legendary Creature — Human',oracle_text='Whenever Example attacks, he deals damage equal to the number of lands you control to any target.')
        self.assertIsNotNone(generate.trigger_creature_template(card))
        for text in ['Whenever another creature you control enters, he deals damage equal to the number of lands you control to any target.',
                     'Whenever Someone Else attacks, he deals damage equal to the number of lands you control to any target.']:
            self.assertIsNone(generate.trigger_creature_template(dict(card,oracle_text=text)))

    def test_scaled_token_quantities_use_exact_shared_count_filters(self):
        cases={
            'create X 2/2 white Cat creature tokens, where X is your life total.':'StatValue::ControllerLife',
            'create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control.':'subtype: Some("Goblin")',
            'create X 2/2 colorless Wizard Soldier creature tokens named Cadet, where X is the amount of life you gained this turn.':'AmountValue::LifeGainedThisTurn',
            'for each land you control, create a Treasure token.':'kind: 2',
            'create a 1/1 green Insect creature token for each artifact and/or creature card in your graveyard.':'kind: 17',
            'create X 2/2 red Mutant creature tokens, where X is the number of nontoken creatures you control.':'cards_only: true',
            'create a 1/1 white Ally creature token for each Plains you control.':'subtype: Some("Plains")',
        }
        for text,expected in cases.items():
            program=generate.scaled_token_program(text);self.assertIsNotNone(program,text)
            self.assertEqual(len(program),1);self.assertIn('CreateTokensScaled',program[0]);self.assertIn(expected,program[0])
        self.assertIn('kind: 1',generate.stat_count_filter('nontoken creatures you control'))
        self.assertIn('cards_only: true',generate.stat_count_filter('nontoken creatures you control'))

    def test_scaled_token_unsupported_quantities_and_extra_factory_clauses_fail_closed(self):
        for text in [
            'create X 2/2 white Cat creature tokens.',
            'create X 2/2 white Cat creature tokens, where X is the amount of damage dealt this turn.',
            'create X 1/1 red Goblin creature tokens, where X is the number of Goblins that entered this turn.',
            'for each land you control, create two Treasure tokens.',
            'create a 1/1 red Goblin creature token with ward {2} for each land you control.',
            'create a 1/1 red Goblin creature token for each land your opponents control.',
            'create X tokens that are copies of target creature, where X is your life total.',
            'create X tapped and attacking 1/1 red Goblin creature tokens, where X is your life total.',
        ]:self.assertIsNone(generate.scaled_token_program(text),text)

    def test_scaled_token_whole_bodies_and_kindred_metadata_are_admitted_exactly(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        for name in ['Aatchik, Emerald Radian','Ajani, Caller of the Pride',"Brass's Bounty",'Clachan Festival','Gather the White Lotus','Germinate Recruits','Krenko, Mob Boss',"Moogles' Valor", "Morcant's Eyes",'Sally Pride, Lioness Leader','The Unbeatable Squirrel Girl']:self.assertIn(name,supported,name)
        for name in ['Anim Pakal, Thousandth Moon','Rise of the Varmints','Revenge of the Rats','Smaug, Wicked Worm','Twitching Doll','Kithkeeper']:self.assertNotIn(name,supported,name)
        self.assertEqual(generate.permanent_type(self.card(type_line='Kindred Enchantment — Elf')),'Enchantment')
        line=next(line for line in rust.splitlines() if 'name: "Morcant' in line)
        self.assertIn('type_line: "Kindred Enchantment — Elf"',line);self.assertIn('kind: 160',line)

    def test_scaled_token_factories_do_not_admit_partial_new_bodies(self):
        import json
        card=next(c for c in json.loads((generate.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name']=='Krenko, Mob Boss')
        self.assertIsNotNone(generate.permanent_program_template(card))
        self.assertIsNone(generate.permanent_program_template(dict(card,oracle_text=card['oracle_text']+'\nWhenever this creature attacks, discover 3.')))
        squirrel=next(c for c in json.loads((generate.ROOT/'data/standard/cards.json').read_text())['cards'] if c['name']=='The Unbeatable Squirrel Girl')
        self.assertIsNotNone(generate.permanent_program_template(squirrel))
        self.assertIsNone(generate.permanent_program_template(dict(squirrel,oracle_text=squirrel['oracle_text']+'\nCreatures you control have protection from red.')))

    def test_loyalty_symbols_are_costs_with_inherent_sorcery_timing(self):
        for cost,change in [('+2',2),('0',0),('−3',-3)]:
            card=self.card(type_line='Legendary Planeswalker — Example',loyalty='5',oracle_text=cost+': You gain 3 life.')
            result=generate.permanent_program_template(card);self.assertIsNotNone(result)
            ability=result[3][0];self.assertEqual(ability[0]['loyalty'],change)
            self.assertFalse(ability[0]['mana_component']);self.assertTrue(ability[3])
            self.assertIn('loyalty: Some('+str(change)+')',generate.rust_activations(result[3]))
        for cost in ['−X','+2147483648','−2147483649','-3','{T}','+2, {T}']:
            self.assertIsNone(generate.permanent_program_template(self.card(type_line='Planeswalker — Example',loyalty='5',oracle_text=cost+': You gain 3 life.')),cost)

    def test_loyalty_mana_compiles_as_a_finite_stack_program(self):
        result=generate.permanent_program_template(self.card(type_line='Planeswalker — Example',loyalty='5',oracle_text='+1: Add {R}{R}.'))
        self.assertIsNotNone(result);self.assertEqual(result[3][0][0]['loyalty'],1)
        self.assertTrue(result[3][0][2][0].startswith('AddMana'))

    def test_professor_admits_every_ability_and_only_referenced_emblem_factories(self):
        import json,copy,hashlib
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(rows,target)
        self.assertTrue(next(r for r in coverage['cards'] if r['name']=='Professor Dellian Fel')['implemented'])
        self.assertEqual(coverage['emblem_definitions'],1)
        self.assertIn('LoseGainedLife(PlayerSet::Targeted)',rust)
        for suffix in ['\n+1: Discover 3.','\nCreatures you control have protection from red.']:
            changed=copy.deepcopy(rows);card=next(c for c in changed['cards'] if c['name']=='Professor Dellian Fel');card['oracle_text']+=suffix
            changed['cards_sha256']=hashlib.sha256(json.dumps(changed['cards'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
            _,report=generate.compile_catalogue(changed,target)
            self.assertFalse(next(r for r in report['cards'] if r['name']=='Professor Dellian Fel')['implemented'])
            self.assertEqual(report['emblem_definitions'],0)

    def test_unknown_emblem_bodies_and_extra_clauses_fail_whole_card_admission(self):
        for effect in ['You get an emblem with "Whenever you gain life, each opponent loses that much life."',
                       'You get an emblem with "Whenever you gain life, target opponent loses that much life." Then draw a card.',
                       'You get an emblem with "Creatures you control get +2/+2."']:
            self.assertIsNone(generate.permanent_program_template(self.card(type_line='Planeswalker — Example',loyalty='5',oracle_text='−6: '+effect)))

    def test_loyalty_damage_unlocks_only_complete_bodies_and_preserves_walker_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(rows,target);supported={r['name'] for r in coverage['cards'] if r['implemented']}
        for name in ['Broadside Barrage','Dragonback Assault','Fulminous Forte','Sear','Splatter Technique']:
            self.assertIn(name,supported,name)
        for name in ['Vivien Reid','Chandra, Torch of Defiance','The Aetherspark','Liliana, Dreadhorde General']:
            self.assertNotIn(name,supported,name)
        self.assertIn('fn starting_loyalty(card: Card) -> u64',rust)
        self.assertEqual(generate.targeted_program('this spell deals 4 damage to target creature or planeswalker.'),('CreaturePlaneswalker',['Damage(4)']))
        self.assertIsNone(generate.targeted_program('this spell deals 4 damage to target creature, planeswalker, or battle.'))
        self.assertIn('kinds: 65',generate.statement_program('this spell deals 1 damage to each creature and planeswalker your opponents control.')[0])

    def test_invalid_starting_loyalty_fails_closed_with_fresh_data_checksum(self):
        import json,hashlib,copy
        original=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        for value in [None,5,'-1','18446744073709551616']:
            rows=copy.deepcopy(original);card=next(c for c in rows['cards'] if c['name']=='Professor Dellian Fel')
            if value is None:del card['loyalty']
            else:card['loyalty']=value
            rows['cards_sha256']=hashlib.sha256(json.dumps(rows['cards'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
            with self.assertRaisesRegex(ValueError,'starting loyalty: Professor Dellian Fel'):
                generate.compile_catalogue(rows,target)

    def test_battlefield_groups_are_whole_non_targeted_batches_with_frozen_results(self):
        cases={
            'Destroy all creatures.':'kinds: 1',
            'Destroy all tapped creatures.':'tapped: Some(true)',
            'Destroy all untapped creatures.':'tapped: Some(false)',
            'Exile all creatures and Spacecraft.':'Some("Spacecraft")',
            'Destroy all nonland permanents with mana value 1 or less.':'max_mana: 1',
            'Exile all artifacts with mana value 3 or less.':'kinds: 16',
        }
        for text,predicate in cases.items():
            program=generate.statement_program(text);self.assertEqual(len(program),1)
            self.assertIn('MoveBattlefield',program[0]);self.assertIn(predicate,program[0])
        self.assertEqual(generate.statement_program('Destroy all creatures. You gain 1 life for each creature destroyed this way.')[-1],'GainLifeMoved')
        self.assertEqual(generate.statement_program('Destroy all creatures and enchantments. Draw a card for each permanent destroyed this way.')[-1],'DrawMoved')
        for text in ['Destroy all creatures. You gain 1 life for each permanent destroyed this way.',
                     'Destroy all creatures and enchantments. Draw a card for each creature destroyed this way.',
                     'Destroy all creatures except Dragons.','Destroy all creatures with power 4 or less.',
                     'Destroy all creatures, then choose a creature type.',
                     'Destroy all creatures and Vehicles. They lose indestructible.']:
            self.assertIsNone(generate.statement_program(text),text)

    def test_group_damage_is_one_source_batch_with_planeswalker_sets(self):
        for text,expected in [
            ('this spell deals 2 damage to each creature.','kinds: 1'),
            ('this spell deals 2 damage to each creature without flying.','without: Keywords(1)'),
            ('this spell deals 1 damage to each opponent and each creature they control.','players: Some(PlayerSet::Opponent)'),
            ('this spell deals 3 damage to each creature your opponents control.','relation: 1'),
            ('this spell deals 2 damage to each creature and planeswalker.','kinds: 65')]:
            program=generate.statement_program(text);self.assertEqual(len(program),1);self.assertIn('DamageBattlefield',program[0]);self.assertIn(expected,program[0])
        self.assertEqual(generate.statement_program('this spell deals 3 damage to each player.'),['DamagePlayers(PlayerSet::Each, 3)'])
        for text in ['this spell deals 2 damage to each creature and battle.',
                     'this spell deals X damage to each creature.',
                     'this spell deals 32768 damage to each creature.',
                     'this spell deals 2 damage to each creature with flying.',
                     'this spell deals 2 damage to each non-Dragon creature.']:
            self.assertIsNone(generate.statement_program(text),text)
        body='Draw two cards. Sample Spell deals 2 damage to each player.'
        compiled=generate.spell_program_template(self.card(name='Sample Spell',type_line='Sorcery',oracle_text=body))
        self.assertEqual(compiled,('None',['Draw(PlayerSet::Controller, 2)','DamagePlayers(PlayerSet::Each, 2)']))
        self.assertIsNone(generate.spell_program_template(self.card(name='Other Spell',type_line='Sorcery',oracle_text=body)))

    def test_complete_battlefield_group_admissions_do_not_discard_missing_mechanics(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text());target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target);supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Beyond the Quiet','Cast Away Doubt','Day of Judgment','Death Begets Life','Extended Absence',
                     'Fuel the Flames','Fumigate','Pest Control','Pyroclasm','Ruinous Rampage',
                     'Seismic Rupture','Slagstorm','Split Up','Tectonic Hazard']:
            self.assertIn(name,supported,name)
        for name in ['No Witnesses',
                     'Kindred Judgment','Villainous Wrath','Expel the Interlopers','Settle the Wreckage',
                     'Spectacular Pileup','Singularity Rupture','Desolation of Smaug']:
            self.assertNotIn(name,supported,name)

    def test_removal_target_predicates_are_exact_and_unions_keep_qualifiers(self):
        cases={
            'Destroy target attacking or blocking creature.':'CreatureAttackingBlocking',
            'Exile target attacking creature.':'CreatureAttacking',
            'Destroy target creature with power or toughness 4 or greater.':'CreatureEitherMin4',
            'Destroy target artifact, enchantment, or creature with flying.':'ArtifactEnchantmentCreatureFlying',
            'Exile target artifact, enchantment, or creature with power 4 or greater.':'ArtifactEnchantmentCreaturePowerMin4',
            'Destroy target token an opponent controls.':'TokenOpponent',
            "Exile target card from an opponent's graveyard.":'CardGraveOpponent',
            'Destroy target non-outlaw creature.':'CreatureNonOutlaw',
            'Destroy target creature an opponent controls that was dealt damage this turn.':'CreatureDamagedOpponent',
            'Destroy target creature an opponent controls that dealt damage this turn.':'CreatureDealtDamageOpponent',
        }
        for text,rule in cases.items():
            self.assertEqual(generate.targeted_program(text),(rule,[text.split()[0]]))
        union=generate.TARGET_RULES['ArtifactEnchantmentCreatureFlying']
        self.assertIn('TargetRule::Union',union);self.assertIn('kinds: 48',union)
        self.assertIn('Keyword(Keywords(1))',union)
        for text in [
            'Destroy target creature an opponent controls that dealt combat damage this turn.',
            'Destroy target non-outlaw creature. Its controller draws a card.',
            'Destroy target creature with total power and toughness 4 or greater.',
            'Destroy target artifact, enchantment, or creature with flying and reach.',
            'Exile target creature or a card from a graveyard.',
            'Destroy target creature with power 3 or less.',
            'Destroy target creature. Its controller manifests dread.',
        ]:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_complete_removal_admissions_keep_other_mechanics_gated(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        admitted=["Airship Crash", "Battle Menu", "Broken Wings", "Bumbleflower's Sharepot",
            "Death in the Family", "Deathmark", "Devout Decree", "Disruptor Wanderglyph", "Embrace Oblivion",
            "Eriette's Lullaby", "Exorcise", "Gallant Strike", "Giant's Boulder", "Goblin Firebomb",
            "Grapple with Death", "Gravkill", "Hero's Downfall", "Kraul Whipcracker", "Make Your Move",
            "Meteor Golem", "Meteor Sword", "Mortify", "Not on My Watch", "Pawpatch Formation",
            "Radiant Strike", "Ray of Ruin", "Red Guardian, Super-Soldier", "Refute Destiny", "Repel Calamity", "Rooftop Assassin",
            "Sarkhan's Resolve", "Scout the City", "Scrap Compactor", "Shattered Wings", "Shoot the Sheriff",
            "Spin Out", "Stand Up for Yourself", "Stingblade Assassin", "Sudden Strike", "Terminal Criticism",
            "Unforgiving Aim", "Valorous Stance", "Withering Torment", "Worthy Cost", "Your Fate Ends Here"]
        self.assertEqual(len(admitted),45)
        for name in admitted:self.assertIn(name,supported)
        for name in ['Unwanted Remake',
                     'Consuming Ashes','Bovine Intervention','Get Lost',
                     'Ride\'s End','Protective Response','Dragon\'s Prey']:
            self.assertNotIn(name,supported,name)
        import re
        colors=json.loads(re.search(r'static COLORS: \[u8; [0-9]+\] = (\[[^;]+\]);',rust)[1])
        definitions=coverage['cards']
        for name,mask in [('Cat-Owl',3),('Forest',0),('Grizzly Bears',16),('Abandon Attachments',10)]:
            self.assertEqual(colors[next(i for i,c in enumerate(definitions) if c['name']==name)],mask)
        self.assertEqual(len(colors),len(definitions)+coverage['token_definitions']+coverage['emblem_definitions'])

    def test_missing_invalid_colors_fail_closed_even_with_fresh_checksum(self):
        import json,hashlib,copy
        original=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        for value in [None,['R','R'],['C'],[5]]:
            rows=copy.deepcopy(original)
            card=next(c for c in rows['cards'] if c['name']=='Bear Cub')
            if value is None:del card['colors']
            else:card['colors']=value
            rows['cards_sha256']=hashlib.sha256(json.dumps(rows['cards'],ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
            with self.assertRaisesRegex(ValueError,'colors: Bear Cub'):
                generate.compile_catalogue(rows,target)

    def test_counter_groups_match_whole_non_targeted_sets_and_distinct_target_ranges(self):
        cases=[('Put a +1/+1 counter on each creature you control.','relation: 0','PlusOne'),
               ('Put two -1/-1 counters on each creature.','relation: -1','MinusOne'),
               ('Put a stun counter on each creature token your opponents control.','tokens: Some(true)','Stun'),
               ('Put a +1/+1 counter on each other Hero you control.','other: true','PlusOne')]
        for text,predicate,kind in cases:
            result=generate.statement_program(text)
            self.assertEqual(len(result),1);self.assertIn('PutCreatureCounters',result[0]);self.assertIn(predicate,result[0]);self.assertIn('CounterKind::'+kind,result[0])
        for phrase,minimum,maximum in [('up to two',0,2),('up to three',0,3),('one or two',1,2),('two',2,2)]:
            for suffix,rule in [('', 'Creature'),(' you control','CreatureYou')]:
                result=generate.targeted_program('Put a +1/+1 counter on each of '+phrase+' target creatures'+suffix+'.')
                self.assertEqual(result[0],(rule,minimum,maximum));self.assertEqual(result[1],[generate.counter_instruction('Targets','a','+1/+1')])
        for text in ['Put a +1/+1 counter on each creature that entered this turn.',
                     'Put a +1/+1 counter on each creature with a +1/+1 counter on it.',
                     'Put a +1/+1 counter on each Hero and Villain you control.',
                     'Put a loyalty counter on each creature you control.',
                     'Put a +1/+1 counter on each of up to two target creatures with total power 4 or less.']:
            self.assertIsNone(generate.statement_program(text));self.assertIsNone(generate.targeted_program(text))
        self.assertEqual(generate.targeted_program('Double the number of each kind of counter on up to two target creatures and/or artifacts you control.'),(('ArtifactCreatureYou',0,2),['DoubleCounters']))
        for text in ['Double the number of +1/+1 counters on up to two target creatures and/or artifacts you control.',
                     'Double the number of each kind of counter on target permanent.',
                     'Put a +1/+1 counter on each other Hero you control. They gain flying.']:
            self.assertIsNone(generate.targeted_program(text));self.assertIsNone(generate.statement_program(text))

    def test_counter_group_complete_admissions_preserve_missing_mechanics(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Abandoned Air Temple','Agent Bishop, Man in Black','Agent Phil Coulson','Darkness Descends',
                     'Duty Beyond Death','Earth Kingdom Soldier','Lyra Dawnbringer','Lyra, Archangel of Dawn',
                     'Minwu, White Mage',"The Crystal's Chosen",'Web-Warriors','Zimone, Paradox Sculptor']:
            self.assertIn(name,supported)
        for name in ['Action News Crew','Atmospheric Greenhouse','Byrke, Long Ear of the Law',
                     'Sandstorm Salvager','Iron Spider, Stark Upgrade','Stirring Hopesinger',
                     'Vraska Joins Up','Wilderland Scrounger','Raucous Entertainer']:
            self.assertNotIn(name,supported)
        for noun in ['each Angel you control','Angels you control','each Cleric you control','Clerics you control']:
            self.assertIsNotNone(generate.creature_filter(noun))
        program=generate.statement_program('Create four 1/1 colorless Hero creature tokens. Then put a +1/+1 counter on each creature you control.')
        self.assertEqual(len(program),2);self.assertIn('CreateTokens',program[0]);self.assertIn('PutCreatureCounters',program[1])
        self.assertIsNone(generate.statement_program('Create four 1/1 colorless Hero creature tokens. Then if you attacked, put a +1/+1 counter on each creature you control.'))

    def test_mandatory_additional_costs_are_exact_and_complete(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        rust,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Corrupted Conviction','Final Vengeance','Thrill of Possibility']:
            self.assertIn(name,supported)
        for name in ['Grab the Prize','Laughing Mad','Pumpkin Bombardment','Souls Lost','Fear of Isolation']:
            self.assertNotIn(name,supported)
        self.assertEqual(rust.count('=> Some(AdditionalCost {'),7)
        prefix='As an additional cost to cast this spell, '
        for cost in ['discard two cards.','you may discard a card.','sacrifice two creatures.',
                     'discard a card or pay {2}.','sacrifice an artifact or pay {2}.',
                     'sacrifice this creature.','return a permanent you control to its owner’s hand.']:
            card=self.card(oracle_text=prefix+cost+'\nDraw two cards.')
            self.assertEqual(generate.extract_additional_cost(card),(card,None))
        card=self.card(oracle_text=prefix+'discard a card.\n'+prefix+'discard a card.\nDraw two cards.')
        self.assertEqual(generate.extract_additional_cost(card),(card,None))
        card=self.card(type_line='Instant',oracle_text=prefix+'discard a card.\nDraw two cards.\nFlashback {2}{R}')
        residual,cost=generate.extract_additional_cost(card)
        self.assertIsNotNone(cost);self.assertIsNone(generate.spell_program_template(residual))

    def test_changeling_intrinsic_flags_never_enable_dynamic_grants_or_type_removal(self):
        result=generate.creature_template(self.card(oracle_text='Changeling (This card is every creature type.)\nReach'))
        self.assertEqual(result[1],524288|2)
        result=generate.trigger_creature_template(self.card(oracle_text='Changeling\nWhen this creature enters, draw a card.'))
        self.assertEqual(result[1],524288)
        token=generate.token_program('create a 1/1 colorless Shapeshifter creature token with changeling.')
        self.assertIsNotNone(token)
        self.assertTrue(any(t['keywords']==524288 for t in generate.TOKEN_DEFS.values()))
        for text in ['Target creature gains changeling until end of turn.',
                     'Target creature gets +3/-3 and loses all creature types until end of turn.',
                     'This creature gains changeling until end of turn.']:
            self.assertIsNone(generate.targeted_program(text));self.assertIsNone(generate.statement_program(text))
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='Changeling\n{0}: This creature gains changeling until end of turn.')))
        self.assertIsNone(generate.creature_template(self.card(oracle_text='Changeling\nThis creature loses all creature types.')))

    def test_changeling_shared_optional_programs_require_complete_shapes(self):
        self.assertEqual(generate.targeted_program('Destroy up to one target artifact or enchantment.'),(('ArtifactEnchantment',0,1),['Destroy']))
        self.assertEqual(generate.targeted_program("Return up to one other target creature you control to its owner's hand."),(('OtherCreatureYou',0,1),['ReturnToHand']))
        self.assertEqual(generate.targeted_program('Exile up to two target cards from graveyards. You gain 3 life.'),(('CardGrave',0,2),['Exile','GainLife(PlayerSet::Controller, 3)']))
        code=generate.statement_program('You may search your library for a basic land card, reveal it, put it into your hand, then shuffle.')
        self.assertEqual(code[0],'May { skip: 1 }');self.assertIn('SearchFilter::BasicLand',code[1])
        for text in ['Destroy up to two target artifacts or enchantments.',
                     'Exile up to two target cards from graveyards. If two cards were exiled, you gain 3 life.',
                     "Return up to one other target creature you control to its owner's hand. If you do, draw a card."]:
            self.assertIsNone(generate.targeted_program(text),text)
        self.assertIsNone(generate.statement_program('You may search your library for a basic land card, reveal it, put it into your hand, then shuffle. If you do, draw a card.'))

    def test_creature_type_snapshot_is_validated_and_changes_catalogue_fingerprint(self):
        import copy,json,tempfile
        from unittest.mock import patch
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        taxonomy=json.loads((generate.ROOT/'data/creature-types.json').read_text())
        self.assertEqual(len(taxonomy['types']),324)
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'data').mkdir();path=root/'data/creature-types.json'
            path.write_text(json.dumps(taxonomy))
            with patch.object(generate,'ROOT',root):
                _,before=generate.compile_catalogue(rows,target)
                changed=copy.deepcopy(taxonomy);changed['normalization']+=' Reviewed.';path.write_text(json.dumps(changed))
                _,after=generate.compile_catalogue(rows,target)
                self.assertNotEqual(before['catalogue_version'],after['catalogue_version'])
                for types in [[],['Elf','Elf'],['Zombie','Elf'],['Elf Warrior']]:
                    changed['types']=types;path.write_text(json.dumps(changed))
                    with self.assertRaisesRegex(ValueError,'creature types'):generate.compile_catalogue(rows,target)

    def test_changeling_complete_admissions_preserve_missing_mechanics(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Changeling Wayfinder','Chitinous Graspling','Chomping Changeling','Curious Farm Animals',
                     'Environmental Scientist','Flock Impostor','Gangly Stompling','Graveshifter',
                     'Mischievous Sneakling','Prideful Feastling','Rooftop Percher','Three Tree Mascot']:
            self.assertIn(name,supported)
        for name in ['Nameless Inversion','Barkform Harvester','Taurean Mauler','Firdoch Core',
                     'Crib Swap','Sizzling Changeling','Mutable Explorer','Omni-Changeling']:
            self.assertNotIn(name,supported)

    def test_hybrid_spell_costs_enumerate_unique_nonhybrid_equivalents(self):
        self.assertEqual(generate.spell_cost_options('{G/W}{G/W}'),[
            ([0,0,0,0,2,0],0),([1,0,0,0,1,0],0),([2,0,0,0,0,0],0)])
        self.assertEqual(len(generate.spell_cost_options('{W/U}{U/B}')),4)
        for cost in ['{3}{R}{R}','{0}','{2}{C}{G}']:
            self.assertEqual(generate.spell_cost_options(cost),[generate.fixed_cost(cost)])
        for pair in ['W/U','W/B','U/B','U/R','B/R','B/G','R/G','R/W','G/W','G/U']:
            costs=generate.spell_cost_options('{1}{'+pair+'}{'+pair+'}{'+pair+'}')
            self.assertEqual(len(costs),4)
            self.assertTrue(all(sum(colored)+generic==4 for colored,generic in costs))
        for cost in ['{3/W}','{W/P}','{W/U/P}','{C/W}','{S}','{X}','{W/W}','{W/G}',
                     '{W/U}junk','','{4294967296}{G/W}']:
            self.assertIsNone(generate.spell_cost_options(cost),cost)
        self.assertIsNone(generate.fixed_cost('{G/W}'))
        self.assertIsNone(generate.mana_cost_spec('{G/W}'))
        self.assertEqual(len(generate.activation_cost('{G/W}')['alternatives']),1)

    def test_two_brid_choices_vary_generic_cost_without_changing_canonical_mana_value(self):
        self.assertEqual(generate.spell_cost_options('{2/W}'),[([0,0,0,0,0,0],2),([1,0,0,0,0,0],0)])
        self.assertEqual(generate.spell_cost_options('{1}{2/R}{2/R}'),[
            ([0,0,0,0,0,0],5),([0,0,0,1,0,0],3),([0,0,0,2,0,0],1)])
        for cost,printed in [('{2/W}{2/B}{2/G}',6),('{W/U}{2/W}{2/U}',5),('{C}{2/G}{R/G}',4)]:
            options=generate.spell_cost_options(cost)
            self.assertEqual(sum(options[0][0])+options[0][1],printed)
            self.assertEqual(len(options),len({(tuple(c),n) for c,n in options}))
            self.assertTrue(all(sum(c)+n<=printed for c,n in options))
        for cost in ['{W/2}','{2/C}','{2/2}','{3/W}','{2/W/P}','{4294967294}{2/G}']:
            self.assertIsNone(generate.spell_cost_options(cost),cost)
        for text in ['{2/W/P}: Draw a card.','Cycling {2/W}','Ward {2/W}']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=text)),text)

    def test_three_two_brid_admissions_require_complete_bodies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,report=generate.compile_catalogue(rows,target);supported={r['name'] for r in report['cards'] if r['implemented']}
        for name in ['Kin-Tree Severance','Reputable Merchant','Temur Tawnyback']:
            self.assertIn(name,supported)
            card=next(c for c in rows['cards'] if c['name']==name)
            extra=dict(card,oracle_text=card['oracle_text']+'\nWhenever you cast a spell, discover 3.')
            self.assertIsNone(generate.spell_program_template(extra),name)
            self.assertIsNone(generate.permanent_program_template(extra),name)
        for name in ['Dragonclaw Strike','Gurmag Nightwatch','Karn, Gilded Guardian',
                     'Magmablood Archaic','Monastery Messenger',"Rakshasa's Bargain",'Reigning Victor','Riverwheel Sweep','Wildgrowth Archaic']:
            self.assertNotIn(name,supported)

    def test_hybrid_spell_admission_never_approximates_other_symbols_or_ability_costs(self):
        row=self.card(mana_cost='{1}{G/W}',oracle_text='When this creature enters, draw a card.')
        self.assertIsNotNone(generate.trigger_creature_template(row))
        self.assertIsNotNone(generate.creature_template(dict(row,oracle_text='')))
        for cost in ['{3/W}','{G/W/P}','{X}{G/W}','{S}{G/W}']:
            self.assertIsNone(generate.trigger_creature_template(dict(row,mana_cost=cost)))
        for text in ['{G/W/P}: Draw a card.','Cycling {G/W}',
                     'Ward {G/W}','When this creature enters, draw a card. Then discover 2.']:
            self.assertIsNone(generate.permanent_program_template(dict(row,oracle_text=text)),text)
        self.assertIsNone(generate.trigger_creature_template(dict(row,layout='adventure')))
        spell=self.card(type_line='Instant',mana_cost='{U/R}',oracle_text='Draw a card.')
        self.assertIsNotNone(generate.spell_program_template(spell))
        modal=dict(spell,oracle_text='Choose one —\n• Draw a card.\n• You gain 2 life.')
        self.assertIsNotNone(generate.modal_spell_template(modal))

    def test_hybrid_complete_admissions_preserve_other_card_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Abandon Attachments','Cat-Owl','Duskwatch Hunter','Foot Elite','Gallant Citizen',
                     'Head of the Homestead','Large Bear','Mary Jane Watson','Mechanized Ninja Cavalry','Mob Lookout',
                     'Moonrise Cleric','Noggle Robber','Pond Prophet','Pretending Poxbearers','Punk Frogs',
                     'Slithering Cryptid','Tempest Angler','Wandering Musicians']:
            self.assertIn(name,supported)
        for name in ['Mouser Mark III',
                     'Sokka, Lateral Strategist','Nori, Teller of Tales']:
            self.assertNotIn(name,supported)

    def test_once_per_turn_wraps_complete_events_and_rejects_other_limit_meanings(self):
        for prefix in ['Whenever you draw a card, ', 'Whenever you discard a card, ',
                       'Whenever you gain life, ', 'Whenever you cast your second spell each turn, ',
                       'Whenever this creature attacks, ', 'At the beginning of your end step, ',
                       'Whenever a creature you control deals combat damage to a player, ']:
            base=generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card.'))
            limited=generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card. This ability triggers only once each turn.'))
            self.assertEqual(limited[2][0],('OncePerTurn(&TriggerCondition::'+base[2][0][0]+')',*base[2][0][1:]))
        for text in ['Whenever you gain life, draw a card. Do this only once each turn.',
                     'Whenever you gain life, draw a card. This ability triggers only twice each turn.',
                     'Whenever you gain life, draw a card. This ability triggers only once.',
                     'Flying This ability triggers only once each turn.',
                     'Prowess This ability triggers only once each turn.',
                     'Whenever you gain life, draw a card. This ability triggers only once each turn. You gain 1 life.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_entry_subtypes_are_exact_and_once_limited_batches_have_no_ambiguous_binding(self):
        for prefix,condition in [('Whenever a Spider you control enters, ','SubtypeEntersYou("Spider")'),
                                 ('Whenever another Faerie you control enters, ','OtherSubtypeEntersYou("Faerie")'),
                                 ('Whenever an Aura you control enters, ','SubtypeEntersYou("Aura")'),
                                 ('Whenever an artifact you control enters, ','ArtifactEntersYou'),
                                 ('Whenever an enchantment you control enters, ','EnchantmentEntersYou')]:
            self.assertEqual(generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card.'))[2][0][0],condition)
        prefixes=['Whenever one or more artifacts you control enter, ',
                  'Whenever one or more enchantments you control enter, ',
                  'Whenever one or more nontoken creatures you control die, ']
        for prefix in prefixes:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card.')))
            self.assertIsNotNone(generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card. This ability triggers only once each turn.')))
        for text in ['Whenever a Spider or Villain you control enters, draw a card.',
                     'Whenever another nontoken Bird with flying you control enters, draw a card.',
                     'Whenever one or more players discard one or more cards, draw that many cards. This ability triggers only once each turn.',
                     'Whenever one or more players discard one or more cards, that player loses 1 life. This ability triggers only once each turn.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_limited_trigger_complete_admissions_and_residual_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Avatar Enthusiasts','Bloombrute','Donatello, Way with Machines','Elvish Archivist','Fateful Discovery',
                     'Flying Octobot','Homicide Investigator','Hostile Investigator','Koth of the Homestead','Mechan Assembler',
                     "Mm'menon, Uthros Exile",'Obyra, Dreaming Duelist','Rimefur Reindeer','Tanglespan Lookout',
                     'Tidus, Blitzball Star','Youthful Valkyrie']:
            self.assertIn(name,supported)
        for name in ['Crossbones, Malicious Mercenary','Caretaker\'s Talent','Enduring Innocence',
                     'Twilight Diviner','Mikey & Leo, Chaos & Order','Deepmuck Desperado']:
            self.assertNotIn(name,supported)

    def test_connive_source_pronouns_names_and_fixed_amounts_require_bound_sources(self):
        for pronoun in ['it','he','she']:
            row=self.card(oracle_text='When this creature enters, '+pronoun+' connives.')
            self.assertEqual(generate.trigger_creature_template(row)[2][0][1],['Connive { subject: Subject::Source, amount: 1 }'])
        row=self.card(name='Test, Named Villain',type_line='Legendary Creature — Human Villain',oracle_text='Whenever Test attacks, he connives.')
        result=generate.trigger_creature_template(row)
        self.assertEqual(result[2][0][0],'SelfAttacks');self.assertIn('Connive',result[2][0][1][0])
        for amount in [0,1,3,65535]:
            self.assertEqual(generate.statement_program(f'This creature connives {amount}.'),[f'Connive {{ subject: Subject::Source, amount: {amount} }}'])
        for text in ['This creature connives 65536.','This creature connives X.','It connives.','That creature connives.']:
            self.assertIsNone(generate.statement_program(text),text)
        self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='Whenever another Villain you control enters, it connives.')))
        row=self.card(name='Test, Named Villain',type_line='Legendary Creature — Human Villain',oracle_text='Whenever another Villain you control enters, Test connives.')
        self.assertEqual(generate.trigger_creature_template(row)[2][0][0],'OtherSubtypeEntersYou("Villain")')

    def test_targeted_connive_groups_and_optional_graveyard_exile_are_complete(self):
        self.assertEqual(generate.targeted_program('Target creature you control connives.'),('CreatureYou',['Connive { subject: Subject::Targets, amount: 1 }']))
        self.assertEqual(generate.targeted_program('Up to one target creature connives 2.'),(('Creature',0,1),['Connive { subject: Subject::Targets, amount: 2 }']))
        groups,program=generate.targeted_program('Target player draws a card, then up to one target creature you control connives.')
        self.assertEqual(groups,['Player',('CreatureYou',0,1)]);self.assertEqual(program[1],'SelectGroup(1)')
        groups,program=generate.targeted_program('Exile up to one target card from a graveyard. Target creature you control connives.')
        self.assertEqual(groups,[('CardGrave',0,1),'CreatureYou']);self.assertEqual(program[0],'Exile');self.assertEqual(program[1],'SelectGroup(1)')
        for text in ['Target creature connives X.','Target creature connives 65536.',
                     'Exile up to one target card from a graveyard. If it was a creature card, draw a card.',
                     'Target player draws a card, then up to one target creature you control connives. If you discarded a creature card, draw a card.']:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_connive_and_shared_vocabulary_complete_admissions_gate_residuals(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['A.I.M. Scientists','Afterthought Sentry','Ambush Wolf','Ascendant Dustspeaker',"Doc Ock's Henchmen",'Heritage Reclamation',
                     'Kang, Temporal Tyrant','Madame Masque','Mechanical Mobster','Prowler, Clawed Thief','Red Room Recruit',
                     'Startled Relic Sloth',"Syr Alin, the Lion's Claw",'Unstable Experiment','Wreck Remover','Yellowjacket, Heartless Marauder']:
            self.assertIn(name,supported)
        for name in ['Baron Helmut Zemo','Baron Strucker, HYDRA Overlord','Leader, Super-Genius','M.O.D.O.K.',
                     'Swordsman, Sharp Scoundrel',"Trickster's Stratagem",'Villainous Hideout']:
            self.assertNotIn(name,supported)

    def test_draw_conditions_and_event_player_are_bound_only_to_draw_triggers(self):
        for prefix,relation,ordinal in [
            ('Whenever you draw a card, ','You',None),('Whenever you draw your first card each turn, ','You',1),
            ('Whenever you draw your second card each turn, ','You',2),('Whenever an opponent draws a card, ','Opponent',None),
            ('Whenever an opponent draws their second card each turn, ','Opponent',2),
            ('Whenever a player draws their second card each turn, ','Any',2),
        ]:
            row=self.card(oracle_text=prefix+'draw a card.')
            self.assertEqual(generate.trigger_creature_template(row)[2][0][0],generate.draw_condition(relation,ordinal))
        for body,program in [('that player loses 1 life.','LoseLife(PlayerSet::DrawingPlayer, 1)'),
                             ('this creature deals 2 damage to them.','DamagePlayers(PlayerSet::DrawingPlayer, 2)')]:
            self.assertEqual(generate.draw_event_program(body),('None',[program]))
            self.assertIsNone(generate.statement_program(body))
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='When this creature enters, '+body)))
        for text in ['Whenever you draw your second card each turn, if you control another Hero that attacked this turn, draw a card.',
                     'Whenever you draw a card, that player loses 1 life. You gain 1 life.',
                     'Whenever you draw a card, you may pay {B}. If you do, draw a card.',
                     'Whenever you draw a card, reveal it.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)
        self.assertIsNone(generate.draw_event_program('that player loses 65536 life.'))
        self.assertIsNone(generate.draw_event_program('this creature deals 32768 damage to them.'))

    def test_draw_milestone_complete_admissions_preserve_residual_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Atlantean Cavalry','Bard the Bowman','Clinquant Skymage','Erudite Wizard','Gleaming Splendor',
                     'Knowledge Seeker','Lakeshore Apothecary','Lightfoot Technique','Loxodon Eavesdropper','Maximum Overdrive',
                     'Mischievous Mystic','Otter-Penguin','Ravenhill Flock','Roxxon Brutes','Scrawling Crawler','Take Up the Shield','Tiger-Seal']:
            self.assertIn(name,supported)
        for name in ['Aether Syphon','Construct a Cosmic Cube','Foggy Swamp Spirit Keeper','Homunculus Horde','Human Torch, Johnny Storm',
                     'Jaded Analyst','Kid Loki','Lady Octopus, Inspired Inventor',
                     "Master's Councillors",'Moon Girl and Devil Dinosaur','Private Eye','Raven Eagle','Razorkin Needlehead',
                     'Starving Revenant','The Astonishing Ant-Man','The Unagi of Kyoshi Island','Thopter Fabricator','Wolfbat']:
            self.assertNotIn(name,supported)

    def test_counter_then_target_keywords_and_two_distinct_player_draws_require_complete_text(self):
        rule,program=generate.targeted_program('Put a +1/+1 counter on target creature. It gains lifelink until end of turn.')
        self.assertEqual(rule,'Creature');self.assertEqual(len(program),2)
        self.assertEqual(generate.targeted_program('Two target players each draw a card.'),(('Player',2,2),['Draw(PlayerSet::Targeted, 1)']))
        self.assertEqual(generate.statement_program('Tap this creature.'),['TapSource'])
        self.assertIsNotNone(generate.statement_program("This creature gets +1/+2 until end of turn and can't be blocked this turn."))
        for text in ['Two target players each draw a card unless they pay {1}.',
                     'Put a +1/+1 counter on target creature. It gains ward {2} until end of turn.',
                     'Put a +1/+1 counter on target creature. It gains flying until end of turn. Draw a card.',
                     'Put a +1/+1 counter on target creature. It gains flying until your next turn.']:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_discard_trigger_prefixes_and_amount_bindings_are_cause_specific(self):
        for prefix,condition in [
            ('When you discard this card, ','SelfDiscarded'),('When you cycle this card, ','SelfCycled'),
            ('Whenever you discard a card, ','Discarded { relation: CastRelation::You, one_or_more: false }'),
            ('Whenever you discard one or more cards, ','Discarded { relation: CastRelation::You, one_or_more: true }'),
            ('Whenever an opponent discards a card, ','Discarded { relation: CastRelation::Opponent, one_or_more: false }'),
            ('Whenever a player discards one or more cards, ','Discarded { relation: CastRelation::Any, one_or_more: true }'),
        ]:
            result=generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card.'))
            self.assertEqual(result[2][0][0],condition)
        for body,expected in [('put that many +1/+1 counters on this creature.','PutDiscardCounters(CounterKind::PlusOne)'),
                              ('this creature deals that much damage to each opponent.','DamageDiscardAmount(PlayerSet::Opponent)')]:
            row=self.card(oracle_text='Whenever you discard one or more cards, '+body)
            self.assertEqual(generate.trigger_creature_template(row)[2][0][1],[expected])
            self.assertIsNone(generate.statement_program(body))
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='When this creature enters, '+body)))
        for text in ['Whenever you discard a nonland card, draw a card.',
                     'When you cycle this card, draw cards equal to the amount of mana spent to cycle it.',
                     'Whenever you discard one or more cards, put that many +1/+1 counters on target creature.',
                     'Whenever you discard a card, exile that card from your graveyard.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_cycling_marker_and_creature_vehicle_target_scope(self):
        _,a=generate.cycling_abilities(self.card(oracle_text='Cycling {2}'))
        self.assertTrue(a[0][0]['cycling']);self.assertIn('cycling: true',generate.rust_activations(a))
        ordinary=generate.permanent_program_template(self.card(oracle_text='{1}: Draw a card.'))
        self.assertIn('cycling: false',generate.rust_activations(ordinary[3]))
        rule,program=generate.targeted_program('Target creature or Vehicle an opponent controls gets -3/-0 until end of turn.')
        self.assertEqual(rule,'CreatureVehicleOpponent');self.assertIn('power: -3',program[0])
        rule,program=generate.targeted_program('Put two +1/+1 counters on up to one target creature or Vehicle. It gains trample and indestructible until end of turn.')
        self.assertEqual(rule,('CreatureVehicle',0,1));self.assertEqual(len(program),2)
        for text in ['Put two +1/+1 counters on up to one target creature or Vehicle. It gains trample until end of turn. Draw a card.',
                     'Target creature or Vehicle an opponent controls gets -3/-0 until your next turn.',
                     'Put two +1/+1 counters on up to one target creature or Vehicle. It gains ward {2} until end of turn.']:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_discard_and_cycling_complete_admissions_gate_residual_mechanics(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Agonasaur Rex','Hobgoblin, Mantled Marauder',"Howler's Heavy",'Magmakin Artillerist',
                     'Marauding Mako','Scrounging Skyray','Tinybones, Pocket Nuisance','Titanbones, Towering Heart']:
            self.assertIn(name,supported)
        for name in ["Basri, Tomorrow's Champion",'Captain Howler, Sea Scourge','Cryptcaller Chariot','Inti, Seneschal of the Sun',
                     'Monument to Endurance','Moonstone, Harsh Mistress','Tinybones, Bauble Burglar','Webstrike Elite',"Valor's Flagship"]:
            self.assertNotIn(name,supported)

    def test_cycling_extracts_only_complete_fixed_cost_hand_abilities(self):
        row=self.card(oracle_text='Flying\nCycling {1}{U} ({1}{U}, Discard this card: Draw a card.)\nBasic landcycling {2}')
        residual,abilities=generate.cycling_abilities(row)
        self.assertEqual(residual['oracle_text'],'Flying')
        self.assertEqual(len(abilities),2)
        self.assertEqual(abilities[0][0]['origin'],'Hand');self.assertTrue(abilities[0][0]['discard'])
        self.assertEqual(abilities[0][0]['colored'],[0,1,0,0,0,0])
        self.assertEqual(abilities[0][2],['Draw(PlayerSet::Controller, 1)'])
        self.assertIn('SearchFilter::BasicLand',abilities[1][2][0])
        for kind in ['Plains','Island','Swamp','Mountain','Forest','Halfling']:
            _,a=generate.cycling_abilities(self.card(oracle_text=kind+'cycling {2}'))
            self.assertIn('SearchFilter::Subtype("'+kind+'")',a[0][2][0])
        for text in ['Cycling {X}{G}','Cycling—Pay 2 life.','Cycling {4294967296}',
                     'Cycling {2}.','Other creatures have cycling {2}.','Cycling {2} only on your turn']:
            residual,a=generate.cycling_abilities(self.card(oracle_text=text))
            self.assertEqual(residual['oracle_text'],text);self.assertEqual(a,[])
        row=self.card(layout='transform',oracle_text='Cycling {2}')
        self.assertEqual(generate.cycling_abilities(row),(row,[]))
        rust=generate.rust_activations(abilities)
        self.assertIn('origin: ActivationOrigin::Hand',rust);self.assertIn('discard: true',rust)

    def test_cycling_complete_admissions_preserve_residual_rules(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Apex Witchstalker','Balamb T-Rexaur','Bedhead Beastie','Borough Backup','Canyon Crawler','Capital City',
                     'Cloudbound Moogle','Daggermaw Megalodon','Hill Gigas','Hobbit Hole','Jennika, Bad Apple Big Sister',
                     'Kree Sentinel','Lightshield Parry','Locust Spray','Marauding Brinefang','Migrating Ketradon','Mongoose Lizard',
                     'Nurturing Bristleback','Rampaging Spiketail','Saber-Tooth Moose-Lion','Savage Land Dinosaur','Shefet Archfiend',
                     'Shepherding Spirits','Skycrash','Slavering Branchsnapper','Soaring Sandwing','Stockman, Mad Fly-entist',
                     'Stratosoarer','Topiary Panther','Undulating Witness','Vinelasher Adept']:
            self.assertIn(name,supported)
        for name in ["Basri, Tomorrow's Champion",'Webstrike Elite',"Valor's Flagship",'Night Market','Clamorous Ironclad']:
            self.assertNotIn(name,supported)

    def test_cycling_combines_with_battlefield_abilities_without_overflowing_slots(self):
        import hashlib,json
        def compile_text(text):
            cards=[self.card(name='Test Cycler',oracle_id='test-cycler',oracle_text=text)]
            canonical=json.dumps(cards,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
            snapshot=dict(cards=cards,card_count=1,cards_sha256=hashlib.sha256(canonical).hexdigest())
            return generate.compile_catalogue(snapshot,dict(banned_card_names=[]))
        rust,c=compile_text('{1}: Draw a card.\nCycling {2}\nCycling {U}')
        self.assertTrue(next(c for c in c['cards'] if c['name']=='Test Cycler')['implemented'])
        self.assertIn('origin: ActivationOrigin::Battlefield',rust);self.assertIn('origin: ActivationOrigin::Hand',rust)
        _,c=compile_text('{1}: Draw a card.\n'+'\n'.join(['Cycling {2}']*64))
        self.assertFalse(next(c for c in c['cards'] if c['name']=='Test Cycler')['implemented'])

    def test_mana_ward_is_triggered_and_repeated_instances_are_preserved(self):
        row=self.card(oracle_text='Flying, ward {2}\nWard {U}\nWhen this creature enters, draw a card.')
        result=generate.trigger_creature_template(row)
        self.assertEqual(result[1],1)
        self.assertEqual([a[0] for a in result[2]],['Ward','Ward','SelfEnters'])
        self.assertIn('generic: 2',result[2][0][1][0]);self.assertIn('CounterSubject::Related',result[2][0][1][0])
        self.assertIn('colored: [0, 1, 0, 0, 0, 0]',result[2][1][1][0])
        for text in ['Ward—Discard two cards.','Ward—Pay X life.','Ward {X}','Ward {2}, pay 2 life',
                     'Ward {2}\nOther creatures have ward {2}.','Equipped creature has ward {1}.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)
        self.assertIsNone(generate.mana_cost_spec('{4294967296}'))

    def test_counter_payment_subjects_conditions_and_spell_filters(self):
        for text,rule in [('Counter target creature spell.','SpellCreature'),('Counter target noncreature spell.','SpellNoncreature'),
                          ('Counter target instant or sorcery spell unless its controller pays {3}.','SpellInstantSorcery')]:
            self.assertEqual(generate.targeted_program(text)[0],rule)
        program=generate.targeted_program("Counter target spell unless its controller pays {3}. If that spell is countered this way, exile it instead of putting it into its owner's graveyard.")[1]
        self.assertIn('exile: true',program[0])
        program=generate.targeted_program('Counter target spell unless its controller pays {2}. If they do, surveil 2.')[1]
        self.assertIn('unpaid_skip: 1',program[0]);self.assertEqual(program[1],'Surveil(2)')
        program=generate.targeted_program('Counter target spell unless its controller pays {2}. If they do, you create a Lander token.')[1]
        self.assertIn('CreateTokens',program[1])
        for text in ['Counter target spell unless its controller pays {X}.','Counter target spell unless its controller pays {2} for each Faerie you control.',
                     'Counter target spell unless its controller pays {2}. If they do, draw a card. You gain 1 life.',
                     'Counter target spell unless its controller pays {2}. If a Dragon was beheld, counter that spell unless its controller pays {4} instead.',
                     'Counter target activated or triggered ability unless its controller pays {2}.']:
            self.assertIsNone(generate.targeted_program(text),text)

    def test_ward_and_resolution_payment_admissions_require_complete_cards(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Archive Dragon','Confusticate and Bebother','Diversion Unit','Divert Disaster',"Don't Make a Sound",'Essence Scatter',
                     'Hulking Raptor',"It'll Quench Ya!",'Negate','No More Lies','Spell Pierce','Spider-Rex, Daring Dino',
                     'Theorix Charm','Toadstool Admirer','Unflinching Hortimancer','Wrecking Gecko','Vein Ripper','Sire of Seven Deaths']:
            self.assertIn(name,supported)
        for name in ['Dazzling Denial','Spell Stutter',"Falcon's Wing Harness"]:
            self.assertNotIn(name,supported)

    def test_player_damage_filters_and_event_amount_scope(self):
        for header,subject,combat,opponent,one in [
            ('Whenever this creature deals combat damage to a player, ','Source',True,False,False),
            ('Whenever equipped creature deals combat damage to a player, ','Attached',True,False,False),
            ('Whenever this creature deals damage to an opponent, ','Source',False,True,False),
            ('Whenever a modified creature you control deals combat damage to a player, ','Modified',True,False,False),
            ('Whenever one or more creatures you control with +1/+1 counters on them deal combat damage to a player, ','PlusOne',True,False,True),
            ('Whenever a creature you control with a counter on it deals combat damage to a player, ','Counter',True,False,False),
            ('Whenever one or more Heroes you control deal damage to a player, ','Hero',False,False,True),
        ]:
            compiled=generate.trigger_creature_template(self.card(oracle_text=header+'draw a card.'))
            self.assertIsNotNone(compiled,header)
            condition=compiled[2][0][0]
            self.assertIn(subject,condition)
            self.assertIn('combat_only: '+str(combat).lower(),condition)
            self.assertIn('opponent_only: '+str(opponent).lower(),condition)
            self.assertIn('one_or_more: '+str(one).lower(),condition)
        row=self.card(oracle_text='Whenever this creature deals combat damage to a player, draw that many cards, then discard a card.')
        self.assertEqual(generate.trigger_creature_template(row)[2][0][1],['DrawDamageAmount','Discard(PlayerSet::Controller, 1)'])
        self.assertIsNone(generate.statement_program('Draw that many cards.'))
        self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text='When this creature enters, draw that many cards.')))
        for text in ['Whenever this creature deals combat damage to a player, draw a card for each modified creature you control.',
                     'Whenever this creature deals combat damage to a player, destroy target artifact that player controls.',
                     'Whenever this creature deals combat damage to a player or planeswalker, draw a card.',
                     'Whenever a creature you control with two counters on it deals combat damage to a player, draw a card.',
                     'Whenever this creature deals combat damage to a player, target creature gets +1/+1 until end of turn. Put a +1/+1 counter on it.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_damage_trigger_complete_admissions_and_residual_gates(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Appendage Amalgam','April, Reporter of the Weird','Ball Lightning','Dragon Mage','Eager Trufflesnout',
                     'Fear of Failed Tests','Fear of Surveillance','Festerleech','Frostbite Pyromental','Goldvein Pick',
                     'Grasping Longneck','Haliya, Ascendant Cadet','Invasion Tactics','Jeskai Shrinekeeper','Locke Cole',
                     'SP//dr, Piloted by Peni','Scream Puff','Syr Vondam, the Lucent','The Thing, Ben Grimm',
                     'Thieving Otter','Ultimecia, Temporal Threat','Yathan Tombguard']:
            self.assertIn(name,supported)
        for name in ['Curious Inquiry','Thief\'s Knife','Enduring Curiosity','Trygon Predator','Kutzil, Malamet Exemplar',
                     'Venus, Torn Between Worlds','Nashi, Searcher in the Dark','Buster Sword']:
            self.assertNotIn(name,supported)
        row=self.card(name='Example, Named',type_line='Legendary Creature — Human',
            oracle_text='Whenever Example deals combat damage to a player, put a +1/+1 counter on it.')
        self.assertIn('Subject::Source',generate.trigger_creature_template(row)[2][0][1][0])

    def test_discard_hand_tap_creatures_and_enchantment_creatures_are_complete(self):
        self.assertEqual(generate.statement_program('Each player discards their hand, then draws seven cards.'),
            ['DiscardHand(PlayerSet::Each)','Draw(PlayerSet::Each, 7)'])
        self.assertIn('TapCreatures',generate.statement_program('Tap all creatures your opponents control.')[0])
        self.assertIn('source_only: true',generate.statement_program('Sacrifice this creature.')[0])
        self.assertIsNotNone(generate.creature_template(self.card(type_line='Enchantment Creature — Nightmare',oracle_text='Flying')))
        self.assertIsNone(generate.creature_template(self.card(type_line='Enchantment Creature — Nightmare',oracle_text='Bestow {2}{U}')))

    def test_mill_quantities_optional_legality_and_exact_recovery_binding(self):
        self.assertEqual(generate.statement_program('Mill two cards.'),[generate.mill_instruction('Controller',2)])
        self.assertEqual(generate.statement_program('You may mill two cards.'),['MayMill(2)',generate.mill_instruction('Controller',2)])
        self.assertEqual(generate.statement_program('Each player mills 4 cards.'),[generate.mill_instruction('Each',4)])
        self.assertEqual(generate.statement_program('Each opponent mills a card.'),[generate.mill_instruction('Opponent',1)])
        self.assertEqual(generate.targeted_program('Target opponent mills three cards.'),('Opponent',[generate.mill_instruction('Targeted',3)]))
        self.assertEqual(generate.targeted_program('Target player mills cards equal to your life total.'),('Player',['Mill(PlayerSet::Targeted, MillAmount::ControllerLife)']))
        program=generate.statement_program('Mill four cards, then you may return a land card from among them to your hand.')
        self.assertEqual(program,[generate.mill_instruction('Controller',4),'RecoverMilled(RecoveryFilter::Land)'])
        program=generate.statement_program('Mill two cards. You may put a permanent card from among the milled cards into your hand. You gain 2 life.')
        self.assertEqual(program,[generate.mill_instruction('Controller',2),'RecoverMilled(RecoveryFilter::Permanent)','GainLife(PlayerSet::Controller, 2)'])
        for text in ['Mill 65536 cards.','Mill X cards.','You may mill two cards. If you do, draw a card.',
                     'Mill four cards, then return a land card from your graveyard to your hand.',
                     'You may put a permanent card from among them into your hand.',
                     'Mill four cards. You may put a permanent card from among them into your hand. Create a Food token if it was a Squirrel.',
                     'Mill two cards. You may put two permanent cards from among them into your hand.']:
            self.assertIsNone(generate.statement_program(text),text)

    def test_milling_and_combined_entry_death_admissions_are_complete(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Crow of Dark Tidings','Daggerfang Duo','Deathcap Marionette','Desperate Bloodseeker',
                     'Lys Alana Informant','Midnight Tilling','Mineshaft Spider',"Mysterio's Phantasm",'Rapid Rescue',
                     'Scarblade Scout','Screaming Phantom','Shinra Reinforcements','Something Worth Saving',
                     'Space-Time Anomaly','Thawbringer','Venomized Cat','Vinereap Mentor','Wary Thespian','Wary Watchdog',
                     'Watcher of the Wayside']:
            self.assertIn(name,supported)
        for name in ['Cache Grab','Aftermath Analyst','Pothole Mole','The Mindskinner','Molt Tender','Town Greeter']:
            self.assertNotIn(name,supported)
        trigger=generate.trigger_creature_template(self.card(oracle_text='When this creature enters or dies, surveil 1.'))
        self.assertEqual(trigger[2][0][0],'SelfEntersOrDies')

    def test_aura_enchant_relations_static_bonuses_and_source_entry_subjects(self):
        for line,relation in [('Enchant creature',-1),('Enchant creature you control',0),('Enchant creature an opponent controls',1)]:
            result=generate.aura_template(self.card(type_line='Enchantment — Aura',oracle_text=line+'\nEnchanted creature gets +2/+1 and has flying.'))
            self.assertIsNotNone(result,line)
            self.assertEqual(result[-4:],(relation,2,1,1))
        self.assertEqual(generate.statement_program('Tap enchanted creature.'),['TapAttached'])
        self.assertIn('Subject::Attached',generate.statement_program('Enchanted creature gains hexproof until end of turn.')[0])
        self.assertIn('Subject::Attached',generate.statement_program('Equipped creature gets +2/+2 until end of turn.')[0])
        self.assertIsNone(generate.permanent_program_template(self.card(type_line='Enchantment — Aura',oracle_text='Enchant creature\nEnchanted creature gets +1/+1.')))

    def test_aura_unsupported_enchant_restrictions_layers_and_events_fail_closed(self):
        for text in ['Enchant creature with flying\nEnchanted creature gets +1/+1.',
                     'Enchant land\nEnchanted creature gets +1/+1.',
                     'Enchant creature\nEnchanted creature has base power and toughness 1/1.',
                     'Enchant creature\nYou control enchanted creature.',
                     'Enchant creature\nEnchanted creature loses all abilities.',
                     'Enchant creature\nEnchanted creature gets +1/+1 for each Forest you control.',
                     'Enchant creature\nWhen enchanted creature dies, draw a card.',
                     'Enchant creature\nEnchanted creature has "Whenever this creature attacks, draw a card."',
                     "Enchant creature\nEnchanted creature doesn't untap during its controller's untap step.",
                     'Enchant creature\nWhen this Aura enters, it deals 1 damage to enchanted creature.']:
            self.assertIsNone(generate.aura_template(self.card(type_line='Enchantment — Aura',oracle_text=text)),text)

    def test_all_aura_milestone_cards_compile_completely(self):
        import json
        cards={c['name']:c for c in json.loads((generate.ROOT/'data/standard/cards.json').read_text())['cards']}
        for name in ["Aquitect's Defenses",'Colossification','Dead Weight','Fae Flight','Feather of Flight',
                     'Fire-Rim Form','Frantic Strength',"Gilt-Leaf's Embrace",'Robotics Mastery','Super Speed',
                     'Twinblade Blessing','Unflinching Courage','Untamed Hunger']:
            self.assertIsNotNone(generate.aura_template(cards[name]),name)
        self.assertIsNotNone(generate.equipment_template(cards['Adventuring Gear']))
        for name in ['Charmed Sleep','Coerced to Kill','Ethereal Armor','Fungal Fortitude',
                     'Pacifism',"Shardmage's Rescue"]:
            self.assertIsNone(generate.aura_template(cards[name]),name)

    def test_equipment_fixed_bonuses_and_equip_costs_compile_completely(self):
        c=self.card(type_line='Artifact — Equipment',mana_cost='{2}',oracle_text='Equipped creature gets +2/+1 and has flying and vigilance.\nEquip {1}{W}')
        result=generate.equipment_template(c)
        self.assertIsNotNone(result)
        self.assertEqual(result[-3:],(2,1,9))
        ability=result[3][0]
        self.assertEqual(ability[0]['colored'],[1,0,0,0,0,0])
        self.assertEqual(ability[0]['generic'],1)
        self.assertEqual(ability[1],'CreatureYou')
        self.assertEqual(ability[2],['AttachSource'])
        self.assertTrue(ability[3])
        self.assertIsNone(generate.permanent_program_template(c))
        self.assertIsNotNone(generate.equipment_template(dict(c,oracle_text='Equipped creature has haste.\nEquip {0}')))

    def test_equipment_unknown_layers_granted_abilities_and_costs_fail_closed(self):
        for text in ['Equipped creature has hexproof from monocolored.\nEquip {2}',
                     'Equipped creature has "Whenever this creature attacks, draw a card."\nEquip {2}',
                     'Equipped creature has base power and toughness 3/3.\nEquip {2}',
                     'Equipped creature gets +1/+1 for each artifact you control.\nEquip {2}',
                     'Equipped creature gets +1/+1 and is a Hero.\nEquip {2}',
                     'During your turn, equipped creature has first strike.\nEquip {2}',
                     'Equipped creature gets +1/+1.\nEquip—Sacrifice a creature.',
                     'Equipped creature gets +1/+1.\nEquip {2}. This ability costs {1} less to activate.',
                     'Equipped creature gets +1/+1.\nEquip Detective {1}',
                     'Equipped creature gets +1/+1.']:
            self.assertIsNone(generate.equipment_template(self.card(type_line='Artifact — Equipment',oracle_text=text)),text)
        self.assertIsNone(generate.equipment_template(self.card(type_line='Enchantment — Aura',oracle_text='Enchant creature\nEnchanted creature gets +1/+1.')))
        self.assertIsNone(generate.targeted_program('Attach this Equipment to target creature you control. That creature gains wither until end of turn.'))
        self.assertIsNone(generate.targeted_program('Attach it to target creature you control.'))

    def test_equipment_milestone_all_admissions_and_unsupported_residuals(self):
        import json
        cards={c['name']:c for c in json.loads((generate.ROOT/'data/standard/cards.json').read_text())['cards']}
        for name in ['Basilisk Collar','Celestial Armor','Coral Sword','Crude Bent Blade','Fireshrieker',
                     'Glimmerlight','Gold Pan','Hard-Won Jitte','Illvoi Light Jammer','Lion Heart',
                     'Malamet Scythe',"Meltstrider's Gear",'Ragged Short Spear','Short Bow','Skateboard',
                     "Squire's Lightblade",'Stolen Stark Tech','Swiftfoot Boots','Sword of Vengeance',
                     'Twin Blades','Vibranium Energy Daggers','Well-Worn Spatula']:
            self.assertIsNotNone(generate.equipment_template(cards[name]),name)
        for name in ['Auxiliary Boosters','Bloodthorn Flail',"Captain America's Shield",
                     'Dragonfire Blade',"Hunter's Blowgun",'Lotus Ring',"Pirate's Cutlass",'Spider-Suit']:
            self.assertIsNone(generate.equipment_template(cards[name]),name)

    def test_vital_event_prefixes_are_complete_and_filtered(self):
        for prefix,condition in generate.VITAL_PREFIXES.items():
            result=generate.trigger_creature_template(self.card(oracle_text=prefix+'draw a card.'))
            self.assertIsNotNone(result,prefix)
            self.assertEqual(result[2][0][0],condition)
        first=generate.trigger_creature_template(self.card(oracle_text='Whenever you gain life for the first time during each of your turns, draw a card.'))
        self.assertIn('first_only: true, own_turn: true',first[2][0][0])
        for text in ['Whenever you gain 3 or more life, draw a card.',
                     'Whenever one or more creatures you control die, draw a card.',
                     'Whenever another creature you control with flying dies, draw a card.',
                     'Whenever another creature or artifact you control dies, draw a card.',
                     'Whenever another creature you control dies, it deals 1 damage to each opponent.',
                     'Whenever a creature you control dies, that player loses 2 life.',
                     'Whenever you gain life, draw that many cards.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_legendary_source_names_and_reminder_whitespace_preserve_complete_scope(self):
        card=self.card(name='Tigra, Feline Fury',type_line='Legendary Creature — Cat Hero',oracle_text='Flash (You may cast this spell any time you could cast an instant.)\nWhenever you gain life, put a +1/+1 counter on Tigra.')
        self.assertIsNotNone(generate.trigger_creature_template(card))
        self.assertIsNone(generate.trigger_creature_template(dict(card,type_line='Creature — Cat Hero')))
        self.assertIsNone(generate.trigger_creature_template(dict(card,oracle_text='Whenever you gain life, put a +1/+1 counter on Some Other Cat.')))
        self.assertIsNotNone(generate.trigger_creature_template(self.card(name='Gladiolus Amicitia',oracle_text='When Gladiolus Amicitia enters, draw a card.')))
        self.assertIsNone(generate.trigger_creature_template(self.card(name='Gladiolus Amicitia',oracle_text='When Gladiolus enters, draw a card.')))

    def test_all_vital_milestone_admissions_and_remaining_mechanics_are_gated(self):
        import json
        cards={c['name']:c for c in json.loads((generate.ROOT/'data/standard/cards.json').read_text())['cards']}
        for name in ["Ajani's Pridemate",'Marauding Blight-Priest','Pest Mascot','Tigra, Feline Fury',
                     'Vanguard Seraph','Cat Collector','Agent Venom','Boggart Cursecrafter',
                     'High-Society Hunter','Midnight Reaper',"Sephiroth, Planet's Heir",'Voracious Vermin',
                     'Drogskol Reaver','Dori, Bearer of Friends','Glacier Godmaw','Gladiolus Amicitia',
                     'Professional Wrestler','Provisions Merchant','Pulse Tracker','Shroudstomper','Starling, Aerial Ally','Vein Ripper']:
            self.assertIsNotNone(generate.trigger_creature_template(cards[name]),name)
        for name in ['Engine Rat',"Thrór's Map"]:
            self.assertIsNotNone(generate.permanent_program_template(cards[name]),name)
        for name in ['Aerith Gainsborough','Twinblade Paladin']:
            self.assertIsNone(generate.trigger_creature_template(cards[name]),name)
            self.assertIsNone(generate.permanent_program_template(cards[name]),name)

    def test_resolution_sacrifices_keep_player_ownership_filters_and_fixed_counts(self):
        for subject,players,targets in [('You','Controller','None'),('Each player','Each','None'),('Each opponent','Opponent','None'),('Target player','Targeted','Player'),('Target opponent','Targeted','Opponent')]:
            result=generate.sacrifice_program(subject.lower()+' sacrifices two creatures of their choice.')
            self.assertEqual(result[0],targets)
            self.assertIn('players: PlayerSet::'+players,result[1][0])
            self.assertIn('amount: 2, optional_skip: None',result[1][0])
        for text,bits in [('an artifact',16),('a creature',1),('a creature or planeswalker',65),('another creature or artifact',17),('a creature or enchantment',33)]:
            spec=generate.sacrifice_spec(text)
            self.assertIn('kinds: '+str(bits),spec[1])
        self.assertIn('other: true',generate.sacrifice_spec('another creature or artifact')[1])
        self.assertIn('source_only: true',generate.sacrifice_spec('this creature')[1])
        self.assertIn('Keywords(1)',generate.sacrifice_spec('a creature with flying')[1])
        self.assertIn('tokens: Some(false)',generate.sacrifice_spec('two nontoken creatures')[1])
        self.assertIn('subtype: Some("Food")',generate.sacrifice_spec('two Foods')[1])
        self.assertIn('amount: 13',generate.statement_program('Each player sacrifices thirteen creatures of their choice.')[0])
        self.assertIsNotNone(generate.modal_spell_template(self.card(type_line='Sorcery',oracle_text='Choose one —\n• Each opponent sacrifices an artifact of their choice.\n• Each opponent sacrifices an enchantment of their choice.\n• Each opponent sacrifices a creature with flying of their choice.')))

    def test_conditional_resolution_costs_require_complete_single_effect_sentences(self):
        result=generate.statement_program('You may discard two cards. If you do, draw a card.')
        self.assertIn('kind: EffectChoiceKind::Discard',result[0])
        self.assertIn('amount: 2, optional_skip: Some(1)',result[0])
        result=generate.statement_program('You may sacrifice another creature or artifact. If you do, draw a card and put a +1/+1 counter on this creature.')
        self.assertIn('optional_skip: Some(2)',result[0])
        self.assertEqual(result[1:],['Draw(PlayerSet::Controller, 1)',generate.counter_instruction('Source','a','+1/+1')])
        result=generate.targeted_program('You may sacrifice an artifact. If you do, put a +1/+1 counter on target creature.')
        self.assertEqual(result[0],'Creature')
        self.assertIn('optional_skip: Some(1)',result[1][0])
        self.assertIsNotNone(generate.statement_program('You may sacrifice a land. If you do, search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.'))
        for text in [
            'You may discard a nonland card. If you do, draw a card.',
            'You may sacrifice an artifact. When you do, draw a card.',
            'You may discard a card. If you do, draw a card. You gain 1 life.',
            'You may sacrifice an artifact or discard a card. If you do, draw a card.',
            'You may sacrifice up to one artifact. If you do, draw a card.',
            'You may discard zero cards. If you do, draw a card.',
            'You may discard a card. If you do, win the game.',
            'You may sacrifice an artifact. If you do, it deals 1 damage to each opponent.',
            'As an additional cost to cast this spell, sacrifice a creature. Draw a card.',
        ]:
            self.assertIsNone(generate.statement_program(text) or generate.targeted_program(text),text)

    def test_sacrifice_unknown_restrictions_and_mixed_union_qualifiers_fail_closed(self):
        for spec in ['a creature with the greatest power','a creature or planeswalker with the greatest mana value',
                     'an artifact or creature token','a nontoken artifact or creature',
                     'an artifact or creature with flying','two creatures with flying or reach','half your creatures',
                     'a creature an opponent controls','another creature you do not control','a creature with ward {2}',
                     'zero creatures','a permanent other than enchanted creature']:
            self.assertIsNone(generate.sacrifice_spec(spec),spec)
        for text in ['Target opponent sacrifices a creature with the greatest power among creatures they control.',
                     'Each player sacrifices all other creatures they control.',
                     'Target creature gets +1/+1 until end of turn. Its controller sacrifices it.']:
            self.assertIsNone(generate.statement_program(text) or generate.targeted_program(text),text)

    def test_step_events_and_intervening_if_are_whole_trigger_clauses(self):
        for prefix, condition in generate.STEP_PREFIXES.items():
            row = self.card(oracle_text=prefix+'Draw a card.')
            self.assertEqual(generate.trigger_creature_template(row)[2], [(condition,['Draw(PlayerSet::Controller, 1)'],'None')])
        for label, event, guard in [
            ('Raid — ', 'your end step', 'you attacked this turn'),
            ('Survival — ', 'your second main phase', 'this creature is tapped'),
            ('Formidable — ', 'combat on your turn', 'creatures you control have total power 8 or greater'),
        ]:
            text=label+'At the beginning of '+event+', if '+guard+', draw a card.'
            result=generate.trigger_creature_template(self.card(oracle_text=text))
            self.assertIn(generate.step_guard(guard),result[2][0][0])
        for text in [
            'At the beginning of your end step, if an opponent gained life this turn, draw a card.',
            'At the beginning of your end step, if you control two or more tapped creatures and a Dragon, draw a card.',
            'At the beginning of your upkeep, draw a card if you control a Dragon.',
            'At the beginning of your upkeep, if you have 40 or more life, you win the game.',
            'At the beginning of your next upkeep, draw a card.',
            'At the beginning of combat on your turn, choose one or both —\n• Draw a card.\n• You gain 2 life.',
            'At the beginning of your end step, draw a card. This ability triggers only once.',
        ]:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)),text)

    def test_step_conditions_preserve_numeric_bounds_and_unknown_predicates_fail_closed(self):
        for text in ['you attacked this turn','this creature is tapped','you control two or more tapped creatures',
                     'you control three or more creatures','you control six or more lands',
                     'you control a creature with power 4 or greater','creatures you control have total power 8 or greater',
                     'creatures you control have total toughness 10 or greater','you have fewer than 8 cards in hand',
                     'you have 20 or more life','you have 5 or less life',"you didn't cast a spell this turn"]:
            self.assertIsNotNone(generate.step_guard(text),text)
        for text in ['you control a creature with power X or greater','you control a creature with power 40000 or greater',
                     'you control 999999 or more lands','you have fewer than 0 cards in hand','you have 999999 or more life',
                     'you attacked with a Hero this turn','this creature has a counter on it',
                     'you control two or more tapped creatures or a Dragon',"you haven't cast a spell from your hand this turn"]:
            self.assertIsNone(generate.step_guard(text),text)

    def test_step_effects_keep_optional_targets_and_complete_atomic_sequences(self):
        result=generate.targeted_program('You may have target creature get +2/+0 until end of turn.')
        self.assertEqual(result,('Creature',['May { skip: 1 }', generate.grow_instruction('Targets',2,0)]))
        self.assertEqual(generate.statement_program('You draw a card and lose 1 life.'),
                         ['Draw(PlayerSet::Controller, 1)','LoseLife(PlayerSet::Controller, 1)'])
        self.assertIsNotNone(generate.statement_program('You lose 1 life and create a 1/1 blue and black Faerie creature token with flying.'))
        self.assertIn('UntapCreatures(',generate.statement_program('Untap each creature you control.')[0])
        for text in ['Untap each artifact you control.','You may have target creature get +X/+0 until end of turn.',
                     'You lose 1 life and create a 1/1 blue Faerie creature token with ward {2}.',
                     'You draw a card and lose 999999 life.']:
            self.assertIsNone(generate.statement_program(text) or generate.targeted_program(text),text)

    def test_cast_trigger_conditions_and_second_spell_are_exact(self):
        for prefix, relation, filter_, ordinal in [
            ('Whenever you cast a spell, ', 'You', 'Any', None),
            ('Whenever you cast a creature spell, ', 'You', 'Creature', None),
            ('Whenever you cast a noncreature spell, ', 'You', 'Noncreature', None),
            ('Whenever you cast an instant or sorcery spell, ', 'You', 'InstantSorcery', None),
            ('Whenever you cast an artifact spell, ', 'You', 'Artifact', None),
            ('Whenever you cast an enchantment spell, ', 'You', 'Enchantment', None),
            ('Whenever an opponent casts a spell, ', 'Opponent', 'Any', None),
            ('Whenever a player casts a spell, ', 'Any', 'Any', None),
            ('Whenever you cast your second spell each turn, ', 'You', 'Any', 2),
            ('Flurry — Whenever you cast your second spell each turn, ', 'You', 'Any', 2),
        ]:
            result = generate.trigger_creature_template(self.card(oracle_text=prefix+'Draw a card.'))
            self.assertEqual(result[2], [(generate.cast_condition(relation, filter_, ordinal), ['Draw(PlayerSet::Controller, 1)'], 'None')])
        for line in [
            'Whenever you cast a noncreature spell, if four mana was spent to cast it, draw a card.',
            'Magecraft — Whenever you cast or copy an instant or sorcery spell, draw a card.',
            'Whenever you cast your second creature spell each turn, draw a card.',
            'Whenever you cast a spell, copy it.',
            'Whenever you cast an instant or sorcery spell that targets a creature, draw a card.',
        ]:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=line)), line)

    def test_prowess_keeps_repeated_instances_and_rejects_residual_abilities(self):
        condition = generate.cast_condition(filter_='Noncreature')
        program = [generate.grow_instruction('Source', 1, 1)]
        row = self.card(oracle_text='Flying, vigilance, prowess, prowess (Each triggers.)')
        result = generate.trigger_creature_template(row)
        self.assertEqual(result[1], 9)
        self.assertEqual(result[2], [(condition, program, 'None')]*2)
        self.assertIsNone(generate.creature_template(row))
        for text in ['Prowess, ward—Pay 1 life.', 'Prowess\nYou may cast spells from your graveyard.',
                     'Other creatures you control have prowess.', 'Prowess\nThis spell costs {1} less to cast.']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)), text)
        self.assertIsNone(generate.keyword_effect('gains prowess until end of turn.', 'Source'))

    def test_nontargeted_source_damage_and_source_untap_preserve_complete_clauses(self):
        self.assertEqual(generate.statement_program('This creature deals 2 damage to each opponent and you gain 2 life.'),
                         ['DamagePlayers(PlayerSet::Opponent, 2)', 'GainLife(PlayerSet::Controller, 2)'])
        self.assertEqual(generate.statement_program('Untap this creature.'), ['UntapSource'])
        row = self.card(name='Black Waltz No. 3', oracle_text='Whenever you cast a noncreature spell, Black Waltz No. 3 deals 2 damage to each opponent.')
        self.assertEqual(generate.trigger_creature_template(row)[2][0][1], ['DamagePlayers(PlayerSet::Opponent, 2)'])
        for text in ['This creature deals X damage to each opponent.',
                     'This creature deals 1 damage to each opponent unless they pay {1}.',
                     'This creature deals 40000 damage to each opponent.',
                     'Untap this creature and another target creature.',
                     'Untap this creature. Win the game.']:
            self.assertIsNone(generate.statement_program(text), text)

    def test_damage_pronouns_require_a_proven_source_antecedent(self):
        for suffix in ['It deals 1 damage to each opponent.', 'It deals 1 damage to target opponent.']:
            self.assertIsNone(generate.statement_program(suffix))
            self.assertIsNone(generate.targeted_program(suffix))
            self.assertIsNone(generate.targeted_program('Target creature gets +1/+1 until end of turn. '+suffix))
            self.assertIsNone(generate.spell_program_template(self.card(type_line='Instant', oracle_text=suffix)))
        for prefix in ['When this creature enters, ', 'When this creature dies, ', 'Whenever this creature attacks, ']:
            self.assertIsNotNone(generate.trigger_creature_template(self.card(oracle_text=prefix+'it deals 1 damage to each opponent.')))
        for prefix in ['Whenever another creature you control enters, ', 'Whenever you cast a creature spell, ']:
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=prefix+'it deals 1 damage to each opponent.')))
        self.assertEqual(generate.spell_program_template(self.card(name='Boltwave',type_line='Sorcery',oracle_text='Boltwave deals 3 damage to each opponent.')),
                         ('None',['DamagePlayers(PlayerSet::Opponent, 3)']))
        activation=generate.permanent_program_template(self.card(oracle_text='{T}, Sacrifice this creature: It deals 1 damage to any target.'))
        self.assertEqual(activation[3][0][2],['Damage(1)'])
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='{T}: Target creature gets +1/+1 until end of turn. It deals 1 damage to each opponent.')))

    def test_distinct_target_groups_and_power_damage_keep_operands(self):
        for verb, action in [('fights', 'Fight'), ('deals damage equal to its power to', 'PowerDamage')]:
            result = generate.targeted_program(f'Target creature you control {verb} target creature an opponent controls.')
            self.assertEqual(result, (['CreatureYou', 'CreatureOpponent'], [f'{action} {{ source: 0, recipient: 1 }}']))
        self.assertEqual(generate.targeted_program('Target creature you control gets +1/+0 until end of turn. It deals damage equal to its power to target creature an opponent controls.')[0], ['CreatureYou','CreatureOpponent'])
        result = generate.targeted_program('Put two +1/+1 counters on target creature you control. Then it fights target creature an opponent controls.')
        self.assertEqual(result[1][0], generate.counter_instruction('Targets', 'two', '+1/+1'))
        result = generate.targeted_program('Target player draws a card. Target opponent loses 1 life.')
        self.assertEqual(result, (['Player','Opponent'], ['Draw(PlayerSet::Targeted, 1)', 'SelectGroup(1)', 'LoseLife(PlayerSet::Targeted, 1)']))
        result = generate.targeted_program('Target player draws a card. Target creature you control fights target creature an opponent controls.')
        self.assertEqual(result[0], ['Player','CreatureYou','CreatureOpponent'])
        self.assertEqual(result[1][-1], 'Fight { source: 1, recipient: 2 }')
        result = generate.targeted_program('Target player draws a card. Target opponent loses 1 life. Tap target creature.')
        self.assertEqual(result[0], ['Player','Opponent',('Creature',1,1)])
        self.assertIn('SelectGroup(2)', result[1])
        self.assertIn('TargetPlan::Groups', generate.rust_targets(result[0]))
        for text in [
            'Target creature you control fights another target creature.',
            'Target creature you control deals damage equal to its power to target creature or planeswalker.',
            'Target creature you control fights target creature an opponent controls. When excess damage is dealt this way, draw a card.',
            'Target creature gets +1/+1 until end of turn. Put a +1/+1 counter on another target creature you control.',
            'Target creature you control gets +X/+X until end of turn. It fights target creature an opponent controls.',
            'Target creature you control fights target creature with power less than its power.']:
            self.assertIsNone(generate.targeted_program(text), text)

    def test_modal_spells_require_every_complete_mode_and_preserve_labels(self):
        row = self.card(name='Test Spell', type_line='Instant', mana_cost='{1}{R}',
            oracle_text='Choose one —\n• Test Spell deals 3 damage to target creature.\n• Destroy target artifact.')
        modes = generate.modal_spell_template(row)
        self.assertEqual(len(modes), 2)
        self.assertEqual(modes[0][1:], ('Creature', ['Damage(3)']))
        self.assertEqual(modes[1][1:], ('Artifact', ['Destroy']))
        labelled = dict(row, oracle_text='Choose one —\n• Do Homework — Draw three cards.\n• Fight Crime — Counter target spell. Draw a card.')
        modes = generate.modal_spell_template(labelled)
        self.assertEqual(modes[0][0], 'Do Homework — Draw three cards.')
        self.assertEqual(modes[0][1:], ('None', ['Draw(PlayerSet::Controller, 3)']))
        self.assertEqual(modes[1][1:], ('Spell', ['Counter', 'Draw(PlayerSet::Controller, 1)']))
        for text in [
            'Choose one —\n• Draw two cards.\n• Win the game.',
            'Choose two —\n• Draw two cards.\n• You gain 2 life.',
            'Choose one or both —\n• Draw two cards.\n• You gain 2 life.',
            'Choose one —\n• Draw two cards.',
            'Choose one —\n• Draw two cards.\n• You gain 2 life.\nThen win the game.',
            'Choose one —\nDraw two cards.\n• You gain 2 life.',
            'Choose one —\n• Counter target spell unless its controller pays {X}.\n• Draw two cards.',
            'Choose one —\n• Draw two cards.\n• Test Spell deals X damage to target creature.',
            'Choose one —\n• Draw two cards.\n• Other creatures you control get +1/+1.']:
            self.assertIsNone(generate.modal_spell_template(dict(row, oracle_text=text)), text)
        self.assertIsNone(generate.modal_spell_template(dict(row, layout='adventure')))
        self.assertIsNone(generate.modal_spell_template(dict(row, mana_cost='{X}{R}')))
        self.assertIsNone(generate.modal_spell_template(dict(row, type_line='Enchantment')))
        self.assertIsNone(generate.modal_spell_template(dict(row, oracle_text=None)))

    def test_search_quality_quantity_destination_and_residual_text(self):
        for quality, name in [('basic land', 'BasicLand'), ('land', 'Land'),
                              ('creature', 'Creature'), ('artifact', 'Artifact'), ('enchantment', 'Enchantment')]:
            result = generate.statement_program(f'Search your library for a {quality} card, reveal it, put it into your hand, then shuffle.')
            self.assertIn('SearchFilter::' + name, result[0])
            self.assertIn('reveal: true', result[0])
            self.assertIn('SearchDestination::Hand', result[0])
        result = generate.statement_program('Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle.')
        self.assertIn('maximum: 2', result[0])
        self.assertIn('tapped: true', result[0])
        result = generate.statement_program('Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. Investigate.')
        self.assertEqual(len(result), 2)
        for text in [
            'Search your library for a basic land card, put it into your hand, then shuffle.',
            'Search your library for a basic land card, reveal it, put it into your hand tapped, then shuffle.',
            'Search your library for a land card with mana value 2, reveal it, put it into your hand, then shuffle.',
            'Search your library for a basic land card or a Desert card, put it onto the battlefield, then shuffle.',
            'Search your library for up to two basic land cards with different names, put them onto the battlefield, then shuffle.',
            'Search your library for a land card, reveal it, put it into your hand.',
            'Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. Then if you control four or more lands, untap that land.',
            'Search your library for up to X land cards, put them onto the battlefield, then shuffle.',
            'Search your library for a basic land cards, put them onto the battlefield, then shuffle.',
            'Search your library for up to two basic land cards, reveal it, put them into your hand, then shuffle.']:
            self.assertIsNone(generate.statement_program(text), text)
        token = generate.predefined_program('create a Lander token.')
        self.assertIsNotNone(token)
        ability = generate.permanent_program_template(self.card(type_line='Artifact', mana_cost='{0}', oracle_text=generate.PREDEFINED['Lander']))
        self.assertTrue(ability[3][0][0]['sacrifice'])
        self.assertIn('SearchFilter::BasicLand', ability[3][0][2][0])

    def test_land_templates_require_complete_text_and_intrinsic_mana(self):
        def land(text, typ='Land', **kw):
            return self.card(type_line=typ, mana_cost='', oracle_text=text, **kw)
        for line, entry in generate.LAND_ENTRY.items():
            result = generate.land_template(land(line + '\n{T}: Add {W} or {U}.'))
            self.assertEqual(result[0], 'Land')
            self.assertEqual(result[5], entry)
            self.assertIn('mask: 3', result[3][0][2][0])
        result = generate.land_template(land('({T}: Add {R} or {G}.)\nThis land enters tapped.\n'
            'When this land enters, surveil 1. (Look at the top card of your library.)', 'Land — Mountain Forest'))
        self.assertEqual(result[5], 'Always')
        self.assertEqual(result[2], [('SelfEnters', ['Surveil(1)'], 'None')])
        self.assertIn('mask: 24', result[3][0][2][0])
        for text, typ in [
            ('({T}: Add {R}.)', 'Land — Mountain Forest'),
            ('{T}: Add {R}.', 'Land — Mountain'),
            ('({T}: Add {R}.)', 'Land'),
            ('This land enters tapped unless you control a Dragon.\n{T}: Add {R}.', 'Land'),
            ('As this land enters, choose a color.\n{T}: Add one mana of the chosen color.', 'Land'),
            ('{T}: Add {R}. Spend this mana only to cast a creature spell.', 'Land'),
            ('{T}: Add {R}.\nYou win the game.', 'Land'),
            ('This land enters tapped.\nThis land enters tapped.\n{T}: Add {R}.', 'Land')]:
            self.assertIsNone(generate.land_template(land(text, typ)), (text, typ))
        self.assertIsNone(generate.land_template(land('{T}: Add {R}.', layout='transform')))
        self.assertIsNone(generate.land_template(land(None)))
        result = generate.land_template(land('{T}: Add {C}.', 'Basic Land'))
        self.assertIsNotNone(result)
        result = generate.land_template(land('This land enters tapped.\n{T}: Add {U} or {R}.\n'
            '{4}, {T}, Sacrifice this land: Draw a card.'))
        self.assertTrue(result[3][1][0]['sacrifice'])
        self.assertEqual(result[3][1][2], ['Draw(PlayerSet::Controller, 1)'])

    def test_scry_and_surveil_compile_whole_fixed_count_programs(self):
        for name in ('scry', 'surveil'):
            for count in (0, 1, 2, 65535):
                self.assertEqual(generate.statement_program(f'{name} {count}.'), [f'{name.capitalize()}({count})'])
            for text in (f'{name} X.', f'{name} 65536.', f'{name} -1.',
                         f'{name} 2. Then win the game.', f'{name} 1, then add mana.'):
                self.assertIsNone(generate.statement_program(text), text)
            row = self.card(type_line='Instant', mana_cost='{U}', oracle_text=f'{name.capitalize()} 2.\nDraw a card.')
            self.assertEqual(generate.spell_program_template(row), ('None', [f'{name.capitalize()}(2)', 'Draw(PlayerSet::Controller, 1)']))
            self.assertEqual(generate.statement_program(f'{name} 2, then draw two cards.'), [f'{name.capitalize()}(2)', 'Draw(PlayerSet::Controller, 2)'])
        for text in ('Whenever you scry, draw a card.', 'Whenever you surveil, draw a card.',
                     'At the beginning of your next upkeep, scry 1.'):
            self.assertIsNone(generate.trigger_creature_template(self.card(oracle_text=text)))

    def test_additive_stats_compile_both_values_and_exact_static_filters(self):
        self.assertEqual(generate.targeted_program('Target creature gets +1/+3 until end of turn.'),
                         ('Creature', [generate.grow_instruction('Targets', 1, 3)]))
        self.assertEqual(generate.statement_program('This creature gets +3/-3 until end of turn.'),
                         [generate.grow_instruction('Source', 3, -3)])
        for phrase, expected in [('Other Elves you control', 'subtype: Some("Elf")'),
                                 ('Creatures your opponents control', 'relation: 1'),
                                 ('Attacking creatures you control', 'attacking: true'),
                                 ('Creature tokens you control', 'tokens: Some(true)'),
                                 ('Artifact creatures you control', 'artifact: true'),
                                 ('Other creatures you control with flying', 'keyword: Keywords(1)')]:
            self.assertIn(expected, generate.creature_filter(phrase))
            row=self.card(oracle_text=phrase + ' get +1/+2.')
            result=generate.static_permanent_template(row)
            self.assertIsNotNone(result)
            self.assertIn('power: 1, toughness: 2', result[5][0])
        self.assertIn('power: -3, toughness: 0', generate.statement_program('Creatures your opponents control get -3/-0 until end of turn.')[0])
        for text in ('Enchanted creatures you control get +2/+2.',
                     'Creatures you control get +X/+X.',
                     'Creatures you control get +1/+1 for each land you control.',
                     'Other creatures you control get +1/+1.\nThis creature has base power and toughness 4/4.',
                     'Other creatures you control get +32768/+0.'):
            self.assertIsNone(generate.static_permanent_template(self.card(oracle_text=text)),text)
        for text in ('Creatures you control get +2/+1 until your next turn.',
                     'Creatures you control get +1/+1 and gain ward {2} until end of turn.',
                     'This creature gets +X/+0 until end of turn.'):
            self.assertIsNone(generate.statement_program(text),text)

    def test_effective_keyword_programs_remain_complete_and_fixed_duration(self):
        self.assertEqual(generate.keyword_list('trample, hexproof, and indestructible'), 256+4096+8192)
        self.assertEqual(generate.keyword_list('flying and flying'), 1)
        row=self.card(oracle_text='Hexproof, indestructible')
        self.assertEqual(generate.creature_template(row), ('Creature(2, 1)',4096+8192))
        for text in ('Target creature gains deathtouch and lifelink until end of turn.',
                     'Target creature you control gets +1/+1 and gains hexproof until end of turn. Untap it.',
                     'Until end of turn, target creature gets +3/+2 and gains trample.'):
            self.assertIsNotNone(generate.targeted_program(text),text)
        for text in ('Creatures you control get +1/+1 and gain haste until end of turn.',
                     'This creature gains reach until end of turn.'):
            self.assertIsNotNone(generate.statement_program(text),text)
        for text in ('Target creature gains hexproof from red until end of turn.',
                     'Target creature loses flying until end of turn.',
                     'Target creature gains ward {2} until end of turn.',
                     'Target creature gains flying until your next turn.',
                     'Target player draws a card. Untap it.'):
            self.assertIsNone(generate.targeted_program(text),text)
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='Other creatures you control have haste.')))

    def test_fixed_blocking_restrictions_and_menace_require_complete_text(self):
        for text, flags in [('Menace', 32768), ("This creature can't block.", 2048),
                            ("This creature can't be blocked.",16384),
                            ("This creature can't be blocked by more than one creature.",65536),
                            ("This creature can't be blocked except by three or more creatures.",131072),
                            ("This creature can't be blocked by tokens.",262144)]:
            self.assertEqual(generate.creature_template(self.card(oracle_text=text)), ('Creature(2, 1)',flags))
        self.assertIsNotNone(generate.targeted_program("Target creature can't be blocked this turn."))
        self.assertIsNotNone(generate.targeted_program("Target creature gains vigilance until end of turn and can't be blocked this turn."))
        for text in ("This creature can't block unless you control a Hero.",
                     "This creature can't be blocked by creatures with power 4 or less.",
                     'This creature must be blocked if able.'):
            self.assertIsNone(generate.creature_template(self.card(oracle_text=text)))
        self.assertIsNone(generate.statement_program("Creatures you control can't be blocked this turn."))
        self.assertIsNotNone(generate.statement_program('Create a 1/1 white Rabbit creature token with menace.'))

    def test_exile_and_linked_return_programs_require_complete_supported_predicates(self):
        for text, rule, program in [
            ('Exile target creature.', 'Creature', ['Exile']),
            ('Exile target creature or enchantment.', 'CreatureEnchantment', ['Exile']),
            ('Exile target card from a graveyard.', 'CardGrave', ['Exile']),
            ('Exile target creature with mana value 3 or greater.', 'CreatureMin3', ['Exile']),
            ('Exile target nonland permanent an opponent controls until this enchantment leaves the battlefield.', 'NonlandOpponent', ['ExileUntilSourceLeaves']),
            ('Exile target tapped creature an opponent controls until this creature leaves the battlefield.', 'CreatureOpponentTapped', ['ExileUntilSourceLeaves']),
            ('Exile target creature an opponent controls with mana value 3 or less until this artifact leaves the battlefield.', 'CreatureOpponentMax3', ['ExileUntilSourceLeaves']),
            ("Exile target player's graveyard.", 'Player', ['ExileGraveyard(PlayerSet::Targeted)']),
        ]:
            self.assertEqual(generate.targeted_program(text), (rule,program))
        self.assertEqual(generate.statement_program("Exile each opponent's graveyard."), ['ExileGraveyard(PlayerSet::Opponent)'])
        for text in ('Exile target permanent with power 4 or greater.',
                     'Exile target creature until the next end step.',
                     'Exile target nonland permanent an opponent controls with mana value X or less until this enchantment leaves the battlefield.',
                     'Exile target creature face down.',
                     'Exile target creature. Its controller draws a card.'):
            self.assertIsNone(generate.targeted_program(text),text)
        self.assertIn('max_mana: 3',generate.TARGET_RULES['CreatureOpponentMax3'])
        self.assertIn('tapped: true',generate.TARGET_RULES['CreatureOpponentTapped'])

    def test_legendary_permanents_require_all_rules_and_spells_stay_gated(self):
        self.assertEqual(generate.creature_template(self.card(type_line='Legendary Creature — Cat')),
                         ('Creature(2, 1)', 0))
        self.assertIsNotNone(generate.trigger_creature_template(self.card(
            type_line='Legendary Creature — Cat', oracle_text='When this creature enters, draw a card.')))
        self.assertIsNotNone(generate.land_template(self.card(
            type_line='Legendary Land', mana_cost='', oracle_text='{T}: Add {R}.')))
        for typ in ('Legendary Instant', 'Legendary Sorcery'):
            self.assertIsNone(generate.spell_program_template(self.card(type_line=typ, oracle_text='Draw a card.')))
        for typ in ('World Creature — Cat', 'Snow Creature — Cat', 'Legendary Snow Creature — Cat'):
            self.assertIsNone(generate.creature_template(self.card(type_line=typ)))
        self.assertIsNone(generate.permanent_program_template(self.card(
            type_line='Legendary Creature — Cat', oracle_text='You may play lands from your graveyard.')))

    def test_exact_creature_template_rejects_other_mechanics(self):
        self.assertEqual(generate.creature_template(self.card()), ('Creature(2, 1)', 0))
        for changes in [dict(oracle_text='Flying\nWhen this creature enters, draw a card.'),
                        dict(type_line='Snow Creature — Cat'), dict(power='*'),
                        dict(mana_cost='{X}{W}'), dict(mana_cost='{W/P}'), dict(layout='transform'),
                        dict(oracle_text='Flying, protection from red')]:
            self.assertIsNone(generate.creature_template(self.card(**changes)), changes)

    def test_keyword_template_requires_every_piece_of_text_to_be_implemented(self):
        self.assertEqual(generate.creature_template(self.card(oracle_text='Flying, vigilance')), ('Creature(2, 1)', 9))
        self.assertEqual(generate.creature_template(self.card(oracle_text='Reach (This creature can block creatures with flying.)')), ('Creature(2, 1)', 2))
        self.assertIsNone(generate.creature_template(self.card(oracle_text='Reach (reminder)\nOther Cats you control get +1/+1.')))

    def test_missing_or_null_rules_text_never_means_vanilla(self):
        row = self.card(oracle_text='When this creature enters, create a token.')
        self.assertIsNone(generate.creature_template(row))
        del row['oracle_text']
        self.assertIsNone(generate.creature_template(row))
        row['oracle_text'] = None
        self.assertIsNone(generate.creature_template(row))
        row['oracle_text'] = ''
        self.assertEqual(generate.creature_template(row), ('Creature(2, 1)', 0))

    def test_cost_parser_does_not_approximate_unimplemented_symbols(self):
        self.assertEqual(generate.fixed_cost('{3}{R}{R}'), ([0, 0, 0, 2, 0, 0], 3))
        for value in ('{S}', '{X}', '{2/R}', '{R/P}', '', '{W}{unrecognized}'):
            self.assertIsNone(generate.fixed_cost(value))

    def test_removal_template_requires_exact_text_types_and_cost(self):
        row = self.card(type_line='Instant', mana_cost='{U}',
                        oracle_text="Return target creature to its owner's hand.")
        self.assertEqual(generate.spell_template(row), 'ReturnToHand')
        self.assertEqual(generate.spell_template(dict(row, oracle_text='Destroy target creature.')), 'Destroy')
        for changes in [dict(oracle_text=None), dict(oracle_text=''),
                        dict(oracle_text=row['oracle_text'] + '\nDraw a card.'),
                        dict(type_line='Creature — Wizard'), dict(layout='adventure'),
                        dict(mana_cost='{X}{U}'), dict(type_line='Kindred Instant — Elf')]:
            self.assertIsNone(generate.spell_template(dict(row, **changes)), changes)

    def test_program_statements_preserve_order_and_reject_residual_text(self):
        self.assertEqual(generate.statement_program('Draw two cards. You lose 2 life.'),
                         ['Draw(PlayerSet::Controller, 2)', 'LoseLife(PlayerSet::Controller, 2)'])
        self.assertEqual(generate.statement_program('you gain 1 life and draw a card.'),
                         ['GainLife(PlayerSet::Controller, 1)', 'Draw(PlayerSet::Controller, 1)'])
        self.assertEqual(generate.statement_program('you may draw a card. If you do, discard a card.'),
                         ['May { skip: 2 }', 'Draw(PlayerSet::Controller, 1)', 'Discard(PlayerSet::Controller, 1)'])
        for text in [None, '', 'Draw two cards. Unknown instruction.', 'You gain X life.',
                     'Draw a card if you control a Hero.', 'Draw a card. You gain 2 life',
                     'You gain 999999 life.', 'you may discard a card. When you do, draw a card.']:
            self.assertIsNone(generate.statement_program(text), text)

    def test_trigger_creatures_require_complete_supported_abilities(self):
        card = self.card(oracle_text='Flying\nWhen this creature enters, draw a card.')
        self.assertEqual(generate.trigger_creature_template(card),
                         ('Creature(2, 1)', 1, [('SelfEnters', ['Draw(PlayerSet::Controller, 1)'], 'None')]))
        card['oracle_text'] += '\nWhen this creature dies, you gain 2 life.'
        self.assertEqual(len(generate.trigger_creature_template(card)[2]), 2)
        for change in [dict(oracle_text=None), dict(oracle_text=card['oracle_text'] + '\n{T}: Draw a card.'),
                       dict(oracle_text='When this creature enters, draw a card if you control an Elf.'),
                       dict(oracle_text='When this creature enters, draw a card.\nWard—Discard two cards.'),
                       dict(mana_cost='{G/U/P}{G/U/P}'), dict(layout='transform'),
                       dict(type_line='Snow Creature — Cat'), dict(toughness='*')]:
            self.assertIsNone(generate.trigger_creature_template(dict(card, **change)), change)

    def test_program_spells_and_trigger_filters_remain_exact(self):
        card = self.card(type_line='Instant', oracle_text='Draw two cards.')
        self.assertEqual(generate.spell_program_template(card), ('None', ['Draw(PlayerSet::Controller, 2)']))
        self.assertIsNone(generate.spell_program_template(dict(card, type_line='Creature — Wizard')))
        self.assertEqual(generate.spell_program_template(dict(card, oracle_text='Draw two cards. Mill a card.')), ('None',['Draw(PlayerSet::Controller, 2)',generate.mill_instruction('Controller',1)]))
        for prefix, condition in [('Whenever another creature you control enters, ', 'OtherCreatureEntersYou'),
                                  ('Whenever this creature or another creature you control enters, ', 'CreatureEntersYou'),
                                  ('Landfall — Whenever a land you control enters, ', 'LandEntersYou'),
                                  ('Whenever another artifact you control enters, ', 'OtherArtifactEntersYou')]:
            row = self.card(oracle_text=prefix + 'you gain 1 life.')
            self.assertEqual(generate.trigger_creature_template(row)[2][0][0], condition)

    def test_targeted_program_specs_and_optional_effects_are_distinct(self):
        self.assertEqual(generate.targeted_program('you may destroy target artifact or enchantment.'),
                         ('ArtifactEnchantment', ['May { skip: 1 }', 'Destroy']))
        self.assertEqual(generate.targeted_program('tap up to two target creatures an opponent controls.'),
                         (('CreatureOpponent', 0, 2), ['Tap']))
        self.assertEqual(generate.targeted_program('Untap one or two target creatures. They each get +2/+2 until end of turn.'),
                         (('Creature', 1, 2), ['Untap', 'Grow { subject: Subject::Targets, power: 2, toughness: 2 }']))
        self.assertEqual(generate.targeted_program('Target creature gets -2/-2 until end of turn. You gain 2 life.'),
                         ('Creature', ['Grow { subject: Subject::Targets, power: -2, toughness: -2 }', 'GainLife(PlayerSet::Controller, 2)']))
        for text in ['Target creature gets +1/+X until end of turn.', 'Tap target creature. Mill X cards.',
                     'Return target creature card from your graveyard to the battlefield with a counter.',
                     'Tap target creature. It does not untap next turn.']:
            self.assertIsNone(generate.targeted_program(text), text)

    def test_targeted_graveyard_and_player_effects_keep_their_filters(self):
        self.assertEqual(generate.targeted_program('Return target creature card from your graveyard to the battlefield.'),
                         ('CreatureGraveYou', ['ReturnToBattlefield']))
        self.assertEqual(generate.targeted_program('target player draws a card and loses 1 life.'),
                         ('Player', ['Draw(PlayerSet::Targeted, 1)', 'LoseLife(PlayerSet::Targeted, 1)']))
        self.assertEqual(generate.targeted_program('target opponent discards a card.'),
                         ('Opponent', ['Discard(PlayerSet::Targeted, 1)']))
        self.assertIn('min: 0, max: 2', generate.rust_targets(('CreatureOpponent', 0, 2)))

    def test_spell_subtypes_do_not_enable_extra_card_types_or_partial_text(self):
        row = self.card(name='Test', type_line='Instant — Lesson',
                        oracle_text='Untap one or two target creatures. They each get +2/+2 until end of turn.')
        self.assertIsNotNone(generate.spell_program_template(row))
        self.assertIsNone(generate.spell_program_template(dict(row, type_line='Kindred Instant — Elf')))
        self.assertIsNone(generate.spell_program_template(dict(row, oracle_text=row['oracle_text'] + ' Draw a card if you control an Elf.')))

    def test_counter_clauses_require_exact_subject_type_and_complete_text(self):
        self.assertEqual(generate.targeted_program('Put four -1/-1 counters on target creature.'),
                         ('Creature', ['PutCounters { subject: Subject::Targets, kind: CounterKind::MinusOne, amount: 4 }']))
        self.assertEqual(generate.targeted_program('Tap up to one target creature and put a stun counter on it.')[0],
                         ('Creature', 0, 1))
        self.assertEqual(generate.statement_program('Put a +1/+1 counter on this creature.'),
                         ['PutCounters { subject: Subject::Source, kind: CounterKind::PlusOne, amount: 1 }'])
        for text in ['Put X +1/+1 counters on target creature.',
                     'Put a +1/+1 counter on target creature. It gains flying.',
                     'Tap up to two target creatures and put a stun counter on it.',
                     'Put a shield counter on target creature.']:
            self.assertIsNone(generate.targeted_program(text), text)
        self.assertIsNone(generate.trigger_creature_template(self.card(
            oracle_text='When this creature enters, put a +1/+1 counter on this creature.\nOther creatures get +1/+1.')))

    def test_fixed_token_characteristics_and_counts_are_not_conflated(self):
        generate.TOKEN_DEFS.clear()
        normal = generate.statement_program('Create two 1/1 red Goblin creature tokens.')
        tapped = generate.statement_program('Create a tapped 1/1 red Goblin creature token.')
        self.assertEqual(len(generate.TOKEN_DEFS), 1)
        self.assertIn('amount: 2, tapped: false', normal[0])
        self.assertIn('amount: 1, tapped: true', tapped[0])
        generate.statement_program('Create a 1/1 black and red Goblin creature token.')
        self.assertEqual(len(generate.TOKEN_DEFS), 2)
        self.assertEqual({factory['colors'] for factory in generate.TOKEN_DEFS.values()}, {8, 12})
        generate.statement_program('Create a 2/2 colorless Wizard Soldier creature token named Cadet with flying.')
        factory = list(generate.TOKEN_DEFS.values())[-1]
        self.assertEqual((factory['name'], factory['subtypes'], factory['keywords']), ('Cadet', 'Wizard Soldier', 1))
        token = generate.statement_program('Create a 1/1 black Rat creature token with "This token can\'t block."')
        self.assertIsNotNone(token)
        self.assertEqual(list(generate.TOKEN_DEFS.values())[-1]['keywords'], 2048)

    def test_token_compilation_rejects_missing_dependencies_and_residual_clauses(self):
        for text in ['Create a Powerstone token.', 'Create a Blood token.', 'Create a Junk token.',
                     'Create X 1/1 white Rabbit creature tokens.',
                     'Create a 1/1 white Rabbit creature token for each creature exiled this turn.',
                     'Create a 1/1 white Rabbit creature token with ward {2}.',
                     'Create a 1/1 white Rabbit creature token with vigilance. It gains haste.',
                     'Create a 1/1 white Rabbit creature token that is tapped and attacking.',
                     'Create a token that is a copy of target creature.',
                     'Create a 1/1 white and colorless Rabbit creature token.',
                     'Create a 1/1 white Rabbit creature token with "Draw a card."']:
            self.assertIsNone(generate.statement_program(text), text)
        result = generate.statement_program('Draw two cards. Create a 1/1 white Glimmer enchantment creature token.')
        self.assertEqual(len(result), 2)
        self.assertIn('Draw', result[0])
        self.assertIn('CreateTokens', result[1])

    def test_factory_emission_uses_only_whole_accepted_cards_and_is_repeatable(self):
        import json, hashlib
        accepted = self.card(name='Factory Test', oracle_id='factory-test', type_line='Sorcery',
                             oracle_text='Create two 1/1 red Goblin creature tokens.')
        rejected = self.card(name='Rejected Factory', oracle_id='rejected-factory',
                             oracle_text='When this creature enters, create a 4/4 green Beast creature token.\nWard—Discard two cards.')
        cards = [accepted, rejected]
        canonical = json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
        snapshot = {'cards': cards, 'card_count': len(cards), 'cards_sha256': hashlib.sha256(canonical).hexdigest()}
        target = {'banned_card_names': []}
        rust, coverage = generate.compile_catalogue(snapshot, target)
        self.assertEqual(coverage['token_definitions'], 1)
        self.assertEqual(coverage['implemented_standard_cards'], 1)
        self.assertNotIn('Beast Token', rust)
        self.assertEqual(generate.compile_catalogue(snapshot, target), (rust, coverage))

    def test_activated_costs_and_restrictions_require_complete_supported_text(self):
        result = generate.permanent_program_template(self.card(
            oracle_text='{1}, {T}: Target creature gets +1/+1 until end of turn. Activate only as a sorcery.'))
        self.assertIsNotNone(result)
        ability = result[3][0]
        self.assertEqual((ability[0]['generic'], ability[0]['tap'], ability[3]), (1, True, True))
        self.assertEqual(generate.activation_cost('Remove two counters from this creature')['remove'], (None, 2))
        self.assertEqual(generate.activation_cost('Remove a stun counter from this artifact')['remove'], ('Stun', 1))
        self.assertEqual(generate.activation_cost('{T}, Pay 2 life')['life'], 2)
        for line in ['{X}: Draw a card.', '{1}, Discard two cards: Draw a card.',
                     '{1}, Sacrifice two creatures: Draw a card.',
                     '{1}: Draw a card. Activate only if you control a Hero with power 4 or greater.',
                     'Power-up — {1}: Draw a card.', '{1}: Add {U}. Draw a card.',
                     '{1}: Draw a card. Unknown instruction.',
                     '{1}: Draw a card. Activate only once.',
                     'Pay 999999 life: Draw a card.']:
            self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=line)), line)
        self.assertIsNotNone(generate.permanent_program_template(self.card(
            type_line='Legendary Creature — Cat', oracle_text='{1}: Draw a card.')))

    def test_permanent_entry_counters_and_all_abilities_are_compiled(self):
        row = self.card(oracle_text='This creature enters with two -1/-1 counters on it.\n'
                                  '{2}, Remove two counters from this creature: Target creature gets -2/-2 until end of turn. Activate only as a sorcery.')
        result = generate.permanent_program_template(row)
        self.assertEqual(result[4], [0, 2, 0])
        self.assertIsNotNone(result[3][0][0]['remove'])
        self.assertIsNone(generate.permanent_program_template(dict(row,
            oracle_text=row['oracle_text'] + '\nOther creatures get +1/+1.')))
        conditional = self.card(oracle_text='This creature enters with a +1/+1 counter on it if a creature died this turn.')
        self.assertIsNone(generate.permanent_program_template(conditional))
        artifact = self.card(type_line='Artifact — Food', oracle_text='Flash\nWhen this artifact enters, draw a card.\n{2}, {T}, Sacrifice this artifact: You gain 3 life.')
        self.assertEqual(generate.permanent_program_template(artifact)[0], 'Permanent')
        restricted = self.card(oracle_text='{1}: Draw a card. Activate only once each turn.')
        self.assertTrue(generate.permanent_program_template(restricted)[3][0][4])

    def test_mana_parser_distinguishes_generic_colorless_and_choice_production(self):
        self.assertEqual(generate.fixed_cost('{2}{C}{G}'), ([0,0,0,0,1,1],2))
        self.assertEqual(generate.mana_program('add {C}{C}.'), ['AddMana(ManaProduction::Fixed([0, 0, 0, 0, 0, 2]))'])
        self.assertIn('mask: 31, amount: 3', generate.mana_program('add three mana of any one color.')[0])
        self.assertIn('mask: 22', generate.mana_program('add {B}, {G}, or {U}.')[0])
        for text in ['add {G}. Spend this mana only to cast a creature spell.',
                     'add an amount of {G} equal to your power.', 'add {X}.']:
            self.assertIsNone(generate.mana_program(text))
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='{T}: Add {G}. Draw a card.')))

    def test_predefined_artifacts_have_complete_activation_definitions(self):
        generate.TOKEN_DEFS.clear()
        for kind in generate.PREDEFINED:
            self.assertIsNotNone(generate.statement_program(f'Create a {kind} token.'))
            result = generate.permanent_program_template(dict(self.card(), type_line='Artifact', mana_cost='{0}',
                                                              oracle_text=generate.PREDEFINED[kind]))
            self.assertIsNotNone(result, kind)
            self.assertEqual(result[0], 'Permanent')
            self.assertEqual(len(result[3]),1)
        for kind in ['Powerstone','Blood','Junk','Map','Shard']:
            self.assertIsNone(generate.statement_program(f'Create a {kind} token.'))
        self.assertIsNotNone(generate.statement_program('Investigate.'))

    def test_funding_kernel_admits_finite_conversion_and_gates_unbounded_free_production(self):
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='{0}: Add {G}.')))
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text='{1}: Add {G}{G}.')))
        for text in ['{2}: Add one mana of any color.',
                     '{0}, {T}: Add {G}.',
                     '{0}: Add {G}. Activate only once each turn.']:
            self.assertIsNotNone(generate.permanent_program_template(self.card(oracle_text=text)),text)
        self.assertTrue(generate.activation_cost('{0}, {T}')['mana_component'])
        self.assertFalse(generate.activation_cost('{T}')['mana_component'])

    def test_continuous_keyword_grants_compile_only_supported_layer_six_scopes(self):
        for subject, expected in [('Creatures you control','relation: 0'),
                                  ('Other creatures','other: true'),
                                  ('Giants you control','subtype: Some("Giant")'),
                                  ('Attacking creature tokens you control','attacking: true'),
                                  ('Artifact creatures you control','artifact: true')]:
            result=generate.static_permanent_template(self.card(oracle_text=subject+' have haste and flying.'))
            self.assertIsNotNone(result,subject)
            self.assertIn(expected,result[6][0]); self.assertIn('keywords: Keywords(5)',result[6][0])
        result=generate.static_permanent_template(self.card(oracle_text='During your turn, this creature has first strike.'))
        self.assertIn('source_only: true, own_turn: true',result[6][0])
        result=generate.static_permanent_template(self.card(oracle_text='Other Cats you control get +1/+1 and have lifelink.'))
        self.assertEqual(len(result[5]),1);self.assertEqual(len(result[6]),1)
        for text in ['Creatures you control with flying have haste.',
                     'Creatures you control have changeling.',
                     'Creatures you control have ward {2}.',
                     'Creatures you control lose flying.',
                     'Creatures you control have flying as long as you control an Island.',
                     'As long as you control a Mountain, creatures you control have haste.',
                     'During your turn, creatures you control get +1/+1.',
                     'Creatures you control have haste.\nWhenever you cast a spell, proliferate.',
                     'Creatures you control have haste.\nThis creature has base power and toughness 4/4.']:
            self.assertIsNone(generate.static_permanent_template(self.card(oracle_text=text)),text)
        self.assertIsNone(generate.static_permanent_template(self.card(type_line='Artifact',oracle_text='This creature has first strike.')))

    def test_continuous_keyword_admissions_are_complete(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Aggressive Mammoth','Bearer of Glory','Boldwyr Aggressor','Cosmic Spider-Man',
                     'Feisty Spikeling','Krang, Utrom Warlord','Null Group Biological Assets',
                     'Okoye, Dora Milaje Leader','Regal Caracal',"Samut, Hazoret's Champion",
                     'Skilled Battlecarver','Vulture, Scheming Scavenger']:
            self.assertIn(name,supported)

    def test_pre_entry_life_payment_requires_exact_complete_land_text(self):
        def land(text,typ='Land'):
            return self.card(type_line=typ,mana_cost='',oracle_text=text)
        clause="As this land enters, you may pay 2 life. If you don't, it enters tapped."
        for amount in [1,2,10,65535]:
            result=generate.land_template(land(clause.replace('2 life',str(amount)+' life')+'\n{T}: Add {C}.'))
            self.assertEqual(result[5],f'PayLife({amount})')
        result=generate.land_template(land('({T}: Add {W} or {U}.)\n'+clause,'Land — Plains Island'))
        self.assertEqual(result[5],'PayLife(2)');self.assertIn('mask: 3',result[3][0][2][0])
        for text in [clause+'\n'+clause+'\n{T}: Add {G}.',
                     clause+'\nThis land enters tapped.\n{T}: Add {G}.',
                     'This land enters tapped.\n'+clause+'\n{T}: Add {G}.',
                     clause+'\n{T}: Add {G}.\nWhen this land enters, proliferate.',
                     clause.replace('2 life','0 life')+'\n{T}: Add {G}.',
                     clause.replace('2 life','65536 life')+'\n{T}: Add {G}.',
                     clause.replace('2 life','X life')+'\n{T}: Add {G}.',
                     clause.replace('2 life','-2 life')+'\n{T}: Add {G}.',
                     clause.replace("If you don't, it enters tapped.",'Draw a card.')+'\n{T}: Add {G}.',
                     'This land enters tapped unless you control two planeswalkers.\n{T}: Add {G}.']:
            self.assertIsNone(generate.land_template(land(text)),text)
        self.assertIsNone(generate.land_template(land('({T}: Add {W}.)\n'+clause,'Land — Plains Island')))
        self.assertIsNone(generate.permanent_program_template(self.card(oracle_text=clause)))

    def test_all_twenty_new_land_admissions_preserve_complete_bodies(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        implemented={c['name'] for c in coverage['cards'] if c['implemented']}
        shocks=['Blood Crypt','Breeding Pool','Godless Shrine','Hallowed Fountain','Overgrown Tomb',
                'Sacred Foundry','Steam Vents','Stomping Ground','Temple Garden','Watery Grave']
        annexes=['Dedicated Commons','Fatehold Annex','Formidable Commons','Innovative Commons','Konstrari Annex',
                 'Meticulous Commons','Stingerquill Annex','Theorix Annex','Transformative Commons','Vigorbloom Annex']
        for name in shocks+annexes:self.assertIn(name,implemented)
        for name in ['Uncharted Haven','Radiant Summit','Prismatic Vista']:
            self.assertNotIn(name,implemented)

    def test_controlled_subtype_activation_guards_are_exact_and_complete(self):
        for phrase,expected in [('a Dragon','["Dragon"]'),('a Hero','["Hero"]'),('an Elf','["Elf"]'),
                                ('a Plains or an Island','["Island", "Plains"]'),
                                ('a Swamp or a Mountain','["Mountain", "Swamp"]')]:
            row=self.card(type_line='Artifact',oracle_text='{T}: Add {G}. Activate only if you control '+phrase+'.')
            result=generate.permanent_program_template(row)
            self.assertIsNotNone(result)
            self.assertEqual(result[3][0][0]['guard'],'ControlSubtypeAny(&'+expected+')')
        for text in ['{T}: Add {G}. Activate only if an opponent controls a Dragon.',
                     '{T}: Add {G}. Activate only if you control a Dragon and an Elf.',
                     '{T}: Add {G}. Activate only if you control two Dragons.',
                     '{T}: Add {G}. Activate only if you control a tapped Dragon.',
                     '{T}: Add {G}. Activate only if you control a creature with toughness 4 or greater.',
                     '{T}: Add {G}. Activate only if you control a Dragon. Draw a card.',
                     '{T}: Add {G}. Activate only if you control a Dragon. Activate only during combat.',
                     '{T}: Add {G}. Activate only if you control a Nonsense.',
                     '{T}: Add {G}. Activate only if you control a Dragon.\nWhenever you tap this artifact, proliferate.']:
            self.assertIsNone(generate.permanent_program_template(self.card(type_line='Artifact',oracle_text=text)),text)
        self.assertIsNone(generate.permanent_program_template(self.card(type_line='Artifact',oracle_text='{0}: Add {G}. Activate only if you control a Dragon.')))

    def test_all_verges_and_jasper_admit_full_bodies_and_wider_conditions_stay_gated(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Blazemire Verge','Bleachbone Verge','Floodfarm Verge','Gloomlake Verge','Hushwood Verge',
                     'Riverpyre Verge','Sunbillow Verge','Thornspire Verge','Wastewood Verge','Willowrush Verge','Mox Jasper']:
            self.assertIn(name,supported)
        for name in ['Baxter Building','Bloom Tender','Springleaf Drum','Supportive Parents','Uncharted Haven']:
            self.assertNotIn(name,supported)

    def test_dynamic_stat_extraction_preserves_unrelated_text_and_exact_formulas(self):
        source=self.card(name='Creature',oracle_text='Flying',power='2',toughness='1')
        proxy,base,scales=generate.extract_stat_rules(source)
        self.assertIs(proxy,source);self.assertIsNone(base);self.assertEqual(scales,[])
        source=self.card(name='Tester',power='*',toughness='1+*',oracle_text="Tester's power is equal to the number of cards you own in exile and its toughness is equal to that number plus 1.")
        proxy,base,scales=generate.extract_stat_rules(source)
        self.assertEqual(proxy['power'],'0');self.assertEqual(proxy['toughness'],'0')
        self.assertIn('CountZone::Exile',base);self.assertIn('offset: 1',base);self.assertEqual(scales,[])
        source=self.card(oracle_text='This creature gets +2/+0 for each other Goblin you control.')
        proxy,base,scales=generate.extract_stat_rules(source)
        self.assertEqual(proxy['oracle_text'],'');self.assertIn('other: true',scales[0]);self.assertIn('power: 2, toughness: 0',scales[0])
        source=self.card(type_line='Enchantment — Aura',oracle_text='Enchant creature\nEnchanted creature gets +1/+1 for each enchantment you control and has first strike.')
        proxy,base,scales=generate.extract_stat_rules(source)
        self.assertIn('Enchanted creature has first strike.',proxy['oracle_text'])
        self.assertIn('Subject::Attached',scales[0]);self.assertIsNotNone(generate.aura_template(proxy))
        for text in ['This creature gets +1/+1 for each creature you control with power 4 or greater.',
                     'This creature gets +1/+1 for each color among permanents you control.',
                     'This creature gets +1/+1 for each Forest you control until end of turn.',
                     'This creature gets +1/+1 for each creature you control if you control a Dragon.',
                     'This creature gets +1/+1 for each opponent.',
                     'During your turn, this creature gets +1/+1 for each Forest you control.']:
            source=self.card(oracle_text=text)
            proxy,base,scales=generate.extract_stat_rules(source)
            self.assertIsNone(base);self.assertEqual(scales,[],text);self.assertIsNone(generate.permanent_program_template(proxy),text)
        source=self.card(name='Tester',power='*',toughness='3',oracle_text="Tester's power and toughness are each equal to the number of lands you control.")
        proxy,base,scales=generate.extract_stat_rules(source);self.assertIsNone(base)
        self.assertIsNone(generate.creature_template(proxy))

    def test_stat_count_predicates_reject_unimplemented_or_ambiguous_facts(self):
        for text,expected in [('basic Island you control','basic: true'),('other Goblin you control','other: true'),
                              ('instant and sorcery cards in your graveyard','kind: 12'),
                              ('noncreature, nonland card in your graveyard','exclude: 3'),
                              ('cards you own in exile','CountZone::Exile')]:
            self.assertIn(expected,generate.stat_count_filter(text))
        for text in ['creatures your opponents control with flying','artifact or creature you control',
                     'cards in all graveyards','cards you control','basic Equipment you control','permanents you own','cards in your library']:
            self.assertIsNone(generate.stat_count_filter(text),text)
        self.assertIn('StatValue::Sum',generate.stat_value('creature you control and each creature card in your graveyard'))
        self.assertEqual(generate.stat_value('card types among cards in all graveyards'),'StatValue::GraveyardCardTypes')

    def test_dynamic_stat_admissions_are_complete_and_keep_previous_typed_lands(self):
        import json
        rows=json.loads((generate.ROOT/'data/standard/cards.json').read_text())
        target=json.loads((generate.ROOT/'data/targets/standard-2026-10-15.json').read_text())
        _,coverage=generate.compile_catalogue(rows,target)
        supported={c['name'] for c in coverage['cards'] if c['implemented']}
        for name in ['Adelbert Steiner','Blanchwood Armor','Boneclub Berserker','Burrowguard Mentor','Cosmogoyf',
                     'Crusader of Odric','Enigma Drake','Ethereal Armor','Guidelight Synergist','Harmonious Grovestrider',
                     'Kraven, Proud Predator','Mirkwood Pathmaker','Moon-Vigil Adherents','Regal Bunnicorn','Snow Villiers',
                     'Tarmogoyf','Tempest Djinn','Xande, Dark Mage','Breeding Pool','Undercity Sewers','Hallowed Fountain']:
            self.assertIn(name,supported)
        for name in ['Diligent Zookeeper','Melek, Reforged Researcher','Namor the Sub-Mariner',
                     'Improvised Arsenal','With Great Power . . .','Zell Dincht','Lumra, Bellow of the Woods']:
            self.assertNotIn(name,supported)


if __name__ == '__main__':
    unittest.main()
