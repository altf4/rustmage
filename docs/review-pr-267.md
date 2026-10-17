# PR267: complete same-graveyard targeting bodies

Issue #266 adds complete Feral Deathgorger // Dusk Sight, Arashin Sunshield and Griffnaut Tracker to reviewed Omen/Saga/Ward main. Coverage is 1,813/5,164 Standard identities, 3,351 gated, 87 token factories, one emblem, nineteen Adventure and ten Omen helpers (5,300 definitions). The all-Standard migration remains incomplete.

## Frozen source and artifact

- Runtime composition: `1b12924d665ec85c926b832fdb0a9bb912569fbe`.
- Standard acceptance driver: `eac8ecc77476fc564b96898387679991d1785bc7`; its four-line assertion fix hides partial target bindings while retaining public announcement metadata. The initial failed fixture log is retained; the corrected full100 run passes without runtime changes.
- Reviewed main documentation/history merge: `787efaf`.
- Final strict compiler and generic dispatch: `400e115932f296eadf0118e38cb3b5e5f836204a`. Every raw/intermediate parenthesis-erasure stage participates in route detection; unchanged raw bodies must still authenticate. New generic graveyard bodies bypass legacy name selectors, preserving complete renamed-body rules and rejecting unsupported residuals. Generated output differs from frozen 1b only in the catalogue fingerprint. All 5,300 definitions, helper programs, colors and factories remain byte-identical.
- Final schema 79 fingerprint: `b80c6e3eb1f68ea84caf2a3bdb24287fdc9ffa8ab57869cfdc997d88db854689`.
- Final optimized immutable: `/home/altf4/.cache/rustmage-immutable/feral267-400e115-b80c6e-schema79-release.so`, SHA256 `14374923a5f2aebdc74ab33859c5660065c14ff3f72637164bc6c11a7dd5cfb9`.
- Preceding frozen optimized immutable: `/home/altf4/.cache/rustmage-immutable/feral267-composed-1b12924-63126a-schema79-release.so`, SHA256 `beff24533fe45174418cfd2ca66f60d270485dc38dc0e1b78a605c2581c9c2a2`, fingerprint `63126a789d0a9b7f6cc1bbf30854a0c2c6d96f21e0e11446de86a27aea08b279`. It remains preserved; full broad gates below cover this precise source/artifact before the compiler-only closure follow-up.

## Rules and rollback

`TargetRule::Grouped` adds immutable `TargetGroupConstraint::SameGraveyard` to the ordinary card-in-graveyard predicate. Group-indexed existing announcement bindings select the graveyard owner; no extra mutable cache or observation fields are added. Actual legal masks and upfront required-target availability enforce the relation. Different occurrences remain independent, distinctness remains enforced, and exact object incarnations control resolution after departure/return.

Seven new both-seat native groups cover zero/one/two targets, both graveyards, mixed/duplicate denial, empty/required availability, partial/all illegal resolution, complete Dusk Sight, both complete adjacent bodies and paid activation. Cross groups include actual Shocks, a foreign Ward Aura and countered Dusk followed by physical same-graveyard selection, plus a declaration-only synthetic two-occurrence test. Each action exercises owned child lifetime, rollback/replay and illegal action atomicity. The printed Feral body has one occurrence. Public cases construct graveyards with real spells and pay using real lands. The separate legal60 Standard driver completes natural games and requires complete live-body behavior before terminal completion.

## Completed author gates

On frozen 1b/631, with the final acceptance driver where applicable:

- 1,675 native tests pass: 1,663 unit in 5.47s and twelve integration. All-target Clippy passes in 12.90s. Logs `/tmp/rustmage-feral-composed-native-final.log` and `-clippy-final.log`.
- Full 363 compiler/data methods pass in 424.902s with pinned Java audit enabled. `/tmp/rustmage-feral-composed-compiler-final.log`.
- All 239 unique public methods pass in 727.508s. Parent and NUL-probe child explicitly select the optimized immutable; parent constructors assert schema 79 and full fingerprint. Collection `/tmp/rustmage-feral-composed-public-collection.json`; log `-public-final.log`.
- Full Standard100 passes with 89,944 parent-closed owned frames, 179,888 wrong-actor/stale-revision denials and 210 counted private frames. All ten Omen pairs cover both faces and seats; Arashin and Griffnaut cover normal bodies in both seats, including actual same-graveyard exiles, activation and attacks. Wall 168.55s, user 167.66s, peak RSS 53,944KiB. `/tmp/rustmage-feral-composed-standard100-final.log/.time`.

