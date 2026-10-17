#!/usr/bin/env python3
"""Fixed, unretried complete Gift Standard workload with owned replay."""
import argparse
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'python'))
from test_gift import GiftTests, NAMES

parser = argparse.ArgumentParser()
parser.add_argument('--library', required=True)
parser.add_argument('--games', type=int, default=100)
args = parser.parse_args()
case = GiftTests()
frames = denials = 0
seen = set()
start = time.monotonic()
for game in range(args.games):
    name = NAMES[(game // 4) % len(NAMES)]
    seat = ['P0', 'P1'][game % 2]
    promise = game % 4 < 2
    seed = 31000 + (game // 4) % len(NAMES) + (0 if promise else 40) + 100 * (game // (4 * len(NAMES)))
    result = case.scenario(name, seat, seed, promise=promise, library=args.library)
    frames += result['frames']
    denials += result['denials']
    seen.add((name, seat, promise))
    print(json.dumps(dict(game=game, seed=seed, **result)), flush=True)
if args.games >= 4 * len(NAMES):
    assert len(seen) == 4 * len(NAMES)
print(json.dumps(dict(games=args.games, complete_vectors=len(seen), owned_frames=frames,
                      illegal_wrong_actor_and_stale_denials=denials,
                      seconds=time.monotonic() - start)), flush=True)
