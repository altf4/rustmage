"""Ordinary Standard full-body/owned-rollback workload for eight Adventure pairs."""
import argparse
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_adventure_land_targets import AdventureLandTargetTests

def main():
    p=argparse.ArgumentParser();p.add_argument('--library',required=True);p.add_argument('--games',type=int,default=100);p.add_argument('--start-seed',type=int,default=2770);a=p.parse_args()
    if a.games<1:p.error('games must be positive')
    checks=AdventureLandTargetTests();frames=0
    for i in range(a.games):
        n=checks.scenario((i//2)%8,'P0' if i%2==0 else 'P1',a.start_seed+i//16,library=a.library);frames+=n
        print(f'game={i} owned_replays={n} full_body=true terminal=true',flush=True)
    print(f'games={a.games} owned_replays={frames} wrong_actor_and_stale_denials={2*frames}',flush=True)
if __name__=='__main__':main()
