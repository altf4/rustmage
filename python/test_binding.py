import concurrent.futures
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest
from rustmage import CapacityError, EngineError, Game


class BindingTests(unittest.TestCase):
    def test_twelve_filtered_selection_bodies_all_hybrids_and_trainer_copies(self):
        names=['Ashe, Princess of Dalmasca','Casey Jones, Jury-Rig Justiciar','Cowabunga!','Eclipsed Boggart','Eclipsed Elf','Eclipsed Flamekin','Eclipsed Kithkin','Eclipsed Merrow','Frontier Seeker','Master Piandao','Staunch Crewmate','Thundertrap Trainer']
        facts={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
        creature_types={'Mutant','Ninja','Turtle','Goblin','Elf','Elemental','Kithkin','Merfolk','Mount','Ally','Pirate'}
        def subtype(n,t):
            c=facts[n];printed=c['type_line'].split(' — ')[1].split() if ' — ' in c['type_line'] else []
            return t in printed or 'Changeling' in c.get('keywords',[]) and t in creature_types
        def eligible(i,n):
            kinds=facts[n]['type_line'].split(' — ')[0].split()
            if i in (0,1):return 'Artifact' in kinds
            if i==2:return 'Land' in kinds or any(subtype(n,t) for t in ['Mutant','Ninja','Turtle'])
            if i in (3,4,5,6,7):return any(subtype(n,t) for t in {3:['Goblin','Swamp','Mountain'],4:['Elf','Swamp','Forest'],5:['Elemental','Island','Mountain'],6:['Kithkin','Forest','Plains'],7:['Merfolk','Plains','Island']}[i])
            if i==8:return 'Creature' in kinds and subtype(n,'Mount') or subtype(n,'Plains')
            if i==9:return any(subtype(n,t) for t in ['Ally','Equipment','Lesson'])
            if i==10:return 'Artifact' in kinds or subtype(n,'Pirate')
            return 'Creature' not in kinds and 'Land' not in kinds
        def cost(i,k):
            c=[0]*6
            if i==3:c[2],c[3]=k,3-k
            if i==4:c[2],c[4]=k,3-k
            if i==5:c[1],c[3]=k,2-k
            if i==6:c[0],c[4]=k,2-k
            if i==7:c[0],c[1]=k,3-k
            return c,int(i==5)
        cases=[(i,k,False) for i in range(12) for k in range(4 if i in (3,4,7) else 3 if i in (5,6) else 1)]+[(11,0,True)]
        for i,alternative,pay in cases:
            wanted=names[i];lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[]) if i not in (3,4,5,6,7) else sum(([n]*13 for n in {3:['Swamp','Mountain'],4:['Swamp','Forest'],5:['Island','Mountain'],6:['Forest','Plains'],7:['Plains','Island']}[i]),[])+[{3:'Forest',4:'Island',5:'Plains',6:'Swamp',7:'Mountain'}[i]]*4
            support=sum(([n]*4 for n in ['Three Tree Mascot','Mischievous Sneakling','Goldvein Pick','Abandon Attachments','Bear Cub','Prophetic Prism']),[])
            with self.subTest(card=wanted,alternative=alternative,pay=pay),Game([lands+[wanted]*8+support+['Corrupted Conviction']*2,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                chosen=None;committed=set();frames=set();replayed=0;hybrid=False;source_left=False;sacrificed_source=False
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome'],wanted);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};mana=list(obs['players'][0]['mana']);basic={'Plains':0,'Island':1,'Swamp':2,'Mountain':3,'Forest':4}
                    for v in objects.values():
                        if v['owner']=='P0' and v['zone']=='Battlefield' and v['card'] in basic and not v['tapped']:mana[basic[v['card']]]+=1
                    ready=True
                    if i in (3,4,5,6,7):required,generic=cost(i,alternative);ready=all(a>=b for a,b in zip(mana,required)) and sum(mana)>=sum(required)+generic
                    if i==11 and pay:ready=mana[1]>=1 and sum(mana)>=6
                    attack_root=next((s for s in obs['stack'] if s['source_card']==wanted and s['kind']=={'TriggeredAbility':{'ability':0}} and i in (0,9) and s['card'] in objects and objects[s['card']]['reference']==s['object'] and objects[s['card']]['zone']=='Battlefield'),None)
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            if obs['actor']!='P0':return -100
                            n=objects[a['card']]['card']
                            if n==wanted:return 400 if k=='AnnounceCast' and chosen is None and ready else -100
                            if n=='Corrupted Conviction':return 600 if attack_root else -100
                            return -100
                        if k=='ChooseAdditionalCost':return 600 if attack_root and a['object']==attack_root['object'] else -100
                        if k=='ChooseSpellCost':return 600 if a['cost']==alternative else -100
                        if k=='ChooseOffspring':return 600 if a['pay']==pay else -100
                        if k=='SelectLibrary':return 600
                        if k=='FinishLibrarySelection':return 500
                        if k=='PayCosts':return 700
                        if k=='TapMana':return 250 if obs['decision']=='pay_costs' else -100
                        if k=='Attack':return 400 if i in (0,9) and a['card']==chosen else -100
                        if k in ('Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        if k=='PlayLand' and i in (3,4,5,6,7):return 400 if required[basic[objects[a['card']]['card']]]>mana[basic[objects[a['card']]['card']]] else 300
                        return self.professor_score(obs,a)
                    f=obs['library_selection'];root=obs['resolving']
                    if f:
                        self.assertIsNone(game.observe('P1')['library_selection']);self.assertEqual(game.observe('P1')['actions'],[])
                        if root['id'] not in frames:
                            frames.add(root['id']);window=f['remaining']+f['selected'];expected=[v['reference'] for v in f['remaining'] if eligible(i,v['card'])];self.assertEqual(f['candidates'],expected);self.assertEqual((f['minimum'],f['maximum'],f['reveal'],f['remainder'],f['ordering']),(0,int(bool(expected)),True,'bottom_random',False));self.assertEqual(len(window),min(5 if i in (0,8) else 4,obs['players'][0]['library_count']))
                            if i in (0,9) and objects[root['card']]['reference']!=root['object']:source_left=True
                            bad=[{'kind':'SelectLibrary','card':4294967295},{'kind':'Pass'}]+[{'kind':'SelectLibrary','card':v['id']} for v in f['remaining'] if v['reference'] not in expected]
                            for action in bad:
                                with self.assertRaisesRegex(EngineError,'IllegalAction'):game.act(obs,action)
                                self.assertEqual(game.observe(obs['actor']),obs)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if action['kind']=='AnnounceCast' and objects[action['card']]['card']==wanted:chosen=action['card']
                    if action['kind']=='ChooseAdditionalCost' and attack_root and action['object']==attack_root['object']:sacrificed_source=True
                    if action['kind']=='ChooseSpellCost':
                        self.assertEqual(action['cost'],alternative)
                        for c in obs['cost_choices']:required_,generic_=cost(i,c['index']);self.assertEqual((c['cost']['colored'],c['cost']['generic']),(required_,generic_))
                        hybrid=True
                    if obs['announcing'] or obs['stack'] or f:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if f:
                        if after['library_selection']:
                            self.assertEqual((after['players'][0]['hand_count'],after['players'][0]['library_count'],after['players'][0]['cards_drawn_this_turn']),(obs['players'][0]['hand_count'],obs['players'][0]['library_count'],obs['players'][0]['cards_drawn_this_turn']));self.assertEqual(after['known_hand'],obs['known_hand']);before={v['id']:v['reference'] for v in f['remaining']+f['selected']}
                            for v in after['library_selection']['remaining']+after['library_selection']['selected']:self.assertEqual((v['reference'],v['zone']),(before[v['id']],'Library'))
                        else:
                            self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count']-len(f['selected']));self.assertEqual(after['players'][0]['hand_count'],obs['players'][0]['hand_count']+len(f['selected']));self.assertEqual(after['players'][0]['cards_drawn_this_turn'],obs['players'][0]['cards_drawn_this_turn'])
                            for v in f['selected']:
                                new=next(x for x in after['objects'] if x['id']==v['id']);self.assertEqual((new['reference']['generation'],new['zone']),(v['reference']['generation']+1,'Hand'));self.assertIn(new['reference'],game.observe('P1')['known_hand'])
                            committed.add(bool(objects[root['card']]['token']))
                            if i!=11 or not pay or committed=={False,True}:break
                else:self.fail('filtered body did not finish '+wanted)
                self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)
                if sacrificed_source:self.assertTrue(source_left)
                if i in (3,4,5,6,7):self.assertTrue(hybrid)
                if i==11:
                    copies=[v for v in after['objects'] if v['owner']=='P0' and v['zone']=='Battlefield' and v['token'] and v['token']['name']==wanted];self.assertEqual(len(copies),int(pay))
                    for v in copies:self.assertEqual((v['power'],v['toughness'],v['card']),(1,1,v['token']['factory_identity']))

    def test_fifteen_impulse_bodies_public_exile_paid_plays_expiry_and_replay(self):
        names=["Alania's Pathmaker",'Blazing Crescendo','Clockwork Percussionist','Cori Mountain Monastery','Count on Luck','Crimson Operative','Diversion Specialist','Emberheart Challenger','Equilibrium Adept','Gundabad Opportunist','Kulrath Zealot',"Outlaws' Fury",'Seize Opportunity','Sizzling Changeling','Zuko, Exiled Prince']
        totals={'spells':0,'lands':0,'expiry':0,'replays':0}
        for family,wanted in enumerate(names):
            deck=['Mountain']*10+['Swamp']*10+['Forest']*10+[wanted]*8+['Swab Goblin']*8+['Corrupted Conviction']*8+['Giant Growth']*6
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                grants=0;played=0;expired=0;held=None;seen=set();castings={}
                for _ in range(18000):
                    obs=game.observe('P0')
                    if obs['outcome'] is not None:break
                    obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};live=[v for v in objects.values() if v['owner']=='P0' and v['zone']=='Battlefield' and v['card']==wanted];victims=[v for v in objects.values() if v['owner']=='P0' and v['zone']=='Battlefield' and v['power']>0]
                    rights={r['object']['card']:r for r in obs['exile_permissions']}
                    def score(a):
                        k=a['kind']
                        if k=='AnnounceExile':return 700 if a['card']!=held else -100
                        if k=='PlayLand':return 690 if a['card'] in rights and a['card']!=held else -100 if a['card']==held else 90
                        if k in ('Cast','AnnounceCast'):
                            if obs['actor']!='P0':return -100
                            n=objects[a['card']]['card']
                            if n==wanted:return 350 if k=='AnnounceCast' and (not live or family in (1,11,12)) else -100
                            if n=='Swab Goblin':return 300
                            if n=='Corrupted Conviction':return 330 if victims and (family in (2,13) and live or grants>0) else -100
                            if n=='Giant Growth':return 340 if family==7 and live else -100
                            return -100
                        if k in ('AnnounceActivate','Activate','BeginActivate'):
                            return 360 if objects[a['card']]['card']==wanted and family in (3,6,14) and (family!=3 or a['ability']==1) else -100
                        if k=='ChooseTarget':
                            target=a['target'];v=objects.get(target.get('Card')) if isinstance(target,dict) else None
                            return 550 if v and v in live else self.professor_score(obs,a)
                        if k=='ChooseAdditionalCost':return 600 if objects[a['object']['card']] in live and family in (2,13) else 500
                        if k=='ChooseMode':return 500 if a['mode']==0 else -100
                        if k=='ChooseSearch':return -100
                        if k=='PayCosts':return 800
                        if k=='TapMana':return 600 if obs['decision']=='pay_costs' else -100
                        if k in ('Attack','Block','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if obs['exile_permissions']:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertEqual(other['exile_permissions'],obs['exile_permissions'])
                        for r in obs['exile_permissions']:
                            v=objects[r['object']['card']];self.assertEqual(v['reference'],r['object']);self.assertEqual((v['zone'],v['owner']),('Exile',r['player']));self.assertGreaterEqual(r['expires_turn'],obs['turn'])
                    saved=game.snapshot()
                    with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                    self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                    after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);totals['replays']+=1
                    new=[r for r in after['exile_permissions'] if r['object']['card'] not in rights]
                    if new:
                        self.assertEqual(len(new),2 if family==12 else 1)
                        self.assertEqual(after['players'][0]['library_count'],obs['players'][0]['library_count']-len(new));self.assertEqual(after['players'][0]['cards_drawn_this_turn'],obs['players'][0]['cards_drawn_this_turn']);grants+=len(new)
                        for r in new:
                            self.assertEqual(r['player'],'P0');self.assertEqual(r['expires_turn'],obs['turn']+(0 if family in (4,6,7,14) else 2 if obs['active']=='P0' else 1))
                        if held is None:held=new[0]['object']['card']
                    if action['kind'] in ('AnnounceExile','PlayLand') and action['card'] in rights:
                        card=action['card'];v=next(v for v in after['objects'] if v['id']==card);self.assertEqual(v['reference']['generation'],rights[card]['object']['generation']+1);self.assertFalse(any(r['object']['card']==card for r in after['exile_permissions']));played+=1
                        if action['kind']=='PlayLand':self.assertEqual(v['zone'],'Battlefield');totals['lands']+=1
                        else:
                            self.assertEqual(v['zone'],'Stack');self.assertEqual(after['announcing']['cast']['origin'],'Exile');self.assertEqual(after['announcing']['cast']['method'],'Normal');totals['spells']+=1
                    for s in after['stack']:
                        if s['kind']=='Spell' and s['id'] not in seen:
                            seen.add(s['id'])
                            if s['cast']['origin']=='Exile':castings[s['id']]=s
                    for sid,s in list(castings.items()):
                        if not any(v['id']==sid for v in after['stack']) and not(after['resolving'] and after['resolving']['id']==sid):
                            v=next(v for v in after['objects'] if v['id']==s['card']);self.assertIn(v['zone'],('Battlefield','Graveyard'));self.assertEqual(s['cast']['method'],'Normal');del castings[sid]
                    for card,r in rights.items():
                        if r['expires_turn']<after['turn'] and next(v for v in after['objects'] if v['id']==card)['zone']=='Exile':
                            self.assertFalse(any(x['object']==r['object'] for x in after['exile_permissions']));expired+=1;totals['expiry']+=1
                            if after['actor']==r['player']:
                                with self.assertRaisesRegex(EngineError,'IllegalAction'):game.act(after,{'kind':'AnnounceExile','card':card})
                                self.assertEqual(game.observe(after['actor']),after)
                    if grants and played and expired and not castings:break
                self.assertGreater(grants,0);self.assertGreater(played,0);self.assertGreater(expired,0);self.assertEqual(after['schema'],96)
        for key in totals:self.assertGreater(totals[key],0,key)

    def test_seventeen_contextual_cast_bodies_targets_effects_private_choices_and_replay(self):
        names=['Colleen Wing, Street Samurai','Danitha, Spear of Agony','Danitha, Sword of Hope','Firespitter Whelp','Forum Necroscribe','Gnarlback Rhino','Graduation Day','Informed Inkwright','Inkling Mascot','Inkshape Demonstrator','Lecturing Scornmage','Madame Hydra','Melancholic Poet','Mockingbird, Ace Agent','Rehearsed Debater','Rodeo Pyromancers','Stirring Hopesinger']
        for i,name in enumerate(names):
            with self.subTest(card=name),Game([['Plains','Island','Swamp','Mountain','Forest']*8+[name]*12+['Bear Cub']*12+['Giant Growth']*12+['Opt']*12+['Lightning Bolt']*8+['Goldvein Pick']*8+['Three Tree Mascot']*8+['Fell']*8,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                done=False;replayed=0;private=0;pending={}
                for _ in range(20000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};sources=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==name];mates=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and v['card']=='Bear Cub'];dead=[v for v in objects.values() if v['zone']=='Graveyard' and v['owner']=='P0' and v['card']=='Bear Cub'];catalyst='Lightning Bolt' if i==1 else 'Goldvein Pick' if i==2 else 'Opt' if i in (3,15) else 'Three Tree Mascot' if i==11 else 'Fell' if i==4 and not dead else 'Giant Growth'
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==name and not sources:return 280
                            if sources and n=='Bear Cub' and i in (4,6) and not mates:return 270
                            return 260 if sources and n==catalyst and (i not in (4,6) or mates) and (i!=15 or int(obs['players'][0]['spells_cast_this_turn'])==0) else -100
                        if k=='ChooseTarget':
                            item=obs['announcing'];t=a['target']
                            if 'Player' in t:return 400 if i==1 and t['Player']=='P1' else -100
                            if 'Card' in t:
                                v=objects[t['Card']]
                                if item['kind']!='Spell':return 400 if (i==4 and v['zone']=='Graveyard') or (i==6 and v['owner']=='P0' and v['zone']=='Battlefield') else 100
                                return 400 if (catalyst=='Fell' and v['card']=='Bear Cub') or (catalyst!='Fell' and ((i!=6 and v['card']==name) or (i==6 and v['card']=='Bear Cub'))) else 100
                        if k=='PayCosts':return 500
                        if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                        if k=='ArrangeLibrary':return 400 if a['destination']==('Graveyard' if i==8 else 'Bottom') else -100
                        if k=='PlayLand':return 330 if not any(v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==objects[a['card']]['card'] for v in objects.values()) else 300
                        if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    j=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][j]
                    if obs['library_choice'] is not None:self.assertIsNone(game.observe('P1')['library_choice']);private+=1
                    if sources or obs['announcing'] or obs['stack'] or obs['resolving']:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),j)
                        self.assertEqual(game.step(obs,j),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,j)
                        after=game.step(game.observe(obs['actor']),j);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,j)
                    if action['kind']=='Pass' and obs['stack']:
                        item=obs['stack'][-1]
                        if item['source_card']==name and item['kind']!='Spell' and item['kind']['TriggeredAbility']['ability']==int(i in (4,9)) and all(v['id']!=item['id'] for v in after['stack']):pending[item['id']]=(item,obs)
                    for sid,(item,before) in list(pending.items()):
                        if any(v['id']==sid for v in after['stack']) or after['resolving'] and after['resolving']['id']==sid:continue
                        self.assertIsNone(item['entering_object']);old={v['id']:v for v in before['objects']};now={v['id']:v for v in after['objects']};source=item['card'];target=next((v['object']['card'] for v in item['targets'] if v['object'] is not None),None)
                        if i in (0,1,10,13):self.assertEqual(int(now[source]['counters']['plus_one']),int(old[source]['counters']['plus_one'])+1)
                        if i in (0,8):self.assertGreater(private,0);self.assertEqual(after['players'][0]['library_count']+int(i==8),before['players'][0]['library_count'])
                        if i in (2,5):self.assertEqual(after['players'][0]['library_count']+1,before['players'][0]['library_count'])
                        elif i==3:self.assertEqual(int(after['players'][1]['life']),int(before['players'][1]['life'])-1)
                        elif i==4:self.assertEqual(old[target]['zone'],'Graveyard');self.assertEqual(now[target]['zone'],'Battlefield');self.assertNotEqual(old[target]['reference'],now[target]['reference'])
                        elif i==6:self.assertEqual(int(now[target]['counters']['plus_one']),int(old[target]['counters']['plus_one'])+1)
                        elif i in (7,11):self.assertEqual(sum(v['zone']=='Battlefield' and v['token'] is not None and v['token']['subtypes']==('Inkling' if i==7 else 'Villain') and v['id'] not in old for v in after['objects']),1)
                        elif i==8:self.assertEqual(now[source]['keywords'],old[source]['keywords']|1)
                        elif i==9:self.assertEqual(int(now[source]['power']),int(old[source]['power'])+1);self.assertEqual(now[source]['keywords'],old[source]['keywords']|64)
                        elif i==12:self.assertEqual(int(after['players'][0]['life']),int(before['players'][0]['life'])+1);self.assertEqual(int(after['players'][1]['life']),int(before['players'][1]['life'])-1)
                        elif i==14:self.assertEqual(int(now[source]['power']),int(old[source]['power'])+1);self.assertEqual(int(now[source]['toughness']),int(old[source]['toughness'])+1)
                        elif i==15:self.assertEqual(after['players'][0]['mana'][3],before['players'][0]['mana'][3]+2)
                        elif i==16:
                            for v in old.values():
                                if v['owner']=='P0' and v['zone']=='Battlefield' and v['card'] in (name,'Bear Cub'):self.assertEqual(int(now[v['id']]['counters']['plus_one']),int(v['counters']['plus_one'])+1)
                        self.assertEqual(after['schema'],96);done=True;break
                    if done:break
                self.assertTrue(done,name);self.assertGreater(replayed,0)

    def test_six_forage_bodies_owned_choices_routes_and_replay(self):
        names=['Bushy Bodyguard','Camellia, the Seedmiser','Corpseberry Cultivator','Curious Forager','Feed the Cycle','Treetop Sentries']
        for n,wanted in enumerate(names):
            for route in (['ExileThree','Food','Black'] if wanted=='Feed the Cycle' else ['ExileThree','Food']):
                deck=['Forest']*12+['Swamp']*12+[wanted]*8+['Daggerfang Duo']*8+['Canyon Crawler']*8+['Bear Cub']*12
                with Game([deck,deck],seed=400+n,deck_rules='Freeform') as game:
                    tracked={};verified=False;replayed=0
                    for _ in range(20000):
                        view=game.observe('P0');self.assertIsNone(view['outcome']);obs=game.observe(view['actor']);objects={o['id']:o for o in obs['objects']};ctx=obs['announcing'] or obs['resolving'];is_wanted=ctx and ctx['source_card']==wanted
                        def score(a):
                            k=a['kind']
                            if k=='Concede':return -10000
                            if k=='Keep':return 1000
                            if k=='PlayLand':return 900
                            if k=='PayCosts':return 850
                            if k=='ChooseForageRoute':return 800 if is_wanted and a['route']==route else (600 if is_wanted else -200)
                            if k=='ChooseForage':return 800 if is_wanted else 100
                            if k=='DeclineForage':return -100 if is_wanted and route in obs['forage']['routes'] else (700 if not is_wanted or obs['forage']['route'] is None else -100)
                            if k=='ChooseOffspring':return 800 if not a['pay'] else 0
                            if k in ('TapMana','ActivateMana','AnnounceMana'):return 500 if obs['decision']=='pay_costs' else -100
                            if k in ('AnnounceCast','Cast'):return 650 if objects[a['card']]['card'] in ('Canyon Crawler','Daggerfang Duo') else (550 if objects[a['card']]['card']==wanted else 250)
                            if k=='AnnounceActivate':return 750 if objects[a['card']]['card']==wanted else -100
                            if k=='ChooseOptional':return 700 if a['accept'] else 0
                            if k=='ChooseSpellCost':return 600
                            if k=='ChooseTarget':return 700
                            if k in ('FinishTargets','PutTrigger','FinishAttackers','FinishBlockers','FinishDamage','ChooseMode','ChooseMana'):return 600
                            if k in ('Attack','Block','Activate','BeginActivate'):return -100
                            return 100
                        choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                        if action['kind']=='ChooseForageRoute' and is_wanted:tracked[ctx['id']]={'route':action['route'],'objects':[],'cost':obs['announcing'] is not None}
                        if action['kind']=='ChooseForage' and is_wanted:tracked[ctx['id']]['objects'].append(action['object'])
                        commit=None
                        if is_wanted and ctx['id'] in tracked:
                            f=tracked[ctx['id']]
                            if (f['cost'] and action['kind']=='PayCosts') or (not f['cost'] and action['kind']=='ChooseForage' and len(f['objects'])==(1 if f['route']=='Food' else 3)):commit=f
                        if obs['forage']:
                            other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['forage']);self.assertEqual(other['actions'],[])
                        if obs['forage'] or obs['announcing'] or obs['resolving']:
                            saved=game.snapshot()
                            with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                            after=game.step(obs,choice);self.assertEqual(after,expected);game.restore(saved)
                            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                            after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                        else:after=game.step(obs,choice)
                        if commit and commit['route']==route:
                            now={v['id']:v for v in after['objects']};refs=commit['objects'];self.assertEqual(len({str(r) for r in refs}),len(refs));self.assertEqual(len(refs),0 if route=='Black' else (1 if route=='Food' else 3))
                            for ref in refs:self.assertEqual(now.get(ref['card'],{}).get('zone','Outside'),'Outside' if route=='Food' else 'Exile')
                            if route=='Black':self.assertEqual(obs['payment_request']['required'][2],2)
                            if wanted=='Bushy Bodyguard':self.assertEqual(now[ctx['card']]['counters']['plus_one'],2)
                            if wanted=='Treetop Sentries':self.assertEqual(after['players'][0 if obs['actor']=='P0' else 1]['cards_drawn_this_turn'],obs['players'][0 if obs['actor']=='P0' else 1]['cards_drawn_this_turn']+1)
                            self.assertEqual(after['schema'],96);self.assertGreater(replayed,0);verified=True;break
                    self.assertTrue(verified,(wanted,route))


    def test_ten_offspring_bodies_paid_unpaid_copies_and_owned_replay(self):
        names=['Coruscation Mage','Finch Formation','Flowerfoot Swordmaster','Intrepid Rabbit','Iridescent Vinelasher','Rust-Shield Rampager','Splash Lasher','Starscape Cleric','Steampath Charger','Tender Wildguide']
        for wanted in names:
            for pay in [False,True]:
                with self.subTest(card=wanted,pay=pay),Game([['Plains','Island','Swamp','Mountain','Forest']*10+[wanted]*12,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                    chosen=False;replayed=0;captured=False
                    for _ in range(15000):
                        obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};parents=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==wanted and v['token'] is None]
                        def score(a):
                            k=a['kind']
                            if k in ('Cast','AnnounceCast'):
                                lands=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and v['card'] in ('Plains','Island','Swamp','Mountain','Forest')];total=[4,6,3,4,3,6,6,5,4,4][names.index(wanted)];color=['Mountain','Island','Plains','Plains','Swamp','Forest','Island','Swamp','Mountain','Forest'][names.index(wanted)];ready=not pay or len(lands)>=total and sum(v['card']==color for v in lands)>=(2 if wanted in ('Splash Lasher','Starscape Cleric') else 1)
                                return 280 if objects[a['card']]['card']==wanted and not parents and ready else -100
                            if k=='ChooseOffspring':return 450 if a['pay']==pay else -100
                            if k=='PayCosts':return 500
                            if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                            if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                            return self.professor_score(obs,a)
                        j=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));a=obs['actions'][j]
                        if a['kind']=='ChooseOffspring':chosen=True;self.assertEqual(obs['decision'],'offspring');self.assertIsNone(obs['payment_request'])
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),j)
                        self.assertEqual(game.step(obs,j),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,j)
                        after=game.step(game.observe(obs['actor']),j);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                        for packet in after['stack']:
                            if packet['source_card']==wanted and packet['kind']=='Spell':self.assertEqual(packet['cast']['offspring'],pay);captured=True
                        parents=[v for v in after['objects'] if v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==wanted and v['token'] is None]
                        if parents and not after['stack'] and after['announcing'] is None and after['resolving'] is None and after['decision']=='priority':
                            tokens=[v for v in after['objects'] if v['zone']=='Battlefield' and v['owner']=='P0' and v['token'] is not None and v['token']['name']==wanted];self.assertEqual(len(tokens),int(pay));self.assertTrue(chosen);self.assertTrue(captured);self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)
                            if pay:self.assertTrue(tokens[0]['token']['factory_identity'].startswith('@token/'));self.assertEqual(tokens[0]['card'],tokens[0]['token']['factory_identity']);self.assertEqual(tokens[0]['counters']['plus_one'],0)
                            break
                    else:self.fail('offspring body did not settle')

    def test_random_bottom_seeded_replay_is_observed_in_the_next_private_window(self):
        deck=['Forest']*4+['Llanowar Elves']*2+['Follow the Lumarets']*2+['Commune with Nature']*2
        observed=set();checked=0
        def reach(game,goal):
            for _ in range(600):
                root=game.observe('P0')
                if root['outcome'] is not None:return None
                obs=game.observe(root['actor']);objects={v['id']:v for v in obs['objects']}
                if obs['library_selection'] and obs['resolving']['source_card']==goal:return obs
                if goal=='Follow the Lumarets' and obs['active']=='P0' and obs['turn']>3:return None
                elves=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and v['card']=='Llanowar Elves']
                def score(a):
                    k=a['kind']
                    if k in ('Cast','AnnounceCast'):
                        n=objects[a['card']]['card']
                        if obs['actor']!='P0':return -100
                        if n=='Llanowar Elves':return 400 if not elves else -100
                        return 350 if n==goal and k=='AnnounceCast' and (goal!='Follow the Lumarets' or elves and not elves[0]['summoning_sick'] and obs['players'][0]['library_count']==2) else -100
                    if k=='PayCosts':return 600
                    if k in ('TapMana','ActivateMana','AnnounceMana'):return 200 if obs['decision']=='pay_costs' else -100
                    if k in ('Attack','Block','Activate','AnnounceActivate'):return -100
                    return self.professor_score(obs,a)
                choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));game.step(obs,choice)
            return None
        for seed in range(40):
            with Game([deck,['Forest']*30],seed=seed,deck_rules='Freeform') as game:
                hand=[v['card'] for v in game.observe('P0')['objects'] if v['owner']=='P0' and v['zone']=='Hand']
                if hand.count('Forest')<2 or not all(n in hand for n in ['Llanowar Elves','Follow the Lumarets','Commune with Nature']):continue
                obs=reach(game,'Follow the Lumarets')
                if obs is None:continue
                f=obs['library_selection'];self.assertEqual(len(f['remaining']),2);self.assertEqual(f['minimum'],0);self.assertEqual(f['remainder'],'bottom_random');original=[v['reference'] for v in f['remaining']]
                choice=next(j for j,a in enumerate(obs['actions']) if a['kind']=='FinishLibrarySelection');saved=game.snapshot()
                with game.clone() as fork:
                    fork.step(fork.observe('P0'),choice);future=reach(fork,'Commune with Nature');self.assertIsNotNone(future);expected=future['library_selection']['remaining']
                game.step(obs,choice);future=reach(game,'Commune with Nature');self.assertIsNotNone(future);actual=future['library_selection']['remaining'];self.assertEqual(actual,expected);self.assertEqual({str(v['reference']) for v in actual},{str(v) for v in original});self.assertEqual(future['players'][0]['library_count'],2)
                self.assertIsNone(game.observe('P1')['library_selection']);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                game.step(game.observe('P0'),choice);again=reach(game,'Commune with Nature');self.assertIsNotNone(again);self.assertEqual(again['library_selection']['remaining'],expected);game.drop_snapshot(saved)
                observed.add(tuple(v['reference']['card'] for v in actual)==tuple(v['card'] for v in original));checked+=1
                if checked>=6 and len(observed)==2:break
        self.assertGreaterEqual(checked,6);self.assertEqual(observed,{False,True})

    def test_selection_empty_short_zero_and_bottom_order_through_future_draw(self):
        cases=[('empty',['Forest']*6+['Commune with Nature'],0),('ordered',['Forest']*7+['Commune with Nature']*2,2),('mandatory',['Swamp']*6+['Resentful Revelation']*2+['Bear Cub'],1)]
        for case,deck,looked in cases:
            with self.subTest(case=case),Game([deck,['Forest']*30],seed=7,deck_rules='Freeform') as game:
                first_order=[];root_id=None;before=None;completed=False;replayed=0;selected_source=None;ordered_turn=None
                for _ in range(1500):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']}
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast','AnnounceFlashback'):
                            if completed or obs['actor']!='P0':return -100
                            return 400 if k=='AnnounceCast' and objects[a['card']]['card'] in ('Commune with Nature','Resentful Revelation') else -100
                        if k=='PayCosts':return 600
                        if k=='TapMana':return 200 if obs['decision']=='pay_costs' else -100
                        if k=='OrderLibraryBottom':return 500-a['card']
                        if k in ('Attack','Block','Activate','AnnounceActivate'):return -100
                        return self.professor_score(obs,a)
                    f=obs['library_selection']
                    if f:
                        self.assertEqual(case,'ordered');self.assertTrue(f['ordering']);self.assertEqual((f['minimum'],f['maximum'],f['candidates']),(0,0,[]));self.assertEqual(len(f['remaining'])+len(f['bottom']),2);self.assertEqual(obs['players'][0]['library_count'],2)
                        for v in f['remaining']+f['bottom']:self.assertEqual(v['zone'],'Library')
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if action['kind']=='OrderLibraryBottom':first_order.append(next(v for v in f['remaining'] if v['id']==action['card']))
                    if obs['stack'] and obs['stack'][-1]['kind']=='Spell' and obs['stack'][-1]['source_card'] in ('Commune with Nature','Resentful Revelation'):
                        root_id=obs['stack'][-1]['id'];selected_source=obs['stack'][-1]['card']
                        if action['kind']=='Pass':before=obs
                    if obs['stack'] or f:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if root_id is not None and not completed and not after['library_selection'] and not any(s['id']==root_id for s in after['stack']) and not(after['resolving'] and after['resolving']['id']==root_id):
                        self.assertEqual(next(v['zone'] for v in after['objects'] if v['id']==selected_source),'Graveyard');self.assertEqual(after['players'][0]['cards_drawn_this_turn'],before['players'][0]['cards_drawn_this_turn']);completed=True;ordered_turn=after['turn']
                        if case=='empty':self.assertEqual(after['players'][0]['library_count'],0);break
                        if case=='mandatory':
                            self.assertEqual(before['players'][0]['library_count'],1);self.assertEqual(after['players'][0]['library_count'],0);self.assertEqual(after['players'][0]['hand_count'],before['players'][0]['hand_count']+1);break
                        self.assertEqual(len(first_order),2);self.assertEqual(after['players'][0]['library_count'],2)
                    if completed and case=='ordered' and after['turn']>ordered_turn and after['active']=='P0' and after['players'][0]['cards_drawn_this_turn']:
                        expected=first_order[0];drawn=next(v for v in game.observe('P0')['objects'] if v['id']==expected['id']);self.assertEqual((drawn['zone'],drawn['reference']['generation'],drawn['card']),('Hand',expected['reference']['generation']+1,expected['card']));break
                else:self.fail('short-window boundary not reached '+case)
                self.assertTrue(completed);self.assertGreater(replayed,0)

    def test_fifteen_library_selection_bodies_private_commit_and_reference_replay(self):
        names=['Accumulate Wisdom','Beastrider Vanguard','Commune with Beavers','Commune with Evil','Commune with Nature','Confounding Riddle','Flow State','Follow the Lumarets','Morbius the Living Vampire',"Rakshasa's Bargain",'Resentful Revelation','Squad Rallier','Stock Up','Boughside Wanderers','Living Phone']
        for family,wanted in enumerate(names):
            lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            with self.subTest(card=wanted),Game([lands+[wanted]*8+['Bear Cub']*4+['Llanowar Elves']*4+['Corrupted Conviction']*8+['Quick Study']*6,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                reached=False;committed=False;replayed=0;window=None;baseline=None;source=None
                for _ in range(20000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome'],wanted);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};live=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==wanted];victims=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and (v['card'] in ('Bear Cub','Llanowar Elves') or family in (8,14) and v['card']==wanted)]
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast','AnnounceFlashback'):
                            n=objects[a['card']]['card']
                            if obs['actor']!='P0':return -100
                            if n==wanted:return -100 if k=='Cast' or live else 350
                            if n=='Corrupted Conviction':return 330 if victims else -100
                            return 250 if n in ('Bear Cub','Llanowar Elves','Quick Study') else -100
                        if k in ('AnnounceActivate','Activate'):return 360 if objects[a['card']]['card']==wanted and family in (1,8,11) else -100
                        if k=='ChooseAdditionalCost':return 500 if objects[a['object']['card']]['card']==wanted and family in (8,14) else 400
                        if k=='ChooseMode':return 500 if a['mode']==0 else -100
                        if k=='SelectLibrary':return 550
                        if k=='FinishLibrarySelection':return 500
                        if k=='OrderLibraryBottom':return 500-a['card']
                        if k=='PayCosts':return 600
                        if k=='TapMana':return 200 if obs['decision']=='pay_costs' else -100
                        if k in ('Attack','Block','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    if obs['library_selection']:
                        f=obs['library_selection'];root=obs['resolving'];other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['library_selection']);self.assertEqual(other['actions'],[])
                        if root['source_card']==wanted and not reached:
                            reached=True;source=root;baseline=obs;window=f['remaining']+f['selected']+f['bottom'];self.assertEqual(len(window),min([3,3,3,4,5,4,3,4,3,4,3,4,5,4,5][family],obs['players'][0]['library_count']));self.assertEqual(f['reveal'],family in (1,2,4,7,11,13,14));self.assertEqual(f['remainder'],'graveyard' if family in (3,5,9,10) else 'bottom_random' if family in (7,11,13,14) else 'bottom_any')
                            expected=2 if family in (9,12) else 1;eligible=len(f['candidates']);self.assertEqual(f['maximum'],min(expected,eligible));self.assertEqual(f['minimum'],0 if family in (1,2,4,7,11,13,14) else f['maximum'])
                            if family in (8,14):
                                old=objects[root['card']];self.assertEqual(old['zone'],'Exile' if family==8 else 'Graveyard');self.assertEqual(old['reference']['generation'],root['object']['generation']+1)
                            for bad in ({'kind':'SelectLibrary','card':4294967295},{'kind':'OrderLibraryBottom','card':4294967295},{'kind':'Pass'}):
                                with self.assertRaisesRegex(EngineError,'IllegalAction'):game.act(obs,bad)
                                unchanged=game.observe(obs['actor']);self.assertEqual(unchanged,obs)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if obs['library_selection'] or obs['resolving'] or obs['stack']:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if reached and obs['library_selection'] and obs['resolving']['source_card']==wanted:
                        f=obs['library_selection'];selected=f['selected'];remaining=f['remaining'];player=obs['players'][0]
                        if after['library_selection']:
                            self.assertEqual((after['players'][0]['hand_count'],after['players'][0]['library_count']),(player['hand_count'],player['library_count']));self.assertEqual(after['known_hand'],obs['known_hand']);current=after['library_selection'];old={v['id']:v for v in window};now=current['remaining']+current['selected']+current['bottom'];self.assertEqual({v['id'] for v in now},set(old))
                            for v in now:self.assertEqual((v['reference'],v['zone']),(old[v['id']]['reference'],'Library'))
                        else:
                            public=game.observe('P1');self.assertEqual(after['players'][0]['hand_count'],player['hand_count']+len(selected));self.assertEqual(after['players'][0]['library_count'],player['library_count']-len(selected)-(len(remaining) if f['remainder']=='graveyard' else 0));self.assertEqual(after['players'][0]['cards_drawn_this_turn'],player['cards_drawn_this_turn'])
                            for v in selected:
                                new=next(x for x in after['objects'] if x['id']==v['id']);self.assertEqual((new['zone'],new['reference']['generation']),( 'Hand',v['reference']['generation']+1));self.assertEqual(new['reference'] in public['known_hand'],f['reveal']);self.assertEqual(any(x['reference']==new['reference'] for x in public['objects']),f['reveal'])
                            if f['remainder']=='graveyard':
                                for v in remaining:
                                    new=next(x for x in after['objects'] if x['id']==v['id']);self.assertEqual((new['zone'],new['reference']['generation']),('Graveyard',v['reference']['generation']+1))
                            if family==3:self.assertEqual(after['players'][0]['life'],baseline['players'][0]['life']+3)
                            committed=True;break
                else:self.fail('selection body did not commit '+wanted)
                self.assertTrue(reached and committed);self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)


    def test_nine_flashback_bodies_two_origins_private_resolution_and_replay(self):
        names=["Auron's Inspiration",'Bestial Incursion','Dig Site Inventory','Dreams of Laguna','Laughing Mad','Predictive Preparations','Revenge of the Rats','Think Twice','Tome Blast']
        for i,wanted in enumerate(names):
            lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            with self.subTest(card=wanted),Game([lands+[wanted]*12+['Bear Cub']*8+['Corrupted Conviction']*8+['Opt']*2,['Forest']*30+['Bear Cub']*30],seed=7,deck_rules='Freeform') as game:
                done=set();pending={};replayed=0;private=0;selected=None
                for _ in range(20000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome'],wanted);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};bears=[v for v in objects.values() if v['owner']=='P0' and v['zone']=='Battlefield' and v['card']=='Bear Cub'];grave=[v for v in objects.values() if v['owner']=='P0' and v['zone']=='Graveyard' and v['card']=='Bear Cub']
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast','AnnounceFlashback'):
                            n=objects[a['card']]['card']
                            if obs['actor']=='P1':return -100
                            if n==wanted:
                                if k=='Cast':return -100
                                if i==0 and not obs['attackers'] or i==6 and not grave:return -100
                                if k=='AnnounceFlashback':return 350 if a['card']==selected and 'Flashback' not in done else -100
                                return 300 if selected is None else -100
                            if n=='Bear Cub':return 290 if len(bears)<2 else -100
                            if n=='Corrupted Conviction':return 280 if i==6 and bears and not grave else -100
                            return -100
                        if k=='ChooseTarget':
                            t=a['target']
                            if i==8:return 400 if t.get('Player')=='P1' else -100
                            if 'Card' in t:return 400 if objects[t['Card']]['owner']=='P0' else -100
                        if k=='PayCosts':return 600
                        if k=='TapMana':return 200 if obs['decision']=='pay_costs' else -100
                        if k=='Attack':return 200 if i==0 and obs['actor']=='P0' else -100
                        if k in ('Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if action['kind'] in ('Cast','AnnounceCast') and objects[action['card']]['card']==wanted:selected=action['card']
                    if obs['library_choice']:
                        hidden=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(hidden['library_choice']);self.assertEqual(hidden['actions'],[]);private+=1
                    if obs['announcing'] or obs['resolving'] or obs['stack']:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if action['kind']=='PayCosts' and obs['announcing']['source_card']==wanted and i==4:self.assertEqual(after['players'][0]['cards_discarded_this_turn'],obs['players'][0]['cards_discarded_this_turn']+1)
                    if obs['stack'] and action['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['source_card']==wanted and root['kind']=='Spell' and not any(v['id']==root['id'] for v in after['stack']):pending[root['id']]=(root,obs)
                    for root_id in list(pending):
                        if after['resolving'] and after['resolving']['id']==root_id:continue
                        root,b=pending.pop(root_id);method=root['cast']['method'];self.assertEqual(root['cast']['origin'],'Graveyard' if method=='Flashback' else 'Hand');self.assertEqual(next(v['zone'] for v in after['objects'] if v['id']==root['card']),'Exile' if method=='Flashback' else 'Graveyard')
                        before={str(v['reference']):v for v in b['objects']};now={str(v['reference']):v for v in after['objects']}
                        if i==0:
                            attacked=[v for v in b['objects'] if v['id'] in b['attackers']];self.assertTrue(attacked)
                            for v in attacked:self.assertEqual(now[str(v['reference'])]['power'],v['power']+2)
                        if i in (1,6):
                            subtype='Beast' if i==1 else 'Rat';tokens=[v for v in after['objects'] if v['token'] and v['token']['subtypes']==subtype and str(v['reference']) not in before];expected=1 if i==1 else sum(v['owner']=='P0' and v['zone']=='Graveyard' and v['card']=='Bear Cub' for v in b['objects']);self.assertEqual(len(tokens),expected);self.assertGreater(expected,0)
                            for v in tokens:self.assertEqual((v['power'],v['toughness'],v['keywords'],v['tapped']),(4,4,256,False) if i==1 else (1,1,0,True))
                        if i in (2,5):
                            for t in root['targets']:
                                ref=str(t['object']);self.assertEqual(now[ref]['counters']['plus_one'],before[ref]['counters']['plus_one']+1)
                                if i==2:self.assertTrue(now[ref]['keywords']&8)
                        if i in (3,4,7):self.assertEqual(after['players'][0]['cards_drawn_this_turn'],b['players'][0]['cards_drawn_this_turn']+(2 if i==4 else 1))
                        if i==8:self.assertEqual(after['players'][1]['life'],b['players'][1]['life']-2)
                        done.add(method)
                    if done=={'Normal','Flashback'}:break
                else:self.fail('both origins did not resolve: '+wanted)
                self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)
                if i==3:self.assertGreater(private,0)

    def test_thirteen_qualified_cast_bodies_and_private_replay(self):
        names=['Angry Rabble','Brineborn Cutthroat','Enraged Flamecaster','Kulrath Mystic','Lurking Lizards','Mage Tower Referee','Nightmare Sower','Skybeast Tracker','Solarium Sentry','Spider Manifestation','Stormkeld Prowler','Tanufel Rimespeaker','Unwelcome Sprite']
        for i,wanted in enumerate(names):
            helper='Lightning Helix' if i==5 else 'Opt' if i in (1,6,12) else 'Shivan Dragon';lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            with self.subTest(card=wanted),Game([lands+[wanted]*8+[helper]*22,['Forest']*30+['Bear Cub']*30],seed=7,deck_rules='Freeform') as game:
                replayed=0;pending={};finished=False;private=0
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};sources=[v for v in objects.values() if v['owner']=='P0' and v['zone']=='Battlefield' and v['card']==wanted]
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if obs['actor']=='P1':return 230 if sources and n=='Bear Cub' and not any(v['owner']=='P1' and v['zone']=='Battlefield' and v['card']=='Bear Cub' for v in objects.values()) else -100
                            if n==wanted:return 260 if not sources else -100
                            if n==helper and sources and i!=8:return 250 if (i not in (1,6,12) or obs['active']=='P1') and (i!=6 or any(v['owner']=='P1' and v['zone']=='Battlefield' and v['card']=='Bear Cub' for v in objects.values())) else -100
                            return -100
                        if k=='ChooseTarget':
                            t=a['target']
                            if 'Player' in t:return 280 if t['Player']=='P1' else -100
                            if 'Card' in t:return 280 if objects[t['Card']]['owner']=='P1' else -100
                        if k=='PayCosts':return 500
                        if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                        if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));a=obs['actions'][choice]
                    if obs['library_choice']:
                        hidden=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(hidden['library_choice']);self.assertEqual(hidden['actions'],[]);private+=1
                    if obs['announcing'] or obs['resolving'] or obs['stack'] or sources:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if obs['stack'] and a['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['source_card']==wanted and root['kind']!='Spell' and not any(v['id']==root['id'] for v in after['stack']):self.assertIsNone(root['entering_object']);pending[root['id']]=(root,obs)
                    for root_id in list(pending):
                        if any(v['id']==root_id for v in after['stack']) or after['resolving'] and after['resolving']['id']==root_id:continue
                        root,b=pending.pop(root_id);old=next(v for v in b['objects'] if v['reference']==root['object']);new=next(v for v in after['objects'] if v['reference']==root['object'])
                        if i in (0,2):self.assertEqual(after['players'][1]['life'],b['players'][1]['life']-(1 if i==0 else 2))
                        if i in (1,4,5,10):self.assertEqual(new['counters']['plus_one'],old['counters']['plus_one']+(2 if i==10 else 1))
                        if i==3:self.assertEqual(new['power'],old['power']+2);self.assertTrue(new['keywords']&8)
                        if i==6:
                            ref=root['targets'][0]['object'];before=next(v for v in b['objects'] if v['reference']==ref);now=next(v for v in after['objects'] if v['reference']==ref);self.assertEqual(now['counters']['minus_one'],before['counters']['minus_one']+1)
                        if i==7:self.assertEqual(sum(v['zone']=='Battlefield' and v['owner']=='P0' and v['token'] is not None and v['token']['subtypes']=='Food' for v in after['objects']),1)
                        if i==8:self.assertEqual(after['players'][0]['life'],b['players'][0]['life']+2)
                        if i==9:self.assertFalse(new['tapped'])
                        if i==11:self.assertEqual(after['players'][0]['library_count'],b['players'][0]['library_count']-1)
                        if i==12:self.assertGreater(private,0)
                        finished=True
                    if finished:break
                else:self.fail('qualified cast body did not resolve')
                self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)

    def test_thirteen_tap_trigger_bodies_external_tap_private_choices_and_replay(self):
        names=['Attentive Sunscribe','Mechan Navigator','Pestered Wellguard','Rescue Leopard','Seedship Agrarian','Silvergill Peddler','Spider-Gwen, Free Spirit','Starfighter Pilot','Swarm Culler','Volatile Wanderglyph','Wanderbrine Preacher','Wanderwine Distracter','Wylie Duke, Atiin Hero']
        colors=[['Plains'],['Island'],['Island'],['Mountain'],['Forest'],['Island'],['Mountain'],['Plains'],['Swamp'],['Mountain'],['Plains'],['Island'],['Plains','Forest']]
        for i,source in enumerate(names):
            mana=list(dict.fromkeys(['Island']+colors[i]));lands=sum(([n]*(30//len(mana)) for n in mana),[])
            with self.subTest(card=source),Game([lands+[source]*12+['Mental Modulation']*10+['Three Tree Mascot']*8,['Forest']*30+['Bear Cub']*30],seed=7,deck_rules='Freeform') as game:
                pending={};done=False;replayed=0;private=0;taps=0;seen=set()
                for _ in range(16000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome'],source);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};live=[v for v in objects.values() if v['owner']=='P0' and v['zone']=='Battlefield' and v['card']==source];mates=[v for v in objects.values() if v['owner']=='P0' and v['zone']=='Battlefield' and v['card']=='Three Tree Mascot']
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            if obs['actor']!='P0':return 260
                            n=objects[a['card']]['card'];return 280 if n==source and not live else 270 if n=='Three Tree Mascot' and live and not mates else 260 if n=='Mental Modulation' and live and not live[0]['tapped'] and (i!=8 or mates) else -100
                        if k=='PayCosts':return 500
                        if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                        if k=='PlayLand' and obs['actor']=='P0':return 330 if not any(v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==objects[a['card']]['card'] for v in objects.values()) else 300
                        if k=='ChooseTarget' and 'Card' in a['target']:
                            v=objects[a['target']['Card']];return 350 if obs['actor']=='P0' and v['card']==source and not v['tapped'] else 200
                        if k=='ChooseOptional':return 400 if a['accept'] else -100
                        if k=='Sacrifice':return 400 if objects[a['object']['card']]['card']=='Three Tree Mascot' else 200
                        if k=='DeclineEffectPayment':return -100
                        if k=='ArrangeLibrary':return 400 if a['destination']==('Graveyard' if i==7 else 'Bottom') else -100
                        if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if obs['library_choice'] is not None:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['library_choice']);self.assertEqual(other['actions'],[]);self.assertEqual(obs['library_choice']['surveil'],i==7);private+=1
                    if live or obs['stack'] or obs['announcing'] or obs['resolving']:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    for v in after['objects']:
                        if v['zone']=='Battlefield' and v['card']==source:
                            old=objects.get(v['id'])
                            if old and old['reference']==v['reference'] and not old['tapped'] and v['tapped']:taps+=1
                    items=after['stack']+after['waiting_triggers']+[s for s in [after['announcing'],after['resolving']] if s is not None]
                    for s in items:
                        if s['id'] in seen:continue
                        seen.add(s['id'])
                        if s['source_card']==source and s['kind']!='Spell' and s['kind']['TriggeredAbility']['ability']==0:self.assertIsNone(s['entering_object']);self.assertGreater(taps,0)
                    if obs['stack'] and action['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['source_card']==source and root['kind']!='Spell' and root['kind']['TriggeredAbility']['ability']==0 and root['id'] not in {s['id'] for s in after['stack']}:pending[root['id']]=(root,obs)
                    for stack_id,(root,before) in list(pending.items()):
                        if any(s['id']==stack_id for s in after['stack']) or after['resolving'] and after['resolving']['id']==stack_id:continue
                        old={v['id']:v for v in before['objects']};new={v['id']:v for v in after['objects']};draw=int(after['players'][0]['cards_drawn_this_turn'])-int(before['players'][0]['cards_drawn_this_turn']);discard=int(after['players'][0]['cards_discarded_this_turn'])-int(before['players'][0]['cards_discarded_this_turn'])
                        if i in (0,7):self.assertGreater(private,0);self.assertEqual(before['players'][0]['library_count']-after['players'][0]['library_count'],int(i==7))
                        elif i in (1,3,5,6,9):self.assertEqual((draw,discard),(1,1))
                        elif i in (2,4):
                            fresh=[v for v in new.values() if v['zone']=='Battlefield' and v['token'] is not None and not any(x['reference']==v['reference'] and x['zone']=='Battlefield' for x in old.values())];self.assertEqual(len(fresh),1);self.assertEqual(fresh[0]['token']['subtypes'],'Faerie' if i==2 else 'Lander');self.assertFalse(fresh[0]['tapped'])
                        elif i==8:
                            self.assertEqual(draw,1);removed=[v for v in old.values() if v['zone']=='Battlefield' and v['owner']=='P0' and not any(x['reference']==v['reference'] and x['zone']=='Battlefield' for x in new.values())];self.assertEqual(len(removed),1);self.assertEqual(removed[0]['card'],'Three Tree Mascot')
                        elif i==10:self.assertEqual(int(after['players'][0]['life']),int(before['players'][0]['life'])+2)
                        elif i==11:
                            target=root['targets'][0]['target']['Card'];self.assertEqual(int(new[target]['power']),int(old[target]['power'])-3);self.assertEqual(new[target]['toughness'],old[target]['toughness'])
                        elif i==12:self.assertEqual(draw,1);self.assertEqual(int(after['players'][0]['life']),int(before['players'][0]['life'])+1);self.assertTrue(new[root['card']]['keywords']&8)
                        done=True;break
                    if done:break
                self.assertTrue(done,source);self.assertGreater(replayed,0);self.assertGreater(taps,0)

    def test_fourteen_attack_bodies_complete_effects_and_rollback(self):
        names=['Ancestor Dragon','Armasaur Guide','Battlesong Berserker','Boggart Prankster','Jolene, Plundering Pugilist','Lionheart Glimmer','Malamet Brawler','Marshal of the Lost','Meddling Youths','Most Valuable Slayer','Nori, Teller of Tales','Razorkin Hordecaller','Seasoned Consultant','The Chief Warg']
        for i,name in enumerate(names):
            for hybrid in range(2 if i==10 else 1):
                lands=(['Plains','Mountain','Forest','Swamp']*10);mate='Gigantoad' if i==13 else "Dragonlord's Servant"
                with self.subTest(card=name,hybrid=hybrid),Game([lands+[name]*12+[mate]*24,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                    done=False;replayed=0;observed=set();chosen=False
                    for _ in range(20000):
                        obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};creatures=[v for v in objects.values() if v['zone']=='Battlefield' and v['owner']=='P0' and v['card'] in (name,mate)];live=any(v['card']==name for v in creatures);ready=live and len(creatures)>=3
                        def score(a):
                            k=a['kind']
                            if k in ('Cast','AnnounceCast'):
                                n=objects[a['card']]['card'];return 280 if n==name and not live else 250 if n==mate and len(creatures)<4 else -100
                            if k=='ChooseSpellCost':return 400 if a['cost']==hybrid else -100
                            if k=='PayCosts':return 500
                            if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                            if k=='Attack':return 220 if ready else -100
                            if k in ('Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                            return self.professor_score(obs,a)
                        j=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][j]
                        if action['kind']=='ChooseSpellCost':chosen=True
                        if live or obs['announcing'] or obs['stack']:
                            saved=game.snapshot()
                            with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),j)
                            self.assertEqual(game.step(obs,j),expected);game.restore(saved)
                            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,j)
                            after=game.step(game.observe(obs['actor']),j);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                        else:after=game.step(obs,j)
                        for item in after['stack']+after['waiting_triggers']+[v for v in [after['announcing'],after['resolving']] if v is not None]:
                            if item['source_card']==name and item['kind']!='Spell':self.assertIsNone(item['entering_object']);observed.add(item['id'])
                        if action['kind']=='Pass' and obs['stack']:
                            item=obs['stack'][-1]
                            if item['source_card']==name and item['kind']!='Spell' and all(v['id']!=item['id'] for v in after['stack']):
                                self.assertEqual(len(observed),1);before={v['reference']['card']:v for v in obs['objects']};now={v['id']:v for v in after['objects']};target=next((v['object']['card'] for v in item['targets'] if v['object'] is not None),None)
                                if i==0:self.assertEqual(int(after['players'][0]['life']),int(obs['players'][0]['life'])+len(obs['attackers']))
                                elif i==1:self.assertEqual(int(now[target]['counters']['plus_one']),int(before[target]['counters']['plus_one'])+1)
                                elif i in (2,3,6,7,9,10):
                                    dp,dt,keyword={2:(1,0,32768),3:(1,0,0),6:(0,0,256),7:(len(obs['attackers']),len(obs['attackers']),0),9:(1,0,512),10:(0,0,512)}[i]
                                    self.assertEqual(int(now[target]['power']),int(before[target]['power'])+dp);self.assertEqual(int(now[target]['toughness']),int(before[target]['toughness'])+dt);self.assertEqual(now[target]['keywords'],before[target]['keywords']|keyword)
                                elif i in (4,8,11):self.assertEqual(sum(v['zone']=='Battlefield' and v['id'] not in before for v in after['objects']),1)
                                elif i==5:
                                    for v in creatures:self.assertEqual(int(now[v['id']]['power']),int(v['power'])+1);self.assertEqual(int(now[v['id']]['toughness']),int(v['toughness'])+1)
                                elif i==12:
                                    source=next(v for v in creatures if v['card']==name);self.assertEqual(int(now[source['id']]['power']),int(source['power'])+2)
                                elif i==13:self.assertEqual(after['players'][0]['library_count']+1,obs['players'][0]['library_count']);self.assertEqual(int(after['players'][0]['life']),int(obs['players'][0]['life'])-1)
                                self.assertEqual(after['schema'],96);done=True;break
                    self.assertTrue(done,name);self.assertGreater(replayed,0)
                    if i==10:self.assertTrue(chosen)

    def test_opponent_entry_tapped_authority_spider_hybrid_and_suppression_replay(self):
        cases=[('Authority of the Consuls','Helpful Hunter',False,0,True),('Authority of the Consuls','Helpful Hunter',True,0,True),('Authority of the Consuls','Mephitic Draught',False,0,False),('Spider-Woman, Stunning Savior','Helpful Hunter',False,0,True),('Spider-Woman, Stunning Savior','Mephitic Draught',False,1,True)]
        for source,wanted,orb,cost,tapped in cases:
            lands=['Island']*30 if cost else ['Plains']*30
            deck=lands+[source]*12+(['Torpor Orb']*18 if orb else ['Bear Cub']*18)
            opponent=['Plains']*30+['Helpful Hunter']*30 if wanted=='Helpful Hunter' else ['Swamp']*30+['Mephitic Draught']*30
            with self.subTest(source=source,entrant=wanted,orb=orb,cost=cost),Game([deck,opponent],seed=7,deck_rules='Freeform') as game:
                done=False;entered=None;replayed=0;chosen_cost=False;life_before=None;trigger_ids=set()
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']}
                    live=any(v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==source for v in objects.values());orb_live=any(v['zone']=='Battlefield' and v['card']=='Torpor Orb' for v in objects.values());ready=live and (not orb or orb_live)
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if obs['actor']=='P0':return 280 if n==source and not live else 270 if n=='Torpor Orb' and live and not orb_live else -100
                            return 260 if ready and entered is None else -100
                        if k=='ChooseSpellCost':return 400 if next(c['cost']['colored'][cost] for c in obs['cost_choices'] if c['index']==a['cost'])==1 else -100
                        if k=='PayCosts':return 500
                        if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                        if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    index=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));action=obs['actions'][index]
                    if action['kind']=='ChooseSpellCost' and obs['actor']=='P0':self.assertEqual(next(c['cost']['colored'][cost] for c in obs['cost_choices'] if c['index']==action['cost']),1);chosen_cost=True
                    if obs['announcing'] or obs['stack'] or live:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),index)
                        self.assertEqual(game.step(obs,index),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,index)
                        after=game.step(game.observe(obs['actor']),index);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,index)
                    items=after['stack']+after['waiting_triggers']+[v for v in [after['announcing'],after['resolving']] if v is not None]
                    new=[v for v in after['objects'] if v['zone']=='Battlefield' and v['owner']=='P1' and v['card']==wanted and not any(old['zone']=='Battlefield' and old['reference']==v['reference'] for old in obs['objects'])]
                    if new:
                        self.assertTrue(ready);self.assertEqual(len(new),1);entered=new[0]['reference'];self.assertEqual(new[0]['tapped'],tapped);life_before=int(obs['players'][0]['life'])
                        if source.startswith('Spider-Woman'):self.assertTrue(next(v for v in after['objects'] if v['zone']=='Battlefield' and v['card']==source)['keywords']&1)
                    if entered is not None:
                        for item in items:
                            if item['source_card']==source and item['kind']!='Spell':self.assertEqual(item['entering_object'],entered);self.assertEqual(item['targets'],[]);trigger_ids.add(item['id'])
                        if not after['stack'] and not after['waiting_triggers'] and after['announcing'] is None and after['resolving'] is None and after['decision']=='priority':
                            gain=source=='Authority of the Consuls' and wanted=='Helpful Hunter' and not orb
                            self.assertEqual(len(trigger_ids),int(gain));self.assertEqual(int(after['players'][0]['life']),life_before+int(gain));self.assertEqual(after['schema'],96);done=True;break
                self.assertTrue(done,(source,wanted,orb,cost));self.assertGreater(replayed,0)
                if source.startswith('Spider-Woman'):self.assertTrue(chosen_cost)

    def test_three_entering_object_bodies_capture_non_target_refs_and_replay(self):
        for i,source in enumerate(['Good-Fortune Unicorn','Serra Redeemer','Wartime Protestors']):
            mate='First-Time Flyer' if i==2 else 'Bear Cub';colors=['Island','Mountain'] if i==2 else ['Plains','Forest'];lands=sum(([n]*15 for n in colors),[])
            with self.subTest(source=source),Game([lands+[source]*12+[mate]*18,['Forest']*30+['Bear Cub']*30],seed=7,deck_rules='Freeform') as game:
                done=False;replayed=0
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome'],source);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};live=any(v['zone']=='Battlefield' and v['card']==source for v in objects.values())
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast') and obs['actor']=='P0':
                            n=objects[a['card']]['card'];return 270 if n==source and not live else 260 if n==mate and live else -100
                        if k=='PayCosts':return 500
                        if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                        if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        if k=='PlayLand' and obs['actor']=='P0':
                            n=objects[a['card']]['card'];count=sum(v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==n for v in objects.values());need=2 if i==1 and n=='Plains' else 1;return 320 if count<need else 300
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if obs['announcing'] or obs['stack'] or live:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if obs['stack'] and action['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['controller']=='P0' and root['source_card']==source and root['kind']!='Spell' and root['id'] not in {v['id'] for v in after['stack']}:
                            self.assertEqual(root['targets'],[]);ref=root['entering_object'];self.assertIsNotNone(ref);old=objects[ref['card']];self.assertEqual(old['reference'],ref);self.assertEqual(old['card'],mate);new=next(v for v in after['objects'] if v['reference']==ref);self.assertEqual(new['counters']['plus_one'],old['counters']['plus_one']+(2 if i==1 else 1));self.assertEqual(bool(new['keywords']&4),i==2);self.assertEqual(after['schema'],96);done=True;break
                self.assertTrue(done,source);self.assertGreater(replayed,0)

    def test_nine_entry_predicate_bodies_effects_private_surveil_and_replay(self):
        names=['HYDRA Assault Robot','Impact Tremors','Marketwatch Phantom','Neighborhood Guardian','Snarling Gorehound','Valley Mightcaller','Vial Smasher, Gleeful Grenadier','Vicious Clown',"Warleader's Call"]
        for i,source in enumerate(names):
            mate='Three Tree Mascot' if i==0 else 'Gigantoad' if i==5 else 'Swab Goblin' if i==6 else 'Bear Cub'
            colors=['Mountain'] if i==0 else ['Forest'] if i==5 else ['Swamp','Mountain'] if i==6 else ['Plains','Mountain','Forest'] if i==8 else ['Swamp','Forest'] if i==4 else ['Plains','Forest'] if i in (2,3) else ['Mountain','Forest']
            lands=sum(([n]*(30//len(colors)) for n in colors),[])
            with self.subTest(card=source),Game([lands+[source]*12+[mate]*18,['Forest']*30+['Bear Cub']*30],seed=7,deck_rules='Freeform') as game:
                done=False;replayed=0;private=0;pending={}
                for _ in range(16000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome'],source);obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']};live=any(v['zone']=='Battlefield' and v['card']==source for v in objects.values())
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast') and obs['actor']=='P0':
                            n=objects[a['card']]['card'];return 270 if n==source and not live else 260 if n==mate and live else -100
                        if k=='PayCosts':return 500
                        if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                        if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        if k=='PlayLand' and obs['actor']=='P0':
                            n=objects[a['card']]['card'];count=sum(v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==n for v in objects.values());return 320 if count==0 else 300
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if obs['library_choice'] is not None:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['library_choice']);self.assertEqual(other['actions'],[]);private+=1
                    if obs['announcing'] or obs['resolving'] or obs['stack'] or live:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if obs['stack'] and action['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['controller']=='P0' and root['source_card']==source and root['kind']!='Spell' and root['id'] not in {x['id'] for x in after['stack']}:pending[root['id']]=(root,obs)
                    for stack_id,(root,before) in list(pending.items()):
                        if any(x['id']==stack_id for x in after['stack']) or after['resolving'] and after['resolving']['id']==stack_id:continue
                        old={v['id']:v for v in before['objects']};new={v['id']:v for v in after['objects']};v=new[root['card']]
                        if i in (0,1,6,8):self.assertEqual(after['players'][1]['life'],int(before['players'][1]['life'])-1)
                        elif i==2:self.assertTrue(v['keywords']&1)
                        elif i==3:
                            target=root['targets'][0]['target']['Card'];self.assertEqual(new[target]['power'],int(old[target]['power'])+1);self.assertEqual(new[target]['toughness'],int(old[target]['toughness'])+1)
                        elif i==4:self.assertGreater(private,0);self.assertIn(before['players'][0]['library_count']-after['players'][0]['library_count'],[0,1])
                        elif i==5:self.assertEqual(v['counters']['plus_one'],int(old[root['card']]['counters']['plus_one'])+1)
                        elif i==7:self.assertEqual(v['power'],int(old[root['card']]['power'])+2)
                        done=True;break
                    if done:break
                self.assertTrue(done,source);self.assertGreater(replayed,0)

    def test_entry_suppression_three_sources_creature_artifact_and_enchantment_replay(self):
        for source in ['Torpor Orb','Doorkeeper Thrull','Karn, Argent Defender']:
            for wanted in ['Helpful Hunter','Mephitic Draught','Hopeful Vigil']:
                with self.subTest(source=source,entrant=wanted),Game([['Plains']*15+['Swamp']*15+[source]*12+[wanted]*18,['Forest']*30+['Bear Cub']*30],seed=7,deck_rules='Freeform') as game:
                    replayed=0;pending=None;entered=False;done=False
                    for _ in range(16000):
                        obs=game.observe('P0');self.assertIsNone(obs['outcome'],(source,wanted,entered));obs=game.observe(obs['actor']);objects={v['id']:v for v in obs['objects']}
                        live=any(v['zone']=='Battlefield' and v['card']==source for v in objects.values())
                        def score(a):
                            k=a['kind']
                            if k in ('Cast','AnnounceCast') and obs['actor']=='P0':
                                n=objects[a['card']]['card'];return 270 if n==source and not live else 260 if n==wanted and live else -100
                            if k=='PayCosts':return 500
                            if k=='TapMana':return 150 if obs['decision']=='pay_costs' else -100
                            if k in ('Attack','Block','Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                            if k=='PlayLand' and obs['actor']=='P0':
                                n=objects[a['card']]['card'];count=sum(v['zone']=='Battlefield' and v['owner']=='P0' and v['card']==n for v in objects.values());return 320 if count==0 else 300
                            return self.professor_score(obs,a)
                        choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                        if obs['announcing'] or obs['stack'] or live:
                            saved=game.snapshot()
                            with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                            self.assertEqual(game.step(obs,choice),expected);game.restore(saved)
                            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                            after=game.step(game.observe(obs['actor']),choice);expected['revision']=after['revision'];self.assertEqual(after,expected);game.drop_snapshot(saved);replayed+=1
                        else:after=game.step(obs,choice)
                        new=[v for v in after['objects'] if v['zone']=='Battlefield' and v['card']==wanted and not any(b['zone']=='Battlefield' and b['reference']==v['reference'] for b in obs['objects'])]
                        if new and live:
                            entered=True
                            items=after['stack']+after['waiting_triggers']+[v for v in [after['announcing'],after['resolving']] if v is not None]
                            triggers={v['id'] for v in items if v['source_card']==wanted and v['kind']!="Spell"}
                            allowed=wanted=='Hopeful Vigil' or wanted=='Mephitic Draught' and source=='Torpor Orb'
                            self.assertEqual(len(triggers),int(allowed),(source,wanted,after['decision']))
                            if not allowed:done=True;break
                            pending=(obs['players'][0]['library_count'],obs['players'][0]['life'])
                        if pending and not after['stack'] and after['resolving'] is None and after['announcing'] is None and after['decision']=='priority':
                            if wanted=='Mephitic Draught':self.assertEqual(after['players'][0]['library_count'],pending[0]-1);self.assertEqual(after['players'][0]['life'],int(pending[1])-1)
                            else:self.assertEqual(sum(v['zone']=='Battlefield' and v['token'] is not None and v['token']['subtypes']=='Knight' for v in after['objects']),1)
                            done=True;break
                    self.assertTrue(entered and done,(source,wanted));self.assertGreater(replayed,0)

    def test_twelve_fixed_effect_bodies_modes_private_discard_and_clue_replay(self):
        names=['Callous Inspector', 'Cosmium Blast', 'Deadly Riposte', 'Defibrillating Current', 'Fear of Lost Teeth', 'Joust Through', 'Keep Out', 'Lightning Helix', 'Ravenous Giant', 'Risky Shortcut', 'Romantic Rendezvous', 'Viashino Pyromancer'];lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
        for i,mode in [(j,0) for j in range(12)]+[(6,1)]:
            wanted=names[i]
            pairs=([('Swamp',12),('Mountain',18)] if i in (0,4,9) else [('Mountain',30)] if i in (8,10,11) else [('Plains',15),('Mountain',15)] if i==7 else [('Plains',10),('Mountain',10),('Swamp',10)] if i==3 else [('Plains',20),('Mountain',10)])
            lands=sum(([n]*amount for n,amount in pairs),[])
            with self.subTest(card=wanted,mode=mode),Game([lands+[wanted]*12+['Shock']*8+['Helpful Hunter']*10,['Mountain']*12+['Swamp']*12+['Ravenous Giant']*18+['Fear of Lost Teeth']*18],seed=7,deck_rules='Freeform') as game:
                casted=False;done=False;clue_done=False;replayed=0;private=0;pending={}
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome'],(i,casted,done,clue_done));obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};sources=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield' and o['card']==wanted]
                    def score(a):
                        k=a['kind']
                        if k=='PlayLand' and obs['actor']=='P0':
                            required=({'Swamp':1,'Mountain':1} if i in (0,4) else {'Plains':1,'Mountain':1,'Swamp':1} if i==3 else {'Plains':1,'Mountain':1} if i==7 else {'Mountain':2} if i==8 else {'Swamp':1} if i==9 else {'Mountain':1} if i in (10,11) else {'Plains':1})
                            n=objects[a['card']]['card'];count=sum(v['owner']=='P0' and v['zone']=='Battlefield' and v['card']==n for v in objects.values());return 220 if count<required.get(n,0) else 200
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if obs['actor']=='P1':return 230
                            if n==wanted:
                                if k=='Cast' and i in (7,11) and a['target']!={'Player':'P1'}:return -100
                                return 260 if not casted else -100
                            if n=='Shock':
                                if i not in (0,4) or not sources:return -100
                                if k=='Cast' and a['target']!={'Card':sources[0]['id']}:return -100
                                return 250
                            return -100
                        if k=='AnnounceActivate':return 280 if i==0 and not clue_done and objects[a['card']]['token'] is not None and objects[a['card']]['token']['subtypes']=='Clue' else -100
                        if k=='ChooseMode':return 320 if a['mode']==mode else -100
                        if k=='ChooseSpellCost':
                            c=next(c['cost'] for c in obs['cost_choices'] if c['index']==a['cost']);return 230-c['generic']-sum(c['colored'])
                        if k=='ChooseTarget':
                            announcing=obs['announcing'];target=a['target']
                            if obs['actor']=='P0' and announcing and announcing['source_card']=='Shock' and i in (0,4):return 300 if 'Card' in target and objects[target['Card']]['card']==wanted and objects[target['Card']]['owner']=='P0' else -100
                            if obs['actor']=='P0' and i in (4,7,11) and target.get('Player')=='P1':return 310
                            if 'Card' in target:return 290 if objects[target['Card']]['owner']!=obs['actor'] else 100
                            return 250 if target.get('Player')!=obs['actor'] else 50
                        if k=='Attack':return 240 if obs['actor']=='P1' else -100
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if obs['actor']=='P0' and action['kind'] in ('Cast','AnnounceCast') and objects[action['card']]['card']==wanted:casted=True
                    if obs['decision']=='effect_discard':
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertEqual(other['effect_discards'],[]);self.assertEqual(other['actions'],[]);private+=1
                    if obs['announcing'] or obs['resolving'] or obs['stack']:
                        saved=game.snapshot()
                        with game.clone() as fork:after=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),after);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if obs['stack'] and action['kind']=='Pass':
                        root=obs['stack'][-1];is_clue=i==0 and root['source_card'].startswith('@token/') and root['kind']=={'ActivatedAbility':{'ability':0}}
                        if root['controller']=='P0' and (root['source_card']==wanted or is_clue) and root['id'] not in {v['id'] for v in after['stack']} and (is_clue or i not in (0,4,8,11) or root['kind']!='Spell'):pending[root['id']]=(root,obs,is_clue)
                    for root_id in list(pending):
                        if any(v['id']==root_id for v in after['stack']) or after['resolving'] and after['resolving']['id']==root_id:continue
                        root,b,is_clue=pending.pop(root_id);old_objects={v['id']:v for v in b['objects']};new_objects={v['id']:v for v in after['objects']}
                        if is_clue:
                            self.assertEqual(after['players'][0]['library_count'],b['players'][0]['library_count']-1);self.assertEqual(after['players'][0]['hand_count'],b['players'][0]['hand_count']+1);clue_done=True;continue
                        if i in (0,8):
                            self.assertEqual(after['players'][0]['life'],b['players'][0]['life']-1)
                            if i==0:self.assertEqual(sum(v['owner']=='P0' and v['zone']=='Battlefield' and v['token'] is not None and v['token']['subtypes']=='Clue' for v in after['objects']),1)
                        elif i in (9,10):
                            self.assertEqual(after['players'][0]['library_count'],b['players'][0]['library_count']-2)
                            if i==9:
                                self.assertEqual(after['players'][0]['life'],b['players'][0]['life']-2);self.assertEqual(after['players'][1]['life'],b['players'][1]['life']-2)
                            else:self.assertEqual(after['players'][0]['hand_count'],b['players'][0]['hand_count']-int(b['players'][0]['hand_count']>0)+2)
                        elif i==6 and mode==1:
                            ref=root['targets'][0]['object'];self.assertEqual(new_objects[ref['card']]['zone'],'Graveyard')
                        else:
                            damage={1:4,2:3,3:4,4:1,5:3,6:4,7:3,11:2}[i];gain={2:2,3:2,4:1,5:1,7:3}.get(i,0);bound=root['targets'][0]
                            if bound['object']:
                                ref=bound['object'];old=old_objects[ref['card']];new=new_objects[ref['card']]
                                if old['damage']+damage>=old['toughness']:self.assertEqual(new['zone'],'Graveyard')
                                else:self.assertEqual(new['damage'],old['damage']+damage)
                            else:self.assertEqual(after['players'][1]['life'],b['players'][1]['life']-damage)
                            self.assertEqual(after['players'][0]['life'],b['players'][0]['life']+gain)
                        done=True
                    if done and (i!=0 or clue_done):break
                else:self.fail('fixed-effect body did not finish')
                self.assertTrue(casted);self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)
                if i==10:self.assertGreater(private,0)

    def test_eight_filtered_entry_bodies_modes_activations_and_private_replay(self):
        names=['Arahbo, the First Fang', 'Arni, Humble Scribe', 'Black Panther, Vanguard', 'Genghis Frog', 'Knightfisher', 'Lathliss, Dragon Queen', 'Pactdoll Terror', 'Thoughtweft Lieutenant'];helpers=['Helpful Hunter','Bear Cub','Agent of Atlas','Mutant Town Musicians',"Healer's Hawk",'Shivan Dragon','Goldvein Pick','Goldmeadow Nomad']
        for i,mode in [(j,0) for j in range(8)]+[(2,1)]:
            wanted=names[i];helper=helpers[i];lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            with self.subTest(card=wanted,mode=mode),Game([lands+[wanted]*12+[helper]*18,['Forest']*36+['Bear Cub']*24],seed=7,deck_rules='Freeform') as game:
                entry=False;activation=False;replayed=0;pending={};private=0
                for _ in range(16000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};board=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];sources=[o for o in board if o['card']==wanted];helper_board=[o for o in board if o['card']==helper]
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources else -100
                            if n==helper:return 240 if sources and not helper_board and (i!=1 or activation) else -100
                            return -100
                        if k=='AnnounceActivate':
                            n=objects[a['card']]['card'];v=objects[a['card']]
                            return 245 if not activation and ((i==1 and n==wanted) or (i==5 and n==wanted and entry) or (i==3 and v['token'] is not None and v['token']['subtypes']=='Mutagen' and entry)) else -100
                        if k=='ChooseMode':return 300 if a['mode']==mode else -100
                        if k=='ChooseTarget' and 'Card' in a['target']:return 280 if objects[a['target']['Card']]['card']==wanted else 100
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    if obs['decision']=='effect_discard':
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertEqual(other['effect_discards'],[]);self.assertEqual(other['actions'],[]);private+=1
                    if obs['announcing'] or obs['resolving'] or obs['stack']:
                        saved=game.snapshot()
                        with game.clone() as fork:after=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),after);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if obs['stack'] and action['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['kind']!='Spell' and root['id'] not in {v['id'] for v in after['stack']}:
                            is_mutagen=i==3 and root['source_card'].startswith('@token/') and root['kind']=={'ActivatedAbility':{'ability':0}}
                            if root['source_card']==wanted or is_mutagen:pending[root['id']]=(root,obs,bool(helper_board),is_mutagen)
                    for root_id in list(pending):
                        if any(v['id']==root_id for v in after['stack']) or after['resolving'] and after['resolving']['id']==root_id:continue
                        root,b,had_helper,is_mutagen=pending.pop(root_id);before_objects={v['id']:v for v in b['objects']};after_objects={v['id']:v for v in after['objects']}
                        if 'ActivatedAbility' in root['kind']:
                            activation=True
                            if i==1:self.assertEqual(after['players'][0]['library_count'],b['players'][0]['library_count']-1);self.assertEqual(after['players'][0]['hand_count'],b['players'][0]['hand_count'])
                            if i==5:
                                old=before_objects[root['card']];new=after_objects[root['card']];self.assertEqual((new['power'],new['toughness']),(old['power']+1,old['toughness']))
                            if is_mutagen:
                                ref=root['targets'][0]['object'];self.assertEqual(after_objects[ref['card']]['counters']['plus_one'],before_objects[ref['card']]['counters']['plus_one']+1)
                        else:
                            if i in (0,3,4,5) or i==2 and mode==0:
                                subtype={0:'Cat',2:'Soldier',3:'Mutagen',4:'Fish',5:'Dragon'}[i];count=lambda o:sum(v['owner']=='P0' and v['zone']=='Battlefield' and v['token'] is not None and v['token']['subtypes']==subtype for v in o['objects']);self.assertEqual(count(after),count(b)+1)
                            if i==1:self.assertFalse(after_objects[root['card']]['tapped'])
                            if i==2 and mode==1:
                                for old in b['objects']:
                                    if old['owner']=='P0' and old['zone']=='Battlefield' and old['card'] in (wanted,helper):new=after_objects[old['id']];self.assertEqual((new['power'],new['toughness']),(old['power']+1,old['toughness']+1))
                            if i==6:self.assertEqual(after['players'][0]['life'],b['players'][0]['life']+1);self.assertEqual(after['players'][1]['life'],b['players'][1]['life']-1)
                            if i==7:
                                ref=root['targets'][0]['object'];old=before_objects[ref['card']];new=after_objects[ref['card']];self.assertEqual((new['power'],new['toughness']),(old['power']+1,old['toughness']+1));self.assertTrue(new['keywords']&256)
                            if had_helper:entry=True
                    if entry and (activation or i not in (1,3,5)):break
                else:self.fail('full filtered-entry body did not complete')
                self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)
                if i==1:self.assertGreater(private,0)

    def test_six_alliance_bodies_cast_and_replay_independent_trigger_effects(self):
        names=['EPF Point Squad', 'East Wind Avatar', 'Mighty Mutanimals', 'Mutant Town Musicians', 'Raphael, Tough Turtle', 'Slash, Reptile Rampager']
        for i,wanted in enumerate(names):
            with self.subTest(card=wanted),Game([['Plains']*16+['Mountain']*14+[wanted]*12+['Helpful Hunter']*18,['Forest']*36+['Bear Cub']*24],seed=7,deck_rules='Freeform') as game:
                replayed=0;finished=False;token_effect=False
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};sources=[o for o in objects.values() if o['card']==wanted and o['owner']=='P0' and o['zone']=='Battlefield']
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources else -100
                            if n=='Helpful Hunter':return 240 if sources and i!=5 else -100
                            return -100
                        if k=='Attack':return 240 if i==5 and objects[a['card']]['card']==wanted else -100
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k=='ChooseTarget' and 'Card' in a['target']:return 230 if objects[a['target']['Card']]['card']==wanted else 100
                        if k in ('Activate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));action=obs['actions'][choice]
                    boundary=obs['announcing'] or obs['stack'] or obs['resolving']
                    if boundary:
                        saved=game.snapshot()
                        with game.clone() as fork:after=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),after);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    if obs['stack'] and action['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['source_card']==wanted and root['kind']!='Spell' and root['id'] not in {v['id'] for v in after['stack']}:
                            before_objects={v['id']:v for v in obs['objects']};after_objects={v['id']:v for v in after['objects']};ability=root['kind']['TriggeredAbility']['ability']
                            if (i==2 and ability==0) or (i==5 and ability==1):
                                count=lambda o:sum(v['owner']=='P0' and v['zone']=='Battlefield' and v['token'] is not None and v['token']['subtypes']=='Mutant' for v in o['objects'])
                                self.assertEqual(count(after),count(obs)+1);token_effect=True
                            else:
                                self.assertEqual(ability,1 if i==2 else 0)
                                if i in (0,2):
                                    ref=root['object'] if i==0 else root['targets'][0]['object'];b=before_objects[ref['card']];a=after_objects[ref['card']];self.assertEqual(a['counters']['plus_one'],b['counters']['plus_one']+1)
                                elif i in (1,3):
                                    b=before_objects[root['card']];a=after_objects[root['card']];self.assertEqual((a['power'],a['toughness']),(b['power']+1,b['toughness']))
                                else:self.assertEqual(after['players'][1]['life'],obs['players'][1]['life']-(1 if i==4 else 2))
                                finished=True
                    if finished:break
                else:self.fail('Alliance body never resolved')
                self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)
                if i in (2,5):self.assertTrue(token_effect)

    def test_twelve_modal_trigger_bodies_choose_before_targets_and_replay_private_programs(self):
        names=['Apothecary Stomper', 'Coliseum Behemoth', 'Cosmogrand Zenith', 'Divining Duelist', 'Downwind Ambusher', 'Elven Raft-Steerer', 'Ferocification', 'Gaius van Baelsar', 'Giant-Sized Flying Ant', 'Glamermite', "Lord Skitter's Butcher", 'Wardens of the Cycle'];preferred=[0,1,0,2,0,1,1,1,1,1,1,1]
        for i,wanted in enumerate(names):
            helper='Reckless Lackey' if i==11 else 'Opt';lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            with self.subTest(card=wanted),Game([lands+[wanted]*12+['Bear Cub']*10+[helper]*8,['Forest']*36+['Bear Cub']*24],seed=7,deck_rules='Freeform') as game:
                pending={};replayed=0;chosen=0;finished=False
                for _ in range(16000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};board=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];sources=[o for o in board if o['card']==wanted];bears=[o for o in board if o['card']=='Bear Cub'];helpers=[o for o in board if o['card']==helper]
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if bears and not sources else -100
                            if n=='Bear Cub':return 240 if not any(o['card']==n and o['zone']=='Battlefield' and o['owner']==obs['actor'] for o in objects.values()) else -100
                            if n==helper:return 230 if sources and (n=='Opt' or not helpers) else -100
                            return -100
                        if k=='ChooseMode':return 300 if a['mode']==preferred[i] else -100
                        if k=='ChooseTarget' and 'Card' in a['target']:
                            o=objects[a['target']['Card']];return 290 if (i in (0,5,6) and o['owner']=='P0' and o['card']=='Bear Cub') or (i in (4,8,9) and o['owner']=='P1') else 100
                        if k=='AnnounceActivate':return 240 if i==11 and sources and objects[a['card']]['card']==helper and not obs['stack'] else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k=='ChooseOptional':return 220 if a['accept'] else -100
                        if k=='Sacrifice':return 260 if objects[a['object']['card']]['card']=='Bear Cub' else 240
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));a=obs['actions'][choice];private=obs['library_choice'] or obs['decision']=='effect_discard'
                    if obs['mode_choices']:
                        self.assertEqual(obs['announcing']['source_card'],wanted);self.assertNotEqual(obs['announcing']['kind'],'Spell');self.assertIsNone(obs['announcing']['mode']);self.assertIsNone(obs['payment_request']);self.assertFalse(any(v['kind']=='ChooseTarget' for v in obs['actions']));other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertEqual(other['mode_choices'],[]);self.assertEqual(other['actions'],[]);self.assertEqual(a['kind'],'ChooseMode');self.assertEqual(a['mode'],preferred[i]);chosen+=1
                    if private:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['library_choice']);self.assertEqual(other['effect_discards'],[]);self.assertEqual(other['actions'],[])
                    if obs['announcing'] or obs['resolving'] or obs['stack'] or private:
                        saved=game.snapshot()
                        with game.clone() as fork:after=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),after);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    live={v['id'] for v in after['stack']}
                    if after['resolving']:live.add(after['resolving']['id'])
                    if obs['stack'] and a['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['source_card']==wanted and root['kind']!='Spell' and root['id'] not in {v['id'] for v in after['stack']}:
                            self.assertEqual(root['mode'],preferred[i]);pending[root['id']]=(root,obs)
                    for root_id in list(pending):
                        if root_id in live:continue
                        root,before=pending.pop(root_id);public=game.observe('P0');before_objects={v['id']:v for v in before['objects']};after_objects={v['id']:v for v in public['objects']};player_before=before['players'][0];player_after=public['players'][0]
                        if i in (1,3,10,11):self.assertEqual(player_after['library_count'],player_before['library_count']-1)
                        if i==11:self.assertEqual(player_after['life'],player_before['life']-1)
                        if i==7:self.assertGreater(public['creatures_died_this_turn'],before['creatures_died_this_turn'])
                        if i==2:self.assertEqual(sum(1 for v in public['objects'] if v['owner']=='P0' and v['zone']=='Battlefield' and v['token'] and v['token']['subtypes']=='Human Soldier'),2)
                        for bound in root['targets']:
                            ref=bound['object'];old=before_objects[ref['card']];new=after_objects[ref['card']]
                            if i==0:self.assertEqual(new['counters']['plus_one'],old['counters']['plus_one']+2)
                            if i in (5,8,9):self.assertFalse(new['tapped'])
                            if i==6:self.assertEqual(new['keywords'],old['keywords']|32768|4)
                            if i==4:self.assertEqual((new['power'],new['toughness']),(old['power']-1,old['toughness']-1))
                        finished=True
                    if finished:break
                else:self.fail('modal trigger body not finished')
                self.assertGreater(chosen,0);self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)

    def test_eighteen_targeted_bodies_resolve_exact_effects_with_private_payment_replay(self):
        names=['Bigfin Bouncer', 'Bounce Off', 'Constrictor Sage', 'Exclusion Mage', 'Exosuit Savior', 'Freeze in Place', 'Grappling Kraken', 'Iceridge Serpent', 'Mental Modulation', 'Mischievous Pup', 'Peerless Ropemaster', "Proctor's Gaze", 'Rimekin Recluse', 'Ring of the Lucii', 'Rowdy Snowballers', 'Spider-Byte, Web Warden', 'Stickytongue Sentinel', 'Tidecaller Mentor']
        bounce={0,1,3,4,7,9,10,11,12,15,16,17};spells={1,5,8,11}
        for i,wanted in enumerate(names):
            lands=sum(([n]*8 for n in ['Island','Plains','Forest','Swamp']),[]);deck=lands+[wanted]*8+['Bear Cub']*8+['Mind Drill Assailant']*8+['Opt']*4
            with self.subTest(card=wanted),Game([deck,['Forest']*36+['Bear Cub']*24],seed=7,deck_rules='Freeform') as game:
                expected={};replayed=0;verified=0
                for _ in range(16000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0'];sources=[o for o in own if o['zone']=='Battlefield' and o['card']==wanted];grave=[o for o in own if o['zone']=='Graveyard' and not o['token']]
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources and (i!=17 or len(grave)>=7) else -100
                            if n=='Mind Drill Assailant':return 230 if i==17 and not any(o['card']==n and o['zone']=='Battlefield' for o in own) else -100
                            if n=='Bear Cub':return 200 if not any(o['card']==n and o['zone']=='Battlefield' and o['owner']==obs['actor'] for o in objects.values()) else -100
                            return -100
                        if k=='AnnounceActivate':return 240 if not obs['stack'] and ((i==13 and objects[a['card']]['card']==wanted and a['ability']==1) or (i==17 and objects[a['card']]['card']=='Mind Drill Assailant' and len(grave)<7)) else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k=='ChooseTarget' and 'Card' in a['target']:
                            o=objects[a['target']['Card']];return 290 if o['owner']!=obs['actor'] else 270 if o['card']=='Bear Cub' else 260
                        if k=='ChooseSearch':return 300
                        if k=='ArrangeLibrary':return 220 if a['destination']=='Graveyard' else 100
                        if k=='Attack':return 200 if i==10 and obs['actor']=='P1' else -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));a=obs['actions'][choice];private=obs['library_choice'] or obs['search_choice'] or obs['decision']=='effect_discard'
                    if private:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertEqual(other['actions'],[]);self.assertIsNone(other['library_choice']);self.assertIsNone(other['search_choice']);self.assertEqual(other['effect_discards'],[])
                    if obs['announcing'] or obs['resolving'] or obs['stack'] or private:
                        saved=game.snapshot()
                        with game.clone() as fork:after=fork.step(fork.observe(obs['actor']),choice)
                        self.assertEqual(game.step(obs,choice),after);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved);replayed+=1
                    else:after=game.step(obs,choice)
                    live={r['id'] for r in after['stack']}
                    if after['resolving']:live.add(after['resolving']['id'])
                    if obs['stack'] and a['kind']=='Pass':
                        root=obs['stack'][-1]
                        if root['source_card']==wanted and root['id'] not in {r['id'] for r in after['stack']} and (root['kind']!='Spell' or i in spells):
                            checks=[]
                            for bound in root['targets']:
                                ref=bound['object'];o=objects.get(ref['card']) if ref else None
                                if o and o['reference']==ref and o['zone']=='Battlefield' and (i!=10 or o['tapped']):checks.append((ref,o['counters']['stun']))
                            expected[root['id']]=checks
                    for root_id in list(expected):
                        if root_id in live:continue
                        for ref,stun in expected.pop(root_id):
                            public=game.observe('P0');o=next(o for o in public['objects'] if o['id']==ref['card'])
                            if i in bounce:self.assertEqual(o['zone'],'Hand');self.assertEqual(o['reference']['generation'],ref['generation']+1);self.assertIn(o['reference'],public['known_hand'])
                            else:self.assertEqual(o['reference'],ref);self.assertTrue(o['tapped']);self.assertEqual(o['counters']['stun'],stun+(3 if i==5 else 1 if i in (2,6,14) else 0))
                            verified+=1
                    if verified:break
                else:self.fail('complete targeted body not resolved')
                self.assertGreater(replayed,0);self.assertEqual(after['schema'],96)

    def test_thirteen_source_predicate_bodies_reach_independent_live_conditions_and_replay(self):
        names=['Akawalli, the Seething Tower','Cephalid Inkmage','Druid of the Spade','First-Time Flyer','Foggy Swamp Hunters','Ghitu Lavarunner','June, Bounty Hunter','Kitesail Corsair','Messenger Hawk','Nightwhorl Hermit','Skyward Spider','Stoic Sphinx','Twinblade Paladin']
        bases=[(3,3),(2,2),(2,3),(1,2),(3,4),(1,2),(2,2),(2,1),(1,2),(1,4),(2,2),(5,3),(3,3)];bonuses=[(2,2),(0,0),(2,0),(1,1),(0,0),(1,0),(0,0),(0,0),(2,0),(1,0),(0,0),(0,0),(0,0)];flags=[256,16384,256,0,64|32768,4,16384,1,0,16384,1,4096,1024];intrinsics=[0,0,0,1,0,0,0,0,1,8,0,1|32,0]
        definitions={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']}
        def types(n):return set(definitions[n]['type_line'].split(' — ')[0].split())
        def subtype(n,t):return t in definitions[n]['type_line'].split(' — ')[-1].split() if ' — ' in definitions[n]['type_line'] else False
        for i,wanted in enumerate(names):
            enabler='Mind Drill Assailant' if i in (0,1,9) else 'Messenger Hawk' if i==2 else 'Abandon Attachments' if i==3 else 'Goldvein Pick' if i==10 else 'Opt'
            lands=sum(([n]*8 for n in ['Tranquil Cove','Blossoming Sands','Scoured Barrens','Dismal Backwater','Mountain']),[])
            with self.subTest(card=wanted),Game([lands+[wanted]*12+[enabler]*8,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                replayed=0
                for _ in range(22000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0'];board=[o for o in own if o['zone']=='Battlefield'];sources=[o for o in board if o['card']==wanted];helpers=[o for o in board if o['card']==enabler];grave=[o for o in own if o['zone']=='Graveyard' and not o['token']];permanents=sum(bool(types(o['card'])&{'Artifact','Creature','Enchantment','Land','Planeswalker','Battle'}) for o in grave);player=obs['players'][0]
                    on=False
                    for source in sources:
                        attached=[o for o in objects.values() if o['zone']=='Battlefield' and o['attached_to']==source['reference']];c=source['counters'];modified=any(c.values()) or any(subtype(o['card'],'Equipment') or (subtype(o['card'],'Aura') and o['owner']==source['owner']) for o in attached)
                        on=(permanents>=4 if i==0 else len(grave)>=7 if i in (1,9) else any(o['token'] for o in board) if i==2 else any(subtype(o['card'],'Lesson') for o in grave) if i==3 else player['cards_drawn_this_turn']>=2 if i in (4,6,8) else sum(bool(types(o['card'])&{'Instant','Sorcery'}) for o in grave)>=2 if i==5 else source['attacking'] and obs['decision']!='attackers' if i==7 else modified if i==10 else player['spells_cast_this_turn']==0 if i==11 else player['life']>=25)
                        extra=2 if i==0 and permanents>=8 else 0;equipment=sum(o['card']=='Goldvein Pick' for o in attached);counter=c['plus_one']-c['minus_one'];base=bases[i];bonus=bonuses[i];expected=(base[0]+(bonus[0] if on else 0)+extra+equipment+counter,base[1]+(bonus[1] if on else 0)+extra+equipment+counter,intrinsics[i]|(flags[i] if on else 0)|(65536 if extra else 0))
                        self.assertEqual((source['power'],source['toughness'],source['keywords']),expected);self.assertEqual(source['modified'],modified)
                    if sources and on:break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources else -100
                            if n==enabler and sources:return 230 if enabler in ('Opt','Abandon Attachments') or not helpers else -100
                            return -100
                        if k=='AnnounceActivate':return 240 if sources and objects[a['card']]['card'] in ('Mind Drill Assailant','Goldvein Pick') and not obs['stack'] and obs['active']=='P0' and obs['step'] in ('Main1','Main2') else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k=='ArrangeLibrary':return 210 if a['destination']=='Graveyard' else -100
                        if k=='ChooseOptional':return 220 if a['accept'] else -100
                        if k=='ChooseTarget' and 'Card' in a['target']:return 240 if objects[a['target']['Card']]['card']==wanted else 100
                        if k=='Attack':return 200 if i==7 else -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda j:score(obs['actions'][j]));private=obs['library_choice'] or obs['search_choice'] or obs['decision']=='effect_discard'
                    if private:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['library_choice']);self.assertIsNone(other['search_choice']);self.assertEqual(other['effect_discards'],[]);self.assertEqual(other['actions'],[])
                    if obs['announcing'] or obs['resolving'] or private:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        after=game.step(obs,choice);self.assertEqual(after,expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);replayed+=1
                    else:game.step(obs,choice)
                else:self.fail('source predicate not reached')
                self.assertGreater(replayed,0);self.assertEqual(obs['schema'],96)

    def test_nine_conditional_keyword_bodies_reach_exact_live_keywords_stats_and_private_replay(self):
        rows=[('Barrow Naughty',(1,3),(0,0),1,64),('Didact Echo',(3,2),(0,0),0,1),('Dreadwing Scavenger',(2,2),(1,1),1,128),('Echo of Dusk',(2,2),(1,1),0,64),('Goblin Tomb Raider',(1,2),(1,0),0,4),('Gravblade Heavy',(3,4),(1,0),0,128),('Kargan Dragonrider',(2,2),(0,0),0,1),('Spineseeker Centipede',(2,1),(1,2),0,8),('Wildfire Wickerfolk',(3,2),(1,1),4,256)]
        definitions={c['name']:c for c in json.loads((Path(__file__).resolve().parents[1]/'data/standard/cards.json').read_text())['cards']};types={n:set(c['type_line'].split(' — ')[0].split()) for n,c in definitions.items()}
        for wanted,base,bonus,intrinsic,flag in rows:
            tribal=wanted in ('Barrow Naughty','Kargan Dragonrider');artifact=wanted in ('Goblin Tomb Raider','Gravblade Heavy');delirium=wanted in ('Spineseeker Centipede','Wildfire Wickerfolk');enabler='Three Tree Mascot' if tribal or artifact else 'Mind Drill Assailant'
            deck=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])+[wanted]*8+[enabler]*8+['Opt']*6+['Scorpion Sentinel']*4+['Anthem of Champions']*4 if delirium else sum(([n]*8 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])+[wanted]*12+[enabler]*8
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                replayed=0
                for _ in range(20000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};owned=[o for o in objects.values() if o['owner']=='P0'];board=[o for o in owned if o['zone']=='Battlefield'];sources=[o for o in board if o['card']==wanted];helpers=[o for o in board if o['card']==enabler];grave=[o for o in owned if o['zone']=='Graveyard' and not o['token']];permanents=[o for o in grave if types[o['card']]&{'Artifact','Creature','Enchantment','Land','Planeswalker','Battle'}];grave_types=set().union(*(types[o['card']]&{'Artifact','Battle','Creature','Enchantment','Instant','Kindred','Land','Planeswalker','Sorcery'} for o in grave))
                    if tribal:
                        subtype='Faerie' if wanted=='Barrow Naughty' else 'Dragon'
                        on=bool(sources and any(o['reference']!=sources[0]['reference'] and ('Changeling' in definitions[o['card']]['keywords'] or subtype in definitions[o['card']]['type_line'].split(' — ')[-1].split()) for o in board))
                    elif artifact:on=any('Artifact' in types[o['card']] for o in board)
                    elif delirium:on=len(grave_types)>=4
                    elif wanted=='Dreadwing Scavenger':on=len(grave)>=7
                    else:on=len(permanents)>=4
                    for o in sources:self.assertEqual((o['power'],o['toughness'],o['keywords']),(base[0]+(bonus[0] if on else 0),base[1]+(bonus[1] if on else 0),intrinsic|(flag if on else 0)))
                    if sources and on:break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources else -100
                            if n==enabler and sources:return 230 if not helpers else -100
                            return -100
                        if k=='AnnounceActivate':return 240 if objects[a['card']]['card']=='Mind Drill Assailant' and sources and not obs['stack'] and not on else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k=='ChooseSearch':return 260
                        if k=='ArrangeLibrary':return 210 if a['destination']=='Graveyard' else -100
                        if k=='Discard':return 200 if not types[objects[a['card']]['card']]<=grave_types else 100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));private=obs['library_choice'] or obs['search_choice'] or obs['decision']=='effect_discard'
                    if private:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['library_choice']);self.assertIsNone(other['search_choice']);self.assertEqual(other['effect_discards'],[]);self.assertEqual(other['actions'],[])
                    if obs['announcing'] or obs['resolving'] or private:
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        after=game.step(obs,choice);self.assertEqual(after,expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);replayed+=1
                    else:game.step(obs,choice)
                else:self.fail('conditional source keyword not reached')
                self.assertGreater(replayed,0);self.assertEqual(obs['schema'],96);public=game.observe('P1');source=next(o for o in public['objects'] if o['id']==sources[0]['id']);self.assertEqual((source['power'],source['toughness'],source['keywords']),(base[0]+bonus[0],base[1]+bonus[1],intrinsic|flag))

    def test_nine_count_threshold_bodies_reach_live_bonus_and_native_python_replay(self):
        rows=[('Basking Capybara',(1,3),(3,0)),('Billowing Shriekmass',(2,3),(2,1)),('Brightspear Zealot',(2,4),(2,0)),('Cloudsculpt Technician',(1,4),(1,0)),('Frilled Cave-Wurm',(2,5),(2,0)),('Gaelicat',(1,3),(2,0)),('Gigantoad',(4,4),(2,2)),('Mind Drill Assailant',(2,5),(3,0)),('Scorpion Sentinel',(1,4),(3,0))]
        types={c["name"]:c["type_line"].split(" — ")[0].split() for c in json.loads((Path(__file__).resolve().parents[1]/"data/standard/cards.json").read_text())["cards"]}
        for wanted,base,bonus in rows:
            enabler='Scorpion Sentinel' if wanted in ('Cloudsculpt Technician','Gaelicat') else 'Opt' if wanted=='Brightspear Zealot' else 'Mind Drill Assailant'
            lands=sum(([n]*8 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            with self.subTest(card=wanted),Game([lands+[wanted]*12+[enabler]*8,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                crossed=False
                for _ in range(16000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};owned=[o for o in objects.values() if o['owner']=='P0'];board=[o for o in owned if o['zone']=='Battlefield'];sources=[o for o in board if o['card']==wanted];helpers=[o for o in board if o['card']==enabler];grave=[o for o in owned if o['zone']=='Graveyard' and not o['token']];permanents=[o for o in grave if any(t in ('Artifact','Creature','Enchantment','Land','Planeswalker','Battle') for t in types[o['card']])]
                    on=(len(permanents)>=4 if wanted in ('Basking Capybara','Frilled Cave-Wurm') else len(grave)>=7 if wanted in ('Billowing Shriekmass','Mind Drill Assailant') else obs['players'][0]['spells_cast_this_turn']>=2 if wanted=='Brightspear Zealot' else sum('Artifact' in types[o['card']] for o in board)>=(2 if wanted=='Gaelicat' else 1) if wanted in ('Cloudsculpt Technician','Gaelicat') else sum('Land' in types[o['card']] for o in board)>=7)
                    for o in sources:self.assertEqual((o['power'],o['toughness']),(base[0]+(bonus[0] if on else 0),base[1]+(bonus[1] if on else 0)))
                    if sources and on:crossed=True;break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources else -100
                            if n==enabler and sources:return 230 if (enabler=='Opt' and obs['active']=='P0' or len(helpers)<(2 if enabler=='Scorpion Sentinel' else 1)) else -100
                            return -100
                        if k=='AnnounceActivate':return 240 if objects[a['card']]['card']=='Mind Drill Assailant' and sources and not obs['stack'] and len(grave)<7 else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k=='ArrangeLibrary':return 210 if a['destination']=='Graveyard' else -100
                        return self.professor_score(obs,a)
                    choice=max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]));action=obs['actions'][choice]
                    # Replay every semantic source/cost/private-library boundary, including the crossing itself.
                    if sources and (obs['announcing'] or obs['resolving'] or obs['library_choice']):
                        saved=game.snapshot()
                        with game.clone() as fork:expected=fork.step(fork.observe(obs['actor']),choice)
                        after=game.step(obs,choice);self.assertEqual(after,expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,choice)
                        again=game.step(game.observe(obs['actor']),choice);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
                    else:game.step(obs,choice)
                    if obs['library_choice']:
                        other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertIsNone(other['library_choice']);self.assertEqual(other['actions'],[])
                else:self.fail('live source threshold not reached')
                self.assertTrue(crossed);self.assertEqual(obs['schema'],96)
                public=game.observe('P1');source=next(o for o in public['objects'] if o['id']==sources[0]['id']);self.assertEqual((source['power'],source['toughness']),(base[0]+bonus[0],base[1]+bonus[1]))

    def test_complete_firebending_bodies_replay_attack_mana_and_combat_expiry(self):
        rows=[('Azula, On the Hunt',2),('Fire Sages',1),('Rough Rhino Cavalry',2),('Sun Warriors',1),('Uncle Iroh',1),('Vindictive Warden',1),('Zhao, Ruthless Admiral',2)]
        for wanted,amount in rows:
            lands=sum(([n]*8 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            with self.subTest(card=wanted),Game([lands+[wanted]*20,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};sources=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield' and o['card']==wanted];roots=[s for s in obs['stack'] if s['source_card']==wanted and s['kind']=={'TriggeredAbility':{'ability':0}}]
                    if roots:break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):return 250 if objects[a['card']]['card']==wanted and not sources else -100
                        if k=='Attack':return 240 if sources and a['card']==sources[0]['id'] else -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 200
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('Activate','AnnounceActivate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('firebending attack not reached')
                root=roots[-1];before=game.observe('P0');self.assertEqual(before['players'][0]['combat_red_mana'],0);saved=game.snapshot()
                with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
                after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(before,0)
                again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
                self.assertEqual(after['schema'],96);self.assertEqual(after['players'][0]['combat_red_mana'],amount);self.assertGreaterEqual(after['players'][0]['mana'][3],amount)
                seen=set()
                for _ in range(300):
                    obs=game.observe('P0');self.assertEqual(obs['players'],game.observe('P1')['players'])
                    if obs['step']=='Main2':break
                    seen.add(obs['step']);self.assertEqual(obs['players'][0]['combat_red_mana'],amount);obs=game.observe(obs['actor'])
                    if not obs['stack'] and obs['decision']=='priority':self.assertEqual(obs['players'][0]['mana'],[0,0,0,amount,0,0])
                    kinds=['Pass','PutTrigger','FinishAttackers','FinishBlockers','Discard'];a=next(a for a in obs['actions'] if a['kind'] in kinds);game.act(obs,a)
                else:self.fail('second main not reached')
                self.assertTrue({'Blockers','CombatDamage','EndCombat'}<=seen);self.assertEqual(obs['players'][0]['combat_red_mana'],0);self.assertEqual(obs['players'][0]['mana'],[0]*6)

    def test_namazu_complete_entry_life_and_treasure_compose_without_retained_mana(self):
        wanted='Namazu Trader'
        with Game([['Swamp']*40+[wanted]*20,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(10000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};roots=[s for s in obs['stack'] if s['source_card']==wanted and s['kind']=={'TriggeredAbility':{'ability':0}}]
                if roots:break
                def score(a):
                    if a['kind'] in ('Cast','AnnounceCast'):return 250 if objects[a['card']]['card']==wanted else -100
                    if a['kind']=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                    return self.professor_score(obs,a)
                game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
            else:self.fail('Namazu entry not reached')
            root=roots[-1];before=game.observe('P0');saved=game.snapshot()
            with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
            after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(before,0)
            again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
            self.assertEqual(after['players'][0]['life'],19);self.assertEqual(after['players'][0]['combat_red_mana'],0);tokens=[o for o in after['objects'] if o['token'] and o['token']['subtypes']=='Treasure' and o['owner']=='P0'];self.assertEqual(len(tokens),1);self.assertEqual(tokens[0]['token']['colors'],0)

    def test_fynn_and_silencer_real_combat_poison_replays_to_ten_counter_loss(self):
        for wanted in ['Fynn, the Fangbearer','Virulent Silencer']:
            with self.subTest(card=wanted),Game([['Forest']*40+[wanted]*20,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                replayed=False
                for _ in range(16000):
                    obs=game.observe('P0')
                    if obs['outcome']:break
                    obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};sources=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield' and o['card']==wanted]
                    roots=[s for s in obs['stack'] if s['source_card']==wanted and s['kind']=={'TriggeredAbility':{'ability':0}}]
                    if roots and not replayed:
                        root=roots[-1];before=game.observe('P0');self.assertEqual(root['damage_event']['player'],'P1');self.assertTrue(root['damage_event']['combat']);saved=game.snapshot()
                        with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
                        after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);game.restore(saved)
                        with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(before,0)
                        again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);self.assertEqual(after['players'][1]['poison_counters'],2);self.assertEqual(after['schema'],96);replayed=True;continue
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):return 250 if objects[a['card']]['card']==wanted and not sources else -100
                        if k=='Attack':return 300 if sources and a['card']==sources[0]['id'] else -100
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('combat poison game did not finish')
                end=game.observe('P0');self.assertTrue(replayed);self.assertEqual(end['outcome'],{'Winner':'P0'});self.assertEqual(end['players'][1]['poison_counters'],10);self.assertEqual(end['players'][1]['life'],15 if wanted.startswith('Fynn') else 10);self.assertIsNone(end['actor']);self.assertEqual(end['actions'],[]);self.assertEqual(game.observe('P1')['players'][1]['poison_counters'],10)

    def test_interrogator_clue_poison_resolves_before_its_private_draw_and_replays(self):
        wanted='Persuasive Interrogators';deck=['Swamp']*40+[wanted]*20
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(16000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];sources=[o for o in own if o['card']==wanted];clues=[o for o in own if o['token'] and o['token']['subtypes']=='Clue'];roots=[s for s in obs['stack'] if s['source_card']==wanted and s['kind']=={'TriggeredAbility':{'ability':1}}]
                if roots:break
                def score(a):
                    k=a['kind']
                    if k in ('Cast','AnnounceCast'):return 250 if objects[a['card']]['card']==wanted and not sources else -100
                    if k=='AnnounceActivate':return 270 if clues and a['card']==clues[0]['id'] and not obs['stack'] else -100
                    if k=='ChooseTarget':return 240 if a['target'].get('Player')=='P1' else -100
                    if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                    if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                    return self.professor_score(obs,a)
                game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
            else:self.fail('Clue poison not reached')
            root=roots[-1];before=game.observe('P0');saved=game.snapshot()
            with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
            after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(before,0)
            again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);self.assertEqual(after['players'][1]['poison_counters'],2);self.assertEqual(after['players'][1]['life'],20);self.assertEqual(after['players'][0]['cards_drawn_this_turn'],before['players'][0]['cards_drawn_this_turn']);self.assertTrue(any(s['source_card'].startswith('Token:') or s['card']!=root['card'] for s in after['stack']))
            self.assertTrue(all(o['zone']!='Hand' or o['owner']!='P0' for o in game.observe('P1')['objects']))

    def test_society_poison_ward_owned_decline_or_two_payments_replay_game_loss(self):
        for accept in [False,True]:
            with self.subTest(accept=accept),Game([['Mountain']*40+['Shock']*20,['Swamp']*20+['Forest']*20+['The Serpent Society']*20],seed=7,deck_rules='Freeform') as game:
                for payment in range(2 if accept else 1):
                    for _ in range(16000):
                        obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};sources=[o for o in objects.values() if o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='The Serpent Society']
                        if obs['decision']=='optional_effect':break
                        def score(a):
                            k=a['kind']
                            if k in ('Cast','AnnounceCast'):
                                n=objects[a['card']]['card'];return 250 if n=='The Serpent Society' and not sources or n=='Shock' and sources and not obs['stack'] else -100
                            if k=='ChooseTarget':return 240 if sources and a['target'].get('Card')==sources[0]['id'] else -100
                            if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                            if k in ('Activate','BeginActivate','AnnounceActivate','ActivateMana','AnnounceMana'):return -100
                            return self.professor_score(obs,a)
                        game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                    else:self.fail('poison ward not reached')
                    self.assertEqual(obs['actor'],'P0');self.assertEqual(game.observe('P1')['actions'],[]);self.assertEqual(obs['players'][0]['poison_counters'],payment*5);saved=game.snapshot()
                    action=next(a for a in obs['actions'] if a['kind']=='ChooseOptional' and a['accept']==accept)
                    with game.clone() as fork:expected=fork.act(fork.observe('P0'),action)
                    after=game.act(obs,action);self.assertEqual(after,expected);game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                    again=game.act(game.observe('P0'),action);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved);self.assertEqual(after['players'][0]['poison_counters'],(payment+1)*5 if accept else 0);self.assertEqual(after['players'][0]['life'],20);self.assertEqual(after['players'][0]['life_lost_this_turn'],0)
                if accept:self.assertEqual(game.observe('P0')['outcome'],{'Winner':'P1'});self.assertEqual(game.observe('P0')['players'][0]['poison_counters'],10)
                else:self.assertIsNone(game.observe('P0')['outcome'])

    def test_seven_draw_token_bodies_and_both_pandora_abilities_replay_native_frames(self):
        cases=[('Biomechan Engineer',0),('Lunatic Pandora',0),('Lunatic Pandora',1),('Popular Egotist',0),('Reckless Lackey',0),('Redrock Sentinel',0),('Seize the Spoils',0),('The Sackville-Bagginses',0)]
        for wanted,ability in cases:
            lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[]);deck=lands+[wanted]*12+['Bear Cub']*18
            with self.subTest(card=wanted,ability=ability),Game([deck,['Forest']*48+['Bear Cub']*12],seed=7,deck_rules='Freeform') as game:
                for _ in range(20000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];sources=[o for o in own if o['card']==wanted];bears=[o for o in own if o['card']=='Bear Cub'];enemy=[o for o in objects.values() if o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Bear Cub']
                    kind='Spell' if wanted=='Seize the Spoils' else {'TriggeredAbility':{'ability':0}} if wanted=='The Sackville-Bagginses' else {'ActivatedAbility':{'ability':ability}}
                    roots=[s for s in obs['stack'] if s['source_card']==wanted and s['kind']==kind]
                    if roots:break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources and (wanted!='The Sackville-Bagginses' or bears) else -100
                            if n=='Bear Cub':return 200 if obs['actor']=='P0' and not bears or obs['actor']=='P1' and not enemy else -100
                            return -100
                        if k=='AnnounceActivate':return 270 if sources and a['card']==sources[0]['id'] and a['ability']==ability and not obs['stack'] and (wanted!='Popular Egotist' or bears) and (wanted!='Lunatic Pandora' or ability==0 or enemy) else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        if k=='ChooseTarget':return 240 if enemy and a['target'].get('Card')==enemy[0]['id'] or a['target'].get('Player')=='P1' else -100
                        if k=='ChooseAdditionalCost':
                            o=objects[a['object']['card']]
                            return 240 if wanted=='Redrock Sentinel' and o['card'] in ('Plains','Island','Swamp','Mountain','Forest') or wanted=='Popular Egotist' and o['card']=='Bear Cub' or wanted=='Seize the Spoils' and o['card']=='Bear Cub' else 100
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 210
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('native body frame not reached')
                root=roots[-1];before=game.observe('P0');source=root['card'];saved=game.snapshot()
                def finish(handle):
                    for _ in range(120):
                        o=handle.observe('P0');live=o['stack']+o['waiting_triggers']+[x for x in (o['announcing'],o['resolving']) if x]
                        if all(s['id']!=root['id'] for s in live):return o
                        o=handle.observe(o['actor'])
                        if o['library_choice']:
                            other=handle.observe('P1' if o['actor']=='P0' else 'P0');self.assertIsNone(other['library_choice']);self.assertEqual(other['actions'],[])
                        def score(a):
                            if a['kind']=='Sacrifice':return 130
                            if a['kind']=='ChooseTarget':return 120 if a['target'].get('Player')=='P1' else -100
                            return {'Pass':110,'PutTrigger':110,'ArrangeLibrary':110,'Discard':110,'ChooseOptional':100}.get(a['kind'],-100)
                        handle.step(o,max(range(len(o['actions'])),key=lambda i:score(o['actions'][i])))
                    self.fail('body frame did not finish')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(before,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
                draws={'Biomechan Engineer':2,'Reckless Lackey':1,'Redrock Sentinel':1,'Seize the Spoils':2,'The Sackville-Bagginses':1}.get(wanted,0);self.assertEqual(after['players'][0]['cards_drawn_this_turn'],before['players'][0]['cards_drawn_this_turn']+draws)
                if draws:
                    subtype='Robot' if wanted=='Biomechan Engineer' else 'Treasure';new=[o for o in after['objects'] if o['owner']=='P0' and o['zone']=='Battlefield' and o['token'] and o['token']['subtypes']==subtype];self.assertEqual(len(new),1);self.assertEqual((new[0]['power'],new[0]['toughness']),(2,2) if subtype=='Robot' else (0,0));self.assertEqual(new[0]['token']['colors'],0)
                if wanted=='Popular Egotist':
                    now=next(o for o in after['objects'] if o['id']==source);self.assertTrue(now['tapped']);self.assertTrue(now['keywords']&8192);self.assertEqual(after['players'][0]['life'],before['players'][0]['life']+1);self.assertEqual(after['players'][1]['life'],before['players'][1]['life']-1)
                if wanted=='Lunatic Pandora':
                    self.assertEqual(next(o for o in after['objects'] if o['id']==source)['zone'],'Graveyard' if ability==1 else 'Battlefield')
                    if ability==1:self.assertEqual(next(o for o in after['objects'] if o['id']==root['targets'][0]['target']['Card'])['zone'],'Graveyard')
                public=game.observe('P1');self.assertTrue(all(o['zone']!='Hand' or o['owner']!='P0' or o['reference'] in public['known_hand'] for o in public['objects']))

    def test_seven_sacrifice_event_bodies_use_real_costs_and_owned_library_replay(self):
        names=['Biotech Specialist','Crime Novelist','Experimental Confectioner','Gleaming Geardrake','Lightless Evangel','Pirate Peddlers','Tolls of War']
        for wanted in names:
            lands=sum(([n]*6 for n in ['Plains','Island','Swamp','Mountain','Forest']),[])
            extras=['Vampiric Rites']*12+['Bear Cub']*6 if wanted in names[4:6] else ['Nutrient Block']*12+['Bear Cub']*6 if wanted=='Crime Novelist' else ['Bear Cub']*18
            deck=lands+[wanted]*12+extras;ability=0 if wanted in names[1:2]+names[4:6] else 1
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']}
                    own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];sources=[o for o in own if o['card']==wanted];bears=[o for o in own if o['card']=='Bear Cub'];rites=[o for o in own if o['card']=='Vampiric Rites'];kind={'Biotech Specialist':'Lander','Experimental Confectioner':'Food','Gleaming Geardrake':'Clue','Tolls of War':'Clue'}.get(wanted)
                    fodder=[o for o in own if (kind and o['token'] and kind in o['token']['subtypes']) or (wanted=='Crime Novelist' and o['card']=='Nutrient Block')]
                    roots=[s for s in obs['stack'] if s['source_card']==wanted and s['kind']=={'TriggeredAbility':{'ability':ability}}]
                    if roots:break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            n=objects[a['card']]['card']
                            if n==wanted:return 250 if not sources else -100
                            if n=='Vampiric Rites':return 210 if sources and not rites else -100
                            if n=='Bear Cub':return 200 if wanted in names[4:6] and not bears else -100
                            if n=='Nutrient Block':return 210 if sources and not fodder else -100
                            return -100
                        if k=='AnnounceActivate':return 260 if sources and not obs['stack'] and (fodder and a['card']==fodder[0]['id'] or rites and bears and a['card']==rites[0]['id']) else -100
                        if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
                        if k=='ChooseAdditionalCost':return 240 if bears and a['object']['card']==bears[0]['id'] else -100
                        if k=='ChooseTarget':return 230 if a['target'].get('Player')=='P1' else -100
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 220
                        if k=='TapMana':return 180 if obs['decision']=='pay_costs' else -100
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('sacrifice trigger not reached')
                root=roots[-1];source=root['card'];before=game.observe('P0');saved=game.snapshot()
                with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
                after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(before,0)
                again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
                if wanted in ('Crime Novelist','Gleaming Geardrake','Lightless Evangel','Pirate Peddlers'):
                    self.assertEqual(next(o for o in after['objects'] if o['id']==source)['counters']['plus_one'],1)
                if wanted=='Crime Novelist':self.assertEqual(after['players'][0]['mana'][3],before['players'][0]['mana'][3]+1)
                if wanted=='Biotech Specialist':self.assertEqual(after['players'][1]['life'],before['players'][1]['life']-2)
                if wanted in ('Experimental Confectioner','Tolls of War'):
                    subtype='Rat' if wanted=='Experimental Confectioner' else 'Ally';created=[o for o in after['objects'] if o['owner']=='P0' and o['zone']=='Battlefield' and o['token'] and subtype in o['token']['subtypes']];self.assertEqual(len(created),1);self.assertEqual((created[0]['power'],created[0]['toughness']),(1,1))
                if wanted!='Crime Novelist':self.assertEqual(after['players'][0]['cards_drawn_this_turn'],before['players'][0]['cards_drawn_this_turn'])
                else:self.assertIn(after['players'][0]['cards_drawn_this_turn']-before['players'][0]['cards_drawn_this_turn'],(0,1))  # Nutrient Block also queues its independent draw.
                other=game.observe('P1');self.assertIsNone(other['search_choice']);self.assertTrue(all(o['zone']!='Hand' or o['owner']!='P0' for o in other['objects']))

    def test_ten_permanent_graveyard_bodies_use_owned_native_events_and_replay(self):
        watchers=["Ashiok's Reaper",'Knight of Doves','Savior of the Sleeping','Warehouse Tabby','Wicked Visitor']
        names=watchers+['Long Feng, Grand Secretariat','Mephitic Draught','Nutrient Block','Hopeful Vigil','Reach for the Sky']
        for wanted in names:
            if wanted in watchers:deck=['Plains']*12+['Swamp']*12+[wanted]*12+['Hopeful Vigil']*12+['Bear Cub']*12
            elif wanted.startswith('Long Feng'):deck=['Swamp']*16+['Forest']*8+[wanted]*12+['Vampiric Rites']*12+['Bear Cub']*12
            elif wanted=='Nutrient Block':deck=['Forest']*24+[wanted]*12+['Bear Cub']*24
            elif wanted=='Reach for the Sky':deck=['Forest']*12+['Plains']*12+[wanted]*12+['Disenchant']*12+['Bear Cub']*12
            else:deck=['Plains']*12+['Swamp']*12+[wanted]*12+['Disenchant']*12+['Bear Cub']*12
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                seen_entry=False
                for _ in range(18000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']}
                    own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];sources=[o for o in own if o['card']==wanted];bears=[o for o in own if o['card']=='Bear Cub'];vigils=[o for o in own if o['card']=='Hopeful Vigil'];rites=[o for o in own if o['card']=='Vampiric Rites']
                    if sources and wanted=='Reach for the Sky':
                        self.assertTrue(sources[0]['keywords']&32);self.assertTrue(any(o['reference']==sources[0]['attached_to'] and (o['power'],o['toughness'])==(5,4) and o['keywords']&2 for o in bears));seen_entry=True
                    if sources and wanted=='Mephitic Draught' and not obs['stack']:self.assertEqual(obs['players'][0]['life'],19);seen_entry=True
                    roots=[s for s in obs['stack'] if s['source_card']==wanted and isinstance(s['kind'],dict) and 'TriggeredAbility' in s['kind'] and (wanted in watchers or wanted.startswith('Long Feng') or objects[s['card']]['zone']=='Graveyard')]
                    if roots:break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            name=objects[a['card']]['card']
                            if name==wanted:return 240 if not sources and (wanted!='Reach for the Sky' or bears) else -100
                            if name=='Bear Cub':return 190 if not bears and (wanted.startswith('Long Feng') or wanted=='Reach for the Sky') else -100
                            if name=='Hopeful Vigil':return 220 if wanted in watchers and sources and not vigils else -100
                            if name=='Vampiric Rites':return 210 if wanted.startswith('Long Feng') and sources and not rites else -100
                            if name=='Disenchant':return 260 if sources and not obs['stack'] and wanted in ('Mephitic Draught','Reach for the Sky') else -100
                            return -100
                        if k=='AnnounceActivate':
                            good=not obs['stack'] and sources and (wanted in watchers and vigils and a['card']==vigils[0]['id'] or wanted in ('Hopeful Vigil','Nutrient Block') and a['card']==sources[0]['id'] or wanted.startswith('Long Feng') and rites and bears and a['card']==rites[0]['id'])
                            return 250 if good else -100
                        if k in ('Activate','ActivateMana','AnnounceMana','BeginActivate'):return -100
                        if k=='ChooseAdditionalCost':return 240 if bears and a['object']['card']==bears[0]['id'] else -100
                        if k=='ChooseTarget':
                            root=obs['announcing'];targets=sources if root['source_card'] in ('Disenchant','Long Feng, Grand Secretariat') else bears
                            return 220 if targets and a['target'].get('Card')==targets[0]['id'] else -100
                        if k=='Sacrifice':return 220
                        if k in ('ChooseSpellCost','ChooseActivationCost'):return 200
                        if k=='TapMana' and obs['decision']=='pay_costs':return 180
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('permanent departure trigger not reached')
                root=roots[-1];source=root['card'];before=game.observe('P0');saved=game.snapshot()
                def finish(handle):return self.finish_self_target(handle,'P0',root['id'])
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(before,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
                if wanted in ("Ashiok's Reaper",'Mephitic Draught','Nutrient Block','Reach for the Sky'):self.assertEqual(after['players'][0]['cards_drawn_this_turn'],before['players'][0]['cards_drawn_this_turn']+1)
                if wanted in ('Mephitic Draught','Reach for the Sky'):self.assertTrue(seen_entry)
                if wanted=='Mephitic Draught':self.assertEqual(after['players'][0]['life'],before['players'][0]['life']-1)
                if wanted=='Wicked Visitor':self.assertEqual(after['players'][1]['life'],before['players'][1]['life']-1)
                if wanted in ('Savior of the Sleeping','Long Feng, Grand Secretariat'):self.assertEqual(next(o for o in after['objects'] if o['id']==source)['counters']['plus_one'],1)
                if wanted in ('Knight of Doves','Warehouse Tabby'):
                    subtype='Bird' if wanted=='Knight of Doves' else 'Rat';ts=[o for o in after['objects'] if o['owner']=='P0' and o['zone']=='Battlefield' and o['token'] and o['token']['subtypes']==subtype];self.assertEqual(len(ts),1);self.assertEqual((ts[0]['power'],ts[0]['toughness']),(1,1));self.assertTrue(ts[0]['keywords']&(1 if subtype=='Bird' else 2048))
                if wanted=='Hopeful Vigil':
                    ks=[o for o in after['objects'] if o['zone']=='Battlefield' and o['token'] and o['token']['subtypes']=='Knight'];self.assertEqual(len(ks),1);self.assertEqual((ks[0]['power'],ks[0]['toughness']),(2,2));self.assertTrue(ks[0]['keywords']&8)
                if wanted=='Nutrient Block':
                    lower=next(s for s in after['stack'] if s['source_card']==wanted and 'ActivatedAbility' in s['kind']);life=after['players'][0]['life'];after=self.finish_self_target(game,'P0',lower['id']);self.assertEqual(after['players'][0]['life'],life+3)
                public=game.observe('P1');self.assertTrue(all(o['owner']!='P0' or o['zone']!='Hand' or o['reference'] in public['known_hand'] for o in public['objects']))

    def test_six_temporary_blocking_bodies_resolve_expire_and_replay_in_process(self):
        names=['Brambleback Brute','Sandstorm Verge','Sower of Chaos','Summit Intimidator','War Squeak','Zog, Triceraton Castaway']
        for wanted in names:
            lands=12 if wanted=='Sandstorm Verge' else 24
            deck=['Mountain']*lands+[wanted]*12+['Bear Cub']*(48-lands)
            # Bears require green; split the ordinary lands without reducing the
            # spell's red sources. Sandstorm itself is the remaining land slot.
            deck[:lands]=['Mountain']*(lands//2)+['Forest']*(lands-lands//2)
            with self.subTest(card=wanted),Game([deck,['Forest']*24+['Bear Cub']*36],seed=7,deck_rules='Freeform') as game:
                for _ in range(16000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']}
                    own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield' and o['card']=='Bear Cub'];enemy=[o for o in objects.values() if o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Bear Cub'];sources=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield' and o['card']==wanted]
                    roots=[s for s in obs['stack'] if s['source_card']==wanted and s['kind']!='Spell' and s['targets']]
                    if roots:break
                    def score(a):
                        k=a['kind']
                        if k=='PlayLand' and wanted=='Sandstorm Verge' and objects[a['card']]['card']==wanted:return 250 if not sources else -100
                        if k in ('Cast','AnnounceCast'):
                            name=objects[a['card']]['card']
                            if name==wanted:return 200 if enemy and not sources and (wanted!='War Squeak' or own) else -100
                            return 160 if name=='Bear Cub' and not (own if obs['actor']=='P0' else enemy) else -100
                        if k=='AnnounceActivate':return 240 if wanted in names[:3] and sources and a['card']==sources[0]['id'] and a['ability']==(1 if wanted=='Sandstorm Verge' else 0) and enemy else -100
                        if k in ('Activate','ActivateMana','AnnounceMana','BeginActivate'):return -100
                        if k=='ChooseTarget':
                            root=obs['announcing'];attachment=root and root['source_card']==wanted and root['kind']=='Spell'
                            choices=own if attachment else enemy
                            return 220 if choices and a['target'].get('Card')==choices[0]['id'] else -100
                        if k=='TapMana' and obs['decision']=='pay_costs':return 180
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('blocking effect was not committed')
                root=roots[-1];target=root['targets'][0]['object'];saved=game.snapshot()
                def finish(handle):
                    after=self.finish_self_target(handle,'P0',root['id']);now=next(o for o in after['objects'] if o['reference']==target);self.assertTrue(now['keywords']&2048);return after
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
                turn=after['turn']
                for _ in range(200):
                    current=game.observe('P0')
                    if current['turn']!=turn:break
                    current=game.observe(current['actor']);i=max(range(len(current['actions'])),key=lambda i:100 if current['actions'][i]['kind']=='Pass' else 90 if current['actions'][i]['kind'].startswith('Finish') else 80 if current['actions'][i]['kind']=='Discard' else -1000 if current['actions'][i]['kind']=='Concede' else -100);game.step(current,i)
                else:self.fail('cleanup not reached')
                now=next(o for o in current['objects'] if o['reference']==target);self.assertFalse(now['keywords']&2048)

    def test_targeted_graveyard_returns_publish_new_hand_refs_and_replay_owned_payment(self):
        for wanted in ['Bonebind Orator', 'Theoretical Necromancer']:
            deck=['Swamp']*18+['Forest']*6+[wanted]*12+['Deathcap Marionette']*12+['Grizzly Bears']*12
            with self.subTest(card=wanted),Game([deck,['Forest']*24+['Grizzly Bears']*36],seed=7,deck_rules='Freeform') as game:
                obs=self.reach_graveyard_activation_payment(game,wanted);root=obs['announcing'];target=root['targets'][0]['object'];action=next(a for a in obs['actions'] if a['kind']=='PayCosts');saved=game.snapshot()
                def finish(handle):
                    handle.act(handle.observe('P0'),action)
                    after=self.finish_self_target(handle,'P0',root['id']);now=next(o for o in after['objects'] if o['id']==target['card']);self.assertEqual(now['zone'],'Hand');self.assertNotEqual(now['reference'],target)
                    public=handle.observe('P1');self.assertEqual(public['known_hand'].count(now['reference']),1);self.assertTrue(any(o['reference']==now['reference'] for o in public['objects']))
                    private=[o for o in after['objects'] if o['owner']=='P0' and o['zone']=='Hand' and o['reference'] not in public['known_hand']];self.assertTrue(private);self.assertTrue(all(o['id'] not in {x['id'] for x in public['objects']} for o in private))
                    return after
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def test_nine_graveyard_returns_pay_before_moving_and_replay_known_hand_or_tapped_entry(self):
        names=['Abzan Devotee','Bestial Bloodline','Blessed Ghoul','Merchant of Many Hats','Project Deathlok Soldier','Reassembling Skeleton','Summoned Dromedary',"Teacher's Pest",'Tunnel Rats']
        for wanted in names:
            deck=['Plains']*6+['Island']*6+['Swamp']*6+['Forest']*6+[wanted]*12+['Deathcap Marionette']*12+['Grizzly Bears']*12
            with self.subTest(card=wanted),Game([deck,['Forest']*24+['Grizzly Bears']*36],seed=7,deck_rules='Freeform') as game:
                obs=self.reach_graveyard_activation_payment(game,wanted);root=obs['announcing'];source=next(o for o in obs['objects'] if o['id']==root['card']);self.assertEqual(source['zone'],'Graveyard');action=next(a for a in obs['actions'] if a['kind']=='PayCosts');saved=game.snapshot()
                def finish(handle):
                    paid=handle.act(handle.observe('P0'),action);now=next(o for o in paid['objects'] if o['id']==source['id']);self.assertEqual(now['zone'],'Graveyard');self.assertEqual(now['reference'],source['reference']);self.assertEqual(paid['players'][0]['mana'],[n-c for n,c in zip(obs['players'][0]['mana'],action['payment'])])
                    return self.finish_self_target(handle,'P0',root['id'])
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);now=next(o for o in after['objects'] if o['id']==source['id']);self.assertNotEqual(now['reference'],source['reference']);battlefield=wanted in ('Reassembling Skeleton',"Teacher's Pest",'Tunnel Rats');self.assertEqual(now['zone'],'Battlefield' if battlefield else 'Hand')
                if battlefield:self.assertTrue(now['tapped'] and now['summoning_sick'])
                else:self.assertIn(now['reference'],game.observe('P1')['known_hand']);self.assertTrue(any(o['reference']==now['reference'] for o in game.observe('P1')['objects']))
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def test_arcanis_and_wingspan_battlefield_returns_preserve_native_replay_and_public_knowledge(self):
        for wanted in ['Arcanis the Omnipotent','Wingspan Stride']:
            with self.subTest(card=wanted),Game([['Island']*18+['Forest']*6+[wanted]*12+['Grizzly Bears']*24,['Forest']*24+['Grizzly Bears']*36],seed=7,deck_rules='Freeform') as game:
                for _ in range(20000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};root=obs['announcing'];sources=[o for o in objects.values() if o['owner']=='P0' and o['card']==wanted and o['zone']=='Battlefield'];bears=[o for o in objects.values() if o['owner']=='P0' and o['card']=='Grizzly Bears' and o['zone']=='Battlefield']
                    if obs['decision']=='pay_costs' and root['source_card']==wanted and root['kind']=={'ActivatedAbility':{'ability':1 if wanted.startswith('Arcanis') else 0}} and any(a['kind']=='PayCosts' for a in obs['actions']):break
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            name=objects[a['card']]['card']
                            if name==wanted:return 180 if not sources and (wanted.startswith('Arcanis') or bears) else -100
                            if obs['actor']=='P0':return 130 if not bears else -100
                        if k=='AnnounceActivate':return 240 if sources and a['card']==sources[0]['id'] and a['ability']==(1 if wanted.startswith('Arcanis') else 0) else -100
                        if k in ('Activate','ActivateMana','AnnounceMana','BeginActivate'):return -100
                        if k=='ChooseTarget':return 220 if bears and a['target'].get('Card')==bears[0]['id'] else -100
                        if k=='TapMana' and obs['decision']=='pay_costs':return 160
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('battlefield return payment not reached')
                root=obs['announcing'];old=root['object'];action=next(a for a in obs['actions'] if a['kind']=='PayCosts');saved=game.snapshot()
                def finish(handle):
                    paid=handle.act(handle.observe('P0'),action);self.assertTrue(any(o['reference']==old and o['zone']=='Battlefield' for o in paid['objects']))
                    return self.finish_self_target(handle,'P0',root['id'])
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);now=next(o for o in after['objects'] if o['id']==root['card']);self.assertEqual(now['zone'],'Hand');self.assertIn(now['reference'],game.observe('P1')['known_hand'])
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def reach_graveyard_activation_payment(self,game,wanted):
        for _ in range(15000):
            obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};root=obs['announcing']
            if obs['decision']=='pay_costs' and root['source_card']==wanted and isinstance(root['kind'],dict) and 'ActivatedAbility' in root['kind'] and any(a['kind']=='PayCosts' for a in obs['actions']):return obs
            def score(a):
                k=a['kind']
                if k in ('Cast','AnnounceCast'):
                    name=objects[a['card']]['card']
                    if name==wanted:return -100
                    if obs['actor']=='P0':return 150 if name=='Deathcap Marionette' else 120 if not any(o['owner']=='P0' and o['zone']=='Battlefield' and o['card']=='Grizzly Bears' for o in objects.values()) else -100
                if k=='AnnounceActivate':return 240 if objects[a['card']]['card']==wanted and objects[a['card']]['zone']=='Graveyard' else -100
                if k in ('Activate','BeginActivate','AnnounceMana','ActivateMana'):return -100
                if k=='Discard':return 230 if objects[a['card']]['card']==wanted else 100
                if k=='ChooseActivationCost':return 200
                if k=='ChooseTarget' and root and root['source_card']==wanted:
                    t=objects.get(a['target'].get('Card'))
                    if not t:return -100
                    if wanted in ('Bonebind Orator','Theoretical Necromancer'):return 220 if t['zone']=='Graveyard' and t['owner']=='P0' and t['id']!=root['card'] else -100
                    if wanted=='Gravestone Strider':return 220 if t['zone']=='Graveyard' and t['id']!=root['card'] else -100
                    return 220 if t['zone']=='Battlefield' and t['owner']=='P0' else -100
                if k=='TapMana':return 170 if obs['decision']=='pay_costs' else self.professor_score(obs,a)
                return self.professor_score(obs,a)
            game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
        self.fail('graveyard activation payment not reached')

    def test_sixteen_graveyard_self_exile_costs_use_owned_native_payment_and_full_replay(self):
        names=['Adorned Crocodile','Agent of Kotis','Beetle, Legacy Criminal','Bonebind Orator','Cryotheory Adept','Eternal Student','Faerie Dreamthief','Goldmeadow Nomad','Gravestone Strider','Leering Onlooker','Rubblebelt Maverick','Stoic Grove-Guide','Stone Docent','Suspicious Shambler','Theoretical Necromancer','Venom, Evil Unleashed']
        for wanted in names:
            deck=['Plains']*6+['Island']*6+['Swamp']*6+['Forest']*6+[wanted]*12+['Deathcap Marionette']*12+['Grizzly Bears']*12
            with self.subTest(card=wanted),Game([deck,['Forest']*24+['Grizzly Bears']*36],seed=7,deck_rules='Freeform') as game:
                obs=self.reach_graveyard_activation_payment(game,wanted);root=obs['announcing'];source=next(o for o in obs['objects'] if o['id']==root['card']);self.assertEqual(source['zone'],'Graveyard');self.assertEqual(source['reference'],root['object']);self.assertEqual(game.observe('P1')['actions'],[]);action=next(a for a in obs['actions'] if a['kind']=='PayCosts');saved=game.snapshot()
                def finish(handle):
                    paid=handle.act(handle.observe('P0'),action);now=next(o for o in paid['objects'] if o['id']==source['id']);self.assertEqual(now['zone'],'Exile');self.assertNotEqual(now['reference'],source['reference']);self.assertEqual(paid['players'][0]['mana'],[n-c for n,c in zip(obs['players'][0]['mana'],action['payment'])]);self.assertTrue(any(s['id']==root['id'] and s['object']==root['object'] for s in paid['stack']))
                    return self.finish_self_target(handle,'P0',root['id'])
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);self.assertEqual(next(o for o in after['objects'] if o['id']==source['id'])['zone'],'Exile')
                if wanted in ('Agent of Kotis','Venom, Evil Unleashed','Adorned Crocodile','Beetle, Legacy Criminal','Rubblebelt Maverick'):
                    target=root['targets'][0]['target']['Card'];prior=next(o for o in obs['objects'] if o['id']==target);body=next(o for o in after['objects'] if o['id']==target);self.assertEqual(body['counters']['plus_one'],prior['counters']['plus_one']+(2 if wanted in ('Agent of Kotis','Venom, Evil Unleashed') else 1))
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def test_uncounterable_eject_preserves_native_stack_through_ward_and_replay(self):
        for ward in ['Spectral Snatcher','Sunset Saboteur','Vein Ripper','Sire of Seven Deaths']:
            with self.subTest(ward=ward),Game([['Swamp']*24+[ward]*36,['Forest']*12+['Island']*12+['Grizzly Bears']*12+['Eject']*24],seed=7,deck_rules='Freeform') as game:
                obs=self.reach_nonmana_ward(game,ward,spell='Eject');victim=obs['resolving']['related_stack']['id'];root=next(s for s in obs['stack'] if s['id']==victim)
                action=next(a for a in obs['actions'] if a['kind']==('ChooseOptional' if ward=='Sire of Seven Deaths' else 'DeclineEffectPayment') and (a['kind']!='ChooseOptional' or not a['accept']))
                saved=game.snapshot()
                def finish(handle):
                    after=handle.act(handle.observe('P1'),action);self.assertTrue(any(s['id']==victim for s in after['stack']))
                    result=self.finish_self_target(handle,'P1',victim)
                    target=root['targets'][0]['target']['Card'];self.assertEqual(next(o for o in handle.observe('P0')['objects'] if o['id']==target)['zone'],'Hand')
                    return result
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);self.assertEqual(next(o for o in after['objects'] if o['id']==root['card'])['zone'],'Graveyard')
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def test_four_uncounterable_bodies_survive_native_counter_and_replay(self):
        for wanted in ['Eject','Gigantic Big Bear','Out Cold','Long Goodbye']:
            with self.subTest(card=wanted),Game([['Island']*12+['Swamp']*6+['Forest']*6+[wanted]*36,['Island']*12+['Forest']*12+['Grizzly Bears']*12+['Counterspell']*24],seed=7,deck_rules='Freeform') as game:
                for _ in range(20000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);objects={o['id']:o for o in obs['objects']};victims=[s for s in obs['stack'] if s['source_card']==wanted]
                    if victims and obs['stack'][-1]['source_card']=='Counterspell':break
                    bears=[o for o in objects.values() if o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Grizzly Bears']
                    def score(a):
                        k=a['kind']
                        if k in ('Cast','AnnounceCast'):
                            name=objects[a['card']]['card']
                            if name==wanted:return 200 if wanted=='Gigantic Big Bear' or bears else -100
                            if name=='Grizzly Bears':return 180 if not bears else -100
                            if name=='Counterspell':return 240 if victims and (k=='AnnounceCast' or a['target'].get('Spell')==victims[0]['card']) else -100
                        if k in ('Activate','BeginActivate','AnnounceActivate','AnnounceMana','Attack'):return -100
                        if k=='ChooseTarget':return 240 if (victims and a['target'].get('Spell')==victims[0]['card']) or (bears and a['target'].get('Card')==bears[0]['id']) else -100
                        if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
                        return self.professor_score(obs,a)
                    game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
                else:self.fail('protected spell and opposing counter not reached')
                root=victims[0];counter=obs['stack'][-1];saved=game.snapshot()
                def finish(handle):
                    after=self.finish_self_target(handle,'P0',counter['id']);self.assertTrue(any(s['id']==root['id'] for s in after['stack']))
                    return self.finish_self_target(handle,'P0',root['id'])
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);self.assertEqual(next(o for o in after['objects'] if o['id']==root['card'])['zone'],'Battlefield' if wanted=='Gigantic Big Bear' else 'Graveyard')
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def reach_activation_sacrifice_payment(self,game,wanted):
        for _ in range(15000):
            obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);root=obs['announcing']
            if obs['decision']=='pay_costs' and root['source_card']==wanted and obs['additional_cost'] is not None and any(a['kind']=='PayCosts' for a in obs['actions']):return obs
            objects={o['id']:o for o in obs['objects']};sources=[o for o in objects.values() if o['card']==wanted and o['owner']=='P0' and o['zone']=='Battlefield'];bears=[o for o in objects.values() if o['card']=='Grizzly Bears' and o['owner']=='P0' and o['zone']=='Battlefield'];grave=any(o['card']=='Grizzly Bears' and o['owner']=='P0' and o['zone']=='Graveyard' for o in objects.values())
            def score(a):
                k=a['kind']
                if k=='PlayLand' and objects[a['card']]['card']==wanted:return 200 if not sources else -100
                if k in ('Cast','AnnounceCast'):
                    name=objects[a['card']]['card']
                    if obs['actor']=='P1':return 200 if wanted=='Cauldron of Essence' and bears and not grave and (k=='AnnounceCast' or a['target'].get('Card')==bears[0]['id']) else -100
                    if name==wanted:return 180 if not sources else -100
                    return 100 if name in ('Grizzly Bears','Sweettooth Witch') else -100
                if k=='AnnounceActivate':return 220 if sources and a['card']==sources[0]['id'] and a['ability']==(1 if wanted=='Fountainport' else 0) else -100
                if k in ('Activate','ActivateMana','BeginActivate','AnnounceMana'):return -100
                if k=='ChooseAdditionalCost':return 230 if wanted=="Kingpin's Enforcers" and sources and a['object']==sources[0]['reference'] else 200
                if k=='ChooseTarget':
                    if obs['actor']=='P1':return 200 if bears and a['target'].get('Card')==bears[0]['id'] else -100
                    if wanted=='Cauldron of Essence' and root and root['source_card']==wanted:return 200 if 'Card' in a['target'] and objects[a['target']['Card']]['zone']=='Graveyard' else -100
                    return 180 if a['target'].get('Player')=='P1' else -100
                if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
                return self.professor_score(obs,a)
            game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
        self.fail('external sacrifice payment not reached')

    def test_nineteen_sacrifice_activations_pay_exact_owned_resource_and_replay_native_bodies(self):
        names=['Acolyte of Aclazotz','Ahriman','Cauldron of Essence','Fountainport','Hardened Tactician','Hungry Ghoul',"Kingpin's Enforcers",'Legion Extruder','Marwyn, the Clearcutter','Ripchain Razorkin','Seismic Monstrosaur','Slagdrill Scrapper','Sweettooth Witch','Umbral Collar Zealot','Unburied Earthcarver','Unrooted Ancestor','Vampiric Rites','Vanguard of the Rose',"Vito's Inquisitor"]
        for wanted in names:
            deck=['Plains']*6+['Swamp']*8+['Forest']*4+['Mountain']*6+[wanted]*12+['Grizzly Bears']*12+['Sweettooth Witch']*12
            with self.subTest(card=wanted),Game([deck,['Mountain']*24+['Shock']*24+['Grizzly Bears']*12],seed=7,deck_rules='Freeform') as game:
                obs=self.reach_activation_sacrifice_payment(game,wanted);root=obs['announcing'];selected=obs['additional_cost']['selected'];self.assertIsNotNone(selected);self.assertEqual(obs['additional_cost']['kind'],'Sacrifice');self.assertIsNone(game.observe('P1')['additional_cost'])
                self.assertTrue(any(o['reference']==selected and o['zone']=='Battlefield' for o in obs['objects']));action=next(a for a in obs['actions'] if a['kind']=='PayCosts');saved=game.snapshot()
                def finish(handle):
                    paid=handle.act(handle.observe('P0'),action);self.assertFalse(any(o['reference']==selected and o['zone']=='Battlefield' for o in paid['objects']));self.assertEqual(paid['players'][0]['mana'],[n-c for n,c in zip(obs['players'][0]['mana'],action['payment'])])
                    return self.finish_self_target(handle,'P0',root['id'])
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected)
                if wanted=="Kingpin's Enforcers":self.assertEqual(selected,root['object']);self.assertEqual(next(o for o in after['objects'] if o['id']==root['card'])['zone'],'Graveyard')
                if wanted in ('Hungry Ghoul','Unburied Earthcarver',"Vito's Inquisitor"):
                    body=next(o for o in after['objects'] if o['reference']==root['object']);self.assertEqual(body['counters']['plus_one'],1)
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def reach_tapped_creature_payment(self,game,wanted,opposing=False):
        for _ in range(12000):
            obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
            root=obs['announcing']
            if obs['decision']=='pay_costs' and root['source_card']==wanted and any(a['kind']=='PayCosts' for a in obs['actions']):return obs
            objects={o['id']:o for o in obs['objects']}
            def score(a):
                k=a['kind']
                if k in ('Cast','AnnounceCast'):
                    name=objects[a['card']]['card']
                    if name==wanted:return 170 if (obs['actor']!=obs['active'])==opposing else -100
                    return self.professor_score(obs,a) if obs['actor']=='P1' else -100
                if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
                return self.professor_score(obs,a)
            game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
        self.fail('tapped creature payment not reached')

    def test_tapped_creature_entries_and_flash_conditions_clone_restore_native_payment(self):
        cases=[('Diregraf Ghoul',False),('Piranha Fly',False),('Daring Thunder-Thief',False),('Sleep-Cursed Faerie',False),('Eddymurk Crab',False),('Eddymurk Crab',True)]
        for wanted,opposing in cases:
            with self.subTest(card=wanted,opposing=opposing),Game([['Island']*12+['Swamp']*12+[wanted]*36,['Forest']*24+['Grizzly Bears']*36],seed=7,deck_rules='Freeform') as game:
                obs=self.reach_tapped_creature_payment(game,wanted,opposing);root=obs['announcing'];card=root['card'];source=next(o for o in obs['objects'] if o['id']==card)
                self.assertEqual(source['zone'],'Stack');self.assertFalse(source['tapped']);self.assertEqual(source['counters']['stun'],0)
                action=next(a for a in obs['actions'] if a['kind']=='PayCosts');saved=game.snapshot()
                def finish(handle):
                    handle.act(handle.observe('P0'),action)
                    return self.resolve_vital_top(handle,'P0')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);body=next(o for o in after['objects'] if o['id']==card)
                self.assertEqual(body['zone'],'Battlefield');self.assertEqual(body['tapped'],wanted!='Eddymurk Crab' or opposing);self.assertEqual(body['counters']['stun'],3 if wanted=='Sleep-Cursed Faerie' else 0)
                if wanted=='Eddymurk Crab':self.assertEqual(after['decision'],'targets')
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def reach_nonmana_ward(self,game,wanted,spell="Shock"):
        for _ in range(10000):
            obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
            root=obs['resolving']
            if root and root['source_card']==wanted and root['related_stack'] is not None and obs['decision'] in ('effect_choice','optional_effect'):
                return obs
            objects={o['id']:o for o in obs['objects']};targets=[o for o in objects.values() if o['card']==wanted and o['owner']=='P0' and o['zone']=='Battlefield']
            def score(a):
                k=a['kind']
                if k in ('Activate','AnnounceActivate','BeginActivate','AnnounceMana'):return -100
                if k=='Cast':
                    name=objects[a['card']]['card']
                    if name==wanted:return 160 if not targets else -100
                    if name==spell:
                        resource=(any(o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Grizzly Bears' for o in objects.values()) if wanted=='Vein Ripper' else any(o['owner']=='P1' and o['zone']=='Hand' and o['id']!=a['card'] for o in objects.values()))
                        return 200 if resource and targets and a['target'].get('Card')==targets[0]['id'] else -100
                return self.professor_score(obs,a)
            game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
        self.fail('nonmana Ward choice not reached')

    def test_nonmana_ward_costs_are_private_legal_and_replay_through_native_binding(self):
        for wanted in ['Spectral Snatcher','Sunset Saboteur','Vein Ripper','Sire of Seven Deaths']:
            for pay in [False,True]:
                with self.subTest(card=wanted,pay=pay),Game([['Swamp']*24+[wanted]*36,['Forest']*12+['Mountain']*12+['Grizzly Bears']*12+['Shock']*24],seed=7,deck_rules='Freeform') as game:
                    obs=self.reach_nonmana_ward(game,wanted);self.assertEqual(obs['actor'],'P1');self.assertEqual(game.observe('P0')['actions'],[])
                    if wanted in ('Spectral Snatcher','Sunset Saboteur'):self.assertIsNone(game.observe('P0')['effect_choice'])
                    root=obs['resolving'];victim=root['related_stack']['id'];kind=('ChooseOptional' if wanted=='Sire of Seven Deaths' else ('Sacrifice' if wanted=='Vein Ripper' else 'Discard')) if pay else ('ChooseOptional' if wanted=='Sire of Seven Deaths' else 'DeclineEffectPayment')
                    action=next(a for a in obs['actions'] if a['kind']==kind and (kind!='ChooseOptional' or a['accept']==pay))
                    before=game.observe('P1');saved=game.snapshot()
                    with self.assertRaises(EngineError):game.act(obs,{'kind':'Pass'})
                    self.assertEqual(before,game.observe('P1'))
                    with game.clone() as fork:expected=fork.act(fork.observe('P1'),action)
                    after=game.act(obs,action);self.assertEqual(after,expected)
                    self.assertEqual(any(s['id']==victim for s in after['stack']),pay)
                    if pay and wanted=='Sire of Seven Deaths':self.assertEqual(after['players'][1]['life'],obs['players'][1]['life']-7)
                    if pay and kind in ('Discard','Sacrifice'):
                        card=action['card'] if kind=='Discard' else action['object']['card'];self.assertEqual(next(o for o in after['objects'] if o['id']==card)['zone'],'Graveyard')
                    game.restore(saved)
                    with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                    again=game.act(game.observe('P1'),action);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def exhaust_index(self,obs,wanted,index,bounce=False,prefer_card=None):
        objects={o['id']:o for o in obs['objects']};source=next((o for o in objects.values() if o['card']==wanted and o['owner']=='P0' and o['zone']=='Battlefield'),None)
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==wanted:return 180 if source is None and (prefer_card is None or a['card']==prefer_card) else -100
                if name=='Unsummon':return 210 if bounce and source and source['exhausted_abilities'] else -100
                return -100
            if k in ('AnnounceActivate','AnnounceMana'):
                if not source or a['card']!=source['id'] or a['ability']!=index:return -100
                if k=='AnnounceMana' and a['production']!=[0,0,0,0,3,0]:return -100
                return 200
            if k in ('Activate','ActivateMana','BeginActivate'):return -100
            if k=='ChooseTarget':
                if wanted=='Loot, the Pathfinder' and index==2:return 230 if a['target'].get('Player')=='P1' else -100
                return 220 if source and a['target'].get('Card')==source['id'] else -100
            if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def reach_exhaust_payment(self,game,wanted,index):
        for _ in range(16000):
            obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
            root=obs['announcing']
            if obs['decision']=='pay_costs' and root['source_card']==wanted and root['kind']=={'ActivatedAbility':{'ability':index}} and any(a['kind']=='PayCosts' for a in obs['actions']):return obs
            game.step(obs,self.exhaust_index(obs,wanted,index))
        self.fail('exhaust payment not reached')

    def finish_exhaust(self,game,obs,wanted,index):
        root=obs['announcing'];a=next(a for a in obs['actions'] if a['kind']=='PayCosts');after=game.act(obs,a)
        source=next(o for o in after['objects'] if o['reference']==root['object']);self.assertEqual(source['exhausted_abilities'],1<<index)
        if wanted=='Loot, the Pathfinder' and index==0:
            self.assertTrue(source['tapped']);self.assertEqual(sum(after['players'][0]['mana']),sum(obs['players'][0]['mana'])+2);return after
        return self.finish_self_target(game,'P0',root['id'])

    def test_thirteen_exhaust_abilities_commit_once_and_replay_all_complete_bodies(self):
        cases=[('Camera Launcher',0,1),('Draconautics Engineer',0,1),('Draconautics Engineer',1,0),('Greenbelt Guardian',1,3),('Hazard of the Dunes',0,3),('Keen Buccaneer',0,1),('Loot, the Pathfinder',0,0),('Loot, the Pathfinder',1,0),('Loot, the Pathfinder',2,0),('Pacesetter Paragon',0,1),('Prowcatcher Specialist',0,2),('Skystreak Engineer',0,2),('Stampeding Scurryfoot',0,1)]
        for wanted,index,counters in cases:
            deck=['Island']*12+['Mountain']*12+['Forest']*12+[wanted]*24
            with self.subTest(card=wanted,ability=index),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                obs=self.reach_exhaust_payment(game,wanted,index);source=next(o for o in obs['objects'] if o['reference']==obs['announcing']['object']);self.assertEqual(source['exhausted_abilities'],0);self.assertEqual(obs['schema'],96);saved=game.snapshot()
                with game.clone() as fork:expected=self.finish_exhaust(fork,fork.observe('P0'),wanted,index)
                after=self.finish_exhaust(game,obs,wanted,index);self.assertEqual(after,expected);b=next(o for o in after['objects'] if o['reference']==source['reference']);self.assertEqual(b['exhausted_abilities'],1<<index);self.assertEqual(b['counters']['plus_one'],counters)
                if wanted=='Keen Buccaneer':self.assertEqual(after['players'][0]['cards_drawn_this_turn'],obs['players'][0]['cards_drawn_this_turn']+1);self.assertEqual(after['players'][0]['cards_discarded_this_turn'],obs['players'][0]['cards_discarded_this_turn']+1)
                if wanted=='Loot, the Pathfinder' and index==1:self.assertEqual(after['players'][0]['cards_drawn_this_turn'],obs['players'][0]['cards_drawn_this_turn']+3)
                if wanted=='Loot, the Pathfinder' and index==2:self.assertEqual(after['players'][1]['life'],obs['players'][1]['life']-3)
                if wanted=='Pacesetter Paragon':self.assertTrue(b['keywords']&1024)
                token={'Camera Launcher':('Thopter',1,1),'Stampeding Scurryfoot':('Elephant',3,3)}.get(wanted)
                if wanted=='Draconautics Engineer' and index==1:token=('Dinosaur Dragon',4,4)
                if token:
                    ts=[o for o in after['objects'] if o['zone']=='Battlefield' and o['token'] is not None];self.assertEqual(len(ts),1);self.assertEqual((ts[0]['token']['subtypes'],ts[0]['power'],ts[0]['toughness']),token)
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=self.finish_exhaust(game,game.observe('P0'),wanted,index);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def test_exhaust_persists_across_turns_but_real_bounce_and_recast_reset_identity(self):
        wanted='Camera Launcher';deck=['Island']*24+['Forest']*18+[wanted]*12+['Unsummon']*6
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            obs=self.reach_exhaust_payment(game,wanted,0);after=self.finish_exhaust(game,obs,wanted,0);source=next(o for o in after['objects'] if o['card']==wanted and o['zone']=='Battlefield');old_turn=after['turn'];saved=game.snapshot()
            def later(handle):
                for _ in range(5000):
                    current=handle.observe('P0');self.assertIsNone(current['outcome']);current=handle.observe(current['actor'])
                    if current['turn']>=old_turn+2 and current['actor']=='P0' and current['decision']=='priority':return current
                    handle.step(current,self.exhaust_index(current,wanted,0))
                self.fail('later turn not reached')
            with game.clone() as fork:expected=later(fork)
            obs=later(game);self.assertEqual(obs,expected);b=next(o for o in obs['objects'] if o['reference']==source['reference']);self.assertEqual(b['exhausted_abilities'],1)
            bad={'kind':'AnnounceActivate','card':source['id'],'ability':0,'counter_payment':dict(plus_one=0,minus_one=0,stun=0,loyalty=0)}
            with self.assertRaisesRegex(EngineError,'IllegalAction'):game.act(obs,bad)
            self.assertEqual(game.observe('P0'),obs)
            game.restore(saved);again=later(game);obs['revision']=again['revision'];self.assertEqual(obs,again);game.drop_snapshot(saved)
            for _ in range(6000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Unsummon':break
                game.step(obs,self.exhaust_index(obs,wanted,0,bounce=True))
            else:self.fail('return spell not reached')
            root=obs['stack'][-1];saved=game.snapshot()
            with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
            after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);hand=next(o for o in after['objects'] if o['id']==source['id']);self.assertEqual(hand['zone'],'Hand');self.assertNotEqual(hand['reference'],source['reference']);self.assertEqual(hand['exhausted_abilities'],0)
            public=game.observe('P1');self.assertEqual(public['known_hand'].count(hand['reference']),1);self.assertTrue(any(o['reference']==hand['reference'] for o in public['objects']))
            private=[o for o in after['objects'] if o['owner']=='P0' and o['zone']=='Hand' and o['reference'] not in public['known_hand']];self.assertTrue(private);self.assertTrue(all(o['id'] not in {x['id'] for x in public['objects']} for o in private))
            game.restore(saved)
            with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
            again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)
            for _ in range(6000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor']);b=next((o for o in obs['objects'] if o['id']==source['id'] and o['zone']=='Battlefield'),None)
                if b:break
                game.step(obs,self.exhaust_index(obs,wanted,0,prefer_card=source['id']))
            else:self.fail('recast not reached')
            self.assertNotEqual(b['reference'],source['reference']);self.assertEqual(b['exhausted_abilities'],0);self.assertEqual(b['counters']['plus_one'],0)
            self.assertNotIn(hand['reference'],game.observe('P1')['known_hand'])
            obs=self.reach_exhaust_payment(game,wanted,0);after=self.finish_exhaust(game,obs,wanted,0);self.assertEqual(next(o for o in after['objects'] if o['reference']==b['reference'])['exhausted_abilities'],1)


    def life_condition_index(self,obs,wanted,spell=None,gain=False,enemy_needed=False):
        objects={o['id']:o for o in obs['objects']};own=next((o for o in objects.values() if o['card']==wanted and o['owner']=='P0' and o['zone']=='Battlefield'),None);enemy=next((o for o in objects.values() if o['owner']=='P1' and o['zone']=='Battlefield' and o['card']=='Hill Giant'),None)
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                b=objects[a['card']];name=b['card']
                if b['owner']=='P1':return 200 if name=='Hill Giant' and enemy is None else 190 if name=='Shock' and wanted=='Front Porch Sentries' and own else -100
                if name==wanted:return 220 if own is None and (not enemy_needed or enemy) else -100
                if name=='Moment of Triumph':
                    ready=spell is None or (any(o['card']==spell and o['zone']=='Hand' and o['owner']=='P0' for o in objects.values()) and obs['active']=='P0' and obs['step'] in ('Main1','Main2') and sum(o['zone']=='Battlefield' and o['owner']=='P0' and o['card'] in ('Plains','Swamp','Forest') and not o['tapped'] for o in objects.values())>=4)
                    return 210 if own and gain and ready and obs['players'][0]['life_gained_this_turn']==0 else -100
                if name==spell:return 200 if own and (not gain or obs['players'][0]['life_gained_this_turn']>0) and (not enemy_needed or enemy) else -100
                return -100
            if k in ('Activate','AnnounceActivate','BeginActivate','ActivateMana','AnnounceMana'):return -100
            if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
            if k=='ChooseTarget':
                b=objects.get(a['target'].get('Card'));root=obs['announcing']
                if root['source_card']=='Moment of Triumph' or root['source_card']=='Efflorescence':return 250 if b and own and b['id']==own['id'] else -100
                if root['source_card']=='Shock':return 250 if b and own and b['id']==own['id'] else -100
                return 240 if b and b['owner']!=root['controller'] else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_continuous_life_gain_bonuses_in_native_library_replay(self):
        for wanted,extra in [('Tenured Concocter',2),('Ulna Alley Shopkeep',2),('Thornfist Striker',1)]:
            deck=['Plains']*12+['Swamp']*12+['Forest']*12+[wanted]*12+['Moment of Triumph']*12
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(12000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']=='Moment of Triumph':break
                    game.step(obs,self.life_condition_index(obs,wanted,gain=True))
                else:self.fail('life gain spell not reached')
                root=obs['stack'][-1];source=next(o for o in obs['objects'] if o['card']==wanted and o['zone']=='Battlefield');self.assertEqual(obs['players'][0]['life_gained_this_turn'],0);saved=game.snapshot()
                with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
                after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);now=next(o for o in after['objects'] if o['reference']==source['reference']);self.assertEqual(now['power'],source['power']+2+extra);self.assertEqual(after['players'][0]['life_gained_this_turn'],2)
                if wanted=='Thornfist Striker':self.assertTrue(now['keywords']&256)
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def test_life_gain_spell_branches_and_counters_in_native_library_replay(self):
        for spell in ['Efflorescence','Withering Curse']:
            for gain in [False,True]:
                deck=['Plains']*10+['Swamp']*10+['Forest']*10+['Bear Cub']*10+['Moment of Triumph']*8+[spell]*12
                with self.subTest(card=spell,gain=gain),Game([deck,['Mountain']*36+['Hill Giant']*24],seed=7,deck_rules='Freeform') as game:
                    for _ in range(15000):
                        obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                        if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']==spell:break
                        game.step(obs,self.life_condition_index(obs,'Bear Cub',spell,gain,enemy_needed=True))
                    else:self.fail('conditional spell not reached')
                    root=obs['stack'][-1];self.assertEqual(obs['players'][0]['life_gained_this_turn']>0,gain);bear=next(o for o in obs['objects'] if o['card']=='Bear Cub' and o['zone']=='Battlefield');enemy=next(o for o in obs['objects'] if o['card']=='Hill Giant' and o['zone']=='Battlefield');saved=game.snapshot()
                    with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
                    after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected)
                    if spell=='Efflorescence':
                        b=next(o for o in after['objects'] if o['reference']==bear['reference']);self.assertEqual(b['counters']['plus_one'],bear['counters']['plus_one']+2);self.assertEqual(bool(b['keywords']&8192),gain);self.assertEqual(bool(b['keywords']&256),gain)
                    else:
                        b=next(o for o in after['objects'] if o['id']==enemy['id']);self.assertEqual(b['zone'],'Graveyard' if gain else 'Battlefield')
                        if not gain:self.assertEqual(b['toughness'],enemy['toughness']-2)
                    game.restore(saved);again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)

    def test_opposing_debuff_and_conditional_entry_bodies_native_replay(self):
        for wanted,p,t in [('Ambush Gigapede',-2,-2),('Burrog Befuddler',-1,0),('Cogwork Wrestler',-2,0),('Front Porch Sentries',-1,-1),('Grim Bauble',-2,-2),('Humbling Elder',-2,0),("Poisoner's Apprentice",0,0),('Old-Growth Educator',0,0)]:
            deck=['Island']*12+['Swamp']*12+['Forest']*12+[wanted]*24;other=['Mountain']*24+['Hill Giant']*24+['Shock']*12
            with self.subTest(card=wanted),Game([deck,other],seed=7,deck_rules='Freeform') as game:
                for _ in range(15000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']==wanted and isinstance(obs['stack'][-1]['kind'],dict) and 'TriggeredAbility' in obs['stack'][-1]['kind']:break
                    game.step(obs,self.life_condition_index(obs,wanted,enemy_needed=True))
                else:self.fail('entry/death trigger not reached')
                root=obs['stack'][-1];enemy=next(o for o in obs['objects'] if o['card']=='Hill Giant' and o['zone']=='Battlefield');saved=game.snapshot()
                if wanted!='Old-Growth Educator':self.assertEqual(root['targets'][0]['object'],enemy['reference'])
                with game.clone() as fork:expected=self.finish_self_target(fork,'P0',root['id'])
                after=self.finish_self_target(game,'P0',root['id']);self.assertEqual(after,expected);b=next(o for o in after['objects'] if o['reference']==enemy['reference']);self.assertEqual((b['power'],b['toughness']),(enemy['power']+p,enemy['toughness']+t))
                game.restore(saved);again=self.finish_self_target(game,'P0',root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)


    def self_target_index(self,obs,wanted):
        objects={o['id']:o for o in obs['objects']};valiant=wanted not in ('Cactarantula','Old Fat Spider','Ruric Thar, Biomagus','Tenured Concocter');owner='P0' if valiant else 'P1'
        source=next((o for o in objects.values() if o['card']==wanted and o['zone']=='Battlefield'),None)
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==wanted:return 200 if source is None else -100
                if name=='Giant Growth':return 190 if source else -100
                return 170
            if k in ('Activate','AnnounceActivate','BeginActivate','AnnounceMana','ActivateMana'):return -100
            if k=='ChooseTarget':
                b=objects.get(a['target'].get('Card'));root=obs['announcing']
                if root['source_card']=='Giant Growth':return 250 if b and source and b['reference']==source['reference'] else -100
                return 240 if b and b['owner']!=root['controller'] else -100
            if k=='TapMana':return 130 if obs['decision']=='pay_costs' else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def finish_self_target(self,game,viewer,root):
        for _ in range(100):
            obs=game.observe(viewer)
            live=obs['stack']+obs['waiting_triggers']+[x for x in (obs['announcing'],obs['resolving']) if x]
            if all(s['id']!=root for s in live):return obs
            obs=game.observe(obs['actor'])
            if obs['decision'] in ('library_choice','effect_discard'):
                other=game.observe('P1' if obs['actor']=='P0' else 'P0');self.assertEqual(other['actions'],[]);self.assertIsNone(other['library_choice']);self.assertEqual(other['effect_discards'],[])
            def score(a):
                k=a['kind']
                if k in ('Pass','PutTrigger','Discard','ArrangeLibrary'):return 100
                if k=='ChooseOptional':return 110 if a['accept'] else -100
                if k=='ChooseTarget':return 90
                return -100
            game.step(obs,max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i])))
        self.fail('self-target trigger did not complete')

    def test_eight_complete_self_target_triggers_use_native_library_and_replay(self):
        for wanted in ('Cactarantula','Old Fat Spider','Ruric Thar, Biomagus','Tenured Concocter','Mouse Trapper','Nettle Guard','Seedglaive Mentor','Veteran Guardmouse','Whiskerquill Scribe'):
            valiant=wanted not in ('Cactarantula','Old Fat Spider','Ruric Thar, Biomagus','Tenured Concocter');owner='P0' if valiant else 'P1';p=0 if valiant else 1
            lands=['Plains']*10+['Island']*10+['Mountain']*10+['Forest']*10
            decks=[lands+[wanted]*12+['Giant Growth']*8,['Forest']*48+['Bear Cub']*12] if valiant else [['Forest']*48+['Giant Growth']*12,lands+['Forest']*8+[wanted]*12]
            with self.subTest(card=wanted),Game(decks,seed=7,deck_rules='Freeform') as game:
                for _ in range(15000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='priority' and obs['stack'] and obs['stack'][-1]['source_card']==wanted and isinstance(obs['stack'][-1]['kind'],dict) and 'TriggeredAbility' in obs['stack'][-1]['kind']:break
                    game.step(obs,self.self_target_index(obs,wanted))
                else:self.fail('self-target trigger not reached')
                root=obs['stack'][-1];self.assertEqual(root['controller'],owner);self.assertIsNone(root['related_stack']);self.assertEqual(obs['stack'][0]['source_card'],'Giant Growth');self.assertEqual(obs['schema'],96)
                source=next(o for o in obs['objects'] if o['reference']==root['object']);drawn=obs['players'][p]['cards_drawn_this_turn'];saved=game.snapshot()
                with game.clone() as fork:expected=self.finish_self_target(fork,owner,root['id'])
                after=self.finish_self_target(game,owner,root['id']);self.assertEqual(after,expected)
                b=next(o for o in after['objects'] if o['reference']==source['reference'])
                if not valiant:self.assertEqual(after['players'][p]['cards_drawn_this_turn'],drawn+1)
                else:
                    self.assertTrue(any(t['source']==root['object'] for t in after['triggered_this_turn']))
                    if wanted=='Nettle Guard':self.assertEqual(b['toughness'],source['toughness']+2)
                    if wanted=='Seedglaive Mentor':self.assertEqual(b['counters']['plus_one'],source['counters']['plus_one']+1)
                    if wanted=='Veteran Guardmouse':self.assertEqual(b['power'],source['power']+1);self.assertTrue(b['keywords']&512)
                    if wanted=='Whiskerquill Scribe':self.assertEqual(after['players'][0]['cards_drawn_this_turn'],drawn+1);self.assertGreater(after['players'][0]['cards_discarded_this_turn'],obs['players'][0]['cards_discarded_this_turn'])
                    if wanted=='Mouse Trapper':self.assertTrue(next(o for o in after['objects'] if o['id']==root['targets'][0]['object']['card'])['tapped'])
                game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,0)
                again=self.finish_self_target(game,owner,root['id']);after['revision']=again['revision'];self.assertEqual(after,again);game.drop_snapshot(saved)


    def game(self):
        return Game([["Mountain"] * 24 + ["LightningBolt"] * 36] * 2, seed=17, deck_rules="Freeform")

    def test_power_blocking_choices_enforce_exact_live_power_and_native_replay(self):
        for wanted,lands in [("April O'Neil, Kunoichi Trainee",['Plains']*24),('Prehistoric Pet',['Plains']*24),('Cavern Stomper',['Forest']*24),('Verdant Outrider',['Forest']*24)]:
            temporary=wanted in ('Cavern Stomper','Verdant Outrider')
            deck=lands+[wanted]*36;opponent=['Mountain']*7+['Forest']*7+['Swamp']*6+['Marauding Mako']*14+['Ordinary Bear']*13+['Highborn Vampire']*13
            def allowed(ap,bp):return bp<=ap if wanted=='Prehistoric Pet' else bp<3 if not temporary else bp>2
            def index(obs):
                objects={o['id']:o for o in obs['objects']};source=next((o for o in objects.values() if o['card']==wanted and o['owner']=='P0' and o['zone']=='Battlefield'),None)
                enemies=[o for o in objects.values() if o['owner']=='P1' and o['zone']=='Battlefield' and o['card'] in ('Marauding Mako','Ordinary Bear','Highborn Vampire') and not o['tapped']]
                ready=source and any(allowed(source['power'],b['power']) for b in enemies) and any(not allowed(source['power'],b['power']) for b in enemies)
                def score(a):
                    k=a['kind']
                    if k in ('Cast','AnnounceCast'):
                        if obs['actor']=='P1':return 170
                        return 180 if objects[a['card']]['card']==wanted and source is None else -100
                    if k=='AnnounceActivate':return 190 if temporary and source and a['card']==source['id'] and obs['active']=='P0' and not source['keywords']&2097152 else -100
                    if k in ('Activate','BeginActivate'):return -100
                    if k=='Attack':return 160 if obs['actor']=='P0' and source and a['card']==source['id'] and ready and (not temporary or source['keywords']&2097152) else -100
                    if k=='TapMana':return 170 if obs['decision']=='pay_costs' else -100
                    return self.professor_score(obs,a)
                return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))
            with self.subTest(card=wanted),Game([deck,opponent],seed=7,deck_rules='Freeform') as game:
                for _ in range(15000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='blockers' and obs['active']=='P0' and obs['attackers']:break
                    game.step(obs,index(obs))
                else:self.fail('power blocking decision not reached')
                self.assertEqual(obs['schema'],96);objects={o['id']:o for o in obs['objects']};source=objects[obs['attackers'][0]]
                enemies=[o for o in objects.values() if o['owner']=='P1' and o['zone']=='Battlefield' and o['card'] in ('Marauding Mako','Ordinary Bear','Highborn Vampire') and not o['tapped']]
                offered={a['blocker'] for a in obs['actions'] if a['kind']=='Block' and a['attacker']==source['id']}
                self.assertEqual(offered,{b['id'] for b in enemies if allowed(source['power'],b['power'])});self.assertTrue(offered)
                bad=next(b['id'] for b in enemies if not allowed(source['power'],b['power']));saved=game.snapshot()
                with self.assertRaisesRegex(EngineError,'IllegalAction'):game.act(obs,{'kind':'Block','blocker':bad,'attacker':source['id']})
                self.assertEqual(game.observe('P1'),obs);a=next(i for i,a in enumerate(obs['actions']) if a['kind']=='Block')
                with game.clone() as fork:expected=fork.step(fork.observe('P1'),a)
                after=game.step(obs,a);self.assertEqual(after,expected);game.restore(saved)
                with self.assertRaisesRegex(EngineError,'StaleRevision'):game.step(obs,a)
                again=game.step(game.observe('P1'),a);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def recovery_index(self,obs,wanted,ability=None):
        objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0'];source=next((o for o in own if o['card']==wanted and o['zone']=='Battlefield'),None)
        grave=any(o['card']=='Shock' and o['zone']=='Graveyard' for o in own)
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==wanted:return 180 if source is None and (wanted in ('Rescue Girl, First Responder','Prehistoric Pet') or grave) else -100
                if name=='Three Tree Mascot' and wanted=='Prehistoric Pet':return 160 if not any(o['card']==name and o['zone']=='Battlefield' for o in own) else -100
                if name=='Shock':return 170 if not grave and (k!='Cast' or a['target']=={'Player':'P1'}) else -100
                return -100
            if k=='AnnounceActivate':return 190 if source and a['card']==source['id'] and a['ability']==ability else -100
            if k in ('Activate','BeginActivate','AnnounceMana','ActivateMana'):return -100
            if k=='ChooseTarget':
                if a['target'].get('Player')=='P1':return 230
                b=objects.get(a['target'].get('Card'))
                if b and b['owner']=='P0' and ((wanted=='Rescue Girl, First Responder' and b['zone']=='Battlefield' and b['card']=='Plains') or (wanted=='Prehistoric Pet' and b['zone']=='Battlefield' and b['card']=='Three Tree Mascot') or (wanted not in ('Rescue Girl, First Responder','Prehistoric Pet') and b['zone']=='Graveyard' and b['card']=='Shock')):return 220
                return -100
            if k=='TapMana':return 160 if obs['decision']=='pay_costs' else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_own_turn_recovery_activations_are_native_and_replayable(self):
        for wanted,ability,deck in [
            ('Professor Zei, Anthropologist',1,['Mountain']*36+['Professor Zei, Anthropologist']*12+['Shock']*12),
            ('Rescue Girl, First Responder',0,['Plains']*36+['Rescue Girl, First Responder']*24),
            ('Prehistoric Pet',0,['Plains']*36+['Prehistoric Pet']*12+['Three Tree Mascot']*12),
        ]:
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['stack'] and obs['stack'][-1]['source_card']==wanted and obs['stack'][-1]['kind']!='Spell':break
                    game.step(obs,self.recovery_index(obs,wanted,ability))
                else:self.fail('own-turn recovery activation not reached')
                root=obs['stack'][-1];target=root['targets'][0]['object']['card'];self.assertEqual(obs['active'],'P0');saved=game.snapshot()
                source=next(o for o in obs['objects'] if o['id']==root['card'])
                self.assertEqual(source['zone'],'Graveyard' if ability==1 else 'Battlefield')
                if ability==0:self.assertTrue(source['tapped'])
                with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
                after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected);self.assertEqual(next(o for o in after['objects'] if o['id']==target)['zone'],'Hand')
                game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_typed_instant_sorcery_recovery_entry_triggers_replay(self):
        for wanted in ['Shipwreck Dowser','Zealous Lorecaster']:
            deck=['Mountain']*18+['Island']*18+['Shock']*12+[wanted]*12
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['stack'] and obs['stack'][-1]['source_card']==wanted and obs['stack'][-1]['kind']!='Spell' and obs['stack'][-1]['targets']:break
                    game.step(obs,self.recovery_index(obs,wanted))
                else:self.fail('typed recovery entry trigger not reached')
                target=obs['stack'][-1]['targets'][0]['object']['card'];saved=game.snapshot()
                with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
                after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected);self.assertEqual(next(o for o in after['objects'] if o['id']==target)['zone'],'Hand')
                game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_misty_draw_uses_public_committed_discard_history_and_native_replay(self):
        wanted='Misty Knight, Hero for Hire';deck=['Mountain']*36+[wanted]*24
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(10000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='additional_cost':break
                game.step(obs,self.activation_discard_index(obs,wanted,0))
            else:self.fail('Misty discard choice not reached')
            self.assertEqual(obs['schema'],96);self.assertEqual(obs['players'][0]['cards_discarded_this_turn'],0)
            game.step(obs,next(i for i,a in enumerate(obs['actions']) if a['kind']=='ChooseAdditionalCost'))
            for _ in range(100):
                obs=game.observe('P0');obs=game.observe(obs['actor'])
                if obs['stack'] and obs['stack'][-1]['source_card']==wanted and obs['stack'][-1]['kind']!='Spell':break
                game.step(obs,self.activation_discard_index(obs,wanted,0))
            else:self.fail('Misty payment stalled')
            self.assertEqual(obs['players'][0]['cards_discarded_this_turn'],1)
            self.assertEqual(game.observe('P1')['players'][0]['cards_discarded_this_turn'],1)
            drawn=obs['players'][0]['cards_drawn_this_turn'];saved=game.snapshot()
            with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
            after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected)
            self.assertEqual(after['players'][0]['cards_drawn_this_turn'],drawn+1)
            game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_goblin_private_discard_resumes_scaled_draw_and_restores_history(self):
        wanted='Green Goblin, Revenant';deck=['Mountain']*18+['Swamp']*18+[wanted]*24
        def index(obs):
            objects={o['id']:o for o in obs['objects']};present=any(o['card']==wanted and o['owner']=='P0' and o['zone']=='Battlefield' for o in obs['objects'])
            def score(a):
                k=a['kind']
                if k in ('Cast','AnnounceCast'):return 170 if objects[a['card']]['card']==wanted and not present else -100
                if k=='Attack':return 160 if objects[a['card']]['card']==wanted else -100
                if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
                return self.professor_score(obs,a)
            return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(10000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='effect_discard' and obs['resolving']['source_card']==wanted:break
                game.step(obs,index(obs))
            else:self.fail('Goblin attack discard not reached')
            prior=obs['players'][0]['cards_discarded_this_turn'];drawn=obs['players'][0]['cards_drawn_this_turn'];saved=game.snapshot()
            self.assertEqual(obs['schema'],96);self.assertEqual(game.observe('P1')['effect_discards'],[])
            def finish(handle):
                current=handle.observe('P0');a=next(i for i,a in enumerate(current['actions']) if a['kind']=='Discard')
                return handle.step(current,a)
            with game.clone() as fork:expected=finish(fork)
            after=finish(game);self.assertEqual(after,expected);self.assertEqual(after['players'][0]['cards_discarded_this_turn'],prior+1)
            self.assertEqual(after['players'][0]['cards_drawn_this_turn'],drawn+prior+1);self.assertEqual(game.observe('P1')['players'][0]['cards_discarded_this_turn'],prior+1)
            game.restore(saved);self.assertEqual(game.observe('P0')['players'][0]['cards_discarded_this_turn'],prior)
            again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def activation_discard_index(self,obs,wanted,ability):
        objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];source=next((o for o in own if o['card']==wanted),None)
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):return 170 if objects[a['card']]['card']==wanted and source is None else -100
            if k=='AnnounceActivate':return 190 if source and a['card']==source['id'] and a['ability']==ability else -100
            if k in ('Activate','BeginActivate','ActivateMana','AnnounceMana'):return -100
            if k=='ChooseAdditionalCost':return 200
            if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_activation_discard_is_private_committed_and_replayable_for_supported_bodies(self):
        for wanted,ability,keyword in [
            ('Flamecache Gecko',0,0),('Charging Strifeknight',0,0),
            ('Inner Demons Gangsters',0,32768),('Iron-Shield Elf',0,8192),
            ('Masked Meower',0,0),('Murmuring Volume',1,0),('Thunderhead Gunner',0,0),('Professor Zei, Anthropologist',0,0),
        ]:
            deck=['Mountain']*18+['Swamp']*18+[wanted]*24
            with self.subTest(card=wanted),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='additional_cost':break
                    game.step(obs,self.activation_discard_index(obs,wanted,ability))
                else:self.fail('activation discard choice not reached')
                self.assertEqual(obs['schema'],96);self.assertIsNone(obs['payment_request']);self.assertTrue(obs['additional_cost']['candidates'])
                self.assertIsNone(game.observe('P1')['additional_cost']);saved=game.snapshot()
                def finish(handle):
                    current=handle.observe('P0');selection=current['additional_cost']['candidates'][0]
                    index=next(i for i,a in enumerate(current['actions']) if a['kind']=='ChooseAdditionalCost' and a['object']==selection)
                    current=handle.step(current,index)
                    self.assertEqual(current['decision'],'pay_costs');self.assertEqual(current['additional_cost']['selected'],selection)
                    self.assertIsNone(handle.observe('P1')['additional_cost']);self.assertFalse(any(o['id']==selection['card'] for o in handle.observe('P1')['objects']))
                    for _ in range(100):
                        current=handle.observe('P0');current=handle.observe(current['actor'])
                        if current['stack'] and current['stack'][-1]['source_card']==wanted and current['stack'][-1]['kind']!='Spell':
                            source=current['stack'][-1]['card'];self.assertTrue(any(o['id']==selection['card'] and o['zone']=='Graveyard' for o in handle.observe('P1')['objects']))
                            drawn=current['players'][0]['cards_drawn_this_turn'];after=self.resolve_vital_top(handle,'P0');obj=next(o for o in after['objects'] if o['id']==source)
                            if keyword:self.assertTrue(obj['keywords']&keyword)
                            else:self.assertEqual(after['players'][0]['cards_drawn_this_turn'],drawn+1)
                            if wanted=='Masked Meower':self.assertEqual(obj['zone'],'Graveyard')
                            if wanted in ('Charging Strifeknight','Murmuring Volume','Iron-Shield Elf'):self.assertTrue(obj['tapped'])
                            return after
                        handle.step(current,self.activation_discard_index(current,wanted,ability))
                    self.fail('activation payment stalled')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved)
                again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def hybrid_activation_index(self,obs,wanted,ability,required,generic):
        objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];source=next((o for o in own if o['card']==wanted),None)
        colors={'Plains':0,'Island':1,'Swamp':2,'Mountain':3,'Forest':4}
        available=obs['players'][0]['mana'][:]
        for o in own:
            if o['card'] in colors and not o['tapped']:available[colors[o['card']]]+=1
        ready=all(a>=b for a,b in zip(available,required)) and sum(available)>=sum(required)+generic
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==wanted:return 170 if source is None and (wanted!='Hovel Hurler' or any(o['card']=='Three Tree Mascot' for o in own)) else -100
                if name=='Three Tree Mascot':return 150 if not any(o['card']==name for o in own) else -100
                return -100
            if k=='AnnounceActivate':return 190 if source is not None and a['card']==source['id'] and a['ability']==ability and ready else -100
            if k in ('Activate','BeginActivate'):return -100
            if k=='ChooseActivationCost':return 200 if any(c['index']==a['cost'] and c['cost']=={'colored':required,'generic':generic} for c in obs['cost_choices']) else -100
            if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
            if k=='ChooseTarget':return 180 if 'Card' in a['target'] and objects[a['target']['Card']]['owner']=='P0' else 80
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_hybrid_activation_costs_are_owned_committed_and_replay_both_colors(self):
        for wanted,ability,required,generic,keyword,lands in [
            ('Cindering Cutthroat',0,[0,0,1,0,0,0],1,32768,['Swamp']*36),
            ('Cindering Cutthroat',0,[0,0,0,1,0,0],1,32768,['Mountain']*36),
            ('Hovel Hurler',0,[1,0,0,1,0,0],0,1,['Plains']*18+['Mountain']*18),
            ('Trostani, Three Whispers',1,[1,0,0,0,0,0],0,8,['Plains']*18+['Forest']*18),
            ('Trostani, Three Whispers',1,[0,0,0,0,1,0],0,8,['Plains']*18+['Forest']*18),
        ]:
            deck=lands+[wanted]*12+['Three Tree Mascot']*12
            with self.subTest(card=wanted,required=required),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['decision']=='activation_cost':break
                    game.step(obs,self.hybrid_activation_index(obs,wanted,ability,required,generic))
                else:self.fail('hybrid activation cost choice not reached')
                self.assertEqual(obs['schema'],96);self.assertIsNone(obs['payment_request']);self.assertEqual(game.observe('P1')['cost_choices'],[])
                self.assertTrue(any(c['cost']=={'colored':required,'generic':generic} for c in obs['cost_choices']));saved=game.snapshot()
                def finish(handle):
                    for _ in range(100):
                        current=handle.observe('P0');current=handle.observe(current['actor'])
                        if current['stack'] and current['stack'][-1]['source_card']==wanted and current['stack'][-1]['kind']!='Spell':
                            item=current['stack'][-1];target=item['targets'][0]['object']['card'] if item['targets'] else item['card'];after=self.resolve_vital_top(handle,'P0')
                            self.assertTrue(next(o for o in after['objects'] if o['id']==target)['keywords']&keyword);return after
                        if current['decision']=='pay_costs':
                            self.assertEqual(current['payment_request']['required'],required);self.assertEqual(current['payment_request']['generic'],generic)
                        handle.step(current,self.hybrid_activation_index(current,wanted,ability,required,generic))
                    self.fail('hybrid activation stalled')
                with game.clone() as fork:expected=finish(fork)
                after=finish(game);self.assertEqual(after,expected);game.restore(saved);again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def loss_index(self,obs,wanted,positive=True):
        objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield'];lands=len([o for o in own if o['card'] in ('Mountain','Swamp')]);lost=obs['players'][1]['life_lost_this_turn']>0
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==wanted:
                    ready=(lost==positive if wanted!='Stromkirk Bloodthief' else not any(o['card']==wanted for o in own))
                    return (180 if k=='AnnounceCast' else 170) if ready else -100
                if name=='Shock':return 160 if positive and not lost and lands>=6 and obs['active']=='P0' and not obs['stack'] else -100
                return -100
            if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
            if k=='ChooseTarget':
                return 190 if a['target'].get('Player')=='P1' else 150
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_opponent_life_loss_entry_counters_replay_for_zero_and_positive_history(self):
        wanted='Frilled Sparkshooter';deck=['Mountain']*36+['Shock']*12+[wanted]*12
        for positive in [False,True]:
            with self.subTest(positive=positive),Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
                for _ in range(10000):
                    obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                    if obs['stack'] and obs['stack'][-1]['source_card']==wanted:break
                    game.step(obs,self.loss_index(obs,wanted,positive))
                else:self.fail('life-loss entry cast not reached')
                self.assertEqual(obs['schema'],96);self.assertEqual(obs['players'][1]['life_lost_this_turn']>0,positive)
                self.assertEqual(game.observe('P1')['players'][1]['life_lost_this_turn'],obs['players'][1]['life_lost_this_turn'])
                card=obs['stack'][-1]['card'];saved=game.snapshot()
                with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
                after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected);obj=next(o for o in after['objects'] if o['id']==card)
                self.assertEqual(obj['counters']['plus_one'],int(positive));self.assertEqual((obj['power'],obj['toughness']),(3+positive,3+positive))
                game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_bloodtithe_opponent_private_discard_replays_after_life_loss(self):
        wanted='Bloodtithe Collector';deck=['Swamp']*18+['Mountain']*18+['Shock']*12+[wanted]*12
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(10000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='effect_discard' and obs['resolving']['source_card']==wanted:break
                game.step(obs,self.loss_index(obs,wanted))
            else:self.fail('Bloodtithe discard not reached')
            self.assertEqual(obs['actor'],'P1');self.assertGreater(obs['players'][1]['life_lost_this_turn'],0);self.assertEqual(game.observe('P0')['effect_discards'],[])
            action=next(a for a in obs['actions'] if a['kind']=='Discard');saved=game.snapshot()
            with game.clone() as fork:expected=fork.act(fork.observe('P1'),action)
            after=game.act(obs,action);self.assertEqual(after,expected);self.assertEqual(after['players'][1]['hand_count'],obs['players'][1]['hand_count']-1)
            game.restore(saved);again=game.act(game.observe('P1'),action);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_stromkirk_guarded_vampire_target_counters_replay_after_opponent_loss(self):
        wanted='Stromkirk Bloodthief';deck=['Swamp']*18+['Mountain']*18+['Shock']*12+[wanted]*12
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(10000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['stack'] and obs['stack'][-1]['source_card']==wanted and obs['stack'][-1]['kind']!='Spell':break
                game.step(obs,self.loss_index(obs,wanted))
            else:self.fail('Stromkirk trigger not reached')
            self.assertGreater(obs['players'][1]['life_lost_this_turn'],0);card=obs['stack'][-1]['targets'][0]['target']['Card'];old=next(o for o in obs['objects'] if o['id']==card)['counters']['plus_one'];saved=game.snapshot()
            with game.clone() as fork:expected=self.resolve_vital_top(fork,'P0')
            after=self.resolve_vital_top(game,'P0');self.assertEqual(after,expected);self.assertEqual(next(o for o in after['objects'] if o['id']==card)['counters']['plus_one'],old+1)
            game.restore(saved);again=self.resolve_vital_top(game,'P0');after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def death_index(self,obs,wanted):
        objects={o['id']:o for o in obs['objects']};own=[o for o in objects.values() if o['owner']=='P0' and o['zone']=='Battlefield']
        n=obs['creatures_died_this_turn'];has_wanted=any(o['card']==wanted for o in own)
        def score(a):
            k=a['kind']
            if k in ('Cast','AnnounceCast'):
                name=objects[a['card']]['card']
                if name==wanted:return (180 if k=='AnnounceCast' else 170) if (n>0 if wanted=='Dreaded Bat-Cloud' else not has_wanted) else -100
                if name=='Three Tree Mascot':return 150 if len([o for o in own if o['card']==name])<2 else -100
                if name=='Corrupted Conviction':return 160 if n==0 and obs['active']=='P0' and (wanted=='Dreaded Bat-Cloud' or has_wanted) and any(o['card']=='Three Tree Mascot' for o in own) else -100
                return -100
            if k=='TapMana':return 140 if obs['decision']=='pay_costs' else -100
            if k=='ChooseAdditionalCost':return 170 if objects[a['object']['card']]['card']=='Three Tree Mascot' else 80
            if k=='ChooseTarget':return 150
            return self.professor_score(obs,a)
        return max(range(len(obs['actions'])),key=lambda i:score(obs['actions'][i]))

    def test_creature_death_history_and_morbid_cost_replay_through_native_library(self):
        wanted='Dreaded Bat-Cloud';deck=['Swamp']*30+['Three Tree Mascot']*12+['Corrupted Conviction']*9+[wanted]*9
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(10000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                self.assertEqual(game.observe('P1')['creatures_died_this_turn'],obs['creatures_died_this_turn'])
                if obs['decision']=='pay_costs' and obs['announcing']['source_card']==wanted:break
                game.step(obs,self.death_index(obs,wanted))
            else:self.fail('morbid payment not reached')
            self.assertGreater(obs['creatures_died_this_turn'],0);self.assertEqual(obs['schema'],96)
            self.assertEqual(obs['payment_request']['generic'],1);self.assertEqual(obs['payment_request']['required'],[0,0,1,0,0,0]);saved=game.snapshot()
            def finish(handle):
                for _ in range(50):
                    current=handle.observe('P0');current=handle.observe(current['actor'])
                    if current['stack'] and current['stack'][-1]['source_card']==wanted:
                        card=current['stack'][-1]['card'];after=self.resolve_vital_top(handle,'P0')
                        obj=next(o for o in after['objects'] if o['id']==card);self.assertEqual(obj['zone'],'Battlefield');self.assertEqual((obj['power'],obj['toughness']),(4,2));return after
                    a=current['actions'][self.death_index(current,wanted)]
                    if a['kind']=='PayCosts':self.assertEqual(sum(a['payment']),2)
                    handle.act(current,a)
                self.fail('morbid payment stalled')
            with game.clone() as fork:expected=finish(fork)
            after=finish(game);self.assertEqual(after,expected);game.restore(saved);again=finish(game);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

    def test_morbid_connive_private_discard_replays_without_repeating_draw(self):
        wanted='Scorpion, Seething Striker';deck=['Swamp']*30+['Three Tree Mascot']*12+[wanted]*12+['Corrupted Conviction']*6
        with Game([deck,['Forest']*60],seed=7,deck_rules='Freeform') as game:
            for _ in range(10000):
                obs=game.observe('P0');self.assertIsNone(obs['outcome']);obs=game.observe(obs['actor'])
                if obs['decision']=='effect_discard' and obs['connive_choice'] is not None:break
                game.step(obs,self.death_index(obs,wanted))
            else:self.fail('morbid connive not reached')
            self.assertGreater(obs['creatures_died_this_turn'],0);library=obs['players'][0]['library_count']
            other=game.observe('P1');self.assertEqual(other['actions'],[]);self.assertEqual(other['effect_discards'],[])
            action=next(a for a in obs['actions'] if a['kind']=='Discard');saved=game.snapshot()
            with game.clone() as fork:expected=fork.act(fork.observe('P0'),action)
            after=game.act(obs,action);self.assertEqual(after,expected);self.assertEqual(after['players'][0]['library_count'],library)
            game.restore(saved);again=game.act(game.observe('P0'),action);after['revision']=again['revision'];self.assertEqual(again,after);game.drop_snapshot(saved)

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
                n=obs['creatures_attacked_this_turn'];self.assertGreaterEqual(n,3);self.assertEqual(obs['schema'],96)
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
                    amount=obs['players'][0]['life_gained_this_turn'];self.assertEqual(obs['schema'],96)
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
                    self.assertEqual(obs['schema'],96)
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
                        self.assertGreater(current['counters']['plus_one'],2**40);self.assertEqual(after['schema'],96);obs=after;break
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
            after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected);self.assertEqual(after['schema'],96)
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
            after=self.resolve_vital_top(game,actor);self.assertEqual(after,expected);self.assertEqual(after['schema'],96)
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
                        view=obs['additional_cost'];self.assertEqual(obs['schema'],96)
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
            self.assertEqual(obs['schema'],96)
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
            self.assertEqual(obs['schema'],96);self.assertIsNone(obs['announcing_cost']);self.assertIsNone(obs['payment_request'])
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
            self.assertEqual(obs['schema'],96)
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
            self.assertEqual(obs['schema'],96);self.assertEqual(obs['connive_choice']['amount'],1);self.assertEqual(obs['connive_choice']['current']['controller'],actor)
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
            self.assertEqual(obs['schema'],96);self.assertEqual(obs['players'][0]['cards_drawn_this_turn'],2)
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
            actor=obs['actor'];self.assertEqual(obs['schema'],96)
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
            self.assertEqual(obs['schema'],96)
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
            self.assertEqual(obs['schema'],96);self.assertEqual(request['payer'],actor);self.assertEqual(request['cost']['generic'],2)
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
            event=obs['resolving']['damage_event'];self.assertEqual(obs['schema'],96)
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
            choice=obs['recovery_choice'];self.assertEqual(obs['schema'],96)
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
            self.assertEqual(obs['schema'],96);self.assertEqual(obs['resolving']['source_card'],'Disenchant')
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
            self.assertEqual(obs['schema'],96)
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
            self.assertEqual(observation["schema"], 96)
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
            self.assertTrue(all(o["token"] == {"factory_identity":o["card"],"name": "Goblin Token", "subtypes": "Goblin", "colors": 8}
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
