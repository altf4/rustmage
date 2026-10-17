"""Fixed, unretried naturally terminal Standard games across six complete bodies."""
import argparse
import json
import sys
import time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_role_semantic_kernels import RoleSemanticTests,NAMES

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--games',type=int,default=100)
    parser.add_argument('--library',required=True)
    args=parser.parse_args();assert args.games>0
    start=time.monotonic();case=RoleSemanticTests();frames=0
    for number in range(args.games):
        card=(number//2)%len(NAMES);player='P0' if number%2==0 else 'P1'
        seed=29500+card+83*(number//12)
        owned=case.scenario(card,player,seed,library=args.library)
        frames+=owned
        print(json.dumps(dict(game=number,card=NAMES[card],player=player,seed=seed,frames=owned)),flush=True)
    print(json.dumps(dict(games=args.games,frames=frames,denials=2*frames,seconds=time.monotonic()-start)),flush=True)
