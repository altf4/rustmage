#!/usr/bin/env python3
"""Rules validation, with explicit regression and comprehensive acceptance tiers.

Each run owns a unique Cargo target. Terminal cleanup retains all shared libraries,
logs and the JSON report. Never run against a source tree being edited.
"""
import argparse,hashlib,json,os,re,subprocess,sys,tempfile,time
from pathlib import Path
if __package__:
    from .resource_limits import bounded_group
else:
    from resource_limits import bounded_group

ROOT=Path(__file__).resolve().parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
COMPREHENSIVE_TEST='tmt_entire_pool_and_theme_acceptance'
def comprehensive_test_passed(output):
    # Cargo succeeds for a filter matching zero tests. A comprehensive report
    # must prove that the selected test executed, rather than trusting exit0.
    return (f'test {COMPREHENSIVE_TEST} ...' in output
            and re.search(r'^test result: ok\. 1 passed; 0 failed; 0 ignored;',output,re.MULTILINE) is not None)
def native_test_commands(release, acceptance):
    profile=['--profile','release-test'] if release else []
    # Ordinary regressions still execute every native test-bearing target. The
    # ignored TMT matrix is explicitly selected only in the comprehensive tier;
    # the ignored production throughput measurement must not run in this profile.
    commands=[['cargo','test',*profile,
               *(['--lib','--bins','--tests','--benches'] if release else ['--all-targets'])]]
    if acceptance == 'comprehensive':
        commands.append(['cargo','test',*profile,'--test','tmt_acceptance',
                         COMPREHENSIVE_TEST,'--','--exact','--ignored','--nocapture'])
    return commands
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release',action='store_true',
                        help='Run optimized assertion-enabled tests and build/run the production release benchmark')
    parser.add_argument('--python', choices=['none', 'smoke', 'full'], default='none',
                        help='Run binding tests against the generated release library')
    parser.add_argument('--acceptance', choices=['regression', 'comprehensive'], default='comprehensive',
                        help='Comprehensive (default) also runs all 172 TMT pool/theme games; regression retains all rules and focused game suites')
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
    # Reject formatting/provenance errors before paying for native code generation.
    commands=[
        ['cargo','fmt','--all','--check'],
        ['rustfmt','--edition','2024','--check',*[str(p)for p in sorted((ROOT/'src/card_definitions').rglob('*.rs'))]],
        ['python3','scripts/link_catalogue.py','--check'],
        ['python3','-m','unittest','discover','-s','scripts','-p','test_*.py'],
        ['cargo','clippy','--all-targets','--','-D','warnings'],
        # Examples contain no tests; Clippy checks them without optimizing
        # empty test executables. Keep every native test-bearing target kind.
        *native_test_commands(args.release,args.acceptance),
    ]
    if args.release or args.python != 'none':commands.extend([['cargo','build','--release','--lib','--bin','benchmark'],[str(target/'release/benchmark'),str(args.games)]])
    if args.python != 'none':
        env['RUSTMAGE_LIBRARY'] = str(target/'release/librustmage.so')
        commands.append(['python3', 'scripts/python_tests.py', '--suite', args.python,
                         '--report', str(report_dir/'python.json')])
    report=dict(head=head,terminal=False,target=str(target),acceptance_suite=args.acceptance,
                acceptance_scope='TMT 172-game comprehensive matrix included' if args.acceptance=='comprehensive' else 'Rules and focused native game regressions; TMT 172-game comprehensive matrix excluded',
                phases=[]);start=time.monotonic();ok=False
    try:
        for index,cmd in enumerate(commands):
            begin=time.monotonic();log=report_dir/f'{index}.log'
            with log.open('w')as out:result=subprocess.run(cmd,env=env,stdout=out,stderr=subprocess.STDOUT)
            code=result.returncode
            checked=COMPREHENSIVE_TEST in cmd
            executed=comprehensive_test_passed(log.read_text()) if checked else None
            if code==0 and checked and not executed:
                code=1
                with log.open('a')as out:out.write('\nValidation failed: the required comprehensive test did not report exactly one executed passing test.\n')
            phase=dict(command=cmd,exit=code,seconds=time.monotonic()-begin,log=str(log))
            if checked:phase.update(process_exit=result.returncode,required_test_executed=executed)
            report['phases'].append(phase)
            print(f'{index+1}/{len(commands)} exit={code} {" ".join(cmd[:4])}',flush=True)
            if code:break
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
