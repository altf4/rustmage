"""Linux cgroup checks shared by the build runner and rustc wrapper."""
from pathlib import Path

MAX_MEMORY = 6 * 1024**3

def bounded_group():
    """Return our actual bounded cgroup, not merely a caller-provided env flag."""
    for line in Path('/proc/self/cgroup').read_text().splitlines():
        if line.startswith('0::'):
            relative = line[3:]
            if not any(part.startswith('rustmage-build-') and part.endswith('.service')
                       for part in Path(relative).parts):
                return None
            if 'rustmage-builds.slice' not in Path(relative).parts:
                return None
            group = Path('/sys/fs/cgroup') / relative.lstrip('/')
            parent = next(p for p in group.parents if p.name == 'rustmage-builds.slice')
            parent_max = (parent / 'memory.max').read_text().strip()
            if parent_max == 'max' or not 0 < int(parent_max) <= MAX_MEMORY:
                return None
            maximum = (group / 'memory.max').read_text().strip()
            if maximum != 'max' and 0 < int(maximum) <= MAX_MEMORY:
                return group
    return None
