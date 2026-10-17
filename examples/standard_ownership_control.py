"""Natural Standard Korvold games with owned rollout and foreign-control witnesses."""
import argparse
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_ownership_control import OwnershipControlTests, PAIRS, BLOOM

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--library',required=True)
    parser.add_argument('--games',type=int,default=100)
    parser.add_argument('--start-seed',type=int,default=2840)
    args=parser.parse_args()
    if args.games<1:parser.error('games must be positive')
    check=OwnershipControlTests();frames=0;coverage={}
    for i in range(args.games):
        player='P0' if i%2==0 else 'P1'
        result=check.scenario(player,args.start_seed+i//2,library=args.library,coverage=coverage)
        key=(result['name'],result['route'],player);coverage[key]=coverage.get(key,0)+1;frames+=result['frames']
        print('game',i,'terminal=true',result,flush=True)
    if args.games>=100:
        expected={(name,'Omen' if name==BLOOM else 'Adventure',player) for name in PAIRS for player in ['P0','P1']}
        assert expected<=coverage.keys(),('missing foreign route/seat witnesses',expected-coverage.keys())
    print('games',args.games,'owned_replays',frames,'denials',2*frames,'coverage',coverage,flush=True)

if __name__=='__main__':main()
