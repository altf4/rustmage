"""Complete recovery grammar, name independence, reminders and residual controls."""
import copy
import json
import re
import unittest
from scripts import generate_catalogue as g
from scripts import test_legacy_metadata as metadata

NAMES = ["Archenemy's Charm", 'Auroral Procession', 'Barkform Harvester',
         'Coati Scavenger', 'Dundoolin Weaver', 'Elvish Regrower',
         'Fateshaper Aspirant', 'Fight On!', "Hazel's Nocturne", 'Macabre Waltz',
         "Morcant's Loyalist", 'Oltec Archaeologists', 'Pull from the Grave',
         'Restore with Empathy', "Rydia's Return", 'Tomb Trawler',
         'Tombstone, Career Criminal', 'True Ancestry', 'White Widow, Free Agent']

class GraveyardRecoveryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cards = {c['name']: c for c in json.loads((g.ROOT/'data/standard/cards.json').read_text())['cards']}

    def test_all_eighteen_complete_programs_are_name_independent(self):
        for name in NAMES:
            if name == "Morcant's Loyalist":
                self.assertFalse(metadata.LegacyMetadataTests().definition(self.cards[name])[1]["implemented"])
                continue
            with self.subTest(name=name):
                card = copy.deepcopy(self.cards[name])
                original, row = metadata.LegacyMetadataTests().definition(card)
                self.assertTrue(row['implemented'])
                renamed = 'Independent Recovery Subject'
                card['oracle_text'] = card['oracle_text'].replace(name, renamed).replace(name.split(',')[0], renamed)
                card['name'] = renamed
                replacement, row = metadata.LegacyMetadataTests().definition(card)
                self.assertTrue(row['implemented'])
                self.assertEqual(original.replace(json.dumps(name), json.dumps(renamed)), replacement)
                # Include helper arms (notably Tombstone's cost reduction), token
                # recipes and colors, allowing only identity and lookup ordering.
                before=metadata.LegacyMetadataTests.compile([self.cards[name]])[0]
                after=metadata.LegacyMetadataTests.compile([card])[0]
                def semantics(text):
                    lines=[line for line in text.splitlines() if not line.startswith('pub const CATALOGUE_VERSION:')]
                    lookups=[line for line in lines if re.fullmatch(r'    \("(?:[^"\\]|\\.)*", [0-9]+\),',line)]
                    return [line for line in lines if line not in lookups],sorted(lookups)
                self.assertEqual(semantics(before.replace(json.dumps(name),json.dumps(renamed))),semantics(after))

    def test_unknown_residuals_and_parentheses_gate_whole_bodies(self):
        mutants = []
        for name in NAMES:
            base = self.cards[name]
            for index, text in enumerate([base['oracle_text'] + tail for tail in
                    ['\nDiscover 4.', ' (You win the game.)', '\n(You win the game.)',
                     ' ((You win the game.))', ' (', ' )']] +
                    [base['oracle_text'].replace('from your graveyard', 'from your (You win the game.)graveyard'),
                     base['oracle_text'].replace('from your graveyard', 'from your ((You win the game.))graveyard')]):
                c = dict(base, name=f'Invalid Recovery {len(mutants)}', oracle_text=text.replace(name, f'Invalid Recovery {len(mutants)}').replace(name.split(',')[0], f'Invalid Recovery {len(mutants)}'))
                mutants.append(c)
        _, coverage = metadata.LegacyMetadataTests.compile(mutants)
        rows = {r['name']: r for r in coverage['cards']}
        for card in mutants:
            self.assertFalse(rows[card['name']]['implemented'], card['oracle_text'])

    def test_reminders_match_actual_production_position_and_parameters(self):
        mutants = []
        for name in ['Barkform Harvester','Fateshaper Aspirant','Oltec Archaeologists','True Ancestry']:
            card = self.cards[name]
            reminder = re.search(r'\([^()]*\)', card['oracle_text'])[0]
            bare = card['oracle_text'].replace(reminder, '').rstrip()
            texts = [card['oracle_text'].replace(reminder, '(You win the game.)'),
                     card['oracle_text'].replace(reminder, '(' + reminder + ')'),
                     bare + '\n' + reminder,
                     card['oracle_text'].replace(reminder, reminder + ' ' + reminder),
                     card['oracle_text'].replace('from your graveyard', reminder+'from your graveyard')]
            if name == 'Oltec Archaeologists': texts += [card['oracle_text'].replace('Scry 3.', 'Scry 2.')]
            if name == 'True Ancestry': texts += [card['oracle_text'].replace('Create a Clue token.', 'Create a Food token.')]
            if name == 'Fateshaper Aspirant': texts += [card['oracle_text'].replace('vigilance and indestructible', 'vigilance')]
            for text in texts:
                new = f'Bad Reminder {len(mutants)}'
                mutants.append(dict(card, name=new, oracle_text=text.replace(name,new)))
        _, cov = metadata.LegacyMetadataTests.compile(mutants)
        rows = {r['name']: r for r in cov['cards']}
        for c in mutants: self.assertFalse(rows[c['name']]['implemented'], c['oracle_text'])

    def test_typed_bounds_filters_destinations_and_cardinality(self):
        for noun, expected in [('permanent','TargetPredicate::Permanent'),
                               ('nonland permanent','nonland: true'),
                               ('legendary','TargetPredicate::Legendary'),
                               ('Elf','TargetPredicate::Subtype("Elf")'),
                               ('creature and/or planeswalker','kinds: 65'),
                               ('artifact or enchantment','kinds: 48'),
                               ('creature with mana value 3 or less','max_mana: 3')]:
            spec, program = g.targeted_program(f'Return up to two target {noun} cards from your graveyard to your hand.')
            self.assertEqual(spec[1:], (0,2)); self.assertEqual(program,['ReturnToHand'])
            self.assertIn(expected,g.TARGET_RULES[spec[0]])
        for text in ['Return target noncreature card from your graveyard to your hand.',
                     'Return two target creature cards that share a creature type from your graveyard to your hand.',
                     'Return target card from exile to your hand.',
                     'Return target permanent card from your graveyard to your hand. Untap it.',
                     'Put target card from your graveyard on the bottom of your library. Untap it.',
                     'Return target creature card with mana value 4294967296 or less from your graveyard to your hand.',
                     'Return up to three target creature cards from your graveyard to your hand.']:
            self.assertIsNone(g.targeted_program(text), text)

    def test_invalid_types_costs_and_physical_source_exclusion_stay_gated(self):
        mutants=[]
        for name in NAMES:
            for change in [dict(type_line='Unknown'), dict(mana_cost='{Y}'),
                           dict(mana_cost='{4294967296}'), dict(oracle_text=self.cards[name]['oracle_text']+'\nDiscover 4.')]:
                renamed=f'Invalid Recovery Metadata {len(mutants)}'
                c=dict(self.cards[name], **change)
                c['oracle_text']=c['oracle_text'].replace(name,renamed).replace(name.split(',')[0],renamed)
                c['name']=renamed; mutants.append(c)
        _,cov=metadata.LegacyMetadataTests.compile(mutants)
        rows={r['name']:r for r in cov['cards']}
        for c in mutants:self.assertFalse(rows[c['name']]['implemented'], c)
        self.assertIsNone(g.targeted_program('Return another target Elf card from your graveyard to your hand.'))

    def test_pinned_java_complete_card_and_direct_dependency_provenance(self):
        import hashlib
        import subprocess
        from pathlib import Path
        manifest=json.loads((g.ROOT/'docs/graveyard-recovery-admissions.json').read_text())
        java=Path('/home/altf4/Code/mage')
        self.assertEqual({r['name'] for r in manifest['cards']},set(NAMES))
        checked=set()
        for row in manifest['cards']:
            for key in ['oracle_id','mana_cost','type_line','oracle_text']:
                self.assertEqual(row[key],self.cards[row['name']][key])
            for source in row['sources']:
                name=source['java_source']
                if name in checked:continue
                raw=(java/name).read_bytes()
                pinned=subprocess.check_output(['git','-C',str(java),'show',manifest['java_revision']+':'+name])
                self.assertEqual(raw,pinned,name)
                self.assertEqual(hashlib.sha256(raw).hexdigest(),source['java_sha256'],name)
                checked.add(name)
        self.assertEqual(len(checked),69)

    def test_all_prior_programs_helpers_colors_recipes_preserved_except_named_admissions(self):
        import subprocess
        old=subprocess.check_output(['git','-C',str(g.ROOT),'show','0977111f57734f6e4e22bf6c3c68a88358d29da9:src/generated/catalogue.rs'],text=True)
        new=(g.ROOT/'src/generated/catalogue.rs').read_text()
        definitions=lambda text:[line for line in text.splitlines() if line.lstrip().startswith('Definition {')]
        before,after=definitions(old),definitions(new)
        expected=set(NAMES)-{"Morcant's Loyalist"}
        self.assertEqual(len(before),5311);self.assertEqual(len(after),5311)
        changed=[(a,b) for a,b in zip(before,after) if a!=b]
        self.assertEqual(len(changed),18)
        changed_names={json.loads(re.search(r'name: ("(?:[^"\\]|\\.)*")',b)[1]) for _,b in changed}
        self.assertEqual(changed_names,expected)
        for a,b in changed:self.assertIn('effect: Effect::Unsupported',a)
        # Existing helper arms stay byte-identical; only the complete newly
        # admitted Tombstone reducer adds a helper entry. Loyalist stays gated.
        import difflib
        removed=[l[2:] for l in difflib.ndiff(old.splitlines(),new.splitlines()) if l.startswith('- ')]
        self.assertEqual(len(removed),20) # 18 definition rows, fingerprint, ALL.
        self.assertTrue(all(l.startswith(('    Definition {','pub const CATALOGUE_VERSION:','    pub const ALL:')) for l in removed))



    def test_infix_reminders_cannot_evade_route_detection_at_any_word_boundary(self):
        routes = ['from your graveyard to your hand',
                  'from your graveyard on the top of your library',
                  'from your graveyard on the bottom of your library']
        for route in routes:
            words = route.split()
            for index in range(1, len(words)):
                for reminder in ['(Draw seven cards.)', '((Draw seven cards.))']:
                    for separator in [' ', '\t', '\n', '  ']:
                        text = separator.join(words[:index]) + separator + reminder + separator + separator.join(words[index:])
                        with self.subTest(text=text):
                            self.assertTrue(g.recovery_route(text))
                            self.assertFalse(g.recovery_reminders_valid({'oracle_text':text}))

    def test_full_bodies_reject_unknown_infix_reminders_across_every_recovery_boundary(self):
        mutants = []
        for name in NAMES:
            if name == "Morcant's Loyalist": continue
            base = self.cards[name]
            route = re.search(r'from your graveyard (?:to your hand|on (?:the )?(?:top|bottom) of your library)', base['oracle_text'])
            self.assertIsNotNone(route, name)
            words = route[0].split()
            for index in range(1,len(words)):
                for reminder in ['(Draw seven cards.)', '((Draw seven cards.))']:
                    for separator in [' ', '\t', '\n']:
                        replacement = ' '.join(words[:index]) + separator + reminder + separator + ' '.join(words[index:])
                        text = base['oracle_text'][:route.start()] + replacement + base['oracle_text'][route.end():]
                        renamed = f'Infix Recovery Mutation {len(mutants)}'
                        mutants.append(dict(base, name=renamed, oracle_text=text.replace(name,renamed).replace(name.split(',')[0],renamed)))
        _, coverage = metadata.LegacyMetadataTests.compile(mutants)
        rows = {row['name']:row for row in coverage['cards']}
        for card in mutants:
            self.assertFalse(rows[card['name']]['implemented'],card['oracle_text'])

if __name__ == '__main__': unittest.main()
