#!/usr/bin/env python3
"""Fixed, unretried complete battlefield-return Standard workload."""
import argparse,json,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_battlefield_returns import BattlefieldReturnTests
p=argparse.ArgumentParser();p.add_argument('--library',required=True);p.add_argument('--games',type=int,default=100);args=p.parse_args();case=BattlefieldReturnTests();frames=0;seen=set();start=time.monotonic()
for game in range(args.games):
 name=['Aetherize',"River's Rebuke"][(game//2)%2];seat=['P0','P1'][game%2];seed=30600+((game//2)%2)+100*(game//4);result=case.scenario(name,seat,seed,library=args.library);frames+=result['frames'];seen.add((name,seat));print(json.dumps(dict(game=game,seed=seed,**result)),flush=True)
if args.games>=4:assert len(seen)==4
print(json.dumps({'games':args.games,'complete_vectors':len(seen),'owned_frames':frames,'wrong_actor_and_stale_denials':frames*2,'seconds':time.monotonic()-start}),flush=True)
