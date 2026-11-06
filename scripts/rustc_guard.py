#!/usr/bin/env python3
"""Keep accidental raw Cargo builds from bypassing the shared memory limit."""
import sys
if __package__:
    from .resource_limits import managed_context
    from .build_platform import exec_process
else:
    from resource_limits import managed_context
    from build_platform import exec_process

if managed_context() is None:
    sys.exit('Rust compilation requires the bounded runner: python3 scripts/resource_guard.py -- cargo <command>')
if len(sys.argv) < 2:
    sys.exit('missing rustc command')
exec_process(sys.argv[1:])
