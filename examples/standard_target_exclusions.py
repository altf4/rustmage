#!/usr/bin/env python3
"""Fixed unretried complete-body filter/modal workload, all26 body/mode/seat cells."""
import argparse,json,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_target_exclusions import TargetExclusionTests,SIMPLE,MODAL
p=argparse.ArgumentParser();p.add_argument('--library',required=True);p.add_argument('--games',type=int,default=100);args=p.parse_args()
vectors=[(name,seat,None,3000+i) for i,name in enumerate(SIMPLE) for seat in ['P0','P1']]+[(name,seat,mode,3010+i*3+mode) for i,name in enumerate(MODAL) for mode in range(3) for seat in ['P0','P1']]
assert len(vectors)==26
case=TargetExclusionTests();frames=0;seen=set();start=time.monotonic()
for game in range(args.games):
 name,seat,mode,seed=vectors[game%len(vectors)];seed+=100*(game//len(vectors));result=case.scenario(name,seat,mode,seed,library=args.library);frames+=result['frames'];seen.add((name,seat,mode));print(json.dumps(dict(game=game,seed=seed,**result)),flush=True)
if args.games>=26:assert len(seen)==26
print(json.dumps({'games':args.games,'complete_vectors':len(seen),'owned_frames':frames,'wrong_actor_and_stale_denials':frames*2,'seconds':time.monotonic()-start}),flush=True)
