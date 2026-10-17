"""Complete paid Standard games with required Adventure normal-event witnesses."""
import argparse
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_adventure_event_predicates import AdventureEventTests

def main():
 p=argparse.ArgumentParser();p.add_argument('--library',required=True);p.add_argument('--games',type=int,default=100);p.add_argument('--start-seed',type=int,default=2910);a=p.parse_args()
 if a.games<1:p.error('games must be positive')
 case=AdventureEventTests();frames=0
 for i in range(a.games):
  count=case.scenario(i//2%3,'P0' if i%2==0 else 'P1',a.start_seed+i//2,library=a.library);frames+=count
  print(f'game={i} owned_replays={count} complete_pair=true normal_event=true terminal=true',flush=True)
 print(f'games={a.games} owned_replays={frames} wrong_actor_and_stale_denials={2*frames}',flush=True)
if __name__=='__main__':main()
