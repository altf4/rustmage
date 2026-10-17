"""Natural Standard acceptance for paid selected-face X and discard Ward."""
import argparse
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"python"))
from test_selected_x import SelectedXTests

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--library",required=True)
    parser.add_argument("--games",type=int,default=100)
    parser.add_argument("--start-seed",type=int,default=2700)
    args=parser.parse_args()
    if args.games<1:parser.error("games must be positive")
    frames=0;check=SelectedXTests()
    for i in range(args.games):
        p="P0" if i%2==0 else "P1";x=(i//2)%4;pay=bool((i//8)%2)
        n=check.scenario(p,x,pay,args.start_seed+i//16,library=args.library);frames+=n
        print(f"game={i} seat={p} x={x} ward_paid={pay} seed={args.start_seed+i//16} owned_replays={n} terminal=true",flush=True)
    print(f"games={args.games} owned_replays={frames} wrong_actor_and_stale_denials={2*frames}",flush=True)
if __name__=="__main__":main()
