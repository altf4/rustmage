#!/usr/bin/env python3
"""Fixed unretried Standard workload; acceptance pending composed library."""
import argparse,json,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_resolved_base_stats import ResolvedBasePublicTests,CASES
p=argparse.ArgumentParser();p.add_argument('--library',required=True);p.add_argument('--games',type=int,default=100);a=p.parse_args();t=ResolvedBasePublicTests();frames=0;seen=set();start=time.monotonic()
for game in range(a.games):
 i=(game//2)%len(CASES);seat=['P0','P1'][game%2];seed=32600+i+100*(game//(2*len(CASES)));result=t.scenario(i,seat,seed,library=a.library);frames+=result['frames'];seen.add((i,seat));print(json.dumps(dict(game=game,**result)),flush=True)
if a.games>=2*len(CASES):assert len(seen)==2*len(CASES)
print(json.dumps(dict(games=a.games,complete_vectors=len(seen),owned_frames=frames,wrong_actor_and_stale_denials=frames*2,seconds=time.monotonic()-start)),flush=True)
