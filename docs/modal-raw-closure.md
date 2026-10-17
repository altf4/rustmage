# Original-raw modal reminder authentication

Issue 334 contributes a focused fix to the wider audit in #261. Triggered, activated,
spell and mixed modal callers previously could erase an unknown parenthetical
before compiling the remaining effects. They now authenticate the original
balanced text before lowering it into a semantic parser. Unknown suffixes,
infixes, detached text, malformed delimiters and nested unknown clauses gate the
whole body, independently of its card name.

Each reminder must prove its actual typed mechanic, count, subject, payment or
token recipe on its own line. Matching words inside names and flavor labels
provide no authority. Nested reminders prove their own mechanic first, then the
containing reminder proves the complete outer production. Valid Royal/Ward and
Sorcerer/Scry nesting is retained; recursion beyond 64 levels fails closed.

Detection examines complete “choose one [or both] —” headers across successive
erasure stages. Erased text is used only to detect a potentially hidden route.
It never becomes executable input. Tiered's “Choose one additional cost” reminder
stays untouched and is authenticated by the complete Tiered parser instead.
Durable tests preserve all six Tiered bodies and both adjacent modal cards while
rejecting altered Tiered reminder text.

The final source is composed with actual merged main `356d798`, schema 100. All
5,332 definitions, 924 helpers, 5,332 colors, 5,438 aliases, 95 token recipes and the
emblem are unchanged. The entire generated Rust file is byte-identical except its
fingerprint, and coverage is identical except that fingerprint. Coverage remains
1,972 Standard identities/1,978 total supported definitions, 40 Adventure and 13 Omen.
No engine Rust or observation-schema change is introduced. The fingerprint is
1551e6ebeb4ee554c01f2f108fd5e9f3fc90b78ac5385ce3e7c99d7c2ffdd7a4.

Validation at source `8532264`:

- All 43 affected methods pass in 6.550s, including ten own methods plus Tiered,
  live-stat, private-hand, intrinsic-reminder and legacy-metadata checks.
- Four additional inherited modal methods pass in 4.265s; generator check passes.
- A fresh optimized immutable library builds in 27.213s from unchanged tracked
  inputs. No parent runtime gate is relabeled as a new run for this compiler fix.
- Five public methods pass in 116.626s (116.843s driver), with zero failures,
  errors or skips: 48 paid Tiered/adjacent cases, 24 paid hand-land cases covering
  every Fungus mode, 12 triggered-modal fixtures and two modal-spell fixtures.
  All 913 tracked inputs and the immutable library remain unchanged.

Independent review controls include 114 positives/1,085 negatives and strict
actual-parent preservation with a mutation control. Two independent nonauthor exact-head approvals
are required before merge. This phase leaves #261's broader raw-authentication audit
open; it introduces no new Standard card admissions.

Earlier prototype evidence is historical: nine tests and a separate composition
with Tiered checkpoint `26adbde` preserved all 5,332 definitions/918 helpers. Development
fixture diagnostics (initial absent-legacy gating placement, structured Fight
matching and imported duplicate test collection) were corrected before the
passing prototype; none are represented as successful gates. Final source/source
and library hashes are preserved in the delivery evidence, with all terminal
build outputs cleaned while every shared-library path/content remains intact.
