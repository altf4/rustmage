import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from rustmage import EngineError, Game


class BindingTests(unittest.TestCase):
    def game(self):
        return Game([["Mountain"] * 24 + ["LightningBolt"] * 36] * 2, seed=17, deck_rules="Freeform")

    def test_draw_ordinals_are_public_without_hand_identity_and_replay_native_triggers(self):
        blue=['Island']*24+['Erudite Wizard']*16+['Clinquant Skymage']*12+['Lightshield Parry']*8
        green=['Forest']*24+['Migrating Ketradon']*36
        with Game([blue,green],seed=6,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                items=obs['stack']+obs['waiting_triggers']+([obs['announcing']] if obs['announcing'] else [])
                if any(s['draw_event'] is not None and s['draw_event']['player']=='P0' and s['draw_event']['ordinal']==2 for s in items):break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('ordinal-two trigger not reached')
            self.assertEqual(obs['schema'],29);self.assertEqual(obs['players'][0]['cards_drawn_this_turn'],2)
            event_item=next(s for s in items if s['draw_event'] is not None and s['draw_event']['ordinal']==2)
            self.assertEqual(set(event_item['draw_event']),{'player','ordinal'})
            public=game.observe('P1');public_items=public['stack']+public['waiting_triggers']+([public['announcing']] if public['announcing'] else [])
            self.assertEqual(next(s['draw_event'] for s in public_items if s['id']==event_item['id']),event_item['draw_event'])
            self.assertTrue(all(o['zone']!='Hand' or o['owner']!='P0' for o in public['objects']))
            saved=game.snapshot()
            def finish(handle):
                for _ in range(40):
                    current=handle.observe(handle.observe('P0')['actor'])
                    if not current['stack'] and not current['waiting_triggers'] and current['announcing'] is None:return current
                    index=next((i for i,a in enumerate(current['actions']) if a['kind']=='PutTrigger'),None)
                    if index is None:index=next(i for i,a in enumerate(current['actions']) if a['kind']=='Pass')
                    handle.step(current,index)
                self.fail('draw trigger stack did not finish')
            with game.clone() as branch:expected=finish(branch)
            actual=finish(game);self.assertEqual(actual,expected);self.assertEqual(actual['players'][0]['cards_drawn_this_turn'],2)
            game.restore(saved);replay=finish(game);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay);game.drop_snapshot(saved)

    def test_drawing_player_binding_and_owned_ordinal_survive_native_replay(self):
        deck=['Island']*24+['Scrawling Crawler']*20+['Mischievous Mystic']*16
        with Game([deck,deck],seed=8,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Scrawling Crawler' and obs['stack'][-1]['draw_event'] is not None:break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('opponent draw trigger not reached')
            top=obs['stack'][-1];player=top['draw_event']['player'];p=0 if player=='P0' else 1
            self.assertNotEqual(player,top['controller']);self.assertGreater(top['draw_event']['ordinal'],0)
            before=[s['life'] for s in obs['players']];counts=[s['cards_drawn_this_turn'] for s in obs['players']];saved=game.snapshot()
            def resolve_top(handle):
                for _ in range(2):
                    current=handle.observe(handle.observe('P0')['actor'])
                    handle.step(current,next(i for i,a in enumerate(current['actions']) if a['kind']=='Pass'))
                return handle.observe(handle.observe('P0')['actor'])
            with game.clone() as branch:expected=resolve_top(branch)
            actual=resolve_top(game);self.assertEqual(actual,expected)
            self.assertEqual(actual['players'][p]['life'],before[p]-1);self.assertEqual(actual['players'][1-p]['life'],before[1-p])
            self.assertEqual([s['cards_drawn_this_turn'] for s in actual['players']],counts)
            game.restore(saved);replay=resolve_top(game);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay);game.drop_snapshot(saved)

    def discard_index(self,obs):
        hand={o['id'] for o in obs['objects'] if o['zone']=='Hand' and o['owner']==obs['actor']}
        def score(a):
            k=a['kind']
            if k=='TapMana':return 70 if obs['decision']=='pay_costs' else -10
            if k in ('AnnounceActivate','Activate') and a['card'] in hand:return 60
            return {'Keep':100,'PlayLand':90,'Cast':80,'AnnounceCast':79,'PayCosts':100,'PutTrigger':60,
                    'Discard':50,'DeclineEffectPayment':-1,'ChooseTarget':60,'FinishTargets':0,
                    'FinishAttackers':40,'FinishBlockers':40,'Pass':20,'Mulligan':-100,'Concede':-1000}.get(k,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_cycling_event_context_and_trigger_stack_replay_in_native_binding(self):
        red=['Mountain']*24+['Marauding Mako']*16+['Magmakin Artillerist']*20
        green=['Forest']*24+['Titanbones, Towering Heart']*36
        with Game([red,green],seed=4,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                items=obs['stack']+obs['waiting_triggers']+([obs['announcing']] if obs['announcing'] else [])
                if any(s['source_card']=='Magmakin Artillerist' and s['discard_event'] is not None and s['discard_event']['cycling'] for s in items):break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('cycling-bound trigger not reached')
            actor=obs['actor'];self.assertEqual(obs['schema'],29)
            item=next(s for s in items if s['source_card']=='Magmakin Artillerist' and s['discard_event'] is not None)
            self.assertEqual(item['discard_event'],{'player':'P0','amount':1,'cycling':True})
            other=game.observe('P1');public=other['stack']+other['waiting_triggers']+([other['announcing']] if other['announcing'] else [])
            self.assertEqual(next(s['discard_event'] for s in public if s['id']==item['id']),item['discard_event'])
            saved=game.snapshot();library=obs['players'][0]['library_count'];life=obs['players'][1]['life']
            def finish(handle):
                for _ in range(40):
                    current=handle.observe(handle.observe('P0')['actor'])
                    if not current['stack'] and not current['waiting_triggers'] and current['announcing'] is None:return current
                    index=next((i for i,a in enumerate(current['actions']) if a['kind']=='PutTrigger'),None)
                    if index is None:index=next(i for i,a in enumerate(current['actions']) if a['kind']=='Pass')
                    handle.step(current,index)
                self.fail('event stack did not finish')
            with game.clone() as branch:expected=finish(branch)
            actual=finish(game);self.assertEqual(actual,expected);self.assertEqual(actual['players'][0]['library_count'],library-1)
            self.assertLess(actual['players'][1]['life'],life)
            game.restore(saved);replay=finish(game);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
            game.drop_snapshot(saved)

    def test_cleanup_discard_batch_is_private_owned_and_repeats_cleanup_after_triggers(self):
        deck=['Mountain']*12+['Island']*12+['Hobgoblin, Mantled Marauder']*16+['Divination']*20
        with Game([deck,deck],seed=17,deck_rules='Freeform') as game:
            ready=False;cast_draw=False;draw_done=False;obs=game.observe('P0')
            for _ in range(4000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']}
                own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield']
                ready=ready or (any(o['card']=='Hobgoblin, Mantled Marauder' for o in own) and sum(o['card'] in ('Mountain','Island') for o in own)>=3 and any(o['card']=='Island' for o in own))
                if obs['actor']=='P0' and obs['decision']=='discard' and draw_done:break
                def score(a):
                    k=a['kind'];name=objects.get(a.get('card'),{}).get('card')
                    if k=='Keep':return 100
                    if k=='TapMana':return 80 if obs['decision']=='pay_costs' else -10
                    if k=='PayCosts':return 100
                    if k=='PlayLand':return 90 if obs['actor']=='P0' and not ready else -10
                    if k in ('Cast','AnnounceCast'):
                        if obs['actor']=='P0' and not ready and name=='Hobgoblin, Mantled Marauder':return 80
                        if ready and not cast_draw and obs['actor']=='P0' and obs['players'][0]['hand_count']>=8 and name=='Divination':return 80
                        return -10
                    return {'Pass':20,'PutTrigger':60,'Discard':50,'FinishAttackers':40,'FinishBlockers':40,'Concede':-1000,'Mulligan':-100}.get(k,-10)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));a=obs['actions'][index]
                if a['kind'] in ('Cast','AnnounceCast') and objects[a['card']]['card']=='Divination':cast_draw=True
                obs=game.step(obs,index)
                if cast_draw and not obs['stack'] and obs['announcing'] is None:draw_done=True
            else:self.fail('nine-card cleanup fixture not reached')
            self.assertEqual(obs['players'][0]['hand_count'],9);turn=obs['turn']
            first=next(a['card'] for a in obs['actions'] if a['kind']=='Discard')
            obs=game.step(obs,next(i for i,a in enumerate(obs['actions']) if a['kind']=='Discard' and a['card']==first))
            self.assertEqual(obs['players'][0]['hand_count'],9);self.assertEqual(next(o['zone'] for o in obs['objects'] if o['id']==first),'Hand')
            self.assertFalse(any(a['kind']=='Discard' and a['card']==first for a in obs['actions']))
            self.assertTrue(all(o['zone']!='Hand' or o['owner']!='P0' for o in game.observe('P1')['objects']))
            saved=game.snapshot();index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Discard')
            with game.clone() as branch:expected=branch.step(branch.observe('P0'),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected);self.assertEqual(actual['players'][0]['hand_count'],7)
            self.assertEqual(actual['step'],'Cleanup');self.assertEqual(actual['turn'],turn)
            game.restore(saved);replay=game.step(game.observe('P0'),index);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay);game.drop_snapshot(saved)
            for _ in range(30):
                obs=game.observe(game.observe('P0')['actor'])
                if obs['turn']>turn:break
                index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='PutTrigger'),None)
                if index is None:index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass')
                game.step(obs,index)
            else:self.fail('extra cleanup did not finish')
            self.assertEqual(obs['active'],'P1')
            self.assertTrue(all(o['power']==1 for o in obs['objects'] if o['card']=='Hobgoblin, Mantled Marauder' and o['zone']=='Battlefield'))

    def cycling_index(self,obs):
        hand={o['id'] for o in obs['objects'] if o['zone']=='Hand' and o['owner']==obs['actor']}
        def score(a):
            k=a['kind']
            if k=='TapMana':return 70 if obs['decision']=='pay_costs' else -10
            if k in ('AnnounceActivate','Activate') and a['card'] in hand:return 85
            return {'Keep':100,'PlayLand':90,'PayCosts':100,'ChooseSearch':80,'FinishSearch':0,
                    'Cast':60,'AnnounceCast':59,'PutTrigger':50,'ChooseTarget':50,'Pass':20,
                    'FinishAttackers':40,'FinishBlockers':40,'Mulligan':-100,'Concede':-1000}.get(k,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_cycling_hand_payment_and_draw_replay_through_native_binding(self):
        deck=['Forest']*28+['Migrating Ketradon']*32
        with Game([deck,deck],seed=5,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(1500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='pay_costs' and obs['announcing']['source_card']=='Migrating Ketradon':break
                obs=game.step(obs,self.cycling_index(obs))
            else:self.fail('hand activation payment not reached')
            actor=obs['actor'];other='P1' if actor=='P0' else 'P0';source=obs['announcing']['card'];player=0 if actor=='P0' else 1
            self.assertEqual(obs['schema'],29)
            self.assertEqual(next(o['zone'] for o in obs['objects'] if o['id']==source),'Hand')
            self.assertEqual(game.observe(other)['announcing']['source_card'],'Migrating Ketradon')
            self.assertEqual(game.observe(other)['actions'],[])
            saved=game.snapshot();hand=obs['players'][player]['hand_count'];library=obs['players'][player]['library_count']
            def commit(handle):
                for _ in range(10):
                    current=handle.observe(actor)
                    if current['announcing'] is None:return current
                    handle.step(current,self.cycling_index(current))
                self.fail('cycling payment not committed')
            with game.clone() as branch:expected=commit(branch)
            actual=commit(game);self.assertEqual(actual,expected)
            self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==source),'Graveyard')
            self.assertEqual(actual['players'][player]['hand_count'],hand-1)
            self.assertEqual(actual['players'][player]['library_count'],library)
            self.assertIn('ActivatedAbility',actual['stack'][-1]['kind'])
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
            replay=commit(game);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
            game.drop_snapshot(saved)
            for _ in range(2):
                current=game.observe(game.observe(actor)['actor'])
                game.step(current,next(i for i,a in enumerate(current['actions']) if a['kind']=='Pass'))
            after=game.observe(actor)
            self.assertEqual(after['players'][player]['hand_count'],hand)
            self.assertEqual(after['players'][player]['library_count'],library-1)
            self.assertEqual(after['players'][player]['spells_cast_this_turn'],0)

    def test_typecycling_private_search_reveal_and_native_rng_replay(self):
        for name,filter_,deck in [
            ('Kree Sentinel','BasicLand',['Forest']*28+['Kree Sentinel']*32),
            ('Balamb T-Rexaur',{'Subtype':'Forest'},['Forest']*12+['Plains']*8+['Lush Portico']*8+['Balamb T-Rexaur']*32),
        ]:
            with self.subTest(name=name),Game([deck,deck],seed=9,deck_rules='Freeform') as game:
                obs=game.observe('P0')
                for _ in range(1500):
                    self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['search_choice'] is not None and obs['resolving']['source_card']==name:break
                    obs=game.step(obs,self.cycling_index(obs))
                else:self.fail('typecycling search not reached')
                actor=obs['actor'];other='P1' if actor=='P0' else 'P0';choice=obs['search_choice']
                self.assertEqual(choice['filter'],filter_);self.assertTrue(choice['reveal']);self.assertEqual(choice['destination'],'Hand')
                hidden=game.observe(other);self.assertIsNone(hidden['search_choice']);self.assertEqual(hidden['actions'],[])
                self.assertTrue(all(o['zone']!='Library' for o in hidden['objects']))
                selected=next((o['id'] for o in choice['cards'] if o['card']=='Lush Portico'),None) if isinstance(filter_,dict) else next(a['card'] for a in obs['actions'] if a['kind']=='ChooseSearch')
                self.assertIsNotNone(selected)
                index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseSearch' and a['card']==selected)
                obs=game.step(obs,index);self.assertEqual(obs['search_choice']['selected'],[selected])
                self.assertIsNone(game.observe(other)['search_choice'])
                saved=game.snapshot();index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='FinishSearch')
                with game.clone() as branch:expected=branch.step(branch.observe(actor),index)
                actual=game.step(obs,index);self.assertEqual(actual,expected)
                public=game.observe(other);self.assertTrue(any(o['id']==selected and o['zone']=='Hand' for o in public['objects']))
                game.restore(saved);replay=game.step(game.observe(actor),index)
                self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
                game.drop_snapshot(saved)

    def ward_index(self,obs,decline=False):
        def score(a):
            k=a['kind']
            if k=='TapMana':return 70 if obs['decision'] in ('pay_costs','resolution_mana') else -10
            if k=='DeclineResolutionMana':return 110 if decline else -1
            if k=='ChooseTarget' and isinstance(a['target'],dict) and 'Card' in a['target']:
                target=next(o for o in obs['objects'] if o['id']==a['target']['Card'])
                if target['owner']!=obs['actor'] and target['card'] in ('Toadstool Admirer','Hulking Raptor'):return 80
            return {'PayResolutionMana':100,'Keep':100,'PlayLand':90,'AnnounceCast':79,'Cast':80,'PayCosts':100,
                    'ChooseTarget':60,'PutTrigger':60,'FinishAttackers':40,'FinishBlockers':40,'Pass':20,
                    'Mulligan':-100,'Concede':-1000}.get(k,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_ward_payment_window_is_public_owned_and_replays_native_actions(self):
        green=['Forest']*24+['Toadstool Admirer']*20+['Hulking Raptor']*16
        red=['Mountain']*28+['Shock']*32
        with Game([green,red],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(3000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='resolution_mana' and obs['resolving']['related_stack'] is not None:break
                obs=game.step(obs,self.ward_index(obs))
            else:self.fail('ward resolution payment not reached')
            actor=obs['actor'];other='P1' if actor=='P0' else 'P0';request=obs['resolution_payment']
            self.assertEqual(obs['schema'],29);self.assertEqual(request['payer'],actor);self.assertEqual(request['cost']['generic'],2)
            self.assertEqual(game.observe(other)['resolution_payment'],request);self.assertEqual(game.observe(other)['actions'],[])
            victim=next(s for s in obs['stack'] if s['id']==request['victim']);self.assertEqual(victim['source_card'],'Shock')
            saved=game.snapshot();index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='DeclineResolutionMana')
            with game.clone() as branch:expected=branch.step(branch.observe(actor),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected);self.assertIsNone(actual['resolving'])
            self.assertFalse(any(s['id']==request['victim'] for s in actual['stack']))
            self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==victim['card']),'Graveyard')
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            replay=game.step(game.observe(actor),index);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
            game.drop_snapshot(saved)

    def test_counter_payment_can_activate_mana_and_preserve_spell_through_native_replay(self):
        blue=['Island']*28+["It'll Quench Ya!"]*32
        green=['Forest']*28+['Llanowar Elves']*16+['Grizzly Bears']*16
        with Game([green,blue],seed=11,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(3000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='resolution_mana' and obs['resolving']['source_card']=="It'll Quench Ya!":
                    available=sum(obs['players'][0 if obs['actor']=='P0' else 1]['mana'])+sum(
                        o['owner']==obs['actor'] and o['zone']=='Battlefield' and not o['tapped']
                        and (o['card']=='Forest' or (o['card']=='Llanowar Elves' and not o['summoning_sick'])) for o in obs['objects'])
                    if available>=2:break
                obs=game.step(obs,self.ward_index(obs))
            else:self.fail('counter resolution payment not reached')
            actor=obs['actor'];request=obs['resolution_payment'];saved=game.snapshot()
            def finish(handle):
                for _ in range(16):
                    current=handle.observe(actor)
                    if current['resolution_payment'] is None:return current
                    index=next((i for i,a in enumerate(current['actions']) if a['kind']=='PayResolutionMana'),None)
                    if index is None:index=next((i for i,a in enumerate(current['actions']) if a['kind']=='TapMana'),None)
                    if index is None:index=next((i for i,a in enumerate(current['actions']) if a['kind']=='ActivateMana'),None)
                    if index is None:self.fail('fixture lacks mana to pay')
                    handle.step(current,index)
                self.fail('payment did not finish')
            with game.clone() as branch:expected=finish(branch)
            actual=finish(game);self.assertEqual(actual,expected)
            self.assertTrue(any(s['id']==request['victim'] for s in actual['stack']))
            game.restore(saved);replay=finish(game);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
            game.drop_snapshot(saved)

    def damage_index(self,obs):
        def score(a):
            k=a['kind']
            if k=='TapMana':return 70 if obs['decision']=='pay_costs' else -10
            return {'Keep':100,'PlayLand':90,'AnnounceCast':79,'Cast':80,'PayCosts':100,
                    'Attack':60,'FinishAttackers':40,'FinishBlockers':40,'PutTrigger':60,'Discard':60,
                    'ChooseTarget':60,'Pass':20,'Mulligan':-100,'Concede':-1000}.get(k,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_damage_event_context_and_native_draw_discard_replay(self):
        deck=['Island']*24+['April, Reporter of the Weird']*20+['Fear of Failed Tests']*16
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='effect_discard' and obs['resolving']['source_card']=='April, Reporter of the Weird':break
                obs=game.step(obs,self.damage_index(obs))
            else:self.fail('April damage discard not reached')
            actor=obs['actor'];other='P1' if actor=='P0' else 'P0'
            event=obs['resolving']['damage_event'];self.assertEqual(obs['schema'],29)
            self.assertEqual(event['amount'],2);self.assertTrue(event['combat']);self.assertEqual(event['player'],other)
            self.assertEqual(event['dealer'],obs['resolving']['object'])
            self.assertEqual(game.observe(other)['resolving']['damage_event'],event)
            self.assertTrue(all(o['zone']!='Hand' or o['owner']==other for o in game.observe(other)['objects']))
            self.assertEqual(game.observe(other)['actions'],[])
            saved=game.snapshot();index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Discard')
            with game.clone() as branch:expected=branch.step(branch.observe(actor),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected);self.assertIsNone(actual['resolving'])
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            replay=game.step(game.observe(actor),index)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
            game.drop_snapshot(saved)

    def test_dragon_mage_pending_hand_discard_is_private_and_snapshot_owned(self):
        deck=['Mountain']*28+['Dragon Mage']*32
        with Game([deck,deck],seed=11,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(4000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='effect_discard' and obs['resolving']['source_card']=='Dragon Mage':break
                obs=game.step(obs,self.damage_index(obs))
            else:self.fail('Dragon Mage damage discard not reached')
            actor=obs['actor'];other='P1' if actor=='P0' else 'P0';event=obs['resolving']['damage_event']
            self.assertEqual(event['amount'],5);self.assertTrue(event['combat']);self.assertEqual(obs['effect_discards'],[])
            saved=game.snapshot();index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Discard')
            card=obs['actions'][index]['card']
            with game.clone() as branch:expected=branch.step(branch.observe(actor),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected)
            self.assertIn(card,actual['effect_discards']);self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==card),'Hand')
            self.assertEqual(game.observe(other)['effect_discards'],[])
            self.assertFalse(any(o['id']==card for o in game.observe(other)['objects']))
            self.assertEqual(actual['players'],obs['players'])
            game.restore(saved);replay=game.step(game.observe(actor),index)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
            game.drop_snapshot(saved)

    def mill_index(self, obs):
        def score(a):
            k=a['kind']
            if k=='TapMana':return 15 if obs['actor']==obs['active'] and obs['step'] in ('Main1','Main2') else -20
            if k=='RecoverMilled':return 60 if a['object'] is not None else 0
            return {'Keep':100,'PlayLand':90,'AnnounceCast':79,'Cast':80,'PayCosts':100,
                    'Mulligan':-100,'Concede':-1000}.get(k,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_milled_recovery_public_projection_native_replay_and_continuation(self):
        deck=['Forest']*24+['Llanowar Elves']*12+['Rapid Rescue']*12+['Something Worth Saving']*12
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(1500):
                self.assertIsNone(obs['outcome'])
                obs=game.observe(obs['actor'])
                if obs['decision']=='recover_milled':break
                obs=game.step(obs,self.mill_index(obs))
            else:self.fail('recovery choice not reached')
            actor=obs['actor'];other='P1' if actor=='P0' else 'P0'
            choice=obs['recovery_choice'];self.assertEqual(obs['schema'],29)
            self.assertEqual(choice,game.observe(other)['recovery_choice']);self.assertEqual(game.observe(other)['actions'],[])
            self.assertEqual(choice['filter'],'Permanent');self.assertTrue(choice['candidates'])
            self.assertTrue(all(any(o['reference']==r and o['zone']=='Graveyard' for o in game.observe(other)['objects']) for r in choice['milled']))
            source=obs['resolving']['card'];card=choice['candidates'][0];old_life=obs['players'][0 if actor=='P0' else 1]['life']
            gain=2 if obs['resolving']['source_card']=='Rapid Rescue' else 1
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='RecoverMilled' and a['object']==card)
            saved=game.snapshot()
            with game.clone() as branch:expected=branch.step(branch.observe(actor),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected)
            returned=next(o for o in game.observe(other)['objects'] if o['id']==card['card'])
            self.assertEqual(returned['zone'],'Hand');self.assertGreater(returned['reference']['generation'],card['generation'])
            self.assertIn(returned['reference'],actual['known_hand']);self.assertIsNone(actual['resolving'])
            self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==source),'Graveyard')
            self.assertEqual(actual['players'][0 if actor=='P0' else 1]['life'],old_life+gain)
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            replay=game.step(game.observe(actor),index)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay)
            game.restore(saved);current=game.observe(actor)
            decline=next(i for i,a in enumerate(current['actions']) if a['kind']=='RecoverMilled' and a['object'] is None)
            declined=game.step(current,decline)
            self.assertEqual(next(o['zone'] for o in declined['objects'] if o['id']==card['card']),'Graveyard')
            self.assertEqual(declined['players'][0 if actor=='P0' else 1]['life'],old_life+gain)
            game.drop_snapshot(saved)

    def aura_index(self, obs, linked=False):
        objects={o['id']:o for o in obs['objects']}
        own_creatures=[o for o in obs['objects'] if o['owner']==obs['actor'] and o['zone']=='Battlefield' and o['card']=="Healer's Hawk"]
        own_auras=[o for o in obs['objects'] if o['owner']==obs['actor'] and o['zone']=='Battlefield' and o['card']=='Feather of Flight']
        enemy_auras=[o for o in obs['objects'] if o['owner']!=obs['actor'] and o['zone']=='Battlefield' and o['card']=='Feather of Flight']
        sources={l['source']['card'] for l in obs['exile_links'] if objects[l['exiled']['card']]['card']=='Feather of Flight'}
        def score(a):
            kind=a['kind']
            if kind=='AnnounceCast':
                if obs['stack'] or obs['active']!=obs['actor'] or obs['step'] not in ('Main1','Main2'):return -5
                name=objects[a['card']]['card']
                if name=="Healer's Hawk":return 85
                if name=='Feather of Flight':return 80 if own_creatures and (not linked or not own_auras) else -5
                if name=='Banishing Light':return 90 if enemy_auras else -5
                if name=='Disenchant':return 95 if any(objects[i]['owner']==obs['actor'] for i in sources) else -5
            if kind=='ChooseTarget':
                target=objects[a['target']['Card']]
                source=obs['announcing']['source_card']
                if source=='Banishing Light':return 100 if target in enemy_auras else 0
                if source=='Disenchant':return 100 if target['id'] in sources and target['owner']==obs['actor'] else 0
                return 80 if target in own_creatures else 0
            return {'Keep':100,'PlayLand':90,'PayCosts':100,'TapMana':70 if obs['decision']=='pay_costs' else -5,
                    'PutTrigger':70,'FinishAttackers':60,'FinishBlockers':60,'Pass':20,'Discard':50,
                    'Mulligan':-100,'Concede':-1000}.get(kind,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_aura_cast_target_payment_and_entry_trigger_clone_restore(self):
        deck=['Plains']*24+["Healer's Hawk"]*16+['Feather of Flight']*20
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2500):
                if obs['outcome'] is not None:self.fail('game ended before Aura cast')
                obs=game.observe(obs['actor'])
                if obs['decision']=='targets' and obs['announcing']['source_card']=='Feather of Flight':break
                obs=game.step(obs,self.aura_index(obs))
            else:self.fail('no Aura cast target reached')
            actor=obs['actor'];aura=obs['announcing']['card'];index=self.aura_index(obs);target=obs['actions'][index]['target']['Card'];saved=game.snapshot()
            def finish(handle):
                handle.step(handle.observe(actor),index)
                for _ in range(16):
                    current=handle.observe(actor)
                    if any(o['id']==aura and o['zone']=='Battlefield' for o in current['objects']):return current
                    current=handle.observe(current['actor'])
                    i=next((i for i,a in enumerate(current['actions']) if a['kind']=='PayCosts'),None)
                    if i is None:i=next((i for i,a in enumerate(current['actions']) if a['kind']=='TapMana'),None)
                    if i is None:i=next(i for i,a in enumerate(current['actions']) if a['kind']=='Pass')
                    handle.step(current,i)
                self.fail('Aura cast did not complete')
            with game.clone() as branch:expected=finish(branch)
            actual=finish(game);self.assertEqual(actual,expected)
            aura_view=next(o for o in actual['objects'] if o['id']==aura);creature=next(o for o in actual['objects'] if o['id']==target)
            self.assertEqual(aura_view['attached_to'],creature['reference']);self.assertEqual(creature['power'],2)
            self.assertTrue(any(t['card']==aura and 'TriggeredAbility' in t['kind'] for t in actual['stack']))
            game.restore(saved);replay=finish(game)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_linked_aura_return_choice_suspends_removal_and_replays_through_library(self):
        deck=['Plains']*24+["Healer's Hawk"]*8+['Feather of Flight']*8+['Banishing Light']*12+['Disenchant']*8
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(6000):
                if obs['outcome'] is not None:self.fail('game ended before Aura return')
                obs=game.observe(obs['actor'])
                if obs['decision']=='aura_entry':break
                obs=game.step(obs,self.aura_index(obs,linked=True))
            else:self.fail('no linked Aura return reached')
            actor=obs['actor'];choice=obs['aura_entry'];aura=choice['aura']['card']
            self.assertEqual(obs['schema'],29);self.assertEqual(obs['resolving']['source_card'],'Disenchant')
            self.assertEqual(next(o['zone'] for o in obs['objects'] if o['id']==aura),'Exile')
            target=next(o['reference'] for o in obs['objects'] if o['zone']=='Battlefield' and o['owner']==actor and o['card']=="Healer's Hawk")
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseAuraAttachment' and a['object']==target)
            other='P1' if actor=='P0' else 'P0';self.assertEqual(game.observe(other)['aura_entry'],choice)
            self.assertEqual(game.observe(other)['actions'],[]);saved=game.snapshot()
            with game.clone() as branch:expected=branch.step(branch.observe(actor),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected)
            aura_view=next(o for o in actual['objects'] if o['id']==aura)
            self.assertEqual(aura_view['zone'],'Battlefield');self.assertEqual(aura_view['attached_to'],target);self.assertIsNone(actual['resolving'])
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            replay=game.step(game.observe(actor),index)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def equipment_index(self, obs):
        own={o['id']:o for o in obs['objects'] if o['owner']==obs['actor']}
        creature=any(o['zone']=='Battlefield' and o['card'] in ("Healer's Hawk",'GoblinPiker','Goblin Piker') for o in own.values())
        def score(a):
            if a['kind']=='AnnounceCast' and own[a['card']]['card'] in ('Short Bow','Coral Sword') and not creature:
                return -5
            return {'Keep':100,'PlayLand':90,'AnnounceCast':80,'PayCosts':100,
                    'TapMana':70 if obs['decision']=='pay_costs' else (40 if obs['active']==obs['actor'] and obs['step'] in ('Main1','Main2') else -5),
                    'ChooseTarget':70,'PutTrigger':70,'FinishAttackers':60,'FinishBlockers':60,
                    'Pass':20,'Mulligan':-100,'Concede':-1000}.get(a['kind'],0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_paid_equipment_attachment_is_public_and_native_clone_restore_owned(self):
        deck=['Plains']*24+["Healer's Hawk"]*12+['Short Bow']*24
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(3000):
                if obs['outcome'] is not None: self.fail('game ended before equip')
                obs=game.observe(obs['actor'])
                index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='Activate'),None)
                if index is not None: break
                obs=game.step(obs,self.equipment_index(obs))
            else: self.fail('no paid equip action reached')
            action=obs['actions'][index];equipment=action['card'];target=action['target']['Card'];actor=obs['actor']
            saved=game.snapshot()
            def activate_and_resolve(handle):
                current=handle.observe(actor);handle.step(current,index)
                self.assertIsNone(next(o['attached_to'] for o in handle.observe(actor)['objects'] if o['id']==equipment))
                return self.resolve_vital_top(handle,actor)
            with game.clone() as branch: expected=activate_and_resolve(branch)
            actual=activate_and_resolve(game);self.assertEqual(actual,expected)
            link=next(o['attached_to'] for o in actual['objects'] if o['id']==equipment)
            self.assertEqual(link['card'],target)
            before=next(o for o in obs['objects'] if o['id']==target)
            after=next(o for o in actual['objects'] if o['id']==target)
            self.assertEqual((after['power'],after['toughness']),(before['power']+1,before['toughness']+1))
            self.assertEqual(next(o['attached_to'] for o in game.observe('P1' if actor=='P0' else 'P0')['objects'] if o['id']==equipment),link)
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            replay=activate_and_resolve(game)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_flash_equipment_entry_target_and_temporary_keyword_replay(self):
        deck=['Mountain']*24+['Coral Sword']*24+['Goblin Piker']*12
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(3000):
                if obs['outcome'] is not None:self.fail('game ended before entry attachment')
                obs=game.observe(obs['actor'])
                if obs['decision']=='targets' and obs['announcing']['source_card']=='Coral Sword' and 'TriggeredAbility' in obs['announcing']['kind']:break
                obs=game.step(obs,self.equipment_index(obs))
            else:self.fail('no entry attachment reached')
            actor=obs['actor'];equipment=obs['announcing']['card']
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseTarget');target=obs['actions'][index]['target']['Card']
            saved=game.snapshot()
            def target_and_resolve(handle):
                handle.step(handle.observe(actor),index)
                return self.resolve_vital_top(handle,actor)
            with game.clone() as branch:expected=target_and_resolve(branch)
            actual=target_and_resolve(game);self.assertEqual(actual,expected)
            self.assertEqual(next(o['attached_to']['card'] for o in actual['objects'] if o['id']==equipment),target)
            creature=next(o for o in actual['objects'] if o['id']==target)
            self.assertNotEqual(creature['keywords'] & 512,0)
            game.restore(saved);replay=target_and_resolve(game)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def vital_index(self, obs):
        own={o['id']:o for o in obs['objects']}
        def score(a):
            kind=a['kind']
            if kind=='AnnounceCast':
                return 85 if own[a['card']]['card'] in ("Ajani's Pridemate",'Agent Venom') else 80
            return {'Keep':100,'PlayLand':90,'PayCosts':100,
                    'TapMana':70 if obs['decision']=='pay_costs' else -5,
                    'Activate':60,'AnnounceActivate':60,'PutTrigger':60,'Sacrifice':60,
                    'FinishAttackers':40,'FinishBlockers':40,'Pass':20,
                    'Mulligan':-100,'Concede':-1000}.get(kind,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def resolve_vital_top(self, game, viewer):
        for _ in range(2):
            obs=game.observe(viewer)
            obs=game.observe(obs['actor'])
            game.step(obs,next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass'))
        return game.observe(viewer)

    def test_life_gain_trigger_and_public_history_clone_restore(self):
        deck=['Plains']*24+["Ajani's Pridemate"]*12+['Cat Collector']*12+['Brightblade Stoat']*12
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(3000):
                if obs['outcome'] is not None: self.fail('game ended before gain trigger')
                obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=="Ajani's Pridemate" and 'TriggeredAbility' in obs['stack'][-1]['kind']: break
                obs=game.step(obs,self.vital_index(obs))
            else: self.fail('no life-gain trigger reached')
            source=obs['stack'][-1];viewer=source['controller'];p=0 if viewer=='P0' else 1
            self.assertEqual(obs['schema'],29)
            self.assertGreater(obs['players'][p]['life_gain_events_this_turn'],0)
            other='P1' if viewer=='P0' else 'P0'
            self.assertEqual(game.observe(other)['players'][p]['life_gain_events_this_turn'],obs['players'][p]['life_gain_events_this_turn'])
            card=source['card'];before=next(o['counters']['plus_one'] for o in obs['objects'] if o['id']==card)
            saved=game.snapshot()
            with game.clone() as branch: expected=self.resolve_vital_top(branch,viewer)
            actual=self.resolve_vital_top(game,viewer)
            self.assertEqual(actual,expected)
            self.assertEqual(next(o['counters']['plus_one'] for o in actual['objects'] if o['id']==card),before+1)
            game.restore(saved)
            replay=self.resolve_vital_top(game,viewer)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_death_trigger_native_continuation_clones_and_restores(self):
        deck=['Swamp']*24+['Agent Venom']*12+['Voracious Vermin']*12+['Abyssal Gorestalker']*12
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(5000):
                if obs['outcome'] is not None: self.fail('game ended before death trigger')
                obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Agent Venom' and 'TriggeredAbility' in obs['stack'][-1]['kind']: break
                obs=game.step(obs,self.vital_index(obs))
            else: self.fail('no death trigger reached')
            source=obs['stack'][-1];viewer=source['controller'];p=0 if viewer=='P0' else 1
            self.assertTrue(any(o['zone']=='Graveyard' for o in obs['objects']))
            saved=game.snapshot()
            with game.clone() as branch: expected=self.resolve_vital_top(branch,viewer)
            actual=self.resolve_vital_top(game,viewer)
            self.assertEqual(actual,expected)
            self.assertEqual(actual['players'][p]['library_count'],obs['players'][p]['library_count']-1)
            self.assertEqual(actual['players'][p]['life'],obs['players'][p]['life']-1)
            game.restore(saved)
            replay=self.resolve_vital_top(game,viewer)
            self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_private_discard_payment_paid_or_declined_clones_and_restores(self):
        deck=["Mountain"]*30+["Discerning Peddler"]*30
        with Game([deck,deck],seed=7,deck_rules="Freeform") as game:
            obs=game.observe("P0")
            for _ in range(1000):
                obs=game.observe(obs["actor"])
                if obs["decision"]=="effect_choice": break
                index=max(range(len(obs["actions"])),key=lambda i:
                    {"Keep":100,"PlayLand":90,"AnnounceCast":80,"PayCosts":100,
                     "TapMana":70 if obs["decision"]=="pay_costs" else -5,"Pass":20,
                     "FinishAttackers":40,"FinishBlockers":40,"Mulligan":-100,"Concede":-1000}
                    .get(obs["actions"][i]["kind"],0))
                obs=game.step(obs,index)
            else: self.fail("no discard cost reached")
            actor=obs["actor"]; player=0 if actor=="P0" else 1; other="P1" if actor=="P0" else "P0"
            self.assertEqual(obs["effect_choice"]["kind"],"Discard")
            self.assertTrue(obs["effect_choice"]["optional"])
            self.assertIsNone(game.observe(other)["effect_choice"])
            self.assertEqual(game.observe(other)["actions"],[])
            index=next(i for i,a in enumerate(obs["actions"]) if a["kind"]=="Discard")
            card=obs["actions"][index]["card"];saved=game.snapshot()
            with game.clone() as branch: expected=branch.step(branch.observe(actor),index)
            actual=game.step(obs,index)
            self.assertEqual(actual,expected)
            self.assertEqual(actual["players"][player]["library_count"],obs["players"][player]["library_count"]-1)
            self.assertTrue(any(o["id"]==card and o["zone"]=="Graveyard" for o in game.observe(other)["objects"]))
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,"StaleRevision"): game.step(obs,index)
            replay=game.step(game.observe(actor),index)
            self.assertGreater(replay.pop("revision"),actual.pop("revision"));self.assertEqual(replay,actual)
            game.restore(saved);current=game.observe(actor)
            decline=next(i for i,a in enumerate(current["actions"]) if a["kind"]=="DeclineEffectPayment")
            declined=game.step(current,decline)
            self.assertEqual(declined["players"][player]["library_count"],obs["players"][player]["library_count"])
            self.assertTrue(any(o["id"]==card and o["zone"]=="Hand" for o in declined["objects"]))
            game.drop_snapshot(saved)

    def test_public_apnap_sacrifices_stay_on_battlefield_until_simultaneous_commit(self):
        deck=["Swamp"]*30+["Agents of HYDRA"]*20+["Abyssal Gorestalker"]*10
        with Game([deck,deck],seed=7,deck_rules="Freeform") as game:
            obs=game.observe("P0")
            for _ in range(5000):
                if obs["outcome"] is not None: self.fail("game ended before simultaneous sacrifice")
                obs=game.observe(obs["actor"])
                choice=obs["effect_choice"]
                if choice and not choice["optional"] and min(choice["required"])>=2: break
                objects={o["id"]:o for o in obs["objects"]}
                def score(a):
                    if a["kind"]=="AnnounceCast": return 85 if objects[a["card"]]["card"]=="Abyssal Gorestalker" else 80
                    return {"Keep":100,"PlayLand":90,"PayCosts":100,
                            "TapMana":70 if obs["decision"]=="pay_costs" else -5,
                            "Sacrifice":60,"PutTrigger":60,"Discard":30,"Pass":20,
                            "FinishAttackers":40,"FinishBlockers":40,"Mulligan":-100,"Concede":-1000}.get(a["kind"],0)
                obs=game.step(obs,max(range(len(obs["actions"])),key=lambda i:score(obs["actions"][i])))
            else: self.fail("no simultaneous sacrifice reached")
            active=obs["active"];player=0 if active=="P0" else 1;other="P1" if active=="P0" else "P0"
            self.assertEqual(obs["actor"],active)
            selected=[]
            for _ in range(2):
                index=next(i for i,a in enumerate(obs["actions"]) if a["kind"]=="Sacrifice")
                selected.append(obs["actions"][index]["object"])
                obs=game.step(obs,index)
                self.assertEqual(game.observe(other)["effect_choice"]["selected"][player],selected)
                self.assertTrue(all(any(o["id"]==ref["card"] and o["zone"]=="Battlefield" for o in obs["objects"]) for ref in selected))
            self.assertEqual(obs["actor"],other)
            saved=game.snapshot()
            def finish(handle):
                for _ in range(3):
                    current=handle.observe(other)
                    if current["effect_choice"] is None: return current
                    current=handle.observe(current["actor"])
                    index=next(i for i,a in enumerate(current["actions"]) if a["kind"]=="Sacrifice")
                    handle.step(current,index)
                self.fail("sacrifice did not commit")
            with game.clone() as branch: expected=finish(branch)
            actual=finish(game);self.assertEqual(actual,expected)
            self.assertTrue(all(any(o["id"]==ref["card"] and o["zone"]=="Graveyard" for o in actual["objects"]) for ref in selected))
            game.restore(saved);replay=finish(game)
            self.assertGreater(replay.pop("revision"),actual.pop("revision"));self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_survival_step_trigger_history_and_source_tapped_clone_restore(self):
        deck=["Forest"]*30+["Cautious Survivor"]*30
        with Game([deck,deck],seed=7,deck_rules="Freeform") as game:
            obs=game.observe("P0")
            for _ in range(1600):
                if obs["outcome"] is not None: self.fail("game ended before survival trigger")
                obs=game.observe(obs["actor"])
                actor=obs["actor"]
                if obs["step"]=="Main2" and obs["decision"]=="priority" and any(x["source_card"]=="Cautious Survivor" for x in obs["stack"]):
                    break
                index=max(range(len(obs["actions"])),key=lambda i:
                    {"Keep":100,"PlayLand":90,"AnnounceCast":80,"PayCosts":100,
                     "TapMana":70 if obs["decision"]=="pay_costs" else -5,
                     "PutTrigger":70,"Attack":60,"Block":-5,"Pass":20,
                     "Discard":30,"Mulligan":-100,"Concede":-1000}.get(obs["actions"][i]["kind"],0))
                obs=game.step(obs,index)
            else: self.fail("no survival trigger reached")
            player=0 if actor=="P0" else 1
            self.assertTrue(obs["players"][player]["attacked_this_turn"])
            self.assertTrue(obs["stack"][-1]["source_tapped"])
            other="P1" if actor=="P0" else "P0"
            self.assertTrue(game.observe(other)["players"][player]["attacked_this_turn"])
            saved=game.snapshot()
            def resolve(handle):
                for _ in range(2):
                    current=handle.observe(actor)
                    current=handle.observe(current["actor"])
                    index=next(i for i,a in enumerate(current["actions"]) if a["kind"]=="Pass")
                    handle.step(current,index)
                return handle.observe(actor)
            with game.clone() as branch: expected=resolve(branch)
            actual=resolve(game)
            self.assertEqual(actual,expected)
            self.assertEqual(actual["players"][player]["life"],obs["players"][player]["life"]+2)
            game.restore(saved)
            replay=resolve(game)
            self.assertGreater(replay.pop("revision"),actual.pop("revision"))
            self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_completed_cast_counts_and_triggers_survive_native_clone_restore(self):
        deck = ["Mountain"] * 24 + ["Firebrand Archer"] * 24 + ["Boltwave"] * 12
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1200):
                obs = game.observe(obs["actor"])
                if obs["outcome"] is not None:
                    self.fail("game ended before a cast-trigger check")
                actor = obs["actor"]
                player = 0 if actor == "P0" else 1
                own = {o["id"]: o for o in obs["objects"] if o["owner"] == actor}
                def score(a):
                    kind = a["kind"]
                    if kind == "AnnounceCast":
                        return 85 if own[a["card"]]["card"] == "Firebrand Archer" else 80
                    return {"Keep":100, "PlayLand":90, "PayCosts":100,
                            "TapMana":70 if obs["decision"] == "pay_costs" else -5,
                            "PutTrigger":60, "FinishAttackers":60, "FinishBlockers":60,
                            "Pass":50, "Mulligan":-100, "Concede":-1000}.get(kind, 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                action = obs["actions"][index]
                if (action["kind"] == "PayCosts" and obs["announcing"]["source_card"] == "Boltwave"
                        and any(o["card"] == "Firebrand Archer" and o["zone"] == "Battlefield" for o in own.values())):
                    before = obs["players"][player]["spells_cast_this_turn"]
                    saved = game.snapshot()
                    with game.clone() as branch:
                        expected = branch.step(branch.observe(actor), index)
                    actual = game.step(obs, index)
                    self.assertEqual(actual, expected)
                    self.assertEqual(actual["players"][player]["spells_cast_this_turn"], before + 1)
                    other = "P1" if actor == "P0" else "P0"
                    self.assertEqual(game.observe(other)["players"][player]["spells_cast_this_turn"], before + 1)
                    self.assertEqual(actual["stack"][0]["source_card"], "Boltwave")
                    self.assertTrue(any(x["source_card"] == "Firebrand Archer" for x in actual["stack"] + actual["waiting_triggers"]))
                    game.restore(saved)
                    replay = game.step(game.observe(actor), index)
                    self.assertGreater(replay.pop("revision"), actual.pop("revision"))
                    self.assertEqual(replay, actual)
                    game.drop_snapshot(saved)
                    break
                obs = game.step(obs, index)
            else:
                self.fail("no cast trigger exercised")

    def test_legend_choices_public_masks_and_native_clone_restore(self):
        deck = ["Swamp"] * 30 + ["Yargle, Glutton of Urborg"] * 30
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(3000):
                obs = game.observe(obs["actor"])
                if obs["decision"] == "legend_rule":
                    break
                # No attacks: keep both legends alive long enough to exercise the rule.
                index = max(range(len(obs["actions"])), key=lambda i:
                    {"Keep": 100, "PlayLand": 90, "AnnounceCast": 80, "Cast": 80, "PayCosts": 100,
                     "TapMana": 70 if obs["decision"] == "pay_costs" else -5,
                     "FinishAttackers": 60, "FinishBlockers": 60,
                     "Pass": 50, "Attack": -50, "Mulligan": -100, "Concede": -1000}
                    .get(obs["actions"][i]["kind"], 0))
                obs = game.step(obs, index)
            else:
                self.fail("no legend decision reached")
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            self.assertEqual(obs["legend_choices"], game.observe(other)["legend_choices"])
            self.assertEqual({a["kind"] for a in obs["actions"]}, {"KeepLegend", "Concede"})
            self.assertEqual(game.observe(other)["actions"], [])
            index = next(i for i,a in enumerate(obs["actions"]) if a["kind"] == "KeepLegend")
            kept = obs["actions"][index]["object"]["card"]
            saved = game.snapshot()
            with game.clone() as branch:
                expected = branch.step(branch.observe(actor), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            self.assertEqual(next(o for o in actual["objects"] if o["id"] == kept)["zone"], "Battlefield")
            self.assertEqual(actual["legend_choices"], [])
            game.restore(saved)
            replay = game.step(game.observe(actor), index)
            self.assertGreater(replay.pop("revision"), actual.pop("revision"))
            self.assertEqual(replay, actual)
            game.drop_snapshot(saved)

    def test_private_views_clone_restore_and_stale_indices(self):
        with self.game() as game:
            observation = game.observe("P0")
            coverage = json.loads((Path(__file__).resolve().parents[1] / "data/standard/coverage.json").read_text())
            self.assertEqual(observation["catalogue"], coverage["catalogue_version"])
            self.assertEqual(observation["schema"], 29)
            self.assertEqual(len(observation["objects"]), 7)
            self.assertTrue(all(o["owner"] == "P0" for o in observation["objects"]))
            self.assertEqual(game.observe("P1")["actions"], [])
            saved = game.snapshot()
            with game.clone() as branch:
                keep = next(i for i, a in enumerate(observation["actions"]) if a["kind"] == "Keep")
                branch.step(observation, keep)
                self.assertEqual(game.observe("P0"), observation)
                self.assertEqual(branch.observe("P1")["actor"], "P1")
            game.restore(saved)
            restored = game.observe("P0")
            self.assertGreater(restored.pop("revision"), observation["revision"])
            before = dict(observation)
            before.pop("revision")
            self.assertEqual(restored, before)
            with self.assertRaisesRegex(EngineError, "StaleRevision"):
                game.step(observation, 1)
            game.drop_snapshot(saved)
            with self.assertRaisesRegex(EngineError, "unknown snapshot"):
                game.restore(saved)

    def test_distinct_target_groups_power_damage_and_partial_restore(self):
        deck = ["Forest"] * 24 + ["Grizzly Bears"] * 18 + ["Quarrel"] * 18
        with Game([deck, deck], seed=17, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1800):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                if obs["targeting_group"] == 0 and obs["announcing"]["source_card"] == "Quarrel":
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 15 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": 79,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, index)
            else:
                self.fail("no distinct target declaration reached")
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            selected = []
            for group in (0, 1):
                self.assertEqual(obs["targeting_group"], group)
                hidden = game.observe(other)
                self.assertIsNone(hidden["targeting_group"])
                self.assertEqual(hidden["announcing"]["targets"], [])
                index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "ChooseTarget")
                chosen = obs["actions"][index]["target"]["Card"]
                obj = next(o for o in obs["objects"] if o["id"] == chosen)
                self.assertEqual(obj["owner"], actor if group == 0 else other)
                selected.append(chosen)
                saved = game.snapshot()
                with game.clone() as branch:
                    expected = branch.step(branch.observe(actor), index)
                actual = game.step(obs, index)
                self.assertEqual(actual, expected)
                game.restore(saved)
                replay = game.step(game.observe(actor), index)
                comparison = dict(replay)
                comparison.pop("revision")
                actual.pop("revision")
                self.assertEqual(actual, comparison)
                game.drop_snapshot(saved)
                obs = replay
            for _ in range(20):
                kinds = [a["kind"] for a in obs["actions"]]
                if "PayCosts" in kinds:
                    obs = game.step(obs, kinds.index("PayCosts"))
                    break
                obs = game.step(obs, kinds.index("TapMana"))
            else:
                self.fail("no completed group payment")
            item = next(i for i in obs["stack"] if i["source_card"] == "Quarrel")
            self.assertEqual([b["group"] for b in item["targets"]], [0, 1])
            for _ in range(2):
                obs = game.observe(obs["actor"])
                obs = game.step(obs, next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "Pass"))
            view = game.observe(actor)
            source = next(o for o in view["objects"] if o["id"] == selected[0])
            victim = next(o for o in view["objects"] if o["id"] == selected[1])
            self.assertEqual(source["zone"], "Battlefield")
            self.assertEqual(source["damage"], 0)
            self.assertEqual(victim["zone"], "Graveyard")

    def test_modal_cast_mode_targets_payment_and_resolution_replay(self):
        fixtures = [(["Mountain"] * 24 + ["Abrade"] * 18 + ["Hill Giant"] * 18, "Abrade", 0),
                    (["Island"] * 36 + ["Spellgyre"] * 24, "Spellgyre", 1)]
        for deck, name, mode in fixtures:
            with Game([deck, deck], seed=17, deck_rules="Freeform") as game:
                obs = game.observe("P0")
                for _ in range(1800):
                    self.assertIsNone(obs["outcome"])
                    if obs["viewer"] != obs["actor"]:
                        obs = game.observe(obs["actor"])
                    if obs["decision"] == "spell_mode":
                        break
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana":
                            return 15 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                        return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": 79,
                                "Mulligan": -100, "Concede": -1000}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                    obs = game.step(obs, index)
                else:
                    self.fail("no mode selection reached: " + name)
                actor = obs["actor"]
                other = "P1" if actor == "P0" else "P0"
                self.assertEqual(obs["announcing"]["source_card"], name)
                self.assertIsNone(obs["announcing"]["mode"])
                self.assertTrue(obs["mode_choices"])
                self.assertFalse(game.observe(other)["mode_choices"])
                self.assertFalse(game.observe(other)["actions"])
                saw_target = False
                saw_payment = False
                for _ in range(30):
                    kinds = [a["kind"] for a in obs["actions"]]
                    if "ChooseMode" in kinds:
                        index = next(i for i, a in enumerate(obs["actions"])
                                     if a["kind"] == "ChooseMode" and a["mode"] == mode)
                    elif "ChooseTarget" in kinds:
                        index = kinds.index("ChooseTarget")
                        saw_target = True
                    elif "PayCosts" in kinds:
                        index = kinds.index("PayCosts")
                        saw_payment = True
                    elif "TapMana" in kinds:
                        index = kinds.index("TapMana")
                    else:
                        break
                    saved = game.snapshot()
                    with game.clone() as branch:
                        expected = branch.step(branch.observe(actor), index)
                    actual = game.step(obs, index)
                    self.assertEqual(actual, expected)
                    game.restore(saved)
                    replay = game.step(game.observe(actor), index)
                    comparison = dict(replay)
                    comparison.pop("revision")
                    actual.pop("revision")
                    self.assertEqual(actual, comparison)
                    game.drop_snapshot(saved)
                    obs = replay
                self.assertTrue(saw_payment)
                self.assertEqual(saw_target, name == "Abrade")
                item = next(item for item in obs["stack"] if item["source_card"] == name)
                self.assertEqual(item["mode"], mode)
                for _ in range(2):
                    obs = game.observe(obs["actor"])
                    index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "Pass")
                    obs = game.step(obs, index)
                if name == "Spellgyre":
                    obs = game.observe(actor)
                    self.assertIsNotNone(obs["library_choice"])
                    self.assertEqual(obs["resolving"]["mode"], 1)
                    self.assertIsNone(game.observe(other)["library_choice"])

    def test_search_private_selection_public_destination_and_clone_restore(self):
        for source in ("Evolving Wilds", "World Map"):
            deck = ["Forest"] * 36 + [source] * 24
            with Game([deck, deck], seed=17, deck_rules="Freeform") as game:
                obs = game.observe("P0")
                for _ in range(1200):
                    self.assertIsNone(obs["outcome"])
                    if obs["viewer"] != obs["actor"]:
                        obs = game.observe(obs["actor"])
                    if obs["search_choice"] is not None:
                        break
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana":
                            return 15 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                        return {"Keep": 100, "PlayLand": 90, "Cast": 80, "Activate": 70,
                                "AnnounceCast": -20, "AnnounceActivate": -20,
                                "Mulligan": -100, "Concede": -1000}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                    obs = game.step(obs, index)
                else:
                    self.fail("no search reached: " + source)
                actor = obs["actor"]
                other = "P1" if actor == "P0" else "P0"
                choice = obs["search_choice"]
                self.assertEqual(choice["filter"], "BasicLand")
                self.assertTrue(choice["cards"])
                self.assertTrue(all(o["zone"] == "Library" for o in choice["cards"]))
                hidden = game.observe(other)
                self.assertIsNone(hidden["search_choice"])
                self.assertFalse(hidden["actions"])
                index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "ChooseSearch")
                selected = obs["actions"][index]["card"]
                obs = game.step(obs, index)
                self.assertEqual(obs["search_choice"]["selected"], [selected])
                self.assertIsNone(game.observe(other)["search_choice"])
                self.assertFalse(any(o["id"] == selected for o in game.observe(other)["objects"]))
                saved = game.snapshot()
                index = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "FinishSearch")
                with game.clone() as branch:
                    expected = branch.step(branch.observe(actor), index)
                actual = game.step(obs, index)
                self.assertEqual(actual, expected)
                obj = next(o for o in game.observe(other)["objects"] if o["id"] == selected)
                if source == "World Map":
                    self.assertEqual(obj["zone"], "Hand")
                    self.assertIn(obj["reference"], actual["known_hand"])
                else:
                    self.assertEqual(obj["zone"], "Battlefield")
                    self.assertTrue(obj["tapped"])
                game.restore(saved)
                replay = game.step(game.observe(actor), index)
                replay.pop("revision")
                actual.pop("revision")
                self.assertEqual(actual, replay)
                game.drop_snapshot(saved)

    def test_nonbasic_land_entry_private_surveil_and_mana_replay(self):
        deck = ["Commercial District"] * 60
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            seen_entry = False
            for _ in range(100):
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                if obs["library_choice"] is not None:
                    break
                index = max(range(len(obs["actions"])), key=lambda i:
                    {"Keep": 100, "PlayLand": 90, "Mulligan": -100, "Concede": -1000}
                    .get(obs["actions"][i]["kind"], 0))
                action = obs["actions"][index]
                if action["kind"] == "PlayLand":
                    actor = obs["actor"]
                    obs = game.step(obs, index)
                    land = next(o for o in obs["objects"] if o["id"] == action["card"])
                    self.assertTrue(land["tapped"])
                    self.assertFalse(any(a.get("card") == land["id"] and a["kind"] == "ActivateMana"
                                         for a in obs["actions"]))
                    seen_entry = True
                else:
                    obs = game.step(obs, index)
            else:
                self.fail("no land surveil choice reached")
            self.assertTrue(seen_entry)
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            self.assertTrue(obs["library_choice"]["surveil"])
            self.assertIsNone(game.observe(other)["library_choice"])
            saved = game.snapshot()
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "ArrangeLibrary" and a["destination"] == "Graveyard")
            with game.clone() as branch:
                expected = branch.step(branch.observe(actor), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            game.restore(saved)
            replay = game.step(game.observe(actor), index)
            replay.pop("revision")
            actual.pop("revision")
            self.assertEqual(actual, replay)
            game.drop_snapshot(saved)
            obs = game.observe(actor)
            # Reach the next untap step without tapping away the available source.
            for _ in range(200):
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                actions = [(i, a) for i, a in enumerate(obs["actions"])
                           if a["kind"] == "ActivateMana"]
                if actions:
                    break
                index = next(i for i, a in enumerate(obs["actions"])
                             if a["kind"] in ("Pass", "FinishAttackers", "FinishBlockers", "Keep",
                                              "Discard", "Bottom", "ArrangeLibrary"))
                obs = game.step(obs, index)
            else:
                self.fail("land never untapped")
            self.assertEqual({tuple(a["production"]) for _, a in actions},
                             {(0, 0, 0, 1, 0, 0), (0, 0, 0, 0, 1, 0)})
            saved = game.snapshot()
            index = actions[0][0]
            with game.clone() as branch:
                expected = branch.step(branch.observe(obs["actor"]), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            self.assertEqual(sum(actual["players"][0 if obs["actor"] == "P0" else 1]["mana"]), 1)
            game.restore(saved)
            replay = game.step(game.observe(obs["actor"]), index)
            replay.pop("revision")
            actual.pop("revision")
            self.assertEqual(actual, replay)
            game.drop_snapshot(saved)

    def test_scry_private_library_cards_and_partial_choice_restore(self):
        deck = ["Plains"] * 36 + ["Wakandan Drone Flock"] * 24
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(600):
                self.assertIsNone(obs["outcome"], "game ended before the scry fixture")
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                if obs["library_choice"] is not None:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, index)
            else:
                self.fail("no scry choice reached")
            choice = obs["library_choice"]
            self.assertFalse(choice["surveil"])
            self.assertEqual(len(choice["remaining"]), 2)
            self.assertTrue(all(o["zone"] == "Library" for o in choice["remaining"]))
            actor = obs["actor"]
            other = "P1" if actor == "P0" else "P0"
            hidden = game.observe(other)
            self.assertIsNone(hidden["library_choice"])
            self.assertEqual(hidden["actions"], [])
            visible_ids = {o["id"] for o in hidden["objects"]}
            self.assertTrue(all(o["id"] not in visible_ids for o in choice["remaining"]))
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "ArrangeLibrary" and a["destination"] == "Bottom")
            obs = game.step(obs, index)
            self.assertEqual(len(obs["library_choice"]["bottom"]), 1)
            selected = obs["library_choice"]["bottom"][0]
            self.assertEqual(selected, next(o for o in choice["remaining"] if o["id"] == selected["id"]))
            self.assertEqual(selected["zone"], "Library")
            self.assertIsNone(game.observe(other)["library_choice"])
            saved = game.snapshot()
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "ArrangeLibrary" and a["destination"] == "Top")
            with game.clone() as branch:
                expected = branch.step(branch.observe(actor), index)
            actual = game.step(obs, index)
            self.assertEqual(actual, expected)
            game.restore(saved)
            replay = game.step(game.observe(actor), index)
            self.assertGreater(replay.pop("revision"), actual.pop("revision"))
            self.assertEqual(actual, replay)
            game.drop_snapshot(saved)

    def test_derived_anthem_stats_clone_and_restore(self):
        deck = ["Plains"] * 20 + ["Forest"] * 20 + ["Anthem of Champions"] * 10 + ["Grizzly Bears"] * 10
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1200):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {o["id"]: o for o in obs["objects"]}
                creatures = {o["id"]: (o["power"], o["toughness"]) for o in objects.values()
                             if o["zone"] == "Battlefield" and o["owner"] == obs["actor"] and o["card"] == "Grizzly Bears"}
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Cast"
                              and objects[a["card"]]["card"] == "Anthem of Champions"), None)
                if index is not None and creatures:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("anthem/creature setup not reached")
            actor = obs["actor"]
            def cast_and_resolve(engine, observation):
                engine.step(observation, index)
                for _ in range(2):
                    current = engine.observe(engine.observe(actor)["actor"])
                    passed = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass")
                    engine.step(current, passed)
                return engine.observe(actor)
            saved = game.snapshot()
            with game.clone() as branch:
                expected = cast_and_resolve(branch, branch.observe(actor))
            after = cast_and_resolve(game, obs)
            self.assertEqual(after, expected)
            by_id = {o["id"]: o for o in after["objects"]}
            for card, (power, toughness) in creatures.items():
                self.assertEqual((by_id[card]["power"], by_id[card]["toughness"]), (power + 1, toughness + 1))
            game.restore(saved)
            again = cast_and_resolve(game, game.observe(actor))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_effective_keyword_gains_replay_through_native_library(self):
        deck = ["Forest"] * 36 + ["Grizzly Bears"] * 12 + ["Overprotect"] * 12
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1000):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {o["id"]: o for o in obs["objects"]}
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Cast"
                              and objects[a["card"]]["card"] == "Overprotect"), None)
                if index is not None:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("keyword fixture not reached")
            actor = obs["actor"]
            target = obs["actions"][index]["target"]["Card"]
            before = objects[target]
            def resolve(engine, observation):
                engine.step(observation, index)
                for _ in range(2):
                    current = engine.observe(engine.observe(actor)["actor"])
                    passed = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass")
                    engine.step(current, passed)
                return engine.observe(actor)
            saved = game.snapshot()
            with game.clone() as branch:
                expected = resolve(branch, branch.observe(actor))
            after = resolve(game, obs)
            self.assertEqual(after, expected)
            protected = next(o for o in after["objects"] if o["id"] == target)
            self.assertEqual(protected["keywords"] & (256 + 4096 + 8192), 256 + 4096 + 8192)
            self.assertEqual((protected["power"], protected["toughness"]), (before["power"] + 3, before["toughness"] + 3))
            game.restore(saved)
            again = resolve(game, game.observe(actor))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_menace_partial_blocks_require_completion_and_restore(self):
        decks = [["Swamp"] * 36 + ["Ravine Raider"] * 24, ["Forest"] * 36 + ["Grizzly Bears"] * 24]
        with Game(decks, seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1500):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Block" and obs["actor"] == "P1"), None)
                if index is not None:
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20, "Attack": 60,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("menace block fixture not reached")
            attacker = obs["actions"][index]["attacker"]
            obs = game.step(obs, index)
            self.assertFalse(any(a["kind"] == "FinishBlockers" for a in obs["actions"]))
            self.assertEqual(game.observe("P0")["blocks"], [])
            saved = game.snapshot()
            second = next(i for i, a in enumerate(obs["actions"]) if a["kind"] == "Block" and a["attacker"] == attacker)
            def complete(engine, observation):
                current = engine.step(observation, second)
                finish = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "FinishBlockers")
                return engine.step(current, finish)
            with game.clone() as branch:
                expected = complete(branch, branch.observe("P1"))
            after = complete(game, obs)
            self.assertEqual(after, expected)
            self.assertEqual(len(after["blocks"]), 2)
            game.restore(saved)
            again = complete(game, game.observe("P1"))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_public_linked_exile_returns_and_replays_through_library(self):
        decks = [["Plains"] * 32 + ["Banishing Light"] * 12 + ["Disenchant"] * 16,
                 ["Forest"] * 36 + ["Grizzly Bears"] * 24]
        with Game(decks, seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(1600):
                self.assertIsNone(obs["outcome"])
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {o["id"]: o for o in obs["objects"]}
                source = obs["exile_links"][0]["source"]["card"] if obs["exile_links"] else None
                index = next((i for i, a in enumerate(obs["actions"]) if a["kind"] == "Cast"
                              and objects[a["card"]]["card"] == "Disenchant" and a["target"] == {"Card": source}), None)
                if index is not None and obs["actor"] == "P0":
                    break
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80, "AnnounceCast": -20, "ChooseTarget": 70,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                selected = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                obs = game.step(obs, selected)
            else:
                self.fail("linked exile fixture not reached")
            link = next(link for link in obs["exile_links"] if link["source"]["card"] == source)
            target = link["exiled"]["card"]
            self.assertEqual(link["return_zone"], "Battlefield")
            self.assertEqual(objects[target]["zone"], "Exile")
            self.assertEqual(game.observe("P1")["exile_links"], obs["exile_links"])
            saved = game.snapshot()
            def resolve(engine, observation):
                engine.step(observation, index)
                for _ in range(2):
                    current = engine.observe(engine.observe("P0")["actor"])
                    passed = next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass")
                    engine.step(current, passed)
                return engine.observe("P0")
            with game.clone() as branch:
                expected = resolve(branch, branch.observe("P0"))
            after = resolve(game, obs)
            self.assertEqual(after, expected)
            returned = next(o for o in after["objects"] if o["id"] == target)
            self.assertEqual((returned["zone"], returned["owner"]), ("Battlefield", "P1"))
            self.assertNotEqual(returned["reference"], link["exiled"])
            self.assertFalse(any(other["source"]["card"] == source for other in after["exile_links"]))
            game.restore(saved)
            again = resolve(game, game.observe("P0"))
            self.assertGreater(again.pop("revision"), after.pop("revision"))
            self.assertEqual(again, after)
            game.drop_snapshot(saved)

    def test_input_and_lifecycle(self):
        with self.assertRaises(EngineError):
            Game([["BlackLotus"] * 60] * 2)
        with self.assertRaises(EngineError):
            Game([["LightningBolt"] * 60] * 2)
        game = self.game()
        with self.assertRaises(EngineError):
            game.request("unknown")
        game.close()
        game.close()
        with self.assertRaisesRegex(EngineError, "closed"):
            game.observe("P0")

    def test_standard_admission_distinguishes_legality_and_rule_coverage(self):
        deck = ["Plains"] * 56 + ["Savannah Lions"] * 4
        with Game([deck, deck], deck_rules="Standard") as game:
            self.assertEqual(game.observe("P0")["players"][0]["library_count"], 53)
        with self.assertRaisesRegex(EngineError, "not Standard legal"):
            Game([["Mountain"] * 56 + ["Lightning Bolt"] * 4] * 2, deck_rules="Standard")
        with self.assertRaisesRegex(EngineError, "rules not implemented"):
            Game([["Hare Apparent"] * 60] * 2, deck_rules="Standard")

    def test_decoded_nul_in_invalid_card_is_an_error_without_aborting_host(self):
        # A regression in this boundary can abort the interpreter, so exercise it
        # in a child and assert normal exit rather than crashing the whole suite.
        result = subprocess.run([sys.executable, "-c", """
from rustmage import EngineError, Game
try:
    Game([["bad\\x00"] * 60] * 2)
except EngineError as error:
    assert "unknown card" in str(error)
else:
    raise AssertionError("invalid card accepted")
"""], env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parent)}, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_token_metadata_and_pending_resolution_replay_through_shared_library(self):
        deck = ["Mountain"] * 10 + ["Dragon Fodder"] * 2
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            obs = game.observe("P0")
            for _ in range(200):
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80,
                            "Mulligan": -100, "Concede": -1000}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                kind = obs["actions"][index]["kind"]
                obs = game.step(obs, index)
                if kind == "Cast":
                    break
                self.assertIsNone(obs["outcome"])
            else:
                self.fail("token spell was never cast")
            saved = game.snapshot()
            def resolve(handle):
                for _ in range(2):
                    current = handle.observe("P0")
                    current = handle.observe(current["actor"])
                    index = next(i for i, action in enumerate(current["actions"]) if action["kind"] == "Pass")
                    handle.step(current, index)
                result = handle.observe("P0")
                result.pop("revision")
                return result
            with game.clone() as branch:
                expected = resolve(branch)
            actual = resolve(game)
            self.assertEqual(actual, expected)
            tokens = [o for o in actual["objects"] if o["token"] is not None]
            self.assertEqual(len(tokens), 2)
            self.assertTrue(all(o["token"] == {"name": "Goblin Token", "subtypes": "Goblin", "colors": 8}
                                for o in tokens))
            self.assertTrue(all(o["summoning_sick"] for o in tokens))
            self.assertTrue(all(o["card"].startswith("@token/") for o in tokens))
            game.restore(saved)
            self.assertEqual(resolve(game), expected)
            with self.assertRaisesRegex(EngineError, "tokens cannot"):
                Game([[tokens[0]["card"]] * 60] * 2, deck_rules="Freeform")

    def test_activated_self_sacrifice_has_independent_stack_identity_and_replay(self):
        deck = ["Mountain"] * 10 + ["Fanatical Firebrand"] * 2
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            for _ in range(100):
                obs = game.observe("P0")
                obs = game.observe(obs["actor"])
                def score(action):
                    if action["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] == "Main1" else -5
                    return {"Keep": 100, "PlayLand": 90, "Cast": 80,
                            "Concede": -1000, "Mulligan": -100}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i]))
                kind = obs["actions"][index]["kind"]
                game.step(obs, index)
                if kind == "Cast":
                    break
            else:
                self.fail("Firebrand never cast")
            def pass_twice():
                for _ in range(2):
                    current = game.observe("P0")
                    current = game.observe(current["actor"])
                    game.step(current, next(i for i, a in enumerate(current["actions"]) if a["kind"] == "Pass"))
            pass_twice()
            obs = game.observe("P0")
            obs = game.observe(obs["actor"])
            opponent = "P1" if obs["actor"] == "P0" else "P0"
            index = next(i for i, a in enumerate(obs["actions"])
                         if a["kind"] == "Activate" and a["target"] == {"Player": opponent})
            action = obs["actions"][index]
            self.assertEqual(action["counter_payment"], {"plus_one": 0, "minus_one": 0, "stun": 0})
            source = next(o for o in obs["objects"] if o["id"] == action["card"])
            saved = game.snapshot()
            game.step(obs, index)
            pending = game.observe("P0")
            self.assertEqual(pending["stack"][-1]["kind"], {"ActivatedAbility": {"ability": 0}})
            self.assertEqual(pending["stack"][-1]["object"], source["reference"])
            self.assertEqual(next(o for o in pending["objects"] if o["id"] == action["card"])["zone"], "Graveyard")
            pass_twice()
            expected = game.observe("P0")
            expected.pop("revision")
            self.assertEqual(expected["players"][0 if opponent == "P0" else 1]["life"], 19)
            game.restore(saved)
            current = game.observe(obs["actor"])
            game.step(current, current["actions"].index(action))
            pass_twice()
            actual = game.observe("P0")
            actual.pop("revision")
            self.assertEqual(actual, expected)

    def test_treasure_mana_is_immediate_and_six_slot_pool_replays(self):
        deck = ["Mountain"] * 16 + ["Plundering Pirate"] * 4
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            for _ in range(1000):
                obs = game.observe("P0")
                obs = game.observe(obs["actor"])
                choices = [(i,a) for i,a in enumerate(obs["actions"])
                           if a["kind"] == "ActivateMana" and a["production"] == [0,0,0,0,1,0]]
                if choices:
                    index, action = choices[0]
                    break
                def score(a):
                    if a["kind"] == "TapMana":
                        return 20 if obs["active"] == obs["actor"] and obs["step"] == "Main1" else -5
                    return {"Keep":100,"PlayLand":90,"Cast":80,"Concede":-1000,"Mulligan":-100}.get(a["kind"],0)
                game.step(obs, max(range(len(obs["actions"])), key=lambda i: score(obs["actions"][i])))
            else:
                self.fail("Treasure mana ability never available")
            actor = obs["actor"]
            self.assertTrue(all(len(p["mana"]) == 6 for p in obs["players"]))
            self.assertEqual(next(o for o in obs["objects"] if o["id"] == action["card"])["token"]["name"], "Treasure Token")
            before_mana = obs["players"][0 if actor == "P0" else 1]["mana"][4]
            saved = game.snapshot()
            game.step(obs, index)
            expected = game.observe(actor)
            expected.pop("revision")
            self.assertEqual(expected["stack"], obs["stack"])
            self.assertEqual(expected["actor"], actor)
            self.assertEqual(expected["players"][0 if actor == "P0" else 1]["mana"][4], before_mana + 1)
            self.assertFalse(any(o["id"] == action["card"] for o in expected["objects"]))
            game.restore(saved)
            current = game.observe(actor)
            game.step(current, current["actions"].index(action))
            actual = game.observe(actor)
            actual.pop("revision")
            self.assertEqual(actual, expected)

    def test_unfunded_cast_targets_then_funds_in_owned_payment_window(self):
        deck = ["Mountain"] * 10 + ["Lightning Bolt"] * 2
        with Game([deck, deck], seed=7, deck_rules="Freeform") as game:
            for _ in range(200):
                obs = game.observe("P0")
                obs = game.observe(obs["actor"])
                index = next((i for i,a in enumerate(obs["actions"]) if a["kind"] == "AnnounceCast"), None)
                if index is not None:
                    game.step(obs,index)
                    break
                def score(a):
                    return {"Keep":100,"PlayLand":90,"Concede":-1000,"Mulligan":-100,"TapMana":-5}.get(a["kind"],0)
                game.step(obs,max(range(len(obs["actions"])),key=lambda i: score(obs["actions"][i])))
            else:
                self.fail("unfunded announcement never offered")
            actor = obs["actor"]
            opponent = "P1" if actor == "P0" else "P0"
            current=game.observe(actor)
            index=next(i for i,a in enumerate(current["actions"]) if a["kind"] == "ChooseTarget" and a["target"] == {"Player":opponent})
            game.step(current,index)
            current=game.observe(actor)
            self.assertEqual(current["decision"],"pay_costs")
            self.assertEqual(current["payment_request"]["required"],[0,0,0,1,0,0])
            self.assertIsNone(game.observe(opponent)["payment_request"])
            self.assertEqual(game.observe(opponent)["actions"],[])
            saved=game.snapshot()
            def finish(handle):
                current=handle.observe(actor)
                index=next(i for i,a in enumerate(current["actions"]) if a["kind"] == "TapMana")
                handle.step(current,index)
                current=handle.observe(actor)
                index=next(i for i,a in enumerate(current["actions"]) if a["kind"] == "PayCosts")
                handle.step(current,index)
                self.assertEqual(len(handle.observe(actor)["stack"]),1)
                for _ in range(2):
                    current=handle.observe("P0")
                    current=handle.observe(current["actor"])
                    handle.step(current,next(i for i,a in enumerate(current["actions"]) if a["kind"] == "Pass"))
                result=handle.observe(actor)
                result.pop("revision")
                return result
            with game.clone() as branch:
                expected=finish(branch)
            self.assertEqual(finish(game),expected)
            self.assertEqual(expected["players"][0 if opponent=="P0" else 1]["life"],17)
            game.restore(saved)
            self.assertEqual(finish(game),expected)

    def test_parallel_independent_games(self):
        def run(seed):
            with self.game() as game:
                for _ in range(100):
                    # Observation calls are read-only; independent handles have no global RNG.
                    game.observe("P0")
                return json.dumps(game.observe("P0"), sort_keys=True)
        with concurrent.futures.ThreadPoolExecutor(4) as pool:
            outputs = list(pool.map(run, range(8)))
        self.assertEqual(len(set(outputs)), 1)

    def test_complete_selfplay_through_shared_library(self):
        # The complete control loop stays in this process. Actor changes require
        # a fresh observation for the new player, never a union of both hands.
        with self.game() as game:
            observation = game.observe("P0")
            cast_count = 0
            for _ in range(5000):
                if observation["outcome"] is not None:
                    break
                actor = observation["actor"]
                if observation["viewer"] != actor:
                    observation = game.observe(actor)
                opponent = "P1" if actor == "P0" else "P0"

                def score(action):
                    kind = action["kind"]
                    if kind == "Cast":
                        return 80 if action["target"] == {"Player": opponent} else -10
                    return {"Keep": 100, "PlayLand": 90, "TapMana": 20,
                            "Discard": 40, "Concede": -1000, "Mulligan": -100}.get(kind, 0)

                index = max(range(len(observation["actions"])), key=lambda i: score(observation["actions"][i]))
                cast_count += observation["actions"][index]["kind"] == "Cast"
                observation = game.step(observation, index)
            else:
                self.fail("self-play did not end naturally")
            self.assertGreaterEqual(cast_count, 7)
            self.assertIn("Winner", observation["outcome"])
            self.assertEqual(game.observe("P0")["outcome"], game.observe("P1")["outcome"])

    def test_standard_keyword_combat_through_shared_library(self):
        white = ["Plains"] * 28
        for name in ["Savannah Lions", "Healer's Hawk", "Bishop's Soldier", "Brightblade Stoat",
                     "Leonin Skyhunter", "Serra Angel", "Shrike Force", "Iron Giant"]:
            white.extend([name] * 4)
        green = ["Forest"] * 24
        for name in ["Ankle Biter", "Dragon Sniper", "Magnigoth Sentry", "Colossadactyl",
                     "Tajuru Pathwarden", "Galewind Moose", "Gigantosaurus", "Bear Cub", "Jibbirik Omnivore"]:
            green.extend([name] * 4)
        seen_steps = set()
        with Game([white, green], seed=2, deck_rules="Standard") as game:
            obs = game.observe("P0")
            for _ in range(10000):
                if obs["outcome"] is not None:
                    break
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                seen_steps.add(obs["step"])
                def score(action):
                    if action["kind"] == "AssignDamage":
                        self.assertIn("target", action)
                        if obs["damage_assigned"]:
                            self.assertIn("source", obs["damage_assigned"][0])
                        return 50 + action["amount"]
                    if action["kind"] == "TapMana":
                        return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                    return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                            "Cast": 80, "Attack": 60, "Block": 60,
                            "Discard": 50}.get(action["kind"], 0)
                index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                obs = game.step(obs, index)
            else:
                self.fail("Standard keyword game did not end")
            self.assertIn("FirstStrikeDamage", seen_steps)
            self.assertIn("CombatDamage", seen_steps)
            self.assertIn("Winner", obs["outcome"])

    def test_standard_removal_and_object_identity_through_shared_library(self):
        removal = {"Unsummon", "Fell", "Murder", "Bilbo's Deadly Slice"}
        blue_black = ["Island"] * 12 + ["Swamp"] * 12
        for name in sorted(removal) + ["Vampire Nighthawk", "Hullcarver", "Monoist Sentry",
                                        "Squirrelanoids", "Highborn Vampire"]:
            blue_black.extend([name] * 4)
        green = ["Forest"] * 24
        for name in ["Ankle Biter", "Dragon Sniper", "Magnigoth Sentry", "Colossadactyl",
                     "Tajuru Pathwarden", "Galewind Moose", "Gigantosaurus", "Bear Cub", "Jibbirik Omnivore"]:
            green.extend([name] * 4)
        replayed = False
        with Game([blue_black, green], seed=9, deck_rules="Standard") as game:
            obs = game.observe("P0")
            for _ in range(10000):
                if obs["outcome"] is not None:
                    break
                if obs["viewer"] != obs["actor"]:
                    obs = game.observe(obs["actor"])
                objects = {obj["id"]: obj for obj in obs["objects"]}
                for obj in objects.values():
                    self.assertEqual(obj["reference"]["card"], obj["id"])
                for spell in obs["stack"]:
                    self.assertEqual(spell["object"], objects[spell["card"]]["reference"])
                def score(action):
                    kind = action["kind"]
                    if kind == "Cast" and action["target"] is not None:
                        target = objects[action["target"]["Card"]]
                        return 85 if target["owner"] != obs["actor"] else -10
                    if kind in ("AnnounceCast", "AnnounceActivate", "AnnounceMana"):
                        return -30  # This fixture checks the original atomic Cast replay.
                    if kind == "TapMana":
                        return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                    if kind == "AssignDamage":
                        return 50 + action["amount"]
                    return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                            "Cast": 80, "Attack": 60, "Block": 60, "Discard": 50}.get(kind, 0)
                index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                action = obs["actions"][index]
                check_replay = (not replayed and action["kind"] == "Cast"
                                and objects[action["card"]]["card"] in removal)
                if check_replay:
                    saved = game.snapshot()
                    after = game.step(obs, index)
                    game.restore(saved)
                    restored = game.observe(obs["viewer"])
                    again = game.step(restored, index)
                    self.assertGreater(again.pop("revision"), after.pop("revision"))
                    self.assertEqual(again, after)
                    game.drop_snapshot(saved)
                    replayed = True
                    obs = game.observe(restored["viewer"])
                else:
                    obs = game.step(obs, index)
            else:
                self.fail("Standard removal game did not end")
            self.assertTrue(replayed)
            self.assertIn("Winner", obs["outcome"])

    def test_standard_trigger_programs_and_pending_choice_replay(self):
        white_blue = ["Plains"] * 12 + ["Island"] * 12
        for name in ["Helpful Hunter", "Inspiring Overseer", "Bellowing Crier", "Buzz Bots",
                     "Summit Sentinel", "Icewind Elemental", "Lifecreed Duo", "Rook Turret", "Cloudblazer"]:
            white_blue.extend([name] * 4)
        black_green = ["Swamp"] * 12 + ["Forest"] * 12
        for name in ["Bogwater Lumaret", "Burglar Rat", "Hecteyes", "Vampire Spawn", "Glidedive Duo",
                     "Virulent Emissary", "Eumidian Terrabotanist", "Pelakka Wurm", "Primordial Pachyderm"]:
            black_green.extend([name] * 4)
        replayed = set()
        seen_trigger = False
        for seed in range(5):
            with Game([white_blue, black_green], seed=seed, deck_rules="Standard") as game:
                obs = game.observe("P0")
                for _ in range(10000):
                    if obs["outcome"] is not None:
                        break
                    if obs["viewer"] != obs["actor"]:
                        obs = game.observe(obs["actor"])
                    if any(item["kind"] != "Spell" for item in obs["stack"]):
                        seen_trigger = True
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana":
                            return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                        if kind == "AssignDamage": return 50 + action["amount"]
                        if kind == "ChooseOptional": return 50 if action["accept"] else 0
                        return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                                "Cast": 80, "Attack": 60, "Block": 60, "Discard": 50,
                                "PutTrigger": 50}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                    decision = obs["decision"]
                    if decision in {"order_triggers", "optional_effect", "effect_discard"} and decision not in replayed:
                        saved = game.snapshot()
                        with game.clone() as branch:
                            expected = branch.step(obs, index)
                        first = game.step(obs, index)
                        self.assertEqual(first, expected)
                        game.restore(saved)
                        restored = game.observe(obs["viewer"])
                        again = game.step(restored, index)
                        self.assertGreater(again.pop("revision"), expected.pop("revision"))
                        self.assertEqual(again, expected)
                        game.drop_snapshot(saved)
                        replayed.add(decision)
                        obs = game.observe(restored["viewer"])
                    else:
                        obs = game.step(obs, index)
                else:
                    self.fail("Standard trigger game did not finish")
                self.assertIn("Winner", obs["outcome"])
            if len(replayed) == 3:
                break
        self.assertTrue(seen_trigger)
        self.assertEqual(replayed, {"order_triggers", "optional_effect", "effect_discard"})

    def test_standard_target_announcements_and_pending_target_replay(self):
        white_black = ["Plains"] * 12 + ["Swamp"] * 12
        for name in ["Fancy Footwork", "Thistledown Players", "Helpful Hunter", "Inspiring Overseer",
                     "Emerge from the Cocoon", "Skeleton Archer", "Vault Plunderer", "Zombify", "Burglar Rat"]:
            white_black.extend([name] * 4)
        blue_green = ["Island"] * 12 + ["Forest"] * 12
        for name in ["Reclamation Sage", "Iron Giant", "Buzz Bots", "Rook Turret", "Quick Study", "Refute",
                     "Aegis Turtle", "Tajuru Pathwarden", "Magnigoth Sentry"]:
            blue_green.extend([name] * 4)
        replayed = set()
        for seed in range(5):
            with Game([white_black, blue_green], seed=seed, deck_rules="Standard") as game:
                obs = game.observe("P0")
                for _ in range(10000):
                    if obs["outcome"] is not None: break
                    if obs["viewer"] != obs["actor"]: obs = game.observe(obs["actor"])
                    objects = {obj["id"]: obj for obj in obs["objects"]}
                    def score(action):
                        kind = action["kind"]
                        if kind == "TapMana": return 10 if obs["active"] == obs["actor"] and obs["step"] in ("Main1", "Main2") else -20
                        if kind == "AssignDamage": return 50 + action["amount"]
                        if kind == "ChooseOptional": return 50 if action["accept"] else 0
                        if kind == "ChooseTarget":
                            source = obs["announcing"]["source_card"]
                            beneficial = source in {"Fancy Footwork", "Thistledown Players", "Vault Plunderer"}
                            target = action["target"]
                            own = (target.get("Player") == obs["actor"] if "Player" in target
                                   else objects[target["Card"]]["owner"] == obs["actor"])
                            return 70 if own == beneficial else -10
                        if kind == "Cast" and action["target"] is not None:
                            target = action["target"]
                            if "Spell" in target: return -10
                            return 80
                        return {"Concede": -1000, "Mulligan": -100, "Keep": 100, "PlayLand": 90,
                                "Cast": 80, "BeginCast": 80, "Attack": 60, "Block": 60, "Discard": 50,
                                "PutTrigger": 50}.get(kind, 0)
                    index = max(range(len(obs["actions"])), key=lambda i: (score(obs["actions"][i]), i))
                    item = obs["announcing"]
                    replay_kind = None if item is None else ("spell" if item["kind"] == "Spell" else "trigger")
                    if obs["decision"] == "targets" and replay_kind not in replayed:
                        saved = game.snapshot()
                        with game.clone() as branch: expected = branch.step(obs, index)
                        self.assertEqual(game.step(obs, index), expected)
                        game.restore(saved)
                        restored = game.observe(obs["viewer"])
                        again = game.step(restored, index)
                        self.assertGreater(again.pop("revision"), expected.pop("revision"))
                        self.assertEqual(again, expected)
                        game.drop_snapshot(saved)
                        replayed.add(replay_kind)
                        obs = game.observe(restored["viewer"])
                    else: obs = game.step(obs, index)
                else: self.fail("Standard targeting game did not end")
                self.assertIn("Winner", obs["outcome"])
            if len(replayed) == 2: break
        self.assertEqual(replayed, {"spell", "trigger"})


if __name__ == "__main__":
    unittest.main()
