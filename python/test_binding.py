import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from rustmage import CapacityError, EngineError, Game


class BindingTests(unittest.TestCase):
    def game(self):
        return Game([["Mountain"] * 24 + ["LightningBolt"] * 36] * 2, seed=17, deck_rules="Freeform")

    def test_conditional_entry_counters_are_immediate_and_replay_in_process(self):
        cases=[('Goblin Boarders',False),('Goblin Boarders',True),('Effortless Master',False),('Effortless Master',True),('Luxknight Breacher',True)]
        for wanted,positive in cases:
            lands=(['Island']*18+['Mountain']*18 if wanted=='Effortless Master' else [('Plains' if wanted=='Luxknight Breacher' else 'Mountain')]*36)
            deck=lands+['Three Tree Mascot']*12+[wanted]*12
            with self.subTest(card=wanted,positive=positive),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                def choose(obs):
                    objects={o['id']:o for o in obs['objects']}
                    own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield']
                    ready=(obs['players'][0]['attacked_this_turn'] if wanted=='Goblin Boarders' else obs['players'][0]['spells_cast_this_turn']>=1 if wanted=='Effortless Master' else bool(own and any(o['card']=='Three Tree Mascot' for o in own)))
                    def score(a):
                        if a['kind'] in ('Cast','AnnounceCast'):
                            if wanted=='Goblin Boarders' and positive and obs['step']=='Main1' and any(o['card']=='Three Tree Mascot' and not o['summoning_sick'] and not o['tapped'] for o in own):return -100
                            name=objects[a['card']]['card']
                            if name==wanted:return (180 if a['kind']=='AnnounceCast' else 170) if ready==positive else -100
                            if name=='Three Tree Mascot':return 150 if positive else -100
                            return -100
                        if a['kind']=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
                        if a['kind']=='Attack':return 80
                        return self.professor_score(obs,a)
                    return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['stack'] and obs['stack'][-1]['source_card']==wanted:break
                    game.step(obs,choose(obs))
                else:self.fail('conditional entry cast not reached')
                own=[o for o in obs['objects'] if o['owner']=='P0' and o['zone']=='Battlefield']
                n=(int(obs['players'][0]['attacked_this_turn']) if wanted=='Goblin Boarders' else 2*int(obs['players'][0]['spells_cast_this_turn']>=2) if wanted=='Effortless Master' else len([o for o in own if o['card']=='Three Tree Mascot']))
                self.assertEqual(n>0,positive)
                card=obs['stack'][-1]['card'];saved=game.snapshot()
                with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
                after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected)
                obj=next(o for o in after['objects'] if o['id']==card)
                self.assertEqual(obj['zone'],'Battlefield');self.assertEqual(obj['counters']['plus_one'],n)
                base={'Goblin Boarders':(3,2),'Effortless Master':(4,3),'Luxknight Breacher':(2,2)}[wanted]
                self.assertEqual((obj['power'],obj['toughness']),(base[0]+n,base[1]+n))
                game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def history_index(self,obs,which):
        objects={o['id']:o for o in obs['objects']};actor=obs['actor'];own=[o for o in objects.values() if o['owner']==actor and o['zone']=='Battlefield']
        ready_creatures=sum(o['card']=='Ordinary Bear' and not o['summoning_sick'] for o in own)
        wanted={'rowdy':'Rowdy Research','frenzy':'Witchstalker Frenzy','chart_raid':'Chart a Course','chart_no_raid':'Chart a Course'}[which]
        raid=obs['players'][0]['attacked_this_turn'];ready=(raid if which=='chart_raid' else not raid if which=='chart_no_raid' else obs['creatures_attacked_this_turn']>=3)
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==wanted:return (180 if k=='AnnounceCast' else 170) if ready else -100
                if name=='Ordinary Bear':return 150 if sum(o['card']=='Ordinary Bear' for o in own)<(1 if which.startswith('chart') else 3) else -100
                return -100
            if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
            if k=='ChooseTarget' and 'Card' in a['target']:
                return 160 if objects[a['target']['Card']]['owner']!=actor else 80
            if k=='Attack':return 80 if ready_creatures>=(1 if which.startswith('chart') else 3) else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_attack_history_discounts_project_public_counts_and_replay_payments(self):
        for which in ['rowdy','frenzy']:
            wanted='Rowdy Research' if which=='rowdy' else 'Witchstalker Frenzy'
            deck=[('Island' if which=='rowdy' else 'Mountain')]*18+['Forest']*18+['Ordinary Bear']*12+[wanted]*12
            with self.subTest(which=which),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='pay_costs' and obs['announcing']['source_card']==wanted:break
                    game.step(obs,self.history_index(obs,which))
                else:self.fail('history-discounted payment not reached')
                n=obs['creatures_attacked_this_turn'];self.assertGreaterEqual(n,3);self.assertEqual(obs['schema'],42)
                self.assertEqual(game.observe('P1')['creatures_attacked_this_turn'],n)
                generic=max(0,(6 if which=='rowdy' else 3)-n);self.assertEqual(obs['payment_request']['generic'],generic);saved=game.snapshot()
                def finish(handle):
                    paid=None
                    for _ in range(100):
                        current=handle.observe('P0');current=handle.observe(current['actor'])
                        if current['stack'] and current['stack'][-1]['source_card']==wanted:
                            self.assertEqual(sum(paid),generic+1)
                            old=current['players'][0]['library_count'];target=current['stack'][-1]['targets'][0]['target']['Card'] if which=='frenzy' else None
                            after=self.resolve_vital_top(handle,'P0')
                            if which=='rowdy':self.assertEqual(after['players'][0]['library_count'],old-3)
                            else:self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==target),'Graveyard')
                            self.assertEqual(after['creatures_attacked_this_turn'],n);return after
                        action=current['actions'][self.history_index(current,which)]
                        if action['kind']=='PayCosts':paid=action['payment']
                        handle.act(current,action)
                    self.fail('history-discounted cast stalled')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved);again=finish(game)
                after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_chart_a_course_private_discard_is_conditional_and_draw_is_not_repeated(self):
        for raid in [False,True]:
            which='chart_raid' if raid else 'chart_no_raid'
            deck=['Island']*18+['Forest']*18+['Ordinary Bear']*12+['Chart a Course']*12
            with self.subTest(raid=raid),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['stack'] and obs['stack'][-1]['source_card']=='Chart a Course':break
                    game.step(obs,self.history_index(obs,which))
                else:self.fail('Chart a Course cast not reached')
                self.assertEqual(obs['players'][0]['attacked_this_turn'],raid);old=obs['players'][0]['library_count']
                obs=self.resolve_vital_top(game,'P0');self.assertEqual(obs['players'][0]['library_count'],old-2)
                if raid:self.assertNotEqual(obs['decision'],'effect_discard');continue
                self.assertEqual(obs['decision'],'effect_discard');self.assertEqual(game.observe('P1')['effect_discards'],[]);self.assertEqual(game.observe('P1')['actions'],[])
                saved=game.snapshot();action=next(a for a in obs['actions'] if a['kind']=='Discard')
                with game.clone() as fork:expected=fork.act(fork.observe('P0'),action)
                after=game.act(obs,action);self.assertEqual(after,expected);game.restore(saved);again=game.act(game.observe('P0'),action)
                after['revision']=again['revision'];self.assertEqual(again,after);self.assertEqual(again['players'][0]['library_count'],old-2);game.drop_snapshot(saved)

    def derived_index(self,obs,which):
        objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield']
        present=any(o['card']=='The Scarlet Witch' for o in own)
        names={o['card'] for o in own if o['card'] in ('Forest','Island','Plains','Mountain')}
        wanted='Arcane Epiphany' if which=='power' else 'Fungal Colossus'
        def score(a):
            if a['kind']=='PlayLand' and objects[a['card']]['card'] not in names:return 190
            if a['kind'] in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name=='The Scarlet Witch':return 150 if not present else -100
                if name==wanted:return (180 if a['kind']=='AnnounceCast' else 170) if (present if which=='power' else len(names)>=2) else -100
                return -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_derived_power_and_distinct_name_payments_replay_complete_bodies(self):
        for which in ['power','names']:
            wanted='Arcane Epiphany' if which=='power' else 'Fungal Colossus'
            deck=(['Mountain']*12+['Island']*12+['The Scarlet Witch']*12+[wanted]*24 if which=='power'
                  else ['Forest']*12+['Island']*12+['Plains']*12+[wanted]*24)
            with self.subTest(which=which),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(6000):
                    obs=game.observe('P0');obs=game.observe(obs['actor']);self.assertIsNone(obs['outcome'])
                    if obs['decision']=='pay_costs' and obs['announcing']['source_card']==wanted:break
                    game.step(obs,self.derived_index(obs,which))
                else:self.fail('derived payment not reached')
                own=[o for o in obs['objects'] if o['owner']=='P0' and o['zone']=='Battlefield']
                if which=='power':expected_generic=max(0,3-next(o['power'] for o in own if o['card']=='The Scarlet Witch'))
                else:expected_generic=6-len({o['card'] for o in own if o['card'] in ('Forest','Island','Plains')})
                self.assertEqual(obs['payment_request']['generic'],expected_generic);saved=game.snapshot()
                def finish(handle):
                    paid=None
                    for _ in range(100):
                        current=handle.observe('P0');current=handle.observe(current['actor'])
                        if current['stack'] and current['stack'][-1]['source_card']==wanted:
                            self.assertEqual(sum(paid),expected_generic+(2 if which=='power' else 1))
                            card=current['stack'][-1]['card'];old=current['players'][0]['library_count'];after=self.resolve_vital_top(handle,'P0')
                            if which=='power':self.assertEqual(after['players'][0]['library_count'],old-3)
                            else:
                                obj=next(o for o in after['objects'] if o['id']==card);self.assertEqual(obj['zone'],'Battlefield');self.assertEqual((obj['power'],obj['toughness']),(5,5))
                            return after
                        action=current['actions'][self.derived_index(current,which)]
                        if action['kind']=='PayCosts':paid=action['payment']
                        handle.act(current,action)
                    self.fail('derived cast stalled')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved);again=finish(game)
                after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def affinity_index(self,obs,wanted):
        objects={o['id']:o for o in obs['objects']}
        artifacts=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield' and o['card'] in ('Three Tree Mascot','Memory Guardian','Valkyrie Aerial Unit')]
        def score(a):
            if a['kind'] in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name=='Three Tree Mascot':return 160
                if name==wanted:return (180 if a['kind']=='AnnounceCast' else 170) if artifacts else -100
                return -100
            if a['kind']=='ArrangeLibrary':return 180 if a['destination']=='Graveyard' else 170
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_artifact_affinity_staged_payment_and_entry_replay_in_process(self):
        wanted='Memory Guardian'
        deck=['Island']*24+['Three Tree Mascot']*12+[wanted]*24
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(6000):
                obs=game.observe('P0');obs=game.observe(obs['actor']);self.assertIsNone(obs['outcome'])
                if obs['decision']=='pay_costs' and obs['announcing']['source_card']==wanted:break
                game.step(obs,self.affinity_index(obs,wanted))
            else:self.fail('affinity payment not reached')
            n=sum(o['owner']=='P0' and o['zone']=='Battlefield' and o['card'] in ('Three Tree Mascot',wanted) for o in obs['objects'])
            expected_generic=max(0,4-n);self.assertLess(expected_generic,4)
            self.assertEqual(obs['payment_request']['generic'],expected_generic);saved=game.snapshot()
            def finish(handle):
                paid=None
                for _ in range(80):
                    current=handle.observe('P0');current=handle.observe(current['actor'])
                    if current['stack'] and current['stack'][-1]['source_card']==wanted:
                        self.assertEqual(sum(paid),expected_generic+1)
                        card=current['stack'][-1]['card'];after=self.resolve_vital_top(handle,'P0')
                        obj=next(o for o in after['objects'] if o['id']==card)
                        self.assertEqual(obj['zone'],'Battlefield');self.assertEqual((obj['power'],obj['toughness']),(3,4));self.assertTrue(obj['keywords']&1)
                        return after
                    action=current['actions'][self.affinity_index(current,wanted)]
                    if action['kind']=='PayCosts':paid=action['payment']
                    handle.act(current,action)
                self.fail('affinity cast stalled')
            with game.clone() as fork:expected=finish(fork)
            after=finish(game);self.assertEqual(after,expected);game.restore(saved);again=finish(game)
            after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_affinity_entry_surveil_keeps_private_choices_and_graveyard_replay(self):
        wanted='Valkyrie Aerial Unit'
        deck=['Island']*24+['Three Tree Mascot']*12+[wanted]*24
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            checked=False
            for _ in range(6000):
                obs=game.observe('P0');obs=game.observe(obs['actor']);self.assertIsNone(obs['outcome'])
                if obs['decision']=='pay_costs' and obs['announcing']['source_card']==wanted:
                    n=sum(o['owner']=='P0' and o['zone']=='Battlefield' and o['card'] in ('Three Tree Mascot',wanted) for o in obs['objects'])
                    self.assertEqual(obs['payment_request']['generic'],max(0,5-n));checked=True
                if obs.get('library_choice') and obs.get('resolving',{}).get('source_card')==wanted:break
                game.step(obs,self.affinity_index(obs,wanted))
            else:self.fail('affinity private surveil not reached')
            self.assertTrue(checked);self.assertTrue(obs['library_choice']['surveil']);self.assertEqual(len(obs['library_choice']['remaining']),2)
            opponent=game.observe('P1');self.assertIsNone(opponent['library_choice']);self.assertEqual(opponent['actions'],[])
            saved=game.snapshot();cards=[o['id'] for o in obs['library_choice']['remaining']]
            def finish(handle):
                for card in cards:handle.act(handle.observe('P0'),{'kind':'ArrangeLibrary','card':card,'destination':'Graveyard'})
                after=handle.observe('P0')
                for card in cards:self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==card),'Graveyard')
                self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count']-2)
                self.assertIsNone(after['library_choice']);return after
            with game.clone() as fork:expected=finish(fork)
            after=finish(game);self.assertEqual(after,expected);game.restore(saved);again=finish(game)
            after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def reduction_index(self,obs,which):
        objects={o['id']:o for o in obs['objects']};bf=[o for o in obs['objects'] if o['owner']=='P0' and o['zone']=='Battlefield']
        source='Geist of Saint Thalia' if which=='sacrifice' else 'Mocking Sprite'
        present=any(o['card']==source for o in bf);mascot=any(o['card']=='Three Tree Mascot' for o in bf)
        wanted={'fixed':'Divination','two_brid':'Kin-Tree Severance','sacrifice':'Duty Beyond Death','focus':'Focus the Mind'}[which]
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==source:return 150 if not present else -100
                if name==wanted:
                    ready=(obs['players'][0]['spells_cast_this_turn']>=1 if which=='focus' else present and (mascot or which!='sacrifice'))
                    if which=='two_brid':ready=ready and sum(obs['players'][0]['mana'])+sum(o['card']=='Island' and not o['tapped'] for o in bf)>=5
                    return (170 if k=='AnnounceCast' else 160) if ready else -100
                if name=='Three Tree Mascot':return 140 if which=='focus' or not mascot else -100
            if k=='ChooseSpellCost':return 180 if a['cost']==0 else -100
            if k=='ChooseAdditionalCost':return 180 if objects[a['object']['card']]['card']==source else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_generic_reduction_fixed_and_two_brid_staged_payments_replay_native_totals(self):
        for which in ['fixed','two_brid']:
            wanted='Divination' if which=='fixed' else 'Kin-Tree Severance'
            deck=['Island']*24+['Mocking Sprite']*12+[wanted]*12+['Three Tree Mascot']*12
            with self.subTest(which=which),Game([deck,['Forest']*36+['Ordinary Bear']*24],seed=7,deck_rules='Freeform') as game:
                for _ in range(6000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    if obs['decision']==('pay_costs' if which=='fixed' else 'spell_cost') and obs['announcing']['source_card']==wanted:break
                    game.step(obs,self.reduction_index(obs,which))
                else:self.fail('reduced staged payment not reached')
                if which=='two_brid':self.assertEqual(next(c['cost']['generic'] for c in obs['cost_choices'] if c['index']==0),6)
                else:self.assertEqual(obs['payment_request']['generic'],1)
                self.assertEqual(game.observe('P1')['actions'],[]);saved=game.snapshot()
                def finish(handle):
                    paid=None
                    for _ in range(100):
                        current=handle.observe('P0');current=handle.observe(current['actor'])
                        if current['stack'] and current['stack'][-1]['source_card']==wanted:
                            self.assertEqual(sum(paid),2 if which=='fixed' else 5)
                            victim=current['stack'][-1]['targets'][0]['target']['Card'] if which=='two_brid' else None
                            old=current['players'][0]['library_count'];after=self.resolve_vital_top(handle,'P0')
                            if which=='fixed':self.assertEqual(after['players'][0]['library_count'],old-2)
                            else:self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==victim),'Exile')
                            return after
                        action=current['actions'][self.reduction_index(current,which)]
                        if action['kind']=='PayCosts':
                            self.assertEqual(current['payment_request']['generic'],1 if which=='fixed' else 5);paid=action['payment']
                        handle.act(current,action)
                    self.fail('reduced cast did not complete')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_sacrificed_reducer_keeps_locked_payment_and_full_spell_body_in_process(self):
        deck=['Island']*12+['Plains']*12+['Geist of Saint Thalia']*12+['Duty Beyond Death']*12+['Three Tree Mascot']*12
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(6000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                if obs['decision']=='additional_cost' and obs['announcing']['source_card']=='Duty Beyond Death':break
                game.step(obs,self.reduction_index(obs,'sacrifice'))
            else:self.fail('discounted sacrifice not reached')
            source=next(o for o in obs['objects'] if o['card']=='Geist of Saint Thalia' and o['zone']=='Battlefield')
            mascot=next(o for o in obs['objects'] if o['card']=='Three Tree Mascot' and o['zone']=='Battlefield');saved=game.snapshot()
            def finish(handle):
                for _ in range(50):
                    current=handle.observe('P0')
                    if current['stack'] and current['stack'][-1]['source_card']=='Duty Beyond Death':
                        self.assertEqual(next(o['zone'] for o in current['objects'] if o['id']==source['id']),'Graveyard')
                        after=self.resolve_vital_top(handle,'P0');new=next(o for o in after['objects'] if o['id']==mascot['id'])
                        self.assertEqual(new['counters']['plus_one'],mascot['counters']['plus_one']+1);self.assertTrue(new['keywords']&8192);return after
                    action=current['actions'][self.reduction_index(current,'sacrifice')]
                    if action['kind']=='PayCosts':self.assertEqual(sum(action['payment']),1);self.assertEqual(current['payment_request']['generic'],0)
                    handle.act(current,action)
                self.fail('sacrificed reducer cast stalled')
            with game.clone() as fork:expected=finish(fork)
            after=finish(game);self.assertEqual(after,expected);game.restore(saved);again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_focus_self_discount_preserves_private_draw_then_discard_restore(self):
        with Game([['Island']*36+['Focus the Mind']*12+['Three Tree Mascot']*12,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(6000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                if obs['decision']=='effect_discard' and obs['resolving']['source_card']=='Focus the Mind':break
                if obs['decision']=='pay_costs' and obs['announcing']['source_card']=='Focus the Mind':self.assertEqual(obs['payment_request']['generic'],2)
                game.step(obs,self.reduction_index(obs,'focus'))
            else:self.fail('reduced focus private discard not reached')
            self.assertEqual(obs['players'][0]['cards_drawn_this_turn'],4);self.assertEqual(game.observe('P1')['effect_discards'],[])
            action=next(a for a in obs['actions'] if a['kind']=='Discard');saved=game.snapshot()
            with game.clone() as fork:expected=fork.act(fork.observe('P0'),action)
            after=game.act(obs,action);self.assertEqual(after,expected);self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count'])
            game.restore(saved);again=game.act(game.observe('P0'),action);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def two_brid_index(self,obs,generic):
        objects={o['id']:o for o in obs['objects']}
        def score(a):
            k=a['kind']
            if k=='ChooseSpellCost':
                cost=next(c['cost'] for c in obs['cost_choices'] if c['index']==a['cost'])
                return 200 if cost['generic']==generic else -100
            if k in ('Cast','AnnounceCast') and objects[a['card']]['owner']=='P0':
                lands=[o for o in obs['objects'] if o['owner']=='P0' and o['zone']=='Battlefield' and o['card'] in ('Island','Forest','Plains','Swamp')]
                ready=len(lands)>=6 if generic==6 else {'Forest','Plains','Swamp'}<={o['card'] for o in lands}
                return (160 if k=='AnnounceCast' else 80) if ready else -100
            if k=='TapMana':return 120 if obs['decision']=='pay_costs' or (obs['active']==obs['actor'] and obs['step'] in ('Main1','Main2')) else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_two_brid_cost_commitment_and_exile_replay_at_three_and_six_mana(self):
        for generic in [0,6]:
            deck=(['Forest']*12+['Plains']*12+['Swamp']*12 if generic==0 else ['Island']*36)+['Kin-Tree Severance']*24
            with self.subTest(generic=generic),Game([deck,['Forest']*36+['Ordinary Bear']*24],seed=7,deck_rules='Freeform') as game:
                for _ in range(5000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    if obs['decision']=='spell_cost':break
                    game.step(obs,self.two_brid_index(obs,generic))
                else:self.fail('two-brid cost decision not reached')
                self.assertEqual(actor,'P0');self.assertEqual(game.observe('P1')['actions'],[])
                self.assertTrue(any(c['cost']['generic']==generic for c in obs['cost_choices']))
                saved=game.snapshot()
                def finish(handle):
                    paid=None
                    for _ in range(100):
                        current=handle.observe('P0');current=handle.observe(current['actor'])
                        if current['stack'] and current['stack'][-1]['source_card']=='Kin-Tree Severance':
                            self.assertEqual(sum(paid),3 if generic==0 else 6)
                            victim=current['stack'][-1]['targets'][0]['target']['Card']
                            after=self.resolve_vital_top(handle,'P0')
                            self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==victim),'Exile');return after
                        action=current['actions'][self.two_brid_index(current,generic)]
                        if action['kind']=='PayCosts':paid=action['payment']
                        handle.act(current,action)
                    self.fail('committed two-brid cast did not finish')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected)
                game.restore(saved);again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_two_brid_entry_counter_and_private_loot_use_native_snapshots(self):
        for name in ['Reputable Merchant','Temur Tawnyback']:
            with self.subTest(card=name),Game([['Island']*36+[name]*24,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(5000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    if name=='Reputable Merchant' and obs['stack'] and obs['stack'][-1]['source_card']==name and 'TriggeredAbility' in obs['stack'][-1]['kind']:
                        item=obs['stack'][-1];target=item['targets'][0]['target']['Card'];saved=game.snapshot()
                        with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
                        after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected)
                        self.assertEqual(next(o['counters']['plus_one'] for o in after['objects'] if o['id']==target),1)
                        game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);break
                    if name=='Temur Tawnyback' and obs['decision']=='effect_discard':
                        self.assertEqual(obs['players'][0]['cards_drawn_this_turn'],2)
                        self.assertEqual(game.observe('P1')['effect_discards'],[]);self.assertEqual(game.observe('P1')['actions'],[])
                        action=next(a for a in obs['actions'] if a['kind']=='Discard');saved=game.snapshot()
                        with game.clone() as fork:expected=fork.act(fork.observe('P0'),action)
                        after=game.act(obs,action);self.assertEqual(after,expected);self.assertEqual(after['players'][0]['cards_drawn_this_turn'],2)
                        self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count'])
                        game.restore(saved);again=game.act(game.observe('P0'),action);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);break
                    game.step(obs,self.two_brid_index(obs,6))
                else:self.fail('two-brid entry body not reached')

    def counter_filter_index(self,obs,delay=False):
        objects={o['id']:o for o in obs['objects']}
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast') and objects[a['card']]['owner']=='P1':
                return 150 if obs['stack'] and (not delay or obs['turn']>=12) else -100
            if k=='TapMana':
                return 120 if obs['decision'] in ('pay_costs','resolution_mana') or (obs['actor']=='P0' and obs['active']=='P0' and obs['step'] in ('Main1','Main2')) or (obs['actor']=='P1' and obs['stack'] and (not delay or obs['turn']>=12)) else -100
            if k in ('Activate','AnnounceActivate','Attack','AttackPlaneswalker'):return -100
            return {'Keep':140,'PlayLand':130,'TapMana':120,'Cast':80,'PutTrigger':100,'ChooseTarget':90,'FinishTargets':70,'Discard':60,'PayCosts':100,'Pass':0,'FinishAttackers':0,'FinishBlockers':0,'Mulligan':-100,'Concede':-1000}.get(k,10)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_restricted_counter_targets_execute_and_restore_through_native_library(self):
        cases=[('Annul','Goldvein Pick','Forest'),('Flashfreeze','Bear Cub','Forest'),
               ('Precise Redaction','Hinterland Sanctifier','Plains'),('Spell Snare','Bear Cub','Forest'),
               ('Disdainful Stroke','Ordinary Bear','Forest')]
        for name,target,land in cases:
            with self.subTest(card=name),Game([[land]*32+[target]*28,['Island']*32+[name]*28],seed=11,deck_rules='Freeform') as game:
                for _ in range(4000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    if obs['stack'] and obs['stack'][-1]['source_card']==name:
                        item=obs['stack'][-1];victim=item['targets'][0]['target']['Spell']
                        self.assertEqual(next(o['card'] for o in obs['objects'] if o['id']==victim),target)
                        saved=game.snapshot()
                        with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                        after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected)
                        self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==victim),'Graveyard')
                        self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==item['card']),'Graveyard')
                        game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);break
                    game.step(obs,self.counter_filter_index(obs))
                else:self.fail('restricted counter did not resolve')

    def test_spectral_four_mana_payment_and_decline_restore_native_continuation(self):
        for paid in [False,True]:
            with self.subTest(paid=paid),Game([['Forest']*36+['Bear Cub']*24,['Island']*36+['Spectral Interference']*24],seed=11,deck_rules='Freeform') as game:
                for _ in range(5000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    if obs['decision']=='resolution_mana':break
                    game.step(obs,self.counter_filter_index(obs,delay=True))
                else:self.fail('spectral payment not reached')
                self.assertEqual(actor,'P0');self.assertEqual(obs['resolving']['source_card'],'Spectral Interference')
                request=obs['resolution_payment'];self.assertEqual(request['cost']['generic'],4)
                self.assertEqual(game.observe('P1')['resolution_payment'],request);self.assertEqual(game.observe('P1')['actions'],[])
                victim=obs['resolving']['targets'][0]['target']['Spell'];saved=game.snapshot()
                def finish(handle):
                    for _ in range(12):
                        current=handle.observe(actor)
                        if current['resolution_payment'] is None:return current
                        index=next((i for i,a in enumerate(current['actions']) if a['kind']==('PayResolutionMana' if paid else 'DeclineResolutionMana')),None)
                        if index is None:index=next((i for i,a in enumerate(current['actions']) if a['kind']=='TapMana'),None)
                        self.assertIsNotNone(index,'fixture needs four mana');handle.step(current,index)
                    self.fail('spectral payment stalled')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected)
                self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==victim),'Stack' if paid else 'Graveyard')
                game.restore(saved);again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_greatshark_flash_entry_counter_target_replays_in_process(self):
        with Game([['Forest']*32+['Bear Cub']*28,['Island']*32+['Voracious Greatshark']*28],seed=11,deck_rules='Freeform') as game:
            for _ in range(5000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                if obs['stack'] and obs['stack'][-1]['source_card']=='Voracious Greatshark' and 'TriggeredAbility' in obs['stack'][-1]['kind']:
                    item=obs['stack'][-1];victim=item['targets'][0]['target']['Spell'];source=next(o for o in obs['objects'] if o['id']==item['card'])
                    self.assertEqual((source['zone'],source['power'],source['toughness']),('Battlefield',5,4))
                    saved=game.snapshot()
                    with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                    after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected)
                    self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==victim),'Graveyard')
                    self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==item['card']),'Battlefield')
                    game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                game.step(obs,self.counter_filter_index(obs))
            self.fail('greatshark counter trigger not reached')

    def test_donatello_intervening_condition_controls_trigger_creation_and_native_replay(self):
        for artifact in [False,True]:
            deck=['Island']*24+['Donatello, Turtle Techie']*(24 if artifact else 36)+(['Three Tree Mascot']*12 if artifact else [])
            with self.subTest(artifact=artifact),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(4000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    if obs['stack'] and obs['stack'][-1]['source_card']=='Donatello, Turtle Techie' and obs['stack'][-1]['kind']=='Spell':
                        old=obs['players'][0]['cards_drawn_this_turn'];entered=self.resolve_vital_top(game,'P0')
                        source=next(o for o in entered['objects'] if o['card']=='Donatello, Turtle Techie' and o['zone']=='Battlefield')
                        self.assertEqual(entered['players'][0]['cards_drawn_this_turn'],old)
                        triggers=[a for a in entered['stack'] if a['source_card']=='Donatello, Turtle Techie' and 'TriggeredAbility' in a['kind']]
                        self.assertEqual(len(triggers),int(artifact))
                        if artifact:
                            self.assertEqual(triggers[0]['object'],source['reference']);saved=game.snapshot()
                            with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
                            after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected)
                            self.assertEqual(after['players'][0]['cards_drawn_this_turn'],old+1)
                            game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)
                        break
                    objects={o['id']:o for o in obs['objects']}
                    index=max(range(len(obs['actions'])),key=lambda i:90 if obs['actions'][i]['kind']=='Cast' and objects[obs['actions'][i]['card']]['card']=='Three Tree Mascot' else self.professor_score(obs,obs['actions'][i]))
                    game.step(obs,index)
                else:self.fail('Donatello entry not reached')

    def test_conditional_rabbit_scry_preserves_one_growth_application_and_private_restore(self):
        deck=['Plains']*24+['Rabbit Response']*12+['Three Tree Mascot']*12+['Hinterland Sanctifier']*12
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(4000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome']);objects={o['id']:o for o in obs['objects']}
                rabbit=any(o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Three Tree Mascot' for o in obs['objects'])
                def score(a):
                    if a['kind']=='Cast' and objects[a['card']]['card']=='Rabbit Response':return 100 if rabbit else -100
                    return self.professor_score(obs,a)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));a=obs['actions'][index]
                if a['kind']=='Cast' and objects[a['card']]['card']=='Rabbit Response':
                    creatures=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['owner']=='P0' and o['card'] in ('Three Tree Mascot','Hinterland Sanctifier')]
                    game.step(obs,index);chosen=self.resolve_vital_top(game,'P0');self.assertEqual(len(chosen['library_choice']['remaining']),2);self.assertIsNone(game.observe('P1')['library_choice'])
                    for o in creatures:
                        new=next(n for n in chosen['objects'] if n['reference']==o['reference']);self.assertEqual((new['power'],new['toughness']),(o['power']+2,o['toughness']+1))
                    choices=[{'kind':'ArrangeLibrary','card':o['id'],'destination':d} for o,d in zip(chosen['library_choice']['remaining'],['Top','Bottom'])]
                    saved=game.snapshot()
                    with game.clone() as fork:
                        for choice in choices:expected=fork.act(fork.observe('P0'),choice)
                    for choice in choices:after=game.act(game.observe('P0'),choice)
                    self.assertEqual(after,expected)
                    for o in creatures:self.assertEqual(next(n['power'] for n in after['objects'] if n['reference']==o['reference']),o['power']+2)
                    game.restore(saved)
                    for choice in choices:again=game.act(game.observe('P0'),choice)
                    after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                game.step(obs,index)
            self.fail('conditional scry not reached')

    def test_replacement_growth_chooses_one_live_branch_in_native_library(self):
        deck=['Plains']*24+["You're Not Alone"]*12+['Hinterland Sanctifier']*12+["Ajani's Pridemate"]*12
        for large in [False,True]:
            with self.subTest(large=large),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(5000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome']);objects={o['id']:o for o in obs['objects']}
                    count=sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['card'] in ('Hinterland Sanctifier',"Ajani's Pridemate") for o in obs['objects'])
                    def score(a):
                        if a['kind']=='Cast' and objects[a['card']]['card']=="You're Not Alone":return 100 if count>0 and (count>=3)==large else -100
                        return self.professor_score(obs,a)
                    index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));a=obs['actions'][index]
                    if a['kind']=='Cast' and objects[a['card']]['card']=="You're Not Alone":
                        old=objects[a['target']['Card']];game.step(obs,index);saved=game.snapshot()
                        with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
                        after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected);new=next(o for o in after['objects'] if o['reference']==old['reference']);bonus=4 if large else 2
                        self.assertEqual((new['power'],new['toughness']),(old['power']+bonus,old['toughness']+bonus))
                        game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);break
                    game.step(obs,index)
                else:self.fail('replacement growth branch not reached')

    def test_conditional_rummage_restores_paid_choice_without_repeating_prior_damage(self):
        deck=['Mountain']*24+['Take Out the Trash']*12+['Three Tree Mascot']*12+['Goblin Piker']*12
        with Game([deck,['Mountain']*24+['Shivan Dragon']*36],seed=7,deck_rules='Freeform') as game:
            for _ in range(5000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome']);objects={o['id']:o for o in obs['objects']}
                if obs['effect_choice'] is not None and obs['resolving']['source_card']=='Take Out the Trash':
                    self.assertIsNone(game.observe('P1')['effect_choice']);victim=obs['resolving']['targets'][0]['target']['Card'];self.assertEqual(objects[victim]['damage'],3)
                    discard=next(a for a in obs['actions'] if a['kind']=='Discard');saved=game.snapshot()
                    with game.clone() as fork:expected=fork.act(fork.observe('P0'),discard)
                    after=game.act(obs,discard);self.assertEqual(after,expected);self.assertEqual(next(o['damage'] for o in after['objects'] if o['id']==victim),3)
                    self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count']-1)
                    self.assertEqual(after['players'][0]['cards_drawn_this_turn'],obs['players'][0]['cards_drawn_this_turn']+1)
                    game.restore(saved);again=game.act(game.observe('P0'),discard);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                raccoon=any(o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Three Tree Mascot' for o in obs['objects'])
                def score(a):
                    if a['kind']=='Cast' and objects[a['card']]['card']=='Take Out the Trash':return 100 if raccoon and obs['players'][0]['hand_count']>=2 and objects[a['target']['Card']]['owner']=='P1' else -100
                    return self.professor_score(obs,a)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));game.step(obs,index)
            self.fail('conditional private rummage not reached')

    def pinned_color_count(self,obs,player):
        rows={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
        colors=set()
        for o in obs['objects']:
            if o['owner']!=player or o['zone']!='Battlefield':continue
            if o['token'] is not None:colors.update(c for i,c in enumerate('WUBRG') if o['token']['colors']&(1<<i))
            else:colors.update(rows[o['card']]['colors'])
        return len(colors)

    def test_vivid_scaled_draw_and_drain_use_color_unions_through_native_replay(self):
        cases=[('Shinestriker',['Island']*12+['Forest']*12+['Shinestriker']*12+['Glister Bairn']*12+['Bear Cub']*12),
               ('Shimmercreep',['Swamp']*12+['Forest']*12+['Shimmercreep']*12+['Luminollusk']*12+['Glister Bairn']*12)]
        for name,deck in cases:
            with self.subTest(card=name),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(5000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    if obs['stack'] and obs['stack'][-1]['source_card']==name and 'TriggeredAbility' in obs['stack'][-1]['kind']:
                        amount=self.pinned_color_count(obs,'P0')
                        if amount>=2:
                            saved=game.snapshot()
                            with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                            after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected)
                            if name=='Shinestriker':
                                self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count']-amount)
                                self.assertEqual(after['players'][0]['hand_count'],obs['players'][0]['hand_count']+amount)
                                self.assertEqual(after['players'][0]['cards_drawn_this_turn'],obs['players'][0]['cards_drawn_this_turn']+amount)
                                self.assertFalse(any(o['owner']=='P0' and o['zone']=='Hand' for o in game.observe('P1')['objects']))
                            else:
                                self.assertEqual(after['players'][1]['life'],obs['players'][1]['life']-amount)
                                self.assertEqual(after['players'][0]['life'],obs['players'][0]['life']+amount)
                                self.assertEqual(after['players'][0]['life_gained_this_turn'],obs['players'][0]['life_gained_this_turn']+amount)
                                self.assertEqual(after['players'][0]['life_gain_events_this_turn'],obs['players'][0]['life_gain_events_this_turn']+1)
                            game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);break
                    index=max(range(len(obs['actions'])),key=lambda i:self.professor_score(obs,obs['actions'][i]))
                    game.step(obs,index)
                else:self.fail('nontrivial distinct-color effect not reached')

    def test_peer_private_batch_then_owned_graveyard_type_draw_replays_without_repeating_discards(self):
        deck=['Forest']*12+['Mountain']*12+['Peer Past the Veil']*12+['Three Tree Mascot']*6+['Clachan Festival']*6+['Bear Cub']*6+['Goldvein Pick']*6
        rows={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(5000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                if obs['decision']=='effect_discard' and obs['resolving']['source_card']=='Peer Past the Veil' and obs['players'][0]['hand_count']>=2:
                    hand=[o for o in obs['objects'] if o['owner']=='P0' and o['zone']=='Hand']
                    typed=[o for o in obs['objects'] if o['owner']=='P0' and o['zone'] in ('Hand','Graveyard') and o['token'] is None]
                    types=set()
                    for o in typed:types.update(set(rows[o['card']]['type_line'].split(' — ')[0].split())&{'Artifact','Battle','Creature','Enchantment','Instant','Kindred','Land','Planeswalker','Sorcery'})
                    amount=len(types);self.assertGreater(amount,1);self.assertGreater(obs['players'][0]['library_count'],amount)
                    first=next(a for a in obs['actions'] if a['kind']=='Discard');chosen=game.act(obs,first)
                    self.assertEqual(next(o['zone'] for o in chosen['objects'] if o['id']==first['card']),'Hand')
                    other=game.observe('P1');self.assertEqual(other['effect_discards'],[]);self.assertFalse(any(o['id'] in {c['id'] for c in hand} for o in other['objects']))
                    saved=game.snapshot()
                    def finish(handle):
                        for _ in range(100):
                            current=handle.observe('P0')
                            if current['decision']!='effect_discard':return current
                            handle.act(current,next(a for a in current['actions'] if a['kind']=='Discard'))
                        self.fail('private discard batch stalled')
                    with game.clone() as fork:expected=finish(fork)
                    after=finish(game);self.assertEqual(after,expected)
                    self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count']-amount)
                    self.assertEqual(after['players'][0]['hand_count'],amount)
                    self.assertEqual(after['players'][0]['cards_drawn_this_turn'],obs['players'][0]['cards_drawn_this_turn']+amount)
                    game.restore(saved);again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                objects={o['id']:o for o in obs['objects']}
                def score(a):
                    if a['kind']=='Cast' and objects[a['card']]['card']=='Peer Past the Veil':return 90
                    return self.professor_score(obs,a)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));game.step(obs,index)
            self.fail('Peer private discard continuation not reached')

    def test_cat_gator_native_scaled_damage_and_lifelink_replay(self):
        deck=['Swamp']*24+['Cat-Gator']*36
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(4000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                if obs['stack'] and obs['stack'][-1]['source_card']=='Cat-Gator' and obs['stack'][-1]['kind']=={'TriggeredAbility':{'ability':0}}:
                    self.assertEqual(obs['stack'][-1]['targets'][0]['target'],{'Player':'P1'})
                    amount=sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Swamp' for o in obs['objects']);self.assertGreater(amount,0)
                    saved=game.snapshot()
                    with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                    after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected)
                    self.assertEqual(after['players'][1]['life'],obs['players'][1]['life']-amount)
                    self.assertEqual(after['players'][0]['life'],obs['players'][0]['life']+amount)
                    self.assertEqual(after['players'][0]['life_gained_this_turn'],obs['players'][0]['life_gained_this_turn']+amount)
                    game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                index=max(range(len(obs['actions'])),key=lambda i:self.professor_score(obs,obs['actions'][i]))
                game.step(obs,index)
            self.fail('scaled Cat-Gator trigger not reached')

    def test_archway_angel_native_gain_quantity_and_event_totals_replay(self):
        deck=['Plains']*24+['Azorius Guildgate']*12+['Archway Angel']*24
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(4000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                amount=2*sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Azorius Guildgate' for o in obs['objects'])
                if amount and obs['stack'] and obs['stack'][-1]['source_card']=='Archway Angel' and obs['stack'][-1]['kind']=={'TriggeredAbility':{'ability':0}}:
                    saved=game.snapshot()
                    with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                    after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected)
                    self.assertEqual(after['players'][0]['life'],obs['players'][0]['life']+amount)
                    self.assertEqual(after['players'][0]['life_gained_this_turn'],obs['players'][0]['life_gained_this_turn']+amount)
                    self.assertEqual(after['players'][0]['life_gain_events_this_turn'],obs['players'][0]['life_gain_events_this_turn']+1)
                    game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                index=max(range(len(obs['actions'])),key=lambda i:self.professor_score(obs,obs['actions'][i]))
                game.step(obs,index)
            self.fail('scaled Archway Angel trigger not reached')

    def test_germinate_uses_exact_life_gained_total_through_native_batch_and_replay(self):
        deck=['Plains']*12+['Swamp']*6+['Forest']*6+['Germinate Recruits']*12+['Professor Dellian Fel']*12+['Hinterland Sanctifier']*12
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(5000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                objects={o['id']:o for o in obs['objects']}
                def score(a):
                    if a['kind']=='Cast' and objects[a['card']]['card']=='Germinate Recruits':
                        p=obs['players'][0];return 105 if p['life_gained_this_turn']>p['life_gain_events_this_turn'] and p['life_gained_this_turn']!=p['life'] else -100
                    return self.professor_score(obs,a,wanted=0)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));a=obs['actions'][index]
                if a['kind']=='Cast' and objects[a['card']]['card']=='Germinate Recruits':
                    amount=obs['players'][0]['life_gained_this_turn'];self.assertEqual(obs['schema'],42)
                    count=lambda view:sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['token'] is not None and o['token']['name']=='Cadet' for o in view['objects'])
                    old=count(obs);game.step(obs,index);saved=game.snapshot()
                    with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                    after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected);self.assertEqual(count(after),old+amount)
                    cadets=[o for o in after['objects'] if o['token'] is not None and o['token']['name']=='Cadet']
                    self.assertTrue(all(o['token']['subtypes']=='Wizard Soldier' and o['power']==2 and o['toughness']==2 for o in cadets))
                    game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                game.step(obs,index)
            self.fail('nontrivial gained-life token batch not reached')

    def test_gather_scaled_tokens_precede_private_scry_and_do_not_repeat_on_restore(self):
        deck=['Plains']*24+['Gather the White Lotus']*12+['Clachan Festival']*12+['Hinterland Sanctifier']*12
        with Game([deck,['Plains']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(4000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome']);objects={o['id']:o for o in obs['objects']}
                index=max(range(len(obs['actions'])),key=lambda i:self.professor_score(obs,obs['actions'][i]));a=obs['actions'][index]
                if a['kind']=='Cast' and objects[a['card']]['card']=='Gather the White Lotus':
                    amount=sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Plains' for o in obs['objects'])
                    game.step(obs,index);after=self.resolve_vital_top(game,actor)
                    count=lambda v:sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['token'] is not None and o['token']['subtypes']=='Ally' for o in v['objects'])
                    self.assertEqual(count(after),amount);self.assertEqual(len(after['library_choice']['remaining']),2);self.assertIsNone(game.observe('P1')['library_choice'])
                    with self.assertRaisesRegex(EngineError,'IllegalAction'):game.act(after,{'kind':'ArrangeLibrary','card':after['library_choice']['remaining'][0]['id'],'destination':'Graveyard'})
                    self.assertEqual(game.observe(actor),after)
                    saved=game.snapshot();actions=[{'kind':'ArrangeLibrary','card':o['id'],'destination':destination} for o,destination in zip(after['library_choice']['remaining'],['Top','Bottom'])]
                    with game.clone() as fork:
                        for choice in actions:expected=fork.act(fork.observe(actor),choice)
                    for choice in actions:result=game.act(game.observe(actor),choice)
                    self.assertEqual(result,expected);self.assertEqual(count(result),amount)
                    game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.act(after,actions[0])
                    for choice in actions:again=game.act(game.observe(actor),choice)
                    result['revision']=again['revision'];self.assertEqual(again,result);self.assertEqual(count(again),amount);game.drop_snapshot(saved);return
                game.step(obs,index)
            self.fail('scaled token/scry continuation not reached')

    def test_krenko_native_resolution_freezes_goblin_count_and_replays_token_generations(self):
        deck=['Mountain']*24+['Krenko, Mob Boss']*12+['Dragon Fodder']*12+['Goblin Piker']*12
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(4000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome']);objects={o['id']:o for o in obs['objects']}
                existing=any(o['zone']=='Battlefield' and o['card']=='Krenko, Mob Boss' and o['owner']=='P0' for o in obs['objects'])
                def score(a):
                    if a['kind']=='Cast' and objects[a['card']]['card']=='Krenko, Mob Boss':return -100 if existing else 90
                    if a['kind']=='Activate' and objects[a['card']]['card']=='Krenko, Mob Boss':return 105
                    return self.professor_score(obs,a)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));a=obs['actions'][index]
                if a['kind']=='Activate' and objects[a['card']]['card']=='Krenko, Mob Boss':
                    goblin=lambda o:o['card'] in ('Krenko, Mob Boss','Goblin Piker') or o['token'] is not None and o['token']['subtypes']=='Goblin'
                    amount=sum(o['zone']=='Battlefield' and o['owner']=='P0' and goblin(o) for o in obs['objects'])
                    count=lambda v:sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['token'] is not None and o['token']['subtypes']=='Goblin' for o in v['objects'])
                    old=count(obs);game.step(obs,index);saved=game.snapshot()
                    with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                    after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected);self.assertEqual(count(after),old+amount)
                    refs=[(o['reference']['card'],o['reference']['generation']) for o in after['objects'] if o['zone']=='Battlefield' and o['token'] is not None];self.assertEqual(len(refs),len(set(refs)))
                    self.assertFalse(any(x['kind']=='Activate' and x['card']==a['card'] for x in after['actions']))
                    game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                game.step(obs,index)
            self.fail('Krenko scaled batch not reached')

    def professor_score(self,obs,action,wanted=None,combat=False):
        objects={o['id']:o for o in obs['objects']};actor=obs['actor'];kind=action['kind']
        walkers=[o for o in obs['objects'] if o['card']=='Professor Dellian Fel' and o['owner']=='P0' and o['zone']=='Battlefield']
        emblems=[o for o in obs['objects'] if o['emblem'] is not None and o['owner']=='P0']
        if kind=='Cast':
            name=objects[action['card']]['card']
            if name=='Professor Dellian Fel':return 90 if not walkers and not emblems else -100
            if name=='Moment of Craving':return 85 if emblems and not walkers and action['target'] is not None and objects[action['target']['Card']]['owner']=='P1' else -100
            return 80
        if kind=='Activate' and objects[action['card']]['card']=='Professor Dellian Fel':
            if actor!='P0' or (combat and emblems):return -100
            ability=action['ability']
            if wanted is not None:return 100 if ability==wanted else 95 if wanted==3 and ability==0 else -100
            return {3:100,0:95}.get(ability,-100)
        if kind=='AttackPlaneswalker':return 60 if combat and emblems and actor=='P1' else -100
        if kind in ('Attack','Block'):return -100
        if kind=='TapMana':return 10 if obs['active']==actor and obs['step'] in ('Main1','Main2') else -100
        if kind=='ChooseTarget':return 70 if action['target'].get('Player')=='P1' else 50
        if kind=='AssignDamage':return 60+action['amount']
        return {'Keep':120,'PlayLand':110,'PutTrigger':80,'ChooseLegend':70,'FinishTargets':60,'PayCosts':100,'Discard':60,'Concede':-1000,'Mulligan':-100,'AnnounceCast':-20,'AnnounceActivate':-20}.get(kind,0)

    def test_professor_all_four_complete_abilities_pay_loyalty_before_native_resolution(self):
        for wanted in [0,1,2,3]:
            with self.subTest(ability=wanted),Game([['Forest']*12+['Swamp']*12+['Professor Dellian Fel']*36,['Forest']*24+['Bear Cub']*36],seed=7,deck_rules='Freeform') as game:
                for _ in range(4000):
                    actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                    index=max(range(len(obs['actions'])),key=lambda i:self.professor_score(obs,obs['actions'][i],wanted=wanted))
                    action=obs['actions'][index]
                    if action['kind']=='Activate' and action['ability']==wanted:
                        source=next(o for o in obs['objects'] if o['id']==action['card']);loyalty=source['counters']['loyalty']
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                        paid=game.step(obs,index);self.assertEqual(paid,expected)
                        now=next(o for o in paid['objects'] if o['id']==source['id'])
                        self.assertEqual(now['counters']['loyalty'],loyalty+[2,0,-3,-6][wanted])
                        self.assertEqual(paid['players'][0]['life'],obs['players'][0]['life'])
                        self.assertEqual(now['used_abilities'],1<<wanted)
                        game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                        paid=game.step(game.observe(actor),index)
                        after=self.resolve_vital_top(game,actor)
                        if wanted==0:self.assertEqual(after['players'][0]['life'],obs['players'][0]['life']+3)
                        if wanted==1:
                            self.assertEqual(after['players'][0]['life'],obs['players'][0]['life']-1)
                            self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count']-1)
                        if wanted==2:self.assertEqual(next(o for o in after['objects'] if o['id']==action['target']['Card'])['zone'],'Graveyard')
                        if wanted==3:self.assertEqual(sum(o['zone']=='Command' and o['emblem'] is not None for o in after['objects']),1)
                        self.assertFalse(any(a['kind'] in ('Activate','AnnounceActivate','BeginActivate') and a['card']==source['id'] for a in after['actions']))
                        game.drop_snapshot(saved);break
                    game.step(obs,index)
                else:self.fail('complete Professor ability not reached')

    def test_real_emblem_survives_planeswalker_combat_death_and_replays_captured_life_loss(self):
        deck=['Forest']*10+['Swamp']*14+['Professor Dellian Fel']*12+['Moment of Craving']*24
        with Game([deck,['Forest']*24+['Bear Cub']*36],seed=7,deck_rules='Freeform') as game:
            attacked=False;checked_factory=False
            for _ in range(6000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor);self.assertIsNone(obs['outcome'])
                emblems=[o for o in obs['objects'] if o['emblem'] is not None]
                if emblems and not checked_factory:
                    for mode in ['Freeform','Constructed','Standard']:
                        with self.assertRaisesRegex(EngineError,'emblems cannot'):Game([[emblems[0]['card']]*60]*2,deck_rules=mode)
                    checked_factory=True
                if obs['stack'] and obs['stack'][-1]['life_event'] is not None and obs['stack'][-1]['source_card'].startswith('@emblem/'):
                    item=obs['stack'][-1]
                    self.assertTrue(attacked);self.assertTrue(checked_factory)
                    self.assertFalse(any(o['card']=='Professor Dellian Fel' and o['zone']=='Battlefield' for o in obs['objects']))
                    self.assertEqual(item['life_event'],{'player':'P0','amount':2})
                    self.assertEqual(item['object'],emblems[0]['reference']);self.assertEqual(item['targets'][0]['target'],{'Player':'P1'})
                    life=obs['players'][1]['life'];saved=game.snapshot()
                    with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
                    after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected)
                    self.assertEqual(after['players'][1]['life'],life-2);self.assertEqual(next(o for o in after['objects'] if o['id']==emblems[0]['id'])['zone'],'Command')
                    game.restore(saved);again=self.resolve_vital_top(game,actor);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);return
                index=max(range(len(obs['actions'])),key=lambda i:self.professor_score(obs,obs['actions'][i],combat=True))
                action=obs['actions'][index]
                if action['kind']=='AttackPlaneswalker':
                    saved=game.snapshot()
                    with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                    chosen=game.step(obs,index);self.assertEqual(chosen,expected)
                    self.assertIn({'card':action['card'],'defender':{'Planeswalker':action['defender']},'blocked':False},chosen['attack_declarations'])
                    game.restore(saved);game.step(game.observe(actor),index);game.drop_snapshot(saved);attacked=True
                else:game.step(obs,index)
            self.fail('emblem did not trigger after real walker combat death')

    def test_attack_declarations_are_private_until_commit_and_owned_by_native_snapshots(self):
        deck=['Forest']*24+['Bear Cub']*36
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            for _ in range(3000):
                actor=game.observe('P0')['actor'];obs=game.observe(actor)
                self.assertIsNone(obs['outcome'])
                attack=next((a for a in obs['actions'] if a['kind']=='Attack'),None)
                if attack is not None:
                    self.assertEqual(obs['schema'],42)
                    other='P1' if actor=='P0' else 'P0'
                    source=next(o for o in obs['objects'] if o['id']==attack['card'])
                    forged={'kind':'AttackPlaneswalker','card':source['id'],'defender':source['reference']}
                    with self.assertRaisesRegex(EngineError,'IllegalAction'):game.act(obs,forged)
                    self.assertEqual(game.observe(actor),obs)
                    saved=game.snapshot()
                    with game.clone() as fork:expected=fork.act(fork.observe(actor),attack)
                    chosen=game.act(obs,attack);self.assertEqual(chosen,expected)
                    self.assertEqual(chosen['attack_declarations'],[{'card':source['id'],'defender':{'Player':other},'blocked':False}])
                    self.assertEqual(game.observe(other)['attack_declarations'],[])
                    finish={'kind':'FinishAttackers'}
                    committed=game.act(chosen,finish)
                    self.assertEqual(game.observe(other)['attack_declarations'],committed['attack_declarations'])
                    self.assertTrue(next(o for o in committed['objects'] if o['id']==source['id'])['attacking'])
                    game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.act(obs,attack)
                    chosen=game.act(game.observe(actor),attack)
                    again=game.act(chosen,finish);committed['revision']=again['revision']
                    self.assertEqual(again,committed);game.drop_snapshot(saved);return
                def score(a):
                    kind=a['kind']
                    if kind=='TapMana':return 10 if obs['active']==actor and obs['step'] in ('Main1','Main2') else -10
                    return {'Keep':100,'PlayLand':90,'Cast':80,'Attack':60,'Block':50,'Discard':40,'Concede':-1000,'Mulligan':-100,'AnnounceCast':-20}.get(kind,0)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))
                game.step(obs,index)
            self.fail('public attack declaration not reached')

    def test_numeric_capacity_error_preserves_native_handle_and_every_checkpoint(self):
        deck=['Forest']*24+['Island']*24+['Zimone, Paradox Sculptor']*96
        with Game([deck,['Forest']*144],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0');checkpoint=None
            for _ in range(25000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                source=next((o for o in obs['objects'] if o['owner']=='P0' and o['zone']=='Battlefield' and o['card']=='Zimone, Paradox Sculptor'),None)
                def score(action):
                    kind=action['kind']
                    if kind=='ChooseTarget':return 110 if source and action['target']=={'Card':source['id']} else -100
                    if kind=='AnnounceActivate':return 90 if source and action['card']==source['id'] else -100
                    if kind=='AnnounceCast':return 90 if source is None else -100
                    if kind=='Discard':return 70 if any(o['id']==action['card'] and o['card']=='Zimone, Paradox Sculptor' for o in obs['objects']) else 0
                    if kind in ('TapMana','ActivateMana'):return 80 if obs['decision']=='pay_costs' else -100
                    return {'Keep':120,'PlayLand':100,'PayCosts':110,'FinishTargets':80,'PutTrigger':90,
                            'FinishAttackers':100,'FinishBlockers':100,'Pass':20,'Concede':-1000}.get(kind,-100)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))
                if source and 2**55<source['counters']['plus_one']<2**60 and checkpoint is None:
                    checkpoint=game.snapshot();checkpoint_view=game.observe('P0')
                try:after=game.step(obs,index)
                except CapacityError as error:
                    self.assertEqual(error.resource,'counter representation exhausted');self.assertIsNotNone(checkpoint)
                    self.assertEqual(game.observe(obs['actor']),obs);self.assertIsNone(game.observe('P0')['outcome'])
                    saved=game.snapshot()
                    with game.clone() as fork:
                        self.assertEqual(fork.observe(obs['actor']),obs)
                        with self.assertRaises(CapacityError):fork.step(fork.observe(obs['actor']),index)
                        self.assertEqual(fork.observe(obs['actor']),obs)
                    with self.assertRaises(CapacityError):game.step(obs,index)
                    self.assertEqual(game.observe(obs['actor']),obs);game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                    game.drop_snapshot(saved);game.restore(checkpoint);restored=game.observe('P0')
                    checkpoint_view['revision']=restored['revision'];self.assertEqual(restored,checkpoint_view);game.drop_snapshot(checkpoint)
                    current=game.observe(restored['actor']);result=game.act(current,{'kind':'Concede'})
                    self.assertIsNotNone(result['outcome']);return
                obs=after
            self.fail('checked counter capacity was not reached by public legal play')

    def test_repeated_zimone_doubling_survives_i32_and_u32_boundaries_in_public_play(self):
        deck=['Forest']*12+['Island']*12+['Zimone, Paradox Sculptor']*36
        with Game([deck,['Forest']*24+['Bear Cub']*36],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0');doubled=0
            for _ in range(15000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                source=next((o for o in obs['objects'] if o['owner']=='P0' and o['zone']=='Battlefield' and o['card']=='Zimone, Paradox Sculptor'),None)
                def score(action):
                    kind=action['kind']
                    if kind=='ChooseTarget':return 110 if source and action['target']=={'Card':source['id']} else -100
                    if kind=='AnnounceActivate':return 90 if source and action['card']==source['id'] else -100
                    if kind=='AnnounceCast':
                        if obs['actor']=='P1':return 90 if sum(o['zone']=='Battlefield' and o['owner']=='P1' and o['card']=='Bear Cub' for o in obs['objects'])<2 else -100
                        return 90 if source is None else -100
                    if kind=='Discard':return 70 if any(o['id']==action['card'] and o['card']=='Zimone, Paradox Sculptor' for o in obs['objects']) else 0
                    if kind in ('TapMana','ActivateMana'):return 80 if obs['decision']=='pay_costs' else -100
                    return {'Keep':120,'PlayLand':100,'PayCosts':110,'FinishTargets':80,'PutTrigger':90,
                            'FinishAttackers':100,'FinishBlockers':100,'Pass':20,'Concede':-1000}.get(kind,-100)
                index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));action=obs['actions'][index]
                top=obs['stack'][-1] if obs['stack'] else None
                resolving=source and action['kind']=='Pass' and top and top['source_card']=='Zimone, Paradox Sculptor' and isinstance(top['kind'],dict) and 'ActivatedAbility' in top['kind']
                saved=game.snapshot() if resolving and doubled>=29 else None
                actor=obs['actor'];before=source['counters']['plus_one'] if source else 0
                if saved is not None:
                    with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                after=game.step(obs,index)
                if saved is not None:
                    self.assertEqual(after,expected);game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                    after=game.step(game.observe(actor),index);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved)
                if resolving and not any(item['id']==top['id'] for item in after['stack']):
                    current=next(o for o in after['objects'] if o['reference']==source['reference'])
                    self.assertEqual(current['zone'],'Battlefield');self.assertEqual(current['counters']['plus_one'],2*before)
                    self.assertEqual(current['power'],2*before+1);self.assertEqual(current['toughness'],2*before+4);doubled+=1
                    if doubled>=40:
                        self.assertGreater(current['counters']['plus_one'],2**40);self.assertEqual(after['schema'],42);obs=after;break
                obs=after
            else:self.fail('forty legal self-targeted doublings not reached')
            for _ in range(100):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='damage':
                    bounds=obs['damage_assignment'];self.assertEqual(bounds['minimum'],0);self.assertGreater(bounds['maximum'],2**40);self.assertLessEqual(len(obs['actions']),4)
                    opponent=game.observe('P1');self.assertIsNone(opponent['damage_assignment']);self.assertEqual(opponent['actions'],[])
                    blocked={o['id'] for o in obs['objects'] if o['zone']=='Battlefield' and o['card']=='Bear Cub' and o['owner']=='P1'};self.assertEqual(len(blocked),2)
                    amount=bounds['maximum']//2+123;action={'kind':'AssignDamage','target':bounds['target'],'amount':amount}
                    self.assertNotIn(action,obs['actions']);saved=game.snapshot();actor=obs['actor']
                    with game.clone() as fork:expected=fork.act(fork.observe(actor),action)
                    after=game.act(obs,action);self.assertEqual(after,expected);game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.act(obs,action)
                    after=game.act(game.observe(actor),action);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved)
                    current=next(o for o in after['objects'] if o['reference']==bounds['source']);self.assertEqual(current['zone'],'Battlefield');self.assertEqual(current['damage'],4)
                    self.assertEqual(sum(o['id'] in blocked and o['zone']=='Graveyard' for o in after['objects']),2);return
                if obs['decision']=='attackers':
                    index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='Attack' and obs['actor']=='P0'),None)
                    if index is None:index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='FinishAttackers')
                elif obs['decision']=='blockers':
                    index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='Block' and obs['actor']=='P1'),None)
                    if index is None:index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='FinishBlockers')
                else:
                    candidates=[i for i,a in enumerate(obs['actions']) if a['kind'] in ('Pass','FinishTargets','FinishAttackers','FinishBlockers','Discard')]
                    self.assertTrue(candidates,str((obs['decision'],obs['step'],obs['actor'],[a['kind'] for a in obs['actions']])))
                    index=candidates[0]
                obs=game.step(obs,index)
            self.fail('huge double-blocked damage range not reached')

    def removal_index(self, obs):
        objects={o['id']:o for o in obs['objects']}
        def score(a):
            k=a['kind']
            if k=='ChooseTarget':
                source=obs['announcing'];beneficial=(source['source_card'],source['mode']) in [('Battle Menu',1),("Sarkhan's Resolve",0),('Valorous Stance',0)]
                target=a['target'];own=objects[target['Card']]['owner']==obs['actor'] if 'Card' in target else target.get('Player')==obs['actor']
                return 80 if own==beneficial else -10
            if k=='TapMana':return 70 if obs['decision']=='pay_costs' else -10
            if k=='AssignDamage':return 80+a['amount']
            if k=='ArrangeLibrary':return 60 if a['destination']=='Top' else 50
            return {'Keep':100,'PlayLand':90,'AnnounceCast':85,'Cast':85,'BeginCast':85,
                    'ChooseMode':80,'ChooseAdditionalCost':80,'PayCosts':100,'PutTrigger':80,
                    'FinishTargets':70,'ChooseOptional':60,'RecoverMilled':60,
                    'FinishAttackers':40,'FinishBlockers':40,'Pass':20,
                    'Concede':-1000,'Mulligan':-100}.get(k,0)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_deathmark_targets_white_hybrid_and_native_resolution_replays(self):
        decks=[['Swamp']*24+['Deathmark']*36,['Plains']*24+['Cat-Owl']*36]
        with Game(decks,seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(5000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['stack'] and obs['stack'][-1]['source_card']=='Deathmark' and obs['decision']=='priority':break
                obs=game.step(obs,self.removal_index(obs))
            else:self.fail('no hybrid-color removal reached')
            item=obs['stack'][-1];target=item['targets'][0]['object'];actor=obs['actor']
            self.assertEqual(next(o['card'] for o in obs['objects'] if o['reference']==target),'Cat-Owl')
            saved=game.snapshot()
            with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
            actual=self.resolve_vital_top(game,actor);self.assertEqual(actual,expected)
            self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==target['card']),'Graveyard')
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass'))
            actual=self.resolve_vital_top(game,actor);actual.pop('revision');expected.pop('revision');self.assertEqual(actual,expected);game.drop_snapshot(saved)

    def test_rooftop_assassin_targets_real_combat_damage_and_replays_entry_trigger(self):
        decks=[['Swamp']*24+['Rooftop Assassin']*36,['Forest']*24+['Magnigoth Sentry']*36]
        with Game(decks,seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(8000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                objects={o['id']:o for o in obs['objects']}
                if obs['decision']=='targets' and obs['announcing']['source_card']=='Rooftop Assassin':break
                index=self.removal_index(obs)
                if obs['decision']=='attackers':
                    index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='Attack' and obs['actor']=='P0'),None)
                    if index is None:index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='FinishAttackers')
                elif obs['decision']=='blockers':
                    index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='Block' and obs['actor']=='P1' and not obs['blocks']),None)
                    if index is None:index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='FinishBlockers')
                elif obs['decision']=='priority':
                    legal=[]
                    for i,a in enumerate(obs['actions']):
                        if a['kind'] in ('AnnounceCast','Cast','BeginCast'):
                            name=objects[a['card']]['card']
                            own=[o for o in obs['objects'] if o['owner']==obs['actor'] and o['zone']=='Battlefield' and o['card']==name]
                            damaged=any(o['owner']=='P1' and o['zone']=='Battlefield' and o['damage']>0 for o in obs['objects'])
                            allowed=(obs['actor']=='P1' and obs['active']=='P1' and obs['step'] in ('Main1','Main2') and not own) or (obs['actor']=='P0' and obs['active']=='P0' and ((obs['step']=='Main1' and not own) or (obs['step']=='Main2' and damaged)))
                            if not allowed:continue
                        legal.append(i)
                    if index not in legal:
                        index=next(i for i in legal if obs['actions'][i]['kind']=='Pass')
                obs=game.step(obs,index)
            else:self.fail('no damaged-creature entry target reached')
            actions=[a for a in obs['actions'] if a['kind']=='ChooseTarget'];self.assertTrue(actions)
            for a in actions:
                target=objects[a['target']['Card']];self.assertEqual(target['owner'],'P1');self.assertGreater(target['damage'],0);self.assertTrue(target['damaged_this_turn'])
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseTarget');victim=obs['actions'][index]['target']['Card'];actor=obs['actor'];saved=game.snapshot()
            def resolve(handle):
                handle.step(handle.observe(actor),index);return self.resolve_vital_top(handle,actor)
            with game.clone() as fork:expected=resolve(fork)
            actual=resolve(game);self.assertEqual(actual,expected);self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==victim),'Graveyard')
            game.restore(saved);actual=resolve(game);actual.pop('revision');expected.pop('revision');self.assertEqual(actual,expected);game.drop_snapshot(saved)

    def test_red_guardian_uses_real_dealer_history_not_recipient_damage(self):
        decks=[['Plains']*40+['Red Guardian, Super-Soldier']*20,['Forest']*40+['Bear Cub']*20]
        with Game(decks,seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(5000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']}
                if obs['decision']=='targets' and obs['announcing']['source_card']=='Red Guardian, Super-Soldier':break
                index=self.removal_index(obs)
                if obs['decision']=='attackers':
                    index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='Attack' and obs['actor']=='P1'),None)
                    if index is None:index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='FinishAttackers')
                elif obs['decision']=='priority' and obs['actions'][index]['kind'] in ('AnnounceCast','Cast','BeginCast'):
                    source=objects[obs['actions'][index]['card']]['card']
                    if source=='Red Guardian, Super-Soldier' and not any(o['owner']=='P1' and o['zone']=='Battlefield' and o['dealt_damage_this_turn'] for o in obs['objects']):
                        index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass')
                obs=game.step(obs,index)
            else:self.fail('no dealer-history target reached')
            choices=[a for a in obs['actions'] if a['kind']=='ChooseTarget'];self.assertTrue(choices)
            self.assertLess(obs['players'][0]['life'],20)
            for a in choices:
                obj=objects[a['target']['Card']];self.assertTrue(obj['dealt_damage_this_turn']);self.assertFalse(obj['damaged_this_turn']);self.assertEqual(obj['damage'],0)
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseTarget');target=obs['actions'][index]['target']['Card'];actor=obs['actor'];saved=game.snapshot()
            def resolve(handle):
                handle.step(handle.observe(actor),index);return self.resolve_vital_top(handle,actor)
            with game.clone() as fork:expected=resolve(fork)
            actual=resolve(game);self.assertEqual(actual,expected);self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==target),'Graveyard')
            game.restore(saved);actual=resolve(game);actual.pop('revision');expected.pop('revision');self.assertEqual(actual,expected);game.drop_snapshot(saved)

    def test_fumigate_counts_real_destroyed_creatures_and_replays_native_batch(self):
        decks=[['Plains']*36+['Fumigate']*24,['Forest']*36+['Bear Cub']*24]
        with Game(decks,seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(8000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Fumigate':break
                index=self.removal_index(obs)
                if obs['actions'][index]['kind'] in ('AnnounceCast','Cast','BeginCast') and obs['actor']=='P0' and sum(o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Bear Cub' for o in obs['objects'])<2:
                    index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass')
                obs=game.step(obs,index)
            else:self.fail('no group destruction reached')
            victims=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['card']=='Bear Cub'];self.assertGreaterEqual(len(victims),2)
            life=obs['players'][0]['life'];events=obs['players'][0]['life_gain_events_this_turn'];actor=obs['actor'];saved=game.snapshot()
            with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
            after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected);self.assertEqual(after['schema'],42)
            self.assertEqual(after['players'][0]['life'],life+len(victims));self.assertEqual(after['players'][0]['life_gain_events_this_turn'],events+1)
            for old in victims:
                current=next(o for o in after['objects'] if o['id']==old['id']);self.assertEqual(current['zone'],'Graveyard');self.assertNotEqual(current['reference'],old['reference'])
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass'))
            after=self.resolve_vital_top(game,actor);after.pop('revision');expected.pop('revision');self.assertEqual(after,expected);game.drop_snapshot(saved)

    def test_tectonic_hazard_hits_opponent_and_their_creatures_in_one_native_resolution(self):
        decks=[['Mountain']*24+['Tectonic Hazard']*36,['Forest']*24+['Llanowar Elves']*36]
        with Game(decks,seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(6000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Tectonic Hazard':break
                index=self.removal_index(obs)
                if obs['actions'][index]['kind'] in ('AnnounceCast','Cast','BeginCast') and obs['actor']=='P0' and not any(o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Llanowar Elves' for o in obs['objects']):
                    index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass')
                obs=game.step(obs,index)
            else:self.fail('no group damage reached')
            victims=[o['id'] for o in obs['objects'] if o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Llanowar Elves'];self.assertTrue(victims)
            before=[p['life'] for p in obs['players']];actor=obs['actor'];saved=game.snapshot()
            with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
            actual=self.resolve_vital_top(game,actor);self.assertEqual(actual,expected);self.assertEqual([p['life'] for p in actual['players']],[before[0],before[1]-1])
            for ident in victims:self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==ident),'Graveyard')
            game.restore(saved);actual=self.resolve_vital_top(game,actor);actual.pop('revision');expected.pop('revision');self.assertEqual(actual,expected);game.drop_snapshot(saved)

    def test_dragonback_assault_resolves_entry_damage_and_real_landfall_token(self):
        decks=[['Mountain']*12+['Forest']*12+['Island']*12+['Dragonback Assault']*24,['Forest']*36+['Bear Cub']*24]
        with Game(decks,seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(10000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Dragonback Assault' and 'TriggeredAbility' in obs['stack'][-1]['kind']:break
                index=self.removal_index(obs)
                if obs['actions'][index]['kind'] in ('AnnounceCast','Cast','BeginCast') and obs['actor']=='P0' and not any(o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Bear Cub' for o in obs['objects']):
                    index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass')
                obs=game.step(obs,index)
            else:self.fail('no Dragonback entry damage reached')
            victims=[o['id'] for o in obs['objects'] if o['zone']=='Battlefield' and o['card']=='Bear Cub'];self.assertTrue(victims);actor=obs['actor'];saved=game.snapshot()
            with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
            after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected);self.assertEqual(after['schema'],42)
            for ident in victims:self.assertEqual(next(o['zone'] for o in after['objects'] if o['id']==ident),'Graveyard')
            game.restore(saved);after=self.resolve_vital_top(game,actor);after.pop('revision');expected.pop('revision');self.assertEqual(after,expected);game.drop_snapshot(saved)
            for _ in range(4000):
                obs=game.observe(game.observe('P0')['actor']);self.assertIsNone(obs['outcome'])
                index=self.removal_index(obs);action=obs['actions'][index];after=game.step(obs,index)
                if action['kind']=='PlayLand' and obs['actor']=='P0':
                    obs=game.observe(after['actor'])
                    while obs['decision']=='order_triggers':
                        order=next(i for i,a in enumerate(obs['actions']) if a['kind']=='PutTrigger');after=game.step(obs,order);obs=game.observe(after['actor'])
                    self.assertTrue(obs['stack']);self.assertEqual(obs['stack'][-1]['source_card'],'Dragonback Assault');self.assertIn('TriggeredAbility',obs['stack'][-1]['kind']);break
            else:self.fail('no landfall trigger reached')
            actor=obs['actor'];before=sum(o['zone']=='Battlefield' and o['token'] is not None and o['token']['subtypes']=='Dragon' for o in obs['objects']);saved=game.snapshot()
            with game.clone() as fork:expected=self.resolve_vital_top(fork,actor)
            after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected)
            dragons=[o for o in after['objects'] if o['zone']=='Battlefield' and o['token'] and o['token']['subtypes']=='Dragon'];self.assertEqual(len(dragons),before+1)
            self.assertTrue(all(o['power']==4 and o['toughness']==4 and o['keywords']&1 for o in dragons));game.drop_snapshot(saved)

    def test_broadside_barrage_keeps_lethal_damage_pending_until_discard_choice_finishes(self):
        decks=[['Mountain']*18+['Island']*18+['Broadside Barrage']*24,['Forest']*36+['Bear Cub']*24]
        with Game(decks,seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(8000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='effect_discard' and obs['resolving']['source_card']=='Broadside Barrage':break
                obs=game.step(obs,self.removal_index(obs))
            else:self.fail('no damage/draw/discard continuation reached')
            target=obs['resolving']['targets'][0]['object'];victim=next(o for o in obs['objects'] if o['reference']==target);self.assertEqual(victim['zone'],'Battlefield');self.assertEqual(victim['damage'],5);self.assertTrue(victim['damaged_this_turn'])
            actor=obs['actor'];index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Discard');saved=game.snapshot()
            with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected);self.assertEqual(next(o['zone'] for o in actual['objects'] if o['id']==target['card']),'Graveyard')
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            actual=game.step(game.observe(actor),index);actual.pop('revision');expected.pop('revision');self.assertEqual(actual,expected);game.drop_snapshot(saved)

    def test_mass_negative_counters_resolve_natively_and_replay(self):
        with Game([['Swamp']*24+['Darkness Descends']*36,['Forest']*24+['Bear Cub']*36],seed=6,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Darkness Descends' and any(o['zone']=='Battlefield' and o['card']=='Bear Cub' for o in obs['objects']):
                    saved=game.snapshot();actor=obs['actor'];index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass')
                    with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                    self.assertEqual(game.step(obs,index),expected);game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                    game.drop_snapshot(saved);after=game.step(game.observe(actor),index);after=game.observe(after['actor'])
                    if after['stack'] and after['stack'][-1]['source_card']=='Darkness Descends':after=game.step(after,next(i for i,a in enumerate(after['actions']) if a['kind']=='Pass'))
                    self.assertTrue(any(o['card']=='Bear Cub' and o['zone']=='Graveyard' for o in after['objects']));self.assertFalse(any(o['card']=='Bear Cub' and o['zone']=='Battlefield' for o in after['objects']));return
                obs=game.step(obs,self.discard_index(obs))
            self.fail('mass counter resolution not reached')

    def test_zimone_counter_doubling_uses_owned_native_targets_and_snapshots(self):
        deck=['Forest']*10+['Island']*10+['Breeding Pool']*4+['Zimone, Paradox Sculptor']*18+['Earth Kingdom Soldier']*18
        with Game([deck,['Forest']*60],seed=6,deck_rules='Freeform') as game:
            obs=game.observe('P0');activated=False
            for _ in range(3000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and isinstance(obs['stack'][-1]['kind'],dict) and 'ActivatedAbility' in obs['stack'][-1]['kind'] and obs['stack'][-1]['source_card']=='Zimone, Paradox Sculptor':
                    item=obs['stack'][-1];refs=[t['object'] for t in item['targets']];old=[o for o in obs['objects'] if o['reference'] in refs]
                    if old and any(o['counters']['plus_one'] for o in old):
                        saved=game.snapshot();actor=obs['actor'];index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Pass')
                        with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                        after=game.step(obs,index);self.assertEqual(after,expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                        game.drop_snapshot(saved);after=game.step(game.observe(actor),index);after=game.observe(after['actor'])
                        if any(s['id']==item['id'] for s in after['stack']):after=game.step(after,next(i for i,a in enumerate(after['actions']) if a['kind']=='Pass'))
                        for source in old:
                            current=next(o for o in after['objects'] if o['reference']==source['reference'])
                            self.assertEqual(current['counters']['plus_one'],2*source['counters']['plus_one'])
                        return
                index=None
                if obs['actor']=='P0' and obs['decision']=='priority' and not obs['stack']:
                    sources={o['id'] for o in obs['objects'] if o['zone']=='Battlefield' and o['card']=='Zimone, Paradox Sculptor' and o['counters']['plus_one']>0}
                    index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='AnnounceActivate' and a['card'] in sources),None)
                    activated=activated or index is not None
                if obs['decision']=='targets' and activated:
                    candidates={o['id']:o['counters']['plus_one'] for o in obs['objects']}
                    target_actions=[(i,a) for i,a in enumerate(obs['actions']) if a['kind']=='ChooseTarget']
                    if target_actions:index=max(target_actions,key=lambda pair:candidates.get(pair[1]['target'].get('Card'),0))[0]
                obs=game.step(obs,self.discard_index(obs) if index is None else index)
            self.fail('counter doubling not reached')

    def test_mandatory_cost_selection_is_private_and_native_replayable(self):
        for spell,deck in [('Thrill of Possibility',['Mountain']*24+['Thrill of Possibility']*36),
                           ('Corrupted Conviction',['Swamp']*20+['Savannah Lions']*20+['Plains']*10+['Corrupted Conviction']*10)]:
            with Game([deck,['Forest']*60],seed=6,deck_rules='Freeform') as game:
                obs=game.observe('P0')
                for _ in range(2500):
                    self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='additional_cost':
                        view=obs['additional_cost'];self.assertEqual(obs['schema'],42)
                        self.assertTrue(view['candidates']);self.assertIsNone(obs['payment_request'])
                        opponent=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(opponent['additional_cost'])
                        index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseAdditionalCost')
                        selected=obs['actions'][index]['object'];saved=game.snapshot();actor=obs['actor']
                        with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                        after=game.step(obs,index);self.assertEqual(after,expected)
                        self.assertEqual(after['additional_cost']['selected'],selected);self.assertEqual(after['decision'],'pay_costs')
                        game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                        game.drop_snapshot(saved)
                        after=game.step(game.observe(actor),index)
                        for _ in range(10):
                            if after['decision']!='pay_costs':break
                            after=game.step(after,self.discard_index(after))
                        self.assertNotEqual(after['decision'],'pay_costs')
                        public=game.observe(actor)
                        self.assertTrue(any(o['reference']['card']==selected['card'] and o['zone']=='Graveyard' for o in public['objects']))
                        break
                    obs=game.step(obs,self.discard_index(obs))
                else:self.fail('additional cost not reached: '+spell)

    def test_dynamic_creature_count_stats_are_native_and_snapshot_owned(self):
        deck=['Plains']*12+['Forest']*12+['Burrowguard Mentor']*12+['Crusader of Odric']*12+['Savannah Lions']*12
        with Game([deck,['Forest']*60],seed=6,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(1800):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                own=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['owner']=='P0' and o['power']!=0]
                source=next((o for o in own if o['card'] in ('Burrowguard Mentor','Crusader of Odric')),None)
                if source and len(own)>=2:
                    self.assertEqual((source['power'],source['toughness']),(len(own),len(own)))
                    saved=game.snapshot();actor=obs['actor'];index=self.discard_index(obs)
                    with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                    self.assertEqual(game.step(obs,index),expected);game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                    game.drop_snapshot(saved);return
                obs=game.step(obs,self.discard_index(obs))
            self.fail('scaled creature board not reached')

    def test_tarmogoyf_power_and_toughness_follow_public_graveyard_card_types(self):
        deck=['Forest']*12+['Mountain']*12+['Tarmogoyf']*12+['Shock']*12+['Giant Growth']*12
        with Game([deck,['Forest']*60],seed=8,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                goyfs=[o for o in obs['objects'] if o['card']=='Tarmogoyf' and o['zone']=='Battlefield']
                # This fixture uses only instants, lands and Tarmogoyfs. Avoid
                # temporary growth while checking its unmodified CDA value.
                types=set()
                for o in obs['objects']:
                    if o['zone']=='Graveyard':
                        types.add('Creature' if o['card']=='Tarmogoyf' else 'Land' if o['card'] in ('Forest','Mountain') else 'Instant')
                if goyfs and types:
                    source=goyfs[0]
                    if source['power']==len(types) and source['toughness']==len(types)+1:
                        other=game.observe('P1')
                        self.assertEqual(next(o for o in other['objects'] if o['reference']==source['reference'])['power'],len(types));return
                obs=game.step(obs,self.discard_index(obs))
            self.fail('public graveyard-scaled Tarmogoyf not reached')

    def test_verge_secondary_color_requires_a_live_owned_land_type(self):
        deck=['Blazemire Verge']*16+['Swamp']*8+['Shock']*36
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0');seen=set()
            for _ in range(1500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['actor']=='P0' and obs['decision']=='priority':
                    verges=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Blazemire Verge' and not o['tapped']]
                    enabled=any(o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Swamp' for o in obs['objects'])
                    for verge in verges:
                        secondary=[i for i,a in enumerate(obs['actions']) if a['kind']=='ActivateMana' and a['card']==verge['id'] and a['ability']==1]
                        self.assertEqual(bool(secondary),enabled);seen.add(enabled)
                        if enabled and len(seen)==2:
                            index=secondary[0];saved=game.snapshot();actor=obs['actor']
                            with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
                            actual=game.step(obs,index);self.assertEqual(actual,expected)
                            self.assertGreaterEqual(actual['players'][0]['mana'][3],1)
                            game.restore(saved)
                            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                            game.drop_snapshot(saved);return
                obs=game.step(obs,self.discard_index(obs))
            self.fail('both disabled and enabled Verge states not reached')

    def test_jasper_native_actions_enable_after_changeling_entry(self):
        deck=['Forest']*24+['Mox Jasper']*18+['Three Tree Mascot']*18
        with Game([deck,['Forest']*60],seed=10,deck_rules='Freeform') as game:
            obs=game.observe('P0');disabled=False
            for _ in range(1500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['actor']=='P0' and obs['decision']=='priority':
                    moxes=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Mox Jasper' and not o['tapped']]
                    dragon=any(o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Three Tree Mascot' for o in obs['objects'])
                    for mox in moxes:
                        actions=[a for a in obs['actions'] if a['kind']=='ActivateMana' and a['card']==mox['id']]
                        self.assertEqual(len(actions),5 if dragon else 0)
                        if not dragon:disabled=True
                        elif disabled:
                            self.assertEqual({tuple(a['production']) for a in actions},{tuple(1 if j==i else 0 for j in range(6)) for i in range(5)})
                            return
                obs=game.step(obs,self.discard_index(obs))
            self.fail('Jasper guard transition not reached')

    def test_shock_land_pre_entry_branches_are_native_owned_and_public(self):
        deck=['Hallowed Fountain']*24+['Opt']*36
        with Game([deck,['Forest']*60],seed=4,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(100):
                obs=game.observe(obs['actor'])
                if obs['decision']=='entry_life':break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('shock entry choice not reached')
            self.assertEqual(obs['schema'],42)
            choice=obs['entry_life'];self.assertEqual(choice['cost'],2)
            self.assertFalse(choice['already_tapped']);self.assertEqual(choice['card'],'Hallowed Fountain')
            actor=obs['actor'];other='P1' if actor=='P0' else 'P0';ident=choice['object']['card']
            opponent=game.observe(other)
            self.assertEqual(opponent['entry_life'],choice);self.assertEqual(opponent['actions'],[])
            self.assertTrue(any(o['id']==ident and o['zone']=='Hand' for o in opponent['objects']))
            saved=game.snapshot();life=obs['players'][0]['life']
            indices={a['pay']:i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseEntryLife'}
            self.assertEqual(set(indices),{False,True})
            for pay in [True,False]:
                with game.clone() as fork:
                    branch=fork.step(fork.observe(actor),indices[pay])
                    card=next(o for o in branch['objects'] if o['id']==ident)
                    self.assertEqual(card['zone'],'Battlefield');self.assertEqual(card['tapped'],not pay)
                    self.assertEqual(branch['players'][0]['life'],life-(2 if pay else 0))
                    self.assertIsNone(branch['entry_life']);self.assertEqual(branch['known_hand'],[])
            actual=game.step(obs,indices[True]);game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,indices[True])
            with game.clone() as fork:expected=fork.step(fork.observe(actor),indices[True])
            self.assertEqual(actual['objects'],expected['objects']);game.drop_snapshot(saved)

    def test_declined_shock_entry_has_no_mana_actions_until_untap(self):
        deck=['Watery Grave']*24+['Opt']*36
        with Game([deck,['Forest']*60],seed=5,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(100):
                obs=game.observe(obs['actor'])
                if obs['decision']=='entry_life':break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('shock entry choice not reached')
            ident=obs['entry_life']['object']['card'];turn=obs['turn']
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseEntryLife' and not a['pay'])
            obs=game.step(obs,index)
            for _ in range(400):
                obs=game.observe(obs['actor'])
                card=next(o for o in obs['objects'] if o['id']==ident)
                if obs['turn']>turn and obs['active']=='P0':break
                self.assertTrue(card['tapped'])
                self.assertFalse(any(a.get('card')==ident and a['kind'] in ('TapMana','ActivateMana') for a in obs['actions']))
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('next own untap not reached')
            self.assertFalse(card['tapped'])
            colors=[a['production'] for a in obs['actions'] if a.get('card')==ident and a['kind']=='ActivateMana']
            self.assertEqual(sorted(colors),sorted([[0,1,0,0,0,0],[0,0,1,0,0,0]]))

    def test_continuous_haste_is_derived_in_native_library_and_replays(self):
        deck=['Mountain']*12+['Forest']*12+["Samut, Hazoret's Champion"]*12+['Llanowar Elves']*24
        with Game([deck,['Forest']*60],seed=9,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                elves=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['card']=='Llanowar Elves' and o['summoning_sick']]
                samut=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['card']=="Samut, Hazoret's Champion"]
                if elves and samut and any(a['kind'] in ('TapMana','ActivateMana') and a['card']==elves[0]['id'] for a in obs['actions']):break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('newly entered mana Elf with continuous haste not reached')
            elf=elves[0];self.assertTrue(elf['keywords']&4)
            index=next(i for i,a in enumerate(obs['actions']) if a['kind'] in ('TapMana','ActivateMana') and a['card']==elf['id'])
            saved=game.snapshot()
            with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected)
            self.assertTrue(next(o for o in actual['objects'] if o['id']==elf['id'])['tapped'])
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            game.drop_snapshot(saved)

    def test_own_turn_first_strike_is_not_written_into_base_state(self):
        deck=['Mountain']*24+['Skilled Battlecarver']*36
        with Game([deck,['Forest']*60],seed=3,deck_rules='Freeform') as game:
            obs=game.observe('P0');reference=None;seen=set()
            for _ in range(2500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                carvers=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Skilled Battlecarver']
                if carvers:
                    if reference is None:reference=carvers[0]['reference']
                    card=next(o for o in carvers if o['reference']==reference)
                    self.assertEqual(bool(card['keywords']&512),obs['active']=='P0')
                    seen.add(obs['active'])
                    if len(seen)==2:break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('both turn scopes not reached')

    def test_changeling_entry_matches_angel_observer_in_native_library_with_replay(self):
        deck=['Plains']*24+['Youthful Valkyrie']*12+['Prideful Feastling']*12+['Mary Jane Watson']*12
        with Game([deck,['Forest']*60],seed=8,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(3000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                changelings=[o for o in obs['objects'] if o['zone']=='Battlefield' and o['owner']=='P0' and o['card']=='Prideful Feastling']
                triggers=obs['stack']+obs['waiting_triggers']+([obs['announcing']] if obs['announcing'] else [])
                angel=next((t for t in triggers if t['source_card']=='Youthful Valkyrie'),None)
                if changelings and angel:break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('changeling Angel-entry trigger not reached')
            self.assertTrue(all(o['keywords']&524288 for o in changelings))
            source=angel['object'];before=next(o['counters']['plus_one'] for o in obs['objects'] if o['reference']==source)
            saved=game.snapshot();index=self.discard_index(obs)
            with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected);game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            obs=game.observe(obs['actor'])
            for _ in range(100):
                current=next(o for o in obs['objects'] if o['reference']==source)
                if current['counters']['plus_one']>before:break
                obs=game.observe(obs['actor']);obs=game.step(obs,self.discard_index(obs))
            else:self.fail('Angel-entry counter did not resolve')
            self.assertEqual(current['counters']['plus_one'],before+1);game.drop_snapshot(saved)

    def test_changeling_wayfinder_optional_search_is_private_and_branch_owned(self):
        deck=['Forest']*28+['Changeling Wayfinder']*32
        with Game([deck,['Forest']*60],seed=6,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='optional_effect' and obs['resolving']['source_card']=='Changeling Wayfinder':break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('Wayfinder optional search not reached')
            saved=game.snapshot();actor=obs['actor']
            decline=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseOptional' and not a['accept'])
            with game.clone() as fork:self.assertIsNone(fork.step(fork.observe(actor),decline)['search_choice'])
            accept=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseOptional' and a['accept'])
            obs=game.step(obs,accept);self.assertEqual(obs['search_choice']['filter'],'BasicLand')
            hidden=game.observe('P1');self.assertIsNone(hidden['search_choice']);self.assertEqual(hidden['actions'],[])
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseSearch')
            card=obs['actions'][index]['card'];obs=game.step(obs,index)
            index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='FinishSearch')
            with game.clone() as fork:expected=fork.step(fork.observe(actor),index)
            self.assertEqual(game.step(obs,index),expected)
            self.assertTrue(any(o['id']==card and o['zone']=='Hand' and o['card']=='Forest' for o in expected['objects']))
            game.restore(saved);self.assertEqual(game.observe(actor)['decision'],'optional_effect');game.drop_snapshot(saved)

    def test_hybrid_cost_choices_are_committed_before_payment_and_snapshot_owned(self):
        deck=['Plains']*14+['Forest']*14+['Gallant Citizen']*32
        with Game([deck,['Forest']*60],seed=5,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(4000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='spell_cost' and len(obs['cost_choices'])==3:break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('three payable hybrid equivalents not reached')
            self.assertEqual(obs['schema'],42);self.assertIsNone(obs['announcing_cost']);self.assertIsNone(obs['payment_request'])
            self.assertTrue(all(a['kind'] in ['ChooseSpellCost','Concede'] for a in obs['actions']))
            other=game.observe('P1');self.assertEqual(other['cost_choices'],[]);self.assertEqual(other['actions'],[])
            saved=game.snapshot()
            for choice in obs['cost_choices']:
                current=game.observe('P0');index=next(i for i,a in enumerate(current['actions']) if a['kind']=='ChooseSpellCost' and a['cost']==choice['index'])
                with game.clone() as fork:expected=fork.step(fork.observe('P0'),index)
                actual=game.step(current,index);self.assertEqual(actual,expected)
                self.assertEqual(actual['decision'],'pay_costs');self.assertEqual(actual['announcing_cost'],choice['cost'])
                self.assertEqual(actual['payment_request']['required'],choice['cost']['colored'])
                self.assertEqual(game.observe('P1')['announcing_cost'],choice['cost'])
                for _ in range(100):
                    if any(a['kind']=='PayCosts' for a in actual['actions']):break
                    actual=game.step(actual,self.discard_index(actual))
                else:self.fail('chosen equivalent cannot be funded')
                payments=[a['payment'] for a in actual['actions'] if a['kind']=='PayCosts']
                self.assertEqual(payments,[choice['cost']['colored']])
                index=next(i for i,a in enumerate(actual['actions']) if a['kind']=='PayCosts');actual=game.step(actual,index)
                self.assertIsNone(actual['announcing']);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(current,0)
            game.drop_snapshot(saved)

    def test_hybrid_mono_green_casts_use_native_library(self):
        deck=['Forest']*28+['Gallant Citizen']*16+['Pond Prophet']*16
        with Game([deck,['Forest']*60],seed=4,deck_rules='Freeform') as game:
            obs=game.observe('P0');seen=False
            for _ in range(2000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='spell_cost':
                    self.assertEqual(len(obs['cost_choices']),1);self.assertEqual(obs['cost_choices'][0]['cost']['colored'],[0,0,0,0,2,0]);seen=True
                if seen and any(o['zone']=='Battlefield' and o['card'] in ['Gallant Citizen','Pond Prophet'] for o in obs['objects']):break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('green-only hybrid cast did not complete')
            self.assertGreater(obs['players'][0]['spells_cast_this_turn'],0)

    def test_once_per_turn_public_ledger_is_owned_and_native_replayable(self):
        deck=['Island']*14+['Swamp']*14+['Flying Octobot']*16+['Common Crook']*16
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['triggered_this_turn']:break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('limited entry trigger not reached')
            self.assertEqual(obs['schema'],42)
            used=obs['triggered_this_turn'];self.assertTrue(all(u['ability']==0 for u in used))
            self.assertEqual(game.observe('P0')['triggered_this_turn'],game.observe('P1')['triggered_this_turn'])
            other=game.observe('P1');self.assertTrue(all(o['zone']!='Hand' or o['owner']!='P0' for o in other['objects']))
            index=self.discard_index(obs);saved=game.snapshot()
            with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected)
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            restored=game.observe(obs['actor']);self.assertEqual(restored['triggered_this_turn'],used)
            replay=game.step(restored,index);replay.pop('revision');actual.pop('revision');self.assertEqual(replay,actual)
            game.drop_snapshot(saved)

    def test_archivist_independent_limits_survive_python_snapshot_and_turn_reset(self):
        deck=['Forest']*12+['Plains']*12+['Island']*12+['Elvish Archivist']*12+['Flying Octobot']*12+['Fear of Surveillance']*12
        with Game([deck,['Forest']*72],seed=3,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(4000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                objects={o['id']:o for o in obs['objects']}
                groups={}
                for u in obs['triggered_this_turn']:
                    if objects.get(u['source']['card'],{}).get('card')=='Elvish Archivist':
                        groups.setdefault((u['source']['card'],u['source']['generation']),set()).add(u['ability'])
                if any(abilities=={0,1} for abilities in groups.values()):break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('both Archivist limits in one turn not reached')
            saved=game.snapshot();used=obs['triggered_this_turn'];turn=obs['turn']
            for _ in range(1000):
                obs=game.observe(obs['actor']);obs=game.step(obs,self.discard_index(obs))
                if obs['turn']!=turn:break
            else:self.fail('next turn not reached')
            self.assertEqual(obs['triggered_this_turn'],[])
            game.restore(saved);self.assertEqual(game.observe('P0')['triggered_this_turn'],used)
            game.drop_snapshot(saved)

    def test_connive_private_discard_branches_and_exact_source_replay(self):
        deck=['Swamp']*28+['Red Room Recruit']*32
        with Game([deck,deck],seed=7,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(1500):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']}
                offered=[a for a in obs['actions'] if a['kind']=='Discard']
                if obs['connive_choice'] is not None and any(objects[a['card']]['card']=='Swamp' for a in offered) and any(objects[a['card']]['card']=='Red Room Recruit' for a in offered):break
                obs=game.step(obs,self.discard_index(obs))
            else:self.fail('mixed-hand connive not reached')
            actor=obs['actor'];other='P1' if actor=='P0' else 'P0';source=obs['connive_choice']['current']['object'];source_id=source['card']
            self.assertEqual(obs['schema'],42);self.assertEqual(obs['connive_choice']['amount'],1);self.assertEqual(obs['connive_choice']['current']['controller'],actor)
            hidden=game.observe(other);self.assertEqual(hidden['connive_choice'],obs['connive_choice']);self.assertEqual(hidden['actions'],[])
            self.assertEqual(hidden['effect_discards'],[]);self.assertTrue(all(o['zone']!='Hand' or o['owner']!=actor for o in hidden['objects']))
            saved=game.snapshot();before=objects[source_id]['counters']['plus_one']
            for name,added in [('Red Room Recruit',1),('Swamp',0)]:
                index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Discard' and objects[a['card']]['card']==name)
                with game.clone() as branch:expected=branch.step(branch.observe(actor),index)
                actual=game.step(game.observe(actor),index);self.assertEqual(actual,expected)
                current=next(o for o in actual['objects'] if o['id']==source_id);self.assertEqual(current['reference'],source)
                self.assertEqual(current['counters']['plus_one'],before+added);self.assertIsNone(actual['connive_choice']);self.assertIsNone(actual['resolving'])
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
            game.drop_snapshot(saved)

    def test_targeted_connive_preserves_two_target_groups_and_native_continuation(self):
        deck=['Island']*16+['Swamp']*12+['Red Room Recruit']*16+['Unstable Experiment']*16
        with Game([deck,['Forest']*60],seed=9,deck_rules='Freeform') as game:
            obs=game.observe('P0')
            for _ in range(2000):
                self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['connive_choice'] is not None and obs['resolving']['source_card']=='Unstable Experiment':break
                index=self.discard_index(obs)
                if obs['announcing'] is not None and obs['announcing']['source_card']=='Unstable Experiment':
                    if obs['targeting_group']==0:
                        index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseTarget' and a['target']=={'Player':'P1'})
                    elif obs['targeting_group']==1:
                        index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseTarget'),index)
                obs=game.step(obs,index)
            else:self.fail('targeted connive not reached')
            self.assertEqual(obs['actor'],'P0');self.assertGreaterEqual(obs['players'][1]['cards_drawn_this_turn'],1)
            targets=obs['resolving']['targets'];self.assertTrue(any(t['group']==0 and t['target']=={'Player':'P1'} for t in targets))
            current=obs['connive_choice']['current'];self.assertEqual(current['controller'],'P0')
            self.assertTrue(any(t['group']==1 and t['object']==current['object'] for t in targets))
            objects={o['id']:o for o in obs['objects']};source=current['object']['card'];before=objects[source]['counters']['plus_one']
            index=next((i for i,a in enumerate(obs['actions']) if a['kind']=='Discard' and objects[a['card']]['card'] in ('Red Room Recruit','Unstable Experiment')),None)
            if index is None:index=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Discard')
            added=int(objects[obs['actions'][index]['card']]['card'] in ('Red Room Recruit','Unstable Experiment'));saved=game.snapshot()
            with game.clone() as branch:expected=branch.step(branch.observe('P0'),index)
            actual=game.step(obs,index);self.assertEqual(actual,expected);self.assertIsNone(actual['connive_choice'])
            self.assertEqual(next(o for o in actual['objects'] if o['id']==source)['counters']['plus_one'],before+added)
            game.restore(saved);replay=game.step(game.observe('P0'),index);self.assertGreater(replay.pop('revision'),actual.pop('revision'));self.assertEqual(actual,replay);game.drop_snapshot(saved)

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
            self.assertEqual(obs['schema'],42);self.assertEqual(obs['players'][0]['cards_drawn_this_turn'],2)
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
            return {'Keep':100,'PlayLand':90,'Cast':80,'AnnounceCast':79,'PayCosts':100,'ChooseSpellCost':100,'PutTrigger':60,
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
            actor=obs['actor'];self.assertEqual(obs['schema'],42)
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
            self.assertEqual(obs['schema'],42)
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
            self.assertEqual(obs['schema'],42);self.assertEqual(request['payer'],actor);self.assertEqual(request['cost']['generic'],2)
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
            event=obs['resolving']['damage_event'];self.assertEqual(obs['schema'],42)
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
            choice=obs['recovery_choice'];self.assertEqual(obs['schema'],42)
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
            self.assertEqual(obs['schema'],42);self.assertEqual(obs['resolving']['source_card'],'Disenchant')
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
            self.assertEqual(obs['schema'],42)
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
            self.assertEqual(observation["schema"], 42)
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
