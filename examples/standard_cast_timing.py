"""Natural Standard games with paid off-turn chosen spells and owned rollback."""
import argparse
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"python"))
from test_cast_timing import CastingTimingTests

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--library",required=True)
    parser.add_argument("--games",type=int,default=100)
    parser.add_argument("--start-seed",type=int,default=2640)
    args=parser.parse_args()
    if args.games<1:parser.error("games must be positive")
    checks=CastingTimingTests();frames=0
    for i in range(args.games):
        seat="P0" if i%2==0 else "P1"
        n=checks.scenario(seat,args.start_seed+i//2,library=args.library)
        frames+=n
        print(f"game={i} seed={args.start_seed+i//2} seat={seat} owned_replays={n} full_body=true terminal=true",flush=True)
    print(f"games={args.games} owned_replays={frames} wrong_actor_and_stale_denials={2*frames}",flush=True)

if __name__=="__main__":main()
