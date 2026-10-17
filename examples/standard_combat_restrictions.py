#!/usr/bin/env python3
"""Fixed, unretried complete combat-restriction Standard workload."""
import argparse,json,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_combat_restrictions import CombatRestrictionTests,NAMES
p=argparse.ArgumentParser();p.add_argument('--library',required=True);p.add_argument('--games',type=int,default=100);args=p.parse_args();case=CombatRestrictionTests();frames=0;seen=set();start=time.monotonic()
for game in range(args.games):
 name=NAMES[(game//2)%len(NAMES)];seat=['P0','P1'][game%2];seed=30800+(game//2)%len(NAMES)+100*(game//(2*len(NAMES)));result=case.scenario(name,seat,seed,library=args.library);frames+=result['frames'];seen.add((name,seat));print(json.dumps(dict(game=game,seed=seed,**result)),flush=True)
if args.games>=2*len(NAMES):assert len(seen)==2*len(NAMES)
print(json.dumps({'games':args.games,'complete_vectors':len(seen),'owned_frames':frames,'wrong_actor_and_stale_denials':frames*2,'seconds':time.monotonic()-start}),flush=True)
