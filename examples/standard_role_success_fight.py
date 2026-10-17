"""Complete paid Werefox games, fixed seeds, fight/decline and alternating seats."""
import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'python'))
from test_role_success_fight import RoleSuccessFightTests


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--games', type=int, default=100)
    parser.add_argument('--seed', type=int, default=304000)
    parser.add_argument('--library', required=True)
    args = parser.parse_args()
    if args.games < 1:
        parser.error('--games must be positive')
    case = RoleSuccessFightTests()
    frames = 0
    for n in range(args.games):
        seat = 'P0' if n % 2 == 0 else 'P1'
        decline = n // 2 % 2 == 1
        count = case.scenario(seat, args.seed + n, library=args.library, decline=decline)
        frames += count
        print(json.dumps(dict(game=n + 1, seat=seat, decline=decline,
                              frames=count, owned_frames=frames, denials=2 * frames)), flush=True)
    print(json.dumps(dict(games=args.games, owned_frames=frames, denials=2 * frames)), flush=True)


if __name__ == '__main__':
    main()
