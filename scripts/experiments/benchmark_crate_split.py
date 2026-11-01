#!/usr/bin/env python3
"""Compare engine, card, and test edits for the fixed matched benchmark cohort.

Run through the generated prototype's scripts/resource_guard.py. Both source
copies must be disposable: this script makes controlled, equivalent edits after
all jobs consuming the previous inputs terminate. Raw evidence is retained under
--report, which should be below ~/.cache. No Cargo output is cleaned.
"""
import os,sys,json,pathlib,subprocess,time,statistics,importlib.util,hashlib,re,datetime
import argparse
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--root', required=True, type=pathlib.Path)
parser.add_argument('--previous', required=True, type=pathlib.Path)
parser.add_argument('--report', required=True, type=pathlib.Path)
parser.add_argument('--target-dir', required=True, type=pathlib.Path)
args = parser.parse_args()
ROOT=args.root.resolve()
PREVIOUS=args.previous.resolve()
REPORT=args.report.resolve()
TARGET=args.target_dir.resolve()
for name in ('monolith', 'split'):
    if (ROOT/name/'.git').exists():
        parser.error('only disposable copies may be benchmarked')
REPORT.mkdir(parents=True, exist_ok=False)
sys.path.insert(0,str(ROOT/'monolith/scripts'))
import run_native_tests as native
assert native.bounded_group() is not None,'must run through resource_guard.py'
expected=json.loads((PREVIOUS/'comparison.json').read_text())['versions']['new']['preflight']['cards_by_scenario']
physical={r['catalogue_id'] for r in json.loads((PREVIOUS/'comparison.json').read_text())['cards']}
assert len(expected)==91 and len(physical)==186
cohort=json.loads((PREVIOUS/'cohort.json').read_text())
env=dict(os.environ,CARGO_TARGET_DIR=str(TARGET),CARGO_PROFILE_TEST_OPT_LEVEL='0',CARGO_PROFILE_TEST_DEBUG='false',CARGO_PROFILE_TEST_INCREMENTAL='false',CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',PYTHONDONTWRITEBYTECODE='1')
result=dict(success=False,scenario_count=91,physical_card_count=186,target=str(TARGET),clean_build=False,previous=str(PREVIOUS/'comparison.json'),runs=[],profiles=dict(opt_level=0,debug=False,incremental=False,jobs=1,test_threads=1),rustc=subprocess.check_output(['rustc','--version'],text=True).strip(),started=datetime.datetime.now(datetime.timezone.utc).isoformat())
(REPORT/'cohort.json').write_text(json.dumps(cohort,indent=2)+'\n');(REPORT/'cards.json').write_bytes((PREVIOUS/'cards.json').read_bytes())
def save():
 p=REPORT/'comparison.json.tmp';p.write_text(json.dumps(result,indent=2)+'\n');p.replace(REPORT/'comparison.json')
def run(label,workload,repeat,trace=False):
 src=ROOT/label;native.ROOT=src;out=REPORT/label/f'{workload}-{repeat}';out.mkdir(parents=True,exist_ok=False)
 e=dict(env,RUSTMAGE_CARD_COVERAGE_REPORT=str(out/'features.jsonl'))
 (out/'features.jsonl').write_text('')
 if trace:e['RUSTMAGE_MATCHED_CARD_TRACE']=str(out/'cards.jsonl')
 else:e.pop('RUSTMAGE_MATCHED_CARD_TRACE',None)
 package='rustmage' if label=='monolith' else 'rustmage-card-tests'
 command=['cargo','test','-p',package,'--lib','--no-run','--message-format=json']
 compiled=native.run_command(command,env=e,log=out/'compile.log',timeout=360)
 if compiled['exit']:
  diagnostics=[]
  for line in (out/'compile.log').read_text().splitlines():
   try:m=json.loads(line)
   except ValueError:continue
   if m.get('reason')=='compiler-message' and m['message']['level']=='error':diagnostics.append(m['message'].get('rendered',m['message']['message']))
  print('\n'.join(diagnostics[-10:]),flush=True);raise RuntimeError(str(out/'compile.log'))
 artifacts=[];binary=[]
 for line in (out/'compile.log').read_text().splitlines():
  try:m=json.loads(line)
  except ValueError:continue
  if m.get('reason')=='compiler-artifact':
   artifacts.append(dict(target=m['target']['name'],package=m['package_id'],fresh=m['fresh'],profile=m['profile']))
   if m.get('executable') and m['profile'].get('test'):binary.append(m['executable'])
 assert len(binary)==1,binary
 listing=native.run_command([binary[0],'','--list','--format=terse'],env=e,log=out/'list.log',timeout=5)
 assert listing['exit']==0
 names=[l.removesuffix(': test') for l in (out/'list.log').read_text().splitlines() if l.endswith(': test')]
 assert len(names)==91 and {n.rsplit('::',1)[-1] for n in names}==set(expected)
 execution=native.run_command([binary[0],'','--include-ignored','--test-threads=1'],env=e,log=out/'execution.log',timeout=60)
 tests,okay=native.suite_results(names,(out/'execution.log').read_text())
 if not okay:print((out/'execution.log').read_text()[-10000:],flush=True)
 assert execution['exit']==0 and okay,(label,workload,repeat)
 item=dict(label=label,workload=workload,repeat=repeat,compile_seconds=compiled['seconds'],execution_seconds=execution['seconds'],total_seconds=compiled['seconds']+execution['seconds'],artifacts=artifacts,tests=tests,compile=compiled,execution=execution)
 if trace:
  rows=[json.loads(l) for l in (out/'cards.jsonl').read_text().splitlines()]
  cards={r['case'].rsplit('::',1)[-1]:sorted(set(r['cards'])&physical) for r in rows}
  differences={k:dict(expected=expected[k],actual=cards.get(k)) for k in expected if expected[k]!=cards.get(k)}
  assert len(rows)==91 and not differences,(len(rows),differences)
  item['cards_by_scenario']=cards
  print('VERIFIED',label,'91 scenarios, exact same 186 physical cards and per-scenario identities',flush=True)
 if label=='split' and workload in ('card-edit','test-edit'):
  fresh={a['target']:a['fresh'] for a in artifacts}
  assert fresh['rustmage_engine'] and fresh['rustmage_cards_other'],fresh
  if workload=='test-edit':assert fresh['rustmage_cards_msh'] and fresh['rustmage'],fresh
  else:assert not fresh['rustmage_cards_msh'] and not fresh['rustmage'],fresh
 if label=='monolith' and workload!='preflight':
  assert any(a['target']=='rustmage' and not a['fresh'] for a in artifacts)
 for a in artifacts:
  if a['target'].startswith('rustmage'):
   assert a['profile']['opt_level']=='0' and a['profile']['debuginfo']==0,a
 # Dependencies outside this project must stay warm in every sample.
 assert all(a['fresh'] for a in artifacts if not a['target'].startswith('rustmage')),artifacts
 result['runs'].append(item);save()
 print(label,workload,repeat,'compile',round(item['compile_seconds'],3),'test',round(item['execution_seconds'],3),'total',round(item['total_seconds'],3),flush=True)
 return item
originals={}
def mutate(label,workload,repeat):
 src=ROOT/label
 if workload=='engine-edit':
  p=src/('src/engine.rs' if label=='monolith' else 'crates/engine/src/engine.rs')
  addition=f'\n#[allow(dead_code)]\npub fn benchmark_engine_edit() -> usize {{ {repeat} }}\n'
 elif workload=='card-edit':
  p=src/('src/card_definitions/msh/arc_reactor.rs' if label=='monolith' else 'crates/cards-msh/src/cards/msh/arc_reactor.rs')
  addition=None
 else:
  file=next(r['new_file'] for r in cohort if r['name']=='msh_arc_reactor_improvise_pays_generic_and_produces_three_after_untap')
  p=src/(file if label=='monolith' else file.replace('src/testing/','crates/card-tests/src/testing/'))
  addition=None
 if str(p) not in originals:originals[str(p)]=p.read_text()
 s=originals[str(p)]
 if workload=='engine-edit':s+=addition
 elif workload=='card-edit':
  assert 'effect: Effect::Permanent,' in s
  s=s.replace('effect: Effect::Permanent,',f'effect: {{ const EDIT: usize = {repeat}; match EDIT {{ _ => Effect::Permanent }} }},',1)
 else:
  name='msh_arc_reactor_improvise_pays_generic_and_produces_three_after_untap'
  pattern=r'(fn '+name+r'\(\)\s*\{)'
  s,n=re.subn(pattern,lambda m:m[0]+f'\nlet _benchmark_test_edit = {repeat};',s,count=1);assert n==1
 p.write_text(s)
try:
 # Initial verification can reuse setup artifacts; not a timing sample.
 for label in ('monolith','split'):run(label,'preflight',0,True)
 for workload in ('engine-edit','card-edit','test-edit'):
  for repeat in range(1,4):
   order=('monolith','split') if repeat%2 else ('split','monolith')
   for label in order:
    mutate(label,workload,repeat)
    run(label,workload,repeat)
 result['summary']={}
 for workload in ('engine-edit','card-edit','test-edit'):
  result['summary'][workload]={}
  for label in ('monolith','split'):
   rows=[r for r in result['runs'] if r['label']==label and r['workload']==workload]
   result['summary'][workload][label]={key:statistics.median(r[key] for r in rows) for key in ('compile_seconds','execution_seconds','total_seconds')}
  old=result['summary'][workload]['monolith']['total_seconds'];new=result['summary'][workload]['split']['total_seconds']
  result['summary'][workload]['speedup']=old/new
  result['summary'][workload]['time_reduction_percent']=100*(1-new/old)
 result['success']=True;save()
 print('COMPLETE',json.dumps(result['summary']),flush=True)
except BaseException as exc:
 result['error']=str(exc);save();raise
