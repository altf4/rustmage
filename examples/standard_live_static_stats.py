#!/usr/bin/env python3
"""Fixed unretried Standard workload for all four complete live-static bodies."""
import argparse,json,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_live_static_stats import LiveStaticPublicTests,CASES
p=argparse.ArgumentParser();p.add_argument('--library',required=True);p.add_argument('--games',type=int,default=100);args=p.parse_args();case=LiveStaticPublicTests();frames=0;seen=set();start=time.monotonic()
for game in range(args.games):
 index=(game//2)%len(CASES);seat=['P0','P1'][game%2];seed=33100+index+100*(game//(2*len(CASES)));result=case.scenario(index,seat,seed,library=args.library);frames+=result['frames'];seen.add((index,seat));print(json.dumps(dict(game=game,seed=seed,**result)),flush=True)
if args.games>=2*len(CASES):assert len(seen)==2*len(CASES)
print(json.dumps({'games':args.games,'complete_vectors':len(seen),'owned_frames':frames,'wrong_actor_and_stale_denials':frames*2,'seconds':time.monotonic()-start}),flush=True)
