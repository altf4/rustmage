"""Fixed complete two-body self-play workload; every step checks owned replay."""
import argparse
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_joint_land import JointLandTests,ROWS
CASES=[(index%len(ROWS),'P0'if index%2==0 else'P1',33900+index)for index in range(100)]
def run(library):
    checks=JointLandTests();results=[]
    for index,(row,seat,seed)in enumerate(CASES):
        result=checks.scenario(row,seat,seed,library=library)
        results.append(result)
        print(json.dumps(dict(completed=index+1,total=len(CASES),result=result)),flush=True)
    vectors={(r['name'],r['x'],r['selection'],r['seat'])for r in results}
    assert len(vectors)==2*len(ROWS)
    frames=sum(r['frames']for r in results)
    return dict(games=len(results),vectors=len(vectors),owned_frames=frames,wrong_actor_and_stale_denials=2*frames)
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--library',required=True)
    args=parser.parse_args();print(json.dumps(run(args.library)),flush=True)
