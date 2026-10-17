#!/usr/bin/env python3
"""Keep accidental raw Cargo builds from bypassing the shared memory limit."""
import os
import sys
if __package__:
    from .resource_limits import bounded_group
else:
    from resource_limits import bounded_group

if bounded_group() is None:
    sys.exit('Rust compilation requires the bounded runner: python3 scripts/resource_guard.py -- cargo <command>')
if len(sys.argv) < 2:
    sys.exit('missing rustc command')
os.execvpe(sys.argv[1], sys.argv[1:], os.environ)
