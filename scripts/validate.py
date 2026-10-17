#!/usr/bin/env python3
"""Routine rules validation; optional existing native performance benchmark.

Each run owns a unique Cargo target. Terminal cleanup retains all shared libraries,
logs and the JSON report. Never run against a source tree being edited.
"""
import argparse,hashlib,json,os,subprocess,sys,tempfile,time
from pathlib import Path
from resource_limits import bounded_group

ROOT=Path(__file__).resolve().parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release',action='store_true')
    parser.add_argument('--games',type=int,default=100)
    args=parser.parse_args()
    if args.games<1:parser.error('--games must be positive')
    os.chdir(ROOT)
    if bounded_group() is None:
        os.execv(sys.executable, [sys.executable, str(ROOT/'scripts/resource_guard.py'), '--', sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    cache=Path(os.environ.get('XDG_CACHE_HOME',Path.home()/'.cache'))
    cache.mkdir(parents=True,exist_ok=True)
    target=Path(tempfile.mkdtemp(prefix='rustmage-validation-',dir=cache))
    report_dir=target/'report';report_dir.mkdir()
    env=dict(os.environ,CARGO_TARGET_DIR=str(target),PYTHONDONTWRITEBYTECODE='1')
    files={p.decode():sha(ROOT/p.decode()) for p in subprocess.check_output(['git','ls-files','-z']).split(b'\0')if p}
    head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
    commands=[['cargo','test','--all-targets'],['cargo','clippy','--all-targets','--','-D','warnings'],['cargo','fmt','--all','--check'],['rustfmt','--edition','2024','--check',*[str(p)for p in sorted((ROOT/'src/card_definitions').glob('*.rs'))]],['python3','scripts/generate_catalogue.py','--check'],['python3','-m','unittest','scripts.test_direct_cards']]
    if args.release:commands.extend([['cargo','build','--release','--lib','--bin','benchmark'],[str(target/'release/benchmark'),str(args.games)]])
    report=dict(head=head,terminal=False,target=str(target),phases=[]);start=time.monotonic();ok=False
    try:
        for index,cmd in enumerate(commands):
            begin=time.monotonic();log=report_dir/f'{index}.log'
            with log.open('w')as out:result=subprocess.run(cmd,env=env,stdout=out,stderr=subprocess.STDOUT)
            report['phases'].append(dict(command=cmd,exit=result.returncode,seconds=time.monotonic()-begin,log=str(log)))
            print(f'{index+1}/{len(commands)} exit={result.returncode} {" ".join(cmd[:4])}',flush=True)
            if result.returncode:break
        else:ok=True
    finally:
        report['changed']=[p for p,h in files.items()if not(ROOT/p).is_file()or sha(ROOT/p)!=h]
        report['files_verified']=len(files);report['success']=ok and not report['changed'];report['terminal']=True;report['seconds']=time.monotonic()-start
        report['shared_libraries']={str(p):sha(p)for p in target.rglob('*')if p.is_file()and not p.is_symlink()and '.so'in p.name}
        freed=0
        for p in target.rglob('*'):
            if p.is_file()and not p.is_symlink()and report_dir not in p.parents and str(p)not in report['shared_libraries']:
                freed+=p.stat().st_blocks*512;p.unlink()
        for path,digest in report['shared_libraries'].items():assert sha(Path(path))==digest
        report['cleaned_allocated_bytes']=freed
        report['coverage']=json.loads((ROOT/'data/standard/coverage.json').read_text())['implemented_standard_cards']
        (report_dir/'result.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'success':report['success'],'report':str(report_dir/'result.json'),'seconds':report['seconds'],'complete_cards':report['coverage'],'cleaned_bytes':freed}),flush=True)
    return 0 if report['success']else 1
if __name__=='__main__':sys.exit(main())