Final 400/b80c follow-up, after all preceding frozen gates ended:

- Ten focused Feral compiler/provenance methods pass in 0.389s; five Omen methods pass in 0.230s. New controls cover all three bodies, raw/wrapped clauses, split literals, whitespace, capitalization and intermediate erasure stages, complete renamed-body semantics and unknown constraints under legacy aliases. `/tmp/rustmage-feral-final-legacy-strict.log`.
- All three unique public methods pass in 21.786s, covering twenty actual-paid cases and both seats. Every constructor explicitly selects the final immutable and asserts schema 79/full b80c fingerprint. `/tmp/rustmage-feral-final-b80c-public.log`.
- Final generated-file, formatting and whitespace checks pass. The two successive compiler-only changes each preserve every generated real-card program and all coverage except the fingerprint: `/tmp/rustmage-feral-every-stage-fp-only.json` and `/tmp/rustmage-feral-legacy-fp-only.json`.

Between these compiler follow-ups, the preserved ecff/354 immutable also passes all twenty paid cases in 27.232s and another complete Standard100 with 89,944 owned frames, 179,888 denials, 210 counted private frames and all required body/face/seat vectors. Wall 197.07s, user 192.94s, peak RSS 54,332KiB. Exact path `/home/altf4/.cache/rustmage-immutable/feral267-ecffdcc-3542ce-schema79-release.so`, SHA256 `5facd9fbde468bc94c32f1ab051cd7b858a901b14dffdf1a6f74e7b547fee7e3`, fingerprint `3542cea960685bc602522261e63c1960aadac4302f3ba3ee4ee24259be196ca6`. Logs `/tmp/rustmage-feral-final354-standard.log/.time` and `-public.log`. This full100 is not claimed to have run on the final b80c fingerprint; final b80c acceptance uses the identical-program proof plus fresh focused compiler and twenty paid public cases.

Public/Standard timings include JSON observations, both-view checks, owned branching/replay and denial verification. They are acceptance timings, not isolated engine throughput. An existing layout diagnostic reports 2,544 inline state bytes and 20,344 logical initial bytes for two 60-card decks, without claiming clone allocation equals inline state size.

## Preservation, provenance and cleanup

Canonical comparison against ead Omen/main preserves 5,296 definitions and changes exactly three ordinary bodies; all 779 existing helper arms, colors, token factories and emblem survive. Against standalone Feral94f, exactly 22 reviewed Saga/Ward bodies change and 5,276 other definitions survive. Combined helper arms total 782. All 1,661 provenance records across 575 unique Java paths match both pinned revision `492369c7844b8d76479f0393ff9eb6b0b0cf30d7` and checkout bytes. `/tmp/rustmage-feral-composed-canonical.py/.json/.log` and `/tmp/rustmage-feral-every-stage-fp-only.json` retain exact proofs.

Completed exclusive Cargo targets are cleaned while preserving every shared-library path and hash. Combined debug target:2,055 disposable files/2,636,336,587 logical bytes; initial optimized target:168/101,877,830; intermediate354 optimized target:168/101,877,938; final b80c optimized target:168/101,877,406. Cleanup totals include hard links and do not measure physical disk-space recovery. All source, worktrees, acceptance logs and immutable libraries remain intact. Earlier standalone evidence is separately scoped in [the feature record](single-graveyard.md).

## Independent review

Two independent nonauthor reviewers must confirm the final published head after inspecting source, strict parser closure, provenance, preservation, paid/rollback probes and terminal evidence. Final exact-head confirmations remain pending. Future Whirl/Bloom integrations are excluded from this record.
