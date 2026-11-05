# Tarkir: Dragonstorm implementation

Tracking issue: [#570](https://github.com/altf4/rustmage/issues/570).
Branch: `codex/tdm-complete-set`; baseline `c87b55079c540edde1c49cc725a1e1475bebf68b`.
Pinned XMage: `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`.

All 276 physical identities are whole-verified: 139 new implementations and 137
existing implementations verified. All 13 Omen pairs and the Reliquary Dragon
factory are complete. Cori-Steel Cutter retains its frozen Standard ban and
separate Freeform coverage. The immutable printed ledger remains
`b6716c0d6f18774988997e1183caaf5b0f977d2049406f56da7dbfaecec2942a`.

The sixteen-family matrix binds 265 named source proofs with no gaps. Two pure
preliminary reviewers read all 276 bodies, 13 Omen bodies, full Oracle text and
603 unique primary/helper Java blobs. Engine86 had full release acceptance before
card authoring. Later engine87 preserves paid child mana-ability use receipts;
engine88 conservatively groups same-source tap capacity and protects future
restored producers. Their focused cases, Clippy and two independent reviews pass.
The actual 830-action slow trace preserves its public X-choice frame, final
snapshot, both final views and outcome. Observation schema remains201.

The fixed 52-game corpus passes: 48 Standard and 4 separate Cutter Freeform,
all natural life-total terminals, all five colors and eight required mechanic
payoffs in Standard, 29,332 actual/replayed actions, 286–1026 decisions per game,
and 3–5 meaningful owned roots per game. Decks, seeds and 4,096-action ceilings
stay fixed. Independent reconstructed parents, divergent raw/index siblings,
parent disposal, restore, atomic rejection and paired full observations are checked.
Twelve source-binding and two compiled stale/tampered corpus rejection contracts
pass; runtime identity pins 11,312 exact execution inputs.

The same verified Standard48 workload repeated3 times measured 2.756 games/s and
1,572.746 decisions/s; separate Freeform4x3 measured 13.495 games/s. This is the
test profile with core/dependencies opt1, harness opt0 and test support/debug
assertions. It includes construction/shuffle, legal actions, generic policy/actor
observations, rules and light cast telemetry. It excludes loader validation,
replay/owned proof work, detailed receipts/JSON and model/network/tensor work.
These are workload measurements rather than production or all-Standard estimates.

The initial PR CI exposed missing default game-evidence plumbing. The validator
now supplies a unique retained report path, and standalone native calls allocate
a fresh retained child even for reports sharing a parent. Explicit paths and old
receipts stay unchanged. Three meaningful orchestration regressions pass. The
following full acceptance used no caller-supplied evidence variable; runtime and
card behavior remain identical. Earlier explicit-path acceptance took577.354s.

Final default full release acceptance passed all9 phases at
`16a9cec6882614f1208c34aa832662ee9f6aaa58`: 4,693 native cases across115 targets,
all265 matrix proofs, 46 runner contracts and10 actual Python binding cases.
All13,040 frozen inputs stayed unchanged; zero timeouts, orphaned processes or
OOM events. Native compilation took 1.118s, execution
153.141s, production release 0.115s and
total validation 165.988s. Peak guarded memory was
2018320384 bytes below the6GiB limit, with one Cargo job/test
thread. All seven required immutable libraries remain at their exact paths/hashes.
The fresh full-suite52-game corpus matches the earlier verified corpus's cases,
outcomes, actions, witnesses and final state/observation digests.

README records TDM276/276, 10/20 complete sets and3,576/5,164 Standard identities
across accepted branches. Current checkout supports3,303 total/3,296 Standard.
The count audit preserves accepted ECL/TMT coverage and pins the union by Oracle
UUID. Through game/benchmark acceptance,139 new physical implementations took
53.203 elapsed hours, including audit, engine work, integration, reviews, validation
and waiting. Final release/publication steps follow that measured interval.

[The acceptance record](tdm-set-acceptance.json) binds all retained reports and
records the documentation-only results delta after validation. Update the existing single completed-set PR573, then obtain two renewed pure
independent nonauthor reviews
of its exact published head and record those reviews on the PR. No executable
changes are needed after passing acceptance. Keep the goal open until renewed published-head
reviews and GitHub CI pass. The wider Standard roadmap remains open.
