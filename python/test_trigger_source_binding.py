"""Paid Standard Morcant sacrifice/recovery with exact public source context."""
import json
import os
from pathlib import Path
import unittest
from rustmage import Game, EngineError
import test_adventure

ROOT=Path(__file__).resolve().parents[1]
EXPECTED=json.loads((ROOT/'data/standard/coverage.json').read_text())['catalogue_version']
LOYALIST="Morcant's Loyalist"
CONVICTION='Corrupted Conviction'


class TriggerSourceBindingTests(unittest.TestCase):
    replay=test_adventure.AdventureTests.replay

    def test_paid_elf_recovery_both_seats(self):
        for p in ['P0','P1']:
            with self.subTest(player=p):self.scenario(p,'Llanowar Elves',2870)

    def test_paid_changeling_recovery_both_seats(self):
        for p in ['P0','P1']:
            with self.subTest(player=p):self.scenario(p,'Three Tree Mascot',2871)

    def test_paid_self_sacrifice_with_no_other_elf_never_recovers_itself_both_seats(self):
        for p in ['P0','P1']:
            with self.subTest(player=p):self.scenario(p,None,2872)

    def scenario(self,player,victim_name,seed,library=None):
        victim_name=victim_name or 'Chrome Companion'
        recover=victim_name!='Chrome Companion'
        filler='Hedron Archive' if not recover else 'Chrome Companion'
        cards=[LOYALIST,CONVICTION,'Candy Trail',victim_name,'Bear Cub',filler,'Bygone Colossus']
        own=['Forest']*16+['Swamp']*16+sum(([n]*4 for n in cards),[])
        self.assertEqual(len(own),60)
        enemy=['Forest']*60
        library=library or os.environ.get('RUSTMAGE_TEST_LIBRARY')
        with Game([own,enemy] if player=='P0' else [enemy,own],seed=seed,starting_player=player,deck_rules='Standard',library=library) as g:
            source=None;victim=None;stage='setup';seen=False;frames=0;paid=0
            for iteration in range(16000):
                head=g.observe('P0');self.assertIsNone(head['outcome'],(player,victim_name,stage,iteration))
                o=g.observe(head['actor']);actor=o['actor']
                self.assertEqual((o['schema'],o['catalogue']),(95,EXPECTED))
                objects={v['id']:v for v in o['objects']}
                private=g.observe(player)
                hand=[v for v in private['objects'] if v['zone']=='Hand' and v['owner']==player]
                field=[v for v in objects.values() if v['zone']=='Battlefield' and v['controller']==player]
                quiet=o['decision']=='priority' and not o['stack'] and not o['announcing'] and not o['resolving']
                if quiet and stage=='sac_victim' and objects[victim]['zone']=='Graveyard':stage='kill_source'
                announcing=o['announcing']
                if announcing and announcing['source_card']==LOYALIST and announcing['kind']!='Spell':
                    context=announcing['source_move'];self.assertIsNotNone(context)
                    self.assertEqual((context['origin'],context['destination']),('Battlefield','Graveyard'))
                    self.assertEqual(announcing['object'],context['before'])
                    self.assertEqual(context['after'],objects[source]['reference'])
                    self.assertEqual(announcing['controller'],player)
                    self.assertFalse(any(a['kind']=='ChooseTarget' and a['target'].get('Card')==source for a in o['actions']))
                    seen=True
                if quiet and stage=='sac_source' and paid==2:
                    self.assertEqual(objects[source]['zone'],'Graveyard')
                    self.assertEqual(objects[victim]['zone'],'Hand' if recover else 'Graveyard')
                    self.assertEqual(seen,recover)
                    return dict(player=player,victim=victim_name,frames=frames,recovered=recover,paid_convictions=paid)
                def score(a):
                    kind=a['kind'];card=objects.get(a.get('card'),{});name=card.get('card')
                    if kind in ['Concede','Mulligan','Cast','BeginCast']:return -2000
                    if kind=='AnnounceCast':
                        if actor!=player:return -1000
                        if name==LOYALIST:return 2900 if source is None else -1000
                        if name==victim_name:return 2800 if victim is None else -1000
                        if name==CONVICTION:
                            ready=source is not None and victim is not None and objects[source]['zone']=='Battlefield' and objects[victim]['zone']=='Battlefield'
                            copies=sum(v['card']==CONVICTION for v in hand)
                            return 3000 if stage=='setup' and ready and copies>=2 or stage=='kill_source' else -1000
                        return -1000
                    if kind=='ChooseAdditionalCost':
                        wanted=victim if stage=='sac_victim' else source
                        return 3100 if a['object']['card']==wanted else -1000
                    if kind=='ChooseTarget':return 3100 if a['target'].get('Card')==victim else -1000
                    if kind=='PlayLand':return 2700 if actor==player else 1000
                    if kind in ['Discard','Bottom']:
                        if actor==player and name in [LOYALIST,victim_name,CONVICTION]:return -500
                        return 2400
                    if kind in ['TapMana','ActivateMana','AnnounceMana']:return 2800 if o['decision'] in ['pay_costs','additional_cost','resolution_mana'] else -1000
                    if kind in ['Attack','Block','AnnounceActivate','Activate','BeginActivate']:return -1000
                    return {'Keep':3500,'PayCosts':3300,'ChooseSpellCost':3200,'FinishTargets':3100,'PutTrigger':3000,'FinishAttackers':2500,'FinishBlockers':2500,'Pass':0}.get(kind,1000)
                index=max(range(len(o['actions'])),key=lambda i:score(o['actions'][i]));action=o['actions'][index]
                self.assertGreaterEqual(score(action),0,(player,victim_name,stage,o['decision'],o['actions']))
                if actor==player and action['kind']=='AnnounceCast':
                    name=objects[action['card']]['card']
                    if name==LOYALIST:source=action['card']
                    if name==victim_name:victim=action['card']
                    if name==CONVICTION:stage='sac_victim' if stage=='setup' else 'sac_source'
                if actor==player and action['kind']=='PayCosts' and announcing and announcing['source_card']==CONVICTION:paid+=1
                before=[g.observe(p) for p in ['P0','P1']]
                with self.assertRaisesRegex(EngineError,'WrongActor'):g.act(dict(o,actor='P1' if actor=='P0' else 'P0'),action)
                self.assertEqual(before,[g.observe(p) for p in ['P0','P1']])
                self.replay(g,o,index);frames+=1
            self.fail((player,victim_name,stage,'no paid recovery witness'))


if __name__=='__main__':unittest.main()
