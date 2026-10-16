# Opponent entry-tapped replacements

Issue: https://github.com/altf4/rustmage/issues/188

Coverage is **1,358/5,164** pinned Standard cards, with **3,806 unsupported**.
Six non-Standard regression identities, 57 token factories and one emblem remain
separate. The full Standard migration remains incomplete, pinned to October 15, 2026.

This stage implements the complete bodies of **Authority of the Consuls** and
**Spider-Woman, Stunning Savior**. Authority makes opposing creatures enter tapped
and separately gains one life for each opposing creature entry. Spider-Woman is a
legendary 2/2 with flying, costs `{1}{W/U}`, and makes opposing artifacts or creatures
enter tapped. Its Venom Blast label adds no rules behavior. Artifact creatures
qualify once, and neither card changes the tapped status of permanents already on
the battlefield.

The library uses an immutable card mask and its existing battlefield source index.
Entry replacement checks sources present before the entry event, including token
entry routes. The replacement applies while the permanent enters; it is not a
trigger or an action window. A source entering in the same batch does not supply
this replacement. Authority's separate triggered ability observes the resulting
entry event and uses the existing exact entering-object context. Torpor Orb can
suppress that creature-entry trigger while the tapped replacement still applies.
There is no new mutable game-state field or observation schema change. Schema
remains **52** (`rustmage::OBSERVATION_SCHEMA`), C ABI **1**, control protocol **2**.
Initial logical state remains **19,848 bytes**, excluding allocator/capacity,
immutable definitions, observations and separately retained snapshots.

Replacement timing follows CR 614.12 in the [pinned official rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf).
Post-event triggered observers use CR 603.6; a simultaneously entering Authority
can observe the creature even though its replacement was not yet present.

The reference implementations are the pinned XMage constructors
`Mage.Sets/src/mage/cards/a/AuthorityOfTheConsuls.java` and
`Mage.Sets/src/mage/cards/s/SpiderWomanStunningSavior.java`. Their complete bodies,
including flying, legendary status, both hybrid payment alternatives, and
Authority's life trigger, are the acceptance target.
`opponent-entry-tapped-admissions.json` records both complete Oracle bodies and
**20 SHA-256 entries across 14 unique Java files** at revision
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Recorded blobs match the pinned
revision and checkout. Exactly these two immutable definitions change; previously
executable definitions and all card/token identities remain unchanged. Unsupported additional rules
remain gated by the compiler instead of admitting partial card bodies.

`examples/standard_opponent_entry_tapped.rs` runs native self-play with legal
60-card Standard decks and four copies per nonbasic card. Public before/after
observations independently determine tapped entry using pre-event sources, while
post-event observers supply an exact `(source incarnation, entering incarnation,
ability)` trigger ledger. Separate resolution witnesses check Authority life gain,
other life and draw effects, and token creation. It also checks creature statistics
and keywords, old creatures when a source enters, ordinary untaps, Torpor Orb
suppression, noncreature artifacts, tokens, both Spider-Woman cost alternatives,
and clone/restore replay with rejection of stale indexed actions.

The Python acceptance test exercises Authority with creature and noncreature
artifact entrants, Authority plus Torpor Orb, and Spider-Woman paid with white and
blue. It checks exact tapped status, entry contexts, life outcomes, preserved flying,
and clone/restore behavior through the C ABI. Native engine tests cover the deeper
simultaneous-entry, source departure, and legal-action boundaries.

Two release runs of 100 games reproduced 50,730 decisions, casts `[166, 195]`,
replacement-source witnesses `[786, 664]`, hybrid choices `[102, 92]`, 668 verified
Authority life effects, 1,414 exact entry contexts, 160 suppressed entry triggers,
373 token entries, 122 noncreature artifact entries, 5,975 ordinary untaps, 1,287
established-creature checks, 246,723 public stat/keyword checks, and 42,016 replay
boundaries. Maximum initialized logical state was 21,696 bytes. Both runs used
catalogue `67869794cc9a2221fe60246a8836b8ba03423f94e13d4f8dfc95dbf088d9a1b8`.
Full checks passed: **1,177 Rust tests**, **225 compiler/data tests**, **149 fresh
release Python tests**, catalogue consistency, formatting, all-target Clippy with
warnings denied and diff checks. The focused Python test passed all five scenarios.
Independent review results are recorded in the pull request review record. These workloads are policy
smoke runs, not an XMage parity comparison or neural-network inference benchmark.

The 1,000-game native smoke benchmark measured 571,049 decisions, approximately
327,156 decisions/s and 573 games/s, 136 ns clone and 183 ns restore on this host.
Timing includes policy and observation construction, excludes model inference,
tensor encoding and JSON, and varies with concurrent host work.
