import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('generate', Path(__file__).with_name('generate_catalogue.py'))
generate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(generate)


class CatalogueCompilerTests(unittest.TestCase):
    def card(self, **overrides):
        row = {'layout': 'normal', 'type_line': 'Creature — Cat', 'power': '2', 'toughness': '1',
               'mana_cost': '{W}', 'oracle_text': ''}
        row.update(overrides)
        return row

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
                     'Whenever a creature you control deals combat damage to a player, draw a card. This ability triggers only once each turn.',
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
                     'Pacifism',"Shardmage's Rescue",'Wingspan Stride']:
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
                     'Whenever you gain life, draw a card. This ability triggers only once each turn.',
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
                     'Professional Wrestler','Provisions Merchant','Pulse Tracker','Shroudstomper','Starling, Aerial Ally']:
            self.assertIsNotNone(generate.trigger_creature_template(cards[name]),name)
        for name in ['Engine Rat',"Thrór's Map"]:
            self.assertIsNotNone(generate.permanent_program_template(cards[name]),name)
        for name in ['Bloombrute','Aerith Gainsborough','Vein Ripper','Twinblade Paladin','Titanbones, Towering Heart']:
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
            'At the beginning of your end step, if you gained life this turn, draw a card.',
            'At the beginning of your end step, if you control two or more tapped creatures and a Dragon, draw a card.',
            'At the beginning of your upkeep, draw a card if you control a Dragon.',
            'At the beginning of your upkeep, if you have 40 or more life, you win the game.',
            'At the beginning of your next upkeep, draw a card.',
            'At the beginning of combat on your turn, choose one —\n• Draw a card.\n• You gain 2 life.',
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
            'Whenever you cast your second spell each turn, draw a card. This ability triggers only once each turn.',
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
        for text in ['Prowess, ward {1}', 'Prowess\nYou may cast spells from your graveyard.',
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
            'Choose one —\n• Counter target spell unless its controller pays {2}.\n• Draw two cards.',
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
                     'Other creatures you control get +1/+1.\nOther creatures have flying.',
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
                     "This creature can't be blocked by creatures with power 2 or less.",
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
                       dict(oracle_text='When this creature enters, draw a card.\nWard {2}'),
                       dict(mana_cost='{G/U}{G/U}'), dict(layout='transform'),
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
                     'Create a 1/1 white Rabbit creature token for each creature you control.',
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
                             oracle_text='When this creature enters, create a 4/4 green Beast creature token.\nWard {2}')
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
        for line in ['{X}: Draw a card.', '{1}, Discard a card: Draw a card.',
                     '{1}, Sacrifice another creature: Draw a card.',
                     '{1}: Draw a card. Activate only if you control a Hero.',
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


if __name__ == '__main__':
    unittest.main()
