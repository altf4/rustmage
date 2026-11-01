#!/usr/bin/env python3
"""Measure full-suite compilation after reversible card/test edits, without cleaning.

Runs in the bounded build slot. Every edit is backed up and restored, including
metadata outputs. Keep the checkout otherwise frozen until this command exits.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import sys
import time

if __package__:
    from . import build_cache, link_catalogue
    from .resource_limits import bounded_group
    from .run_native_tests import run_command
else:
    import build_cache, link_catalogue
    from resource_limits import bounded_group
    from run_native_tests import run_command

ROOT = Path(__file__).resolve().parents[1]
CASE = 'msh_arc_reactor_improvise_pays_generic_and_produces_three_after_untap'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path, required=True)
    parser.add_argument('--report-dir', type=Path, required=True)
    parser.add_argument('--repeats', type=int, default=3)
    args = parser.parse_args()
    if args.repeats < 1:
        parser.error('repeats must be positive')
    if bounded_group() is None:
        os.execv(sys.executable, [sys.executable, str(ROOT/'scripts/resource_guard.py'), '--',
                                 sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]])
    report = args.report_dir.resolve()
    report.mkdir(parents=True, exist_ok=False)
    target = args.target_dir.resolve()
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), RUST_TEST_THREADS='1')
    card = ROOT/'src/card_definitions/msh/arc_reactor.rs'
    case = ROOT/'tests/legacy/assurance/msh_a.rs'
    metadata, records, version = link_catalogue.load()
    generated = link_catalogue.render(metadata, records, version)
    backups = {path: path.read_bytes() for path in [card, case, *generated]}
    result = dict(success=False, clean_build=False, samples=[], target=str(target),
                  rustc=subprocess.check_output(['rustc', '--version'], text=True).strip(),
                  profiles=dict(test=0, engine=1, external_dependencies=1,
                                incremental=False, debug=False, jobs=1, test_threads=1))

    def save():
        (report/'result.json').write_text(json.dumps(result, indent=2)+'\n')

    def regenerate():
        with (report/'linker.log').open('a') as log:
            subprocess.run([sys.executable, 'scripts/link_catalogue.py'], cwd=ROOT,
                           stdout=log, stderr=subprocess.STDOUT, check=True)

    def compile(label, expected=None):
        log = report/(label+'.log')
        compiled = run_command(['cargo', 'test', '-p', 'rustmage', '--test', 'legacy',
                                '--no-run', '--message-format=json'],
                               env=env, log=log, timeout=900)
        if compiled['exit']:
            raise RuntimeError('compilation failed: '+str(log))
        artifacts, binaries = {}, []
        for line in log.read_text().splitlines():
            try:
                item = json.loads(line)
            except ValueError:
                continue
            if item.get('reason') == 'compiler-artifact':
                artifacts[item['target']['name']] = dict(fresh=item['fresh'],
                                                        profile=item['profile'],
                                                        features=item['features'])
                if 'test' in item['target']['kind'] and item.get('executable'):
                    binaries.append(item['executable'])
        if len(binaries) != 1:
            raise RuntimeError('expected the one inherited integration harness')
        if expected:
            assert artifacts['rustmage_core']['fresh'], 'engine rebuilt for '+label
            if expected == 'card':
                assert not artifacts['rustmage_cards_msh']['fresh'], 'changed card was not rebuilt'
                assert all(v['fresh'] for k,v in artifacts.items()
                           if k.startswith('rustmage_cards_') and k!='rustmage_cards_msh')
            if expected in ('test', 'fresh'):
                assert all(v['fresh'] for k,v in artifacts.items() if k.startswith('rustmage_cards_'))
            if expected == 'fresh':
                assert all(v['fresh'] for k,v in artifacts.items() if k in ('rustmage','legacy'))
        execution = run_command([binaries[0], CASE, '--test-threads=1'], env=env,
                                log=report/(label+'-case.log'), timeout=45)
        assert execution['exit'] == 0 and '1 passed; 0 failed; 0 ignored;' in Path(execution['log']).read_text()
        listing = subprocess.check_output([binaries[0], '--list', '--format=terse'], env=env, text=True)
        count = sum(line.endswith(': test') for line in listing.splitlines())
        item = dict(label=label, compile_seconds=compiled['seconds'],
                    focused_execution_seconds=execution['seconds'], compiled_cases=count,
                    artifacts=artifacts, compile=compiled, execution=execution)
        result['samples'].append(item)
        save()
        print(f'{label}: {compiled["seconds"]:.3f}s compile; engine fresh={artifacts["rustmage_core"]["fresh"]}; {count} compiled cases', flush=True)
        return item

    try:
        compile('warm-up')
        for repeat in range(1, args.repeats+1):
            text = backups[card].decode()
            assert 'effect: Effect::Permanent,' in text
            text = text.replace('effect: Effect::Permanent,',
                                f'effect: {{ const _: usize = {repeat}; Effect::Permanent }},', 1)
            card.write_text(text)
            regenerate()
            compile('card-'+str(repeat), 'card')
        card.write_bytes(backups[card])
        regenerate()
        compile('restore-card', 'card')
        for repeat in range(1, args.repeats+1):
            text, count = re.subn(r'(fn '+CASE+r'\(\)\s*\{)',
                                 lambda m: m[0]+f'\nlet _build_probe = {repeat};',
                                 backups[case].decode(), count=1)
            assert count == 1
            case.write_text(text)
            compile('test-'+str(repeat), 'test')
        case.write_bytes(backups[case])
        compile('restore-test', 'test')
        # Simulate a fresh checkout of an unchanged engine file. Content checks,
        # not timestamp guesses, authorize restoring its previous input time.
        result['cache_save'] = build_cache.save(ROOT, target)
        engine = ROOT/'crates/engine/src/engine.rs'
        os.utime(engine, None)
        result['cache_restore'] = build_cache.restore(ROOT, target)
        compile('checkout-cache', 'fresh')
        result['medians'] = {kind: statistics.median(
            sample['compile_seconds'] for sample in result['samples']
            if re.fullmatch(kind+r'-\d+', sample['label'])) for kind in ('card','test')}
        result['success'] = True
    except BaseException as error:
        result['error'] = str(error)
        raise
    finally:
        for path, original in backups.items():
            if path.read_bytes() != original:
                path.write_bytes(original)
        result['sources_restored'] = all(path.read_bytes() == original for path,original in backups.items())
        result['source_sha256'] = {str(path.relative_to(ROOT)):hashlib.sha256(backups[path]).hexdigest()
                                   for path in (card,case)}
        save()
        print('Build benchmark: '+str(report/'result.json'), flush=True)


if __name__ == '__main__':
    main()
