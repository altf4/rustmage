# Collect evidence: deferred exact payments

Issue [#226](https://github.com/altf4/rustmage/issues/226) adds ten complete Standard bodies to the in-process engine. On the standalone bargain base, the catalogue implements **1,597 / 5,164 Standard cards**, with 3,567 gated, 74 existing immutable token factories and one emblem. Its fingerprint is `22f548933f6d5c805c97e02e898f905d30dfbf8d04053525603900617b861d09`.

| Complete body | Behavior |
| --- | --- |
| Analyze the Pollen | Optional evidence 8; one basic-land search, replaced by one creature-or-land search when paid; reveal, Hand and shuffle. |
| Axebane Ferox | 4/4, deathtouch and haste; ward asks the targeted spell or ability's controller to collect evidence 4. |
| Bite Down on Crime | Optional evidence 6 reduces the locked generic cost by two; the controlled target gets +2/+0, then deals its current power to the opposing target. |
| Crimestopper Sprite | 2/2 flying; optional evidence 6; entry taps any target creature and adds one stun counter only for the paid cast. |
| Evidence Examiner | 2/2; optional evidence 4 at its controller's beginning of combat; every genuine collection investigates through the complete Clue factory. |
| Extract a Confession | Optional evidence 6; the opponent chooses a creature to sacrifice, restricted to every tie for greatest current signed power when paid. |
| Forensic Researcher | 1/3; tap to untap another controlled permanent, or tap and collect evidence 3 to tap an opposing target creature. |
| Sample Collector | 2/3; an actual attack offers evidence 3; successful payment creates a separate reflexive trigger targeting a controlled creature for a +1/+1 counter. |
| Surveillance Monitor | 3/3; entry offers evidence 4; genuine collections create the complete 1/1 colorless artifact Thopter with flying. |
| Vitu-Ghazi Inspector | 1/3 reach; optional evidence 6; qualified entry targets any creature for a counter and gains two life, subject to normal all-target fizzle. |

The [provenance manifest](evidence-admissions.json) records every full admitted and gated Oracle body, constructor and supporting source. All **360 SHA-256 records across 147 Java files** match checkout bytes and XMage Git revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. The twelve other evidence cards remain gated: their missing composites include global alternative casting, variable thresholds, type/base transformations, multizone named exile, crew/linkage, death/self-exile replacement, four-token sacrifice and arbitrary target-value allocation. Fixed evidence spell selectors independently require a valid bounded fixed mana cost; malformed symbols, Phyrexian costs, overflow and X cannot revive an unsupported body.

## Public declarations and payment

Observation schema **64** adds `CastInfo.evidence: Option<bool>`, `ChooseCollectEvidence { collect }`, `ChooseEvidence { object }`, `FinishEvidence` and `DeclineEvidence`. An optional casting declaration is chosen at `evidence_intent` before targets, mana abilities and payment. `Some(true)` describes the declared additional cost; it does not claim that cards have already been exiled. Actual subset selection starts only after targets. A mandatory activated evidence cost uses the same subset transaction after its targets are selected. Direct casting and activation shortcuts cannot bypass these costs. A real authorized normal cast from Exile follows the same declaration and payment path.

The choosing player's private `evidence` observation contains its player, threshold, exact selected references, `total_mana`, remaining exact candidates, optional/completion flags and payment status. Other viewers receive no frame or actions. Any eligible Graveyard subset whose nonnegative mana-value sum reaches the threshold is legal. Off-stack X contributes zero. The engine checks existence with a linear sum, without enumerating subsets. Zero-value cards may accompany sufficient evidence. Reaching the threshold never auto-finishes: more cards remain selectable until explicit `FinishEvidence`.

All partial choices remain in Graveyard at their original generations. For casting and activation costs, even `FinishEvidence` only completes the declaration: the actual exiles occur atomically with real cost payment. The selected resources cannot be reused. Supported mana recipes cannot consume Graveyard cards; composite evidence/self-exile and graveyard mana recipes remain gated. Conditional generic reductions lock through the ordinary total-cost transaction before mana abilities and payments. Colored requirements are preserved.

Resolving optional payments use the same owned selection. Declining, including after partial selection, moves no cards and emits no collection. Successful `FinishEvidence` commits the whole payment, then continues the surrounding program. Sample Collector's reflexive targets are chosen afterward. Ward captures the associated stack identity and its controller as payer; it counters that exact packet on decline or insufficient evidence, independently of the ward source's departure. The payer's observers receive the collection event, rather than the ward source controller's observers.

## Events, entry history and resolution

Only a committed successful collection emits the event. Indexed battlefield observers are captured from the current controlled sources and each queues once for that collection. The paying source need not remain on the battlefield. The bounded public `last_evidence` record contains a monotonically increasing sequence, payer, historical source, requested threshold, actual mana-value sum and new exact Exile references. Failed or declined actions leave it unchanged.

Entry qualification travels from the exact cast through its staged entry event into the owned trigger packet's `evidence_collected` context. No object carries an inherited evidence marker. A queued paid Sprite or Inspector trigger retains its history after departure/reentry; the new incarnation is unqualified. Entry-trigger suppression prevents the entry residuals, while actual collection observers still fire during cost payment. Clues use their complete shared two-mana/sacrifice/draw recipe; Thopters preserve printed artifact, color, size and flying characteristics.

Bite Down on Crime uses the existing current target-group filter for growth and both exact bound groups for power damage. It never boosts the opposing target. If only the owned target becomes illegal, there is no growth or power damage; if only the opposing target becomes illegal, the valid owned target still grows. If both become illegal, the whole spell fizzles. Extract's non-targeted sacrifice choice caches the actual live signed maximum at resolution and includes every tied controlled creature, including when all powers are negative.

## Ownership and memory

`EvidenceSelection` occupies **64 bytes plus 16 bytes per selected exact reference**. Optional `EvidenceState` occupies **112 bytes**, plus four bytes per indexed observer and sixteen bytes per reference in its bounded latest event. Logical accounting includes these initialized allocations. Objects remain **144 bytes** with no evidence qualification field. Clones and snapshots own selections, event history, observer indexes, continuations and RNG.

Initial standalone state is **20,024 logical bytes**, 48 more than the 19,976-byte bargain parent. The Standard evidence workload reaches **21,192 bytes**. These figures exclude spare capacity, allocator overhead, static catalogue data, external snapshots and observations. A contended native 1,000-game smoke run recorded 571,049 decisions at about **188,961 decisions/s**, 166 ns clone and 164 ns restore. It includes policy and observation construction, excludes model/tensors/JSON and supplies no Java comparison.

## Standalone acceptance

Seventeen new native groups cover every body and both controllers; declaration and subset timing; exact references and illegal shortcuts; real authorized Exile casts; ordinary and all-negative greatest-power ties; each partial-invalid Bite target; genuine payer-owned ward collections for spells and abilities; source departure/reentry; suppression; actual Clue payment/draw; reflexive timing; optional partial decline; zero-value extras; memory ownership; partial-exile and late observer/event/cast allocation rollback. The complete all-targets native suite passes **1,456 tests** (1,444 unit and twelve integration tests). Clippy with warnings denied, formatting, generated-catalogue checks and all eleven Python-module ASTs pass.

The [public Python suite](../python/test_evidence.py) has **three methods and 48 real paid scenarios**: forty whole-card/controller/payment-profile cases, four actual ward payments or declines after Forensic's mandatory activated payment, and four real Evolving Wilds sacrifices whose zero-value cards are selected after the threshold is already reached. Every action checks clone/snapshot/restore, stale revisions and both viewers. The 48 scenarios pass in 24.975 seconds in this concurrent run. Profiles select optional casting, resolution or ward choices where present; Forensic's evidence activation remains mandatory in both profiles.

A fresh isolated release-library run passes the complete **190-test Python suite in 429.741 seconds**, including these three methods. The library is explicitly selected in the parent process and its subprocess probes.

The [Standard driver](../examples/standard_evidence.rs) uses legal sixty-card decks and reproduces the same full vectors twice across 100 games: **77,046 decisions and owned replay boundaries**, with every forty-cell cast/effect ledger positive. It observes 65 committed collections, 130 exiled cards, 50 real zero-value extras, 332 private frames, 664 atomic invalid-action denials, 22 ward frames, 54 optional declines, ten revealed search puts, ten actual untaps, ten evidence taps, four reflexive effects, four Clues and four Thopters. Adjacent bargain Standard100 preserves all vectors and 85,454 decisions; its peak becomes 23,132 logical bytes.

Canonical comparison preserves **5,248 definitions**, with exactly ten complete bodies changed; all 74 token recipes and metadata, ordinary identities, colors and 504 prior numeric helper arms remain unchanged. Five helper arms encode optional evidence cast metadata.

Rules basis: [Comprehensive Rules, September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf), CR701.59a–c, 601.2b/c/f–h, 400.7, 603.12 and 702.21. Native late-failure tests restore mana, source tap state, all exiles, history, observer queues, private frames, generations, revision and RNG to the pre-action checkpoint.
