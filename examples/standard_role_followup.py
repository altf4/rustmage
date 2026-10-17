"""Paid complete Role bodies in naturally terminal Standard games, alternating seats."""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from test_role_followup import NAMES, RoleFollowupTests


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--games", type=int, default=100)
    parser.add_argument("--seed", type=int, default=294000)
    parser.add_argument("--library", required=True)
    args = parser.parse_args()
    if args.games < 1:
        parser.error("--games must be positive")
    case = RoleFollowupTests()
    frames = 0
    for n in range(args.games):
        index = n % len(NAMES)
        seat = "P0" if n // len(NAMES) % 2 == 0 else "P1"
        count = case.scenario(index, seat, args.seed + n, library=args.library)
        frames += count
        print(json.dumps(dict(game=n + 1, card=NAMES[index], seat=seat,
                              frames=count, owned_frames=frames, denials=2 * frames)), flush=True)
    print(json.dumps(dict(games=args.games, owned_frames=frames, denials=2 * frames)), flush=True)


if __name__ == "__main__":
    main()
