# Public hand knowledge

Issue: https://github.com/altf4/rustmage/issues/151

Targeted and batch returns previously moved cards correctly but omitted their
new hand identities from the public knowledge ledger. A memoryless RL observer
could forget the card immediately after seeing it return. Knowledge is now
recorded in the shared zone-event path, covering battlefield, graveyard, stack
and supported face-up exile returns, including targeted graveyard recovery.

## Visibility and lifecycle

The observer remembers the exact new `ObjectRef`, with its new generation. It
sees the returned card in the owner's hand. Unrelated cards already in that hand,
ordinary library draws and private searches stay hidden. Explicitly revealed
library searches still use their existing reveal flag. The ledger deduplicates
facts, ignores failed moves and old events, and excludes tokens and emblems.
Tokens returned from the battlefield cease to exist through state-based actions.

When that hand incarnation leaves, its fact is removed by the existing shared
transition path. Recasting, discarding, moving to a library and drawing again
cannot reuse an earlier reference. Caller-specific source-return and mill-recovery
updates were removed; they now share the same public-zone rule. Search reveals
and announced land choices retain their existing explicit visibility handling.

This is retained observation history for the supported information model. It
does not turn hands into public zones or change legal action ownership. Future
face-down exile, concealed exchanges, copy identities or information-erasing
effects need explicit information semantics before their whole bodies can be
admitted. All exile in the current supported subset is face up.

The [official Comprehensive Rules](https://magic.wizards.com/en/rules), pinned
September 25, 2026 text, distinguish public and hidden zones in CR400.2, assign
owner destinations in CR400.3, create new objects on zone changes in CR400.7,
keep opponents' hands hidden in CR402.3, and make exile face up by default in
CR406.3. The knowledge ledger retains already observed identity information;
it grants no permission to inspect a private hand.

## Validation

Seven new Rust tests cover real Unsummon casts by either player, both owners,
both targeted graveyard recovery bodies, shared simultaneous events across all
four supported public origin zones, stale events, duplicate events, staged
reveals, private draws, token cleanup and capacity-failure rollback. Actual
recasts clear known references. Clone/restore repeats the same state, and stale
action revisions fail without mutation. An existing search test now correctly
expects a played land returned from the battlefield to become known again.

A new public Python test exercises Bonebind Orator and Theoretical Necromancer
through actual self-mill, staged owned payment, targeted resolution and both
observer views. The existing Exhaust bounce/recast fixture now checks public
hand knowledge, unrelated private cards, cloned resolution, restored replay and
removal of the old fact on recast. Existing search, mill-recovery and source-return
regressions protect those paths against duplication or leaks.

Complete checks passed: **938 Rust tests** (926 unit and 12 integration),
**176 compiler/data tests**, **128 fresh release Python tests**, Clippy on all
targets with warnings denied, formatting, catalogue consistency and diff checks.

The native Standard graveyard workload now checks ledger agreement between
observers, uniqueness, exact current hand references, and all public-to-hand
transitions on every decision. Its 100 games retain the previous 51,529
decisions; commits and resolutions
`[26,43,20,55,41,51,45,31,45,10,43,19,28,9,32,24]`; 1,044 exact cost/source
checks; 285 private choices; 9,496 replay boundaries; and 22,024 maximum
initialized logical state bytes. It additionally validates **84 public-to-hand
transitions**. The source-return workload retains all previous 100-game results:
57,128 decisions, 1,490 checks and 11,730 replay boundaries.

There are no new state fields or card admissions. Schema 49, C ABI 1 and control
protocol 2 remain unchanged. The initial logical state remains 19,784 bytes.
Retained hand facts use the existing snapshot-owned vector and are included in
logical state accounting. Coverage remains **1,212/5,164**, with **3,952
unsupported**; six regression identities, 55 token factories and one emblem
are separate. Catalogue:
`95664285e02e408361e0e9e37153edf94d5ba6122039ac31e54b3f7243556b50`.
