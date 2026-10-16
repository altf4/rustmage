"""Public in-process paid X casts, complete domains and exact whole-body effects."""
import json
from pathlib import Path
import unittest
from rustmage import EngineError, Game
NAMES=['Choco-Comet','Mind Spring','Pedal to the Metal','Procrastinate','Slice from the Shadows','Song of Totentanz','Spectral Denial','Syncopate','Traumatic Critique','United Front','Morlun, Devourer of Spiders','Pterafractyl']
FACTS={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}

class ScalarXTests(unittest.TestCase):
    def replay(self,g,o,a):
        saved=g.snapshot()
        with g.clone() as fork: expected=fork.act(fork.observe(o['actor']),a)
        self.assertEqual(g.act(o,a),expected)
        g.restore(saved)
        with self.assertRaisesRegex(EngineError,'StaleRevision'):g.act(o,a)
        after=g.act(g.observe(o['actor']),a)
        expected['revision']=after['revision'];self.assertEqual(after,expected)
        g.drop_snapshot(saved)
        return after

    def test_all_twelve_whole_bodies_zero_and_positive_both_controllers(self):
        for seat in ('P0','P1'):
            for x in (0,3):
                for i in range(12):
                    with self.subTest(card=NAMES[i],seat=seat,x=x):self.scenario(i,seat,x)

    def scenario(self,i,seat,x):
        other='P1' if seat=='P0' else 'P0';p=int(seat[1]);q=1-p
        deck=sum(([n]*12 for n in ('Plains','Island','Swamp','Mountain','Forest')),[])+[NAMES[i]]*12+['Bear Cub']*12
        foe=['Mountain']*24+['Shock']*24+['Forest']*24
        with Game([deck,foe] if p==0 else [foe,deck],seed=73,starting_player=seat,deck_rules='Freeform') as g:
            source=packet=base=target=paid=None;seen=False;entry_x=False
            for _ in range(16000):
                o=g.observe('P0');self.assertIsNone(o['outcome']);o=g.observe(o['actor']);objects={v['id']:v for v in o['objects']}
                body=next((v for v in objects.values() if v['owner']==seat and v['zone']=='Battlefield' and v['card']=='Bear Cub'),None)
                if o['x_choice']:
                    self.assertEqual(o['actor'],seat);self.assertIsNone(o['payment_request']);f=o['x_choice'];self.assertIsNone(g.observe(other)['x_choice']);self.assertEqual(g.observe(other)['actions'],[])
                    self.assertTrue(any(r['minimum']<=x<=r['maximum'] for r in f['intervals']))
                    self.assertEqual(f['intervals'],sorted(f['intervals'],key=lambda r:r['minimum']))
                    before=g.observe(seat)
                    for actor,value in [(other,x),(seat,f['intervals'][-1]['maximum']+1)]:
                        with self.assertRaises(EngineError):g.request('act',actor=actor,revision=o['revision'],action={'kind':'ChooseX','value':value},viewer=seat)
                        self.assertEqual(g.observe(seat),before)
                    a={'kind':'ChooseX','value':x};self.replay(g,o,a);continue
                packets=o['stack']+o['waiting_triggers']+([o['announcing']] if o['announcing'] else [])+([o['resolving']] if o['resolving'] else [])
                own=next((s for s in packets if s['source_card']==NAMES[i] and s['kind']=='Spell'),None)
                if own:
                    source=own['card']
                    if own['x'] is not None:self.assertEqual(own['x'],x)
                    if own['targets']:target=own['targets'][0]['target']
                    if o['stack'] and o['stack'][-1]['id']==own['id'] and o['decision']=='priority':packet=own;seen=True;base=o
                for s in packets:
                    if s['source_card']==NAMES[i] and s['kind']!='Spell':self.assertEqual(s['x'],x);entry_x=True
                if seen and not any(s['source_card']==NAMES[i] for s in packets) and not o['stack'] and o['decision']=='priority':
                    self.assertIsNotNone(paid);self.assertIsNotNone(base)
                    view=g.observe(seat);ob={v['id']:v for v in view['objects']}
                    bird=lambda z,t:sum(v['owner']==seat and v['zone']=='Battlefield' and v['token'] is not None and v['token']['subtypes']==t for v in z['objects'])
                    if i==0:self.assertEqual(view['players'][q]['life'],base['players'][q]['life']-x);self.assertEqual(bird(view,'Bird'),bird(base,'Bird')+1)
                    if i==1:self.assertEqual(view['players'][p]['cards_drawn_this_turn'],base['players'][p]['cards_drawn_this_turn']+x)
                    if i in (2,3,4):
                        old=next(v for v in base['objects'] if v['id']==target['Card']);new=ob[target['Card']]
                        if i==2:self.assertEqual(new['power'],old['power']+x);self.assertTrue(new['keywords']&512)
                        if i==3:self.assertTrue(new['tapped']);self.assertEqual(new['counters']['stun'],old['counters']['stun']+2*x)
                        if i==4:
                            if x>=old['toughness']:self.assertEqual(new['zone'],'Graveyard')
                            else:self.assertEqual(new['toughness'],old['toughness']-x)
                    if i==5:
                        self.assertEqual(bird(view,'Rat'),bird(base,'Rat')+x)
                        for v in view['objects']:
                            if v['owner']==seat and v['zone']=='Battlefield' and (v['card']=='Bear Cub' or v['token']):self.assertTrue(v['keywords']&4)
                    if i in (6,7):
                        self.assertEqual(ob[target['Spell']]['zone'],'Exile' if i==7 and x else 'Graveyard')
                        if not x:self.assertEqual(view['players'][p]['life'],base['players'][p]['life']-2)
                    if i==8:self.assertEqual(view['players'][q]['life'],base['players'][q]['life']-x);self.assertEqual(view['players'][p]['cards_drawn_this_turn'],base['players'][p]['cards_drawn_this_turn']+2);self.assertEqual(view['players'][p]['cards_discarded_this_turn'],base['players'][p]['cards_discarded_this_turn']+1)
                    if i==9:
                        self.assertEqual(bird(view,'Ally'),bird(base,'Ally')+x)
                        for v in view['objects']:
                            if v['owner']==seat and v['zone']=='Battlefield' and (v['card']=='Bear Cub' or v['token']):self.assertEqual(v['counters']['plus_one'],1)
                    if i==10:self.assertEqual(ob[source]['counters']['plus_one'],x);self.assertEqual(view['players'][q]['life'],base['players'][q]['life']-x);self.assertEqual(view['players'][p]['life'],base['players'][p]['life']+x);self.assertTrue(entry_x)
                    if i==11:self.assertEqual(ob[source]['zone'],'Battlefield' if x else 'Graveyard');self.assertEqual(view['players'][p]['life'],base['players'][p]['life']+2);self.assertTrue(entry_x)
                    return
                def score(a):
                    k=a['kind'];v=objects.get(a.get('card'));actor=o['actor']
                    if k in ('Cast','AnnounceCast','BeginCast'):
                        if actor==seat:
                            if v['card']==NAMES[i] and source is None and (body or i not in (2,3,4,5,9)):
                                lands=sum(v['owner']==seat and v['zone']=='Battlefield' and v['card'] in ('Plains','Island','Swamp','Mountain','Forest') for v in objects.values())
                                if lands>=7 and (i not in (6,7) or (o['stack'] and o['stack'][-1]['source_card']=='Shock' and o['stack'][-1]['controller']==other)):return 2500
                            if v['card']=='Bear Cub' and body is None:return 2000
                        elif v['card']=='Shock' and i in (6,7) and source is None and not o['stack'] and sum(v['owner']==seat and v['zone']=='Battlefield' and v['card'] in ('Plains','Island','Swamp','Mountain','Forest') for v in objects.values())>=7:return 2000
                        return -1000
                    if k=='ChooseTarget':
                        preferred={'Card':body['id']} if body and i in (2,3,4) and actor==seat else {'Player':other if actor==seat else seat}
                        return 3000 if a['target']==preferred or 'Spell' in a['target'] else -1000
                    if k=='TapMana':return 3000 if o['decision']=='pay_costs' else -1000
                    if k=='PayCosts':return 4000
                    if k=='DeclineResolutionMana':return 4000 if x else -1000
                    if k=='PayResolutionMana':return 4000 if not x else -1000
                    if k in ('PayResolutionMana','ActivateMana','AnnounceMana','Activate','AnnounceActivate','Attack','Block'):return -1000
                    if k=='Discard':return 2500 if v['card'] in ('Plains','Island','Swamp','Mountain','Forest') else 1500
                    if k=='PlayLand':
                        count=sum(z['owner']==actor and z['zone']=='Battlefield' and z['card']==v['card'] for z in objects.values())
                        return 1800-count*100
                    return {'Keep':5000,'Mulligan':-1000,'Concede':-10000,'PutTrigger':4000,'FinishTargets':4000,'FinishAttackers':1000,'FinishBlockers':1000,'Pass':0}.get(k,100)
                j=max(range(len(o['actions'])),key=lambda j:(score(o['actions'][j]),-j));a=o['actions'][j]
                if a['kind']=='PayCosts' and own:
                    paid=a['payment'];self.assertEqual(sum(paid),sum(o['payment_request']['required'])+o['payment_request']['generic'])
                self.replay(g,o,a)
            self.fail(('unfinished',NAMES[i],seat,x))

    def test_disconnected_and_nonrepresentative_domains_from_actual_paid_cards(self):
        for seat in ('P0','P1'):
            other='P1' if seat=='P0' else 'P0'
            # Only two Islands: spending all remaining red on Pedal leaves UU.
            deck=['Island']*8+['Mountain']*24+['The Scarlet Witch']*12+['Goblin Piker']*12+['Pedal to the Metal']*40+['Mind Spring']*16
            with Game([deck,['Forest']*180] if seat=='P0' else [['Forest']*180,deck],seed=7,starting_player=seat,deck_rules='Freeform') as g:
                burn=False;disconnected=False;chosen=None
                for _ in range(30000):
                    o=g.observe('P0');self.assertIsNone(o['outcome'],(seat,burn,disconnected,witch if 'witch' in locals() else None,islands if 'islands' in locals() else None,o['turn']));o=g.observe(o['actor']);objects={v['id']:v for v in o['objects']}
                    witch=next((v for v in objects.values() if v['owner']==seat and v['zone']=='Battlefield' and v['card']=='The Scarlet Witch'),None)
                    bear=next((v for v in objects.values() if v['owner']==seat and v['zone']=='Battlefield' and v['card']=='Goblin Piker'),None)
                    islands=sum(v['owner']==seat and v['zone']=='Battlefield' and v['card']=='Island' for v in objects.values())
                    if o['x_choice']:
                        f=o['x_choice'];ann=o['announcing'];name=ann['source_card']
                        if name=='Pedal to the Metal':
                            if disconnected:chosen=max(3,witch['power'])
                            else:
                                red=o['players'][int(seat[1])]['mana'][3]+sum(v['owner']==seat and v['zone']=='Battlefield' and v['card']=='Mountain' and not v['tapped'] for v in objects.values())
                                chosen=red+1 if red>=2 else 0
                        elif not disconnected:
                            self.assertEqual(f['intervals'],[{'minimum':0,'maximum':0},{'minimum':2,'maximum':2}])
                            for illegal in (1,3,2**32-1):
                                with self.assertRaisesRegex(EngineError,'IllegalAction'):g.act(o,{'kind':'ChooseX','value':illegal})
                                self.assertEqual(g.observe(seat),o)
                            chosen=2;disconnected=True
                        else:
                            # Choose a concrete integer deliberately absent from bounded representatives.
                            reps={a['value'] for a in o['actions'] if a['kind']=='ChooseX'}
                            selected=next((v for v in range(65,130) if v not in reps and any(r['minimum']<=v<=r['maximum'] for r in f['intervals'])),None)
                            if selected is not None:
                                self.assertLessEqual(len(reps),6)
                                after=self.replay(g,o,{'kind':'ChooseX','value':selected})
                                self.assertEqual(after['announcing']['x'],selected)
                                break
                            chosen=f['intervals'][-1]['maximum']
                        self.replay(g,o,{'kind':'ChooseX','value':chosen});continue
                    def score(a):
                        k=a['kind'];v=objects.get(a.get('card'))
                        if k in ('Cast','AnnounceCast','BeginCast'):
                            if o['actor']!=seat or o['stack']:return -1000
                            n=v['card']
                            if n=='The Scarlet Witch':return 2500 if witch is None else -1000
                            if n=='Goblin Piker':return 2400 if bear is None else -1000
                            if not witch or not bear or islands!=2:return -1000
                            if n=='Pedal to the Metal':
                                ready=o['active']==seat and o['step'] in ('Main1','Main2') and sum(z['owner']==seat and z['zone']=='Battlefield' and z['card']=='Island' and not z['tapped'] for z in objects.values())==2 and sum(z['owner']==seat and z['zone']=='Hand' and z['card']=='Mind Spring' for z in objects.values())>=2
                                growth_ready=witch['power']>2 or (sum(z['owner']==seat and z['zone']=='Hand' and z['card']=='Pedal to the Metal' for z in objects.values())>=5 and sum(z['owner']==seat and z['zone']=='Battlefield' and z['card']=='Mountain' and not z['tapped'] for z in objects.values())>=6 and any(z['owner']==seat and z['zone']=='Hand' and z['card']=='Mind Spring' for z in objects.values()))
                                return 2300 if (disconnected and growth_ready and o['active']==seat and o['step'] in ('Main1','Main2')) or (not burn and ready) else -1000
                            if n=='Mind Spring':return 2700 if (burn and not disconnected) or (disconnected and witch['power']>=65) else -1000
                            return -1000
                        if k=='ChooseTarget':
                            wanted={'Card':witch['id'] if disconnected else bear['id']}
                            return 4000 if a['target']==wanted else -1000
                        if k=='PlayLand':
                            if v['card']=='Island' and islands>=2:return -1000
                            return 2000 if not burn or disconnected else -1000
                        if k=='Discard':
                            if v['card']=='Mind Spring':return 3000 if sum(z['owner']==seat and z['zone']=='Hand' and z['card']=='Mind Spring' for z in objects.values())>(1 if disconnected else 2) else -1000
                            return 3000 if v['card'] in ('The Scarlet Witch','Island','Goblin Piker') else 2000 if v['card']=='Mountain' else -1000
                        if k=='TapMana':
                            if o['decision']!='pay_costs':return -1000
                            return 3500 if v['card']=='Mountain' else 3000
                        if k in ('Activate','ActivateMana','AnnounceMana','AnnounceActivate','Attack','Block'):return -1000
                        return {'Keep':5000,'Mulligan':-1000,'Concede':-10000,'PayCosts':4000,'PutTrigger':4000,'FinishTargets':4000,'FinishAttackers':1000,'FinishBlockers':1000,'Pass':0}.get(k,100)
                    j=max(range(len(o['actions'])),key=lambda j:(score(o['actions'][j]),-j));a=o['actions'][j]
                    if a['kind']=='PayCosts' and o['announcing'] and o['announcing']['source_card']=='Pedal to the Metal':
                        burn=True
                    self.replay(g,o,a)
                else:self.fail(('did not reach complete nonenumerative domain',seat,disconnected))
                self.assertTrue(disconnected)

    def test_numeric_json_bounds_are_atomic(self):
        with Game([['Forest']*60]*2,seed=1,deck_rules='Standard') as g:
            o=g.observe('P0');before=dict(o)
            for value in (-1,2**32,1.5,'3'):
                with self.assertRaises(EngineError):g.act(o,{'kind':'ChooseX','value':value})
                self.assertEqual(g.observe('P0'),before)

if __name__=='__main__':unittest.main()
