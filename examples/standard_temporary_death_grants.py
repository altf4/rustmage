"""Fixed, unretried naturally terminal Standard temporary-death-grant games."""
import argparse,json,sys,time
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'python'))
from test_temporary_death_grants import TemporaryDeathGrantTests
if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--games',type=int,default=100);parser.add_argument('--library');args=parser.parse_args();assert args.games>0
 start=time.monotonic();case=TemporaryDeathGrantTests();rows=[]
 for i in range(args.games):
  row=case.scenario((i//2)%3,'P0' if i%2==0 else 'P1',29900+i//2,library=args.library);rows.append(row);print(json.dumps(dict(game=i,**row)),flush=True)
 frames=sum(r['frames'] for r in rows);print(json.dumps(dict(games=len(rows),frames=frames,denials=2*frames,seconds=time.monotonic()-start)))
