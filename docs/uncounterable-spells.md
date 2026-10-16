# Self-spell counter protection

Issue: https://github.com/altf4/rustmage/issues/146
Pinned Standard: October 15, 2026. Four complete bodies bring executable coverage
to **1,185/5,164**; **3,979 remain unsupported**. Six regression identities,
53 token factories and one emblem remain separate.

## Complete bodies and provenance

`uncounterable-admissions.json` records pinned Oracle text and 23 SHA-256 entries
across eleven unique Java files at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. All four card classes, stack-only
prohibition ability/effect, explicit counter and Ward implementations, investigate,
and the Clue token/ability were read.

| Card | Whole body |
| --- | --- |
| Eject | Uncounterable instant; return target nonland permanent to its owner's hand, then draw one |
| Gigantic Big Bear | Uncounterable 10/7 Bear creature spell; hexproof and haste |
| Out Cold | Uncounterable instant; tap zero to two distinct target creatures, put a stun counter on each, investigate |
| Long Goodbye | Uncounterable instant; destroy target creature or planeswalker with mana value at most three |

The compiler removes only one standalone exact `This spell can't be countered.`
line, optionally followed by reminder text, and compiles the entire residual body.
It emits immutable metadata only for a wholly accepted body. Conditional/global/
next-spell protection, duplicate protection clauses, unknown residual text,
unsupported layouts, X costs and other remaining dependencies stay gated.
Inevitable Defeat, Slice from the Shadows, Koma, World-Eater, Frenzied Baloth and
Hexing Squelcher remain unsupported. No incidental bodies were admitted.

## Rules and implementation

`Card::cannot_be_countered()` reads generated immutable executable metadata.
The shared explicit-counter operation looks up the exact stack ID and checks the
prohibition only for `StackKind::Spell`. An activated or triggered ability is
independent of its source card's own spell prohibition. Missing old stack IDs do
not alias later incarnations. Ordinary targeted counters additionally retain the
bound object generation. Ordinary counters, declined counter-unless-mana payments
(including exile replacement), and declined discard/sacrifice/life Ward all use
this operation. No counter removes a protected spell or refunds its cast costs.

Protection does not change targeting legality. Counterspells can still target the
spell, Ward still triggers, and the payer can pay or decline its cost. Unrelated
instructions still execute: Refute draws and discards even when its counter fails;
Don't Make a Sound's paid surveil follows the payment branch. No More Lies exiles
only when its counter succeeds. A countering ability can itself resolve normally
while leaving its protected victim on the stack.

Existing resolution legality remains authoritative. If every chosen target is
illegal, Eject does not draw and Out Cold does not investigate. With no targets,
Out Cold investigates; with some legal targets, it affects those and investigates.
The Clue retains its full {2}, sacrifice, draw ability. Long Goodbye's new exact
union filter covers creatures and planeswalkers with mana value zero through
three. Destroy still respects indestructible. Bear's hexproof restricts opposing
battlefield targets after resolution; it does not prevent a legal counterspell
target while the Bear is on the stack.

No mutable state, public observation shape or continuation fields were added.
Schema 49, C ABI 1, control protocol 2 and **19,784 bytes** of initial logical state
stay unchanged. Snapshots retain exact stack associations, targets, owned payment
choices, private discard/surveil and RNG. Restore invalidates stale indexed actions.
Catalogue fingerprint:
`5fa8ec107ec2a9b01c64712624110d572dcdd21eadca4528df897d23440174b5`.

Rules references: CR 113.6g (stack-only prohibition), 113.9 (abilities independent
of spells), 400.7 (new incarnation), 608.2b (illegal targets), 701.6 (counter),
702.21 (Ward), in the [official rules](https://magic.wizards.com/en/rules).
[Official MKM release notes](https://magic.wizards.com/en/news/feature/murders-at-karlov-manor-release-notes)
explicitly cover Long Goodbye/Out Cold's optional Ward payments and Out Cold's
zero-target and all-targets-illegal outcomes.

## Validation

- 903 Rust tests: 891 unit and twelve integration tests. Nine new native methods
  cover both seats, direct/restricted/mana/exile counters, paid and declined Ward,
  private follow-up choices, complete bodies, token ability, zero/partial/total
  target invalidity, spell-versus-ability scope, mana-value boundaries and replay.
- 172 compiler/data tests, including exact extraction, residual rejection,
  metadata omission for unsupported bodies and unchanged unsupported dependencies.
- 124 fresh release Python tests, including eight new real-library scenarios:
  all four bodies surviving Counterspell, and Eject surviving four nonmana Ward
  bodies, with private observation, clone/restore and stale-action checks.
- Catalogue consistency, formatting, Clippy with warnings denied and diff checks.

`cargo run --release --example standard_uncounterable -- 100` uses complete legal
Standard decks. The deterministic smoke policy preserves Snatcher's Ward body,
delays attacks until turn twelve, prefers legal protected counter targets, and
alternates payment decline to exercise counter branches. It is a coverage policy,
not a trained player or a Standard metagame benchmark.

The workload completes 100 games with 78,003 decisions, resolutions
`[244, 93, 293, 171]` in manifest order, failed direct/mana/Ward counters
`[120, 107, 328]`, 555 exact stack protection checks, 228 declined payments,
14,103 clone/restore replay boundaries and 21,952 maximum initialized logical
state bytes. All four complete bodies and all three explicit counter paths are
required to occur. Logical byte counts exclude allocator/capacity overhead,
immutable definitions, observations and externally retained snapshots.

The existing 1,000-game native Freeform benchmark completes 571,049 decisions at
397,058 decisions/s and 695 games/s, with 135.0 ns clone and 127.8 ns restore at
the initial 19,784-byte state. This is one host sample including smoke policy and
observation construction; it excludes neural-network/tensor work and JSON, and
is not an XMage parity or speedup claim.
