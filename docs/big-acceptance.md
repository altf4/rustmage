# The Big Score acceptance

[Issue 469](https://github.com/altf4/rustmage/issues/469) tracks all printed
obligations; [PR 470](https://github.com/altf4/rustmage/pull/470) integrates the
complete batch. `data/sets/big.json` freezes all 30 Oracle identities. Five were
already admitted; this batch implements the remaining 25 and verifies the
inherited five. Cosmetic variants do not expand the denominator.

The pinned XMage revision is
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Each directly authored typed Rust
definition has paired provenance; generated catalogue links remain metadata.
Authenticate membership and source pins with:

```sh
python3 -m scripts.set_status BIG --check
python3 scripts/check_sources.py
```

The focused BIG modules cover paid costs, optional and modal choices, replacement
and delayed effects, equipment, private library/exile permissions, distinct-name
costs, source/controller/incarnation changes and acquired activated abilities.
Territory Forge tests include intrinsic land mana, inherited additional costs,
actual-source counter LKI, tagged mana, exhaust/once-per-turn accounting, more
than 64 acquired slots, and abilities whose choices or triggers are not inherited.
Illegal actions preserve state; scenario helpers exercise owned raw/index
branching, parent disposal, restore and stale revisions. Private observations
and delayed/copy packets have independent interaction assertions.

`src/engine_big_game_tests.rs` runs 12 complete games in four bounded tests using
four Standard-legal 60-card decks. Every nonbasic card is from BIG; all 30 members
occur in the deck pool. The seeds are 56710–56712 for each pairing; every game
must reach a natural outcome within 2048 decisions. The games exercise all five
spell colors across the pool, record actually cast/activated cards, replay every
action, and sample owned indexed branches and observation equality after parent
disposal and restore. Card inclusion or casting coverage alone does not prove
all obligations; focused rules cases provide that evidence.

The upstream assurance migration removed older development tests. The BIG tests
use a small adapter around its existing scenario helpers rather than restoring
that suite. The new cast/entry baseline adds 25 completing BIG rows. All 2,196
existing rows keep identical action journals, decision counts, fixture roots,
completion status and final zones. Their observation digests change because the
wire schema is 132 and exposes acquired abilities and delayed artifact copies.
These fingerprints supplement the independent rules cases.

Run routine acceptance without changing inputs until the process terminates:

```sh
python3 scripts/link_catalogue.py
python3 scripts/validate.py --release --python smoke
```

Validation retains logs, JSON reports and required shared libraries at their
exact paths and hashes under `~/.cache`; completed disposable build outputs are
removed. The resource guard retains its shared 6 GiB cap and single build/job/test
thread. `docs/big-acceptance.json` records measured evidence and workload scope.
The release Freeform simulation sample is a smoke workload and does not establish
all-Standard rules correctness or general search/model performance.

No BIG printed obligations remain open after these acceptance gates and two
independent nonauthor reviews of the published batch. The full Standard pool
still has gated cards and unverified interactions; its goal remains open.
