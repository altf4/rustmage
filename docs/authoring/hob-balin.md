# Balin, Loremaster: complete-card proof

Base: f6df3621. Isolated author checkout `/tmp/hob-balin`, branch
`codex/hob-balin`. Root owns generated registration and final full release validation.
No PR, push, merge, identity additions, public state, or observation vocabulary changes.

Oracle: 0d420e41-43e9-41d6-832c-5a9f410c994e, complete body from pinned
`data/standard/cards.json`. Read complete XMage BalinLoremaster.java and every helper
listed in paired provenance at revision 492369c7844b8d76479f0393ff9eb6b0b0cf30d7.
The XMage discard helper and custom damage effect use initial hand size. Oracle
instead says the number discarded this way; actual completed discard events are
authoritative. Count replaced-to-exile/library/BF discard events, exclude prevented
or skipped moves. No card characteristics are inspected to determine that count.

Rules authority: official 2026-09-25 CR cached at
`<cache>/rustmage-authoring/rules/MagicCompRules-20260925.txt`, SHA256
8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca.
CR701.9a/c (discard and replacement destinations), 608.2c/h (ordered instructions,
resolution-time information and LKI), 609.3 (do possible actions), 113.7a (source
independence), 614.6 (replacement event) govern this implementation.

| Obligation | Discriminating proof |
| --- | --- |
| 3RR paid4/4 legendary Dwarf Bard; optional hand discard, zero and decline | paid_body test: exact paid mana, body, self trigger;0/2decline/3accept, private hands, illegal Discard rollback |
| Self OR each other current controlled Dwarf, per actual entry | entry tests: enemy Dwarf/own Bear excluded,2 simultaneous tokens=>2 packets; source made Bear still self; Bear made Dwarf triggers; other printed Dwarf made Bear excluded; entry-time facts survive later type changes |
| Actual discard result, drawX, story damageX | exile and shuffle replacement tests count actual events and exact draw/damage; oldhand identities leave; new draws private |
| Story check occurs after draws and remains permanent | story gained after entry, threshold lost, source gone; damage still occurs; no-story positive discard does not damage |
| Ability controller/source snapshot | control change and source departure preserve original controller's hand/opponent; source LKI damage after departure |
| Multiple entries, trigger doubling, independent X | Bifur:2 tokens=>4 Balin packets; one declined=>3 independent discard/draw2/damage2 outcomes |
| Replacements affect damage independently | Twinflame damage4 while discard/draw remain2 |
| Private suspended draw then later damage | Lorehold Miracle first draw pauses withX3; life unchanged until remaining draws finish; source departure, private observations, raw/index/restore, ancestor disposal |
| Optional roots, failure and ownership | wrong action rejects atomically; shared branch_contract plus distinct decline/accept children and restored decisions |
| Zero resets old result | core result starts with99, accepted empty hand=>draw0 and damage0 |
| Linked entry result pause | synthetic entry-destination core contract: Adaptive Automaton private choice, Shock graveyard, skipped Forest; count2 after pause, draw2/damage2, raw/index/restore and ancestor disposal |

The linked-entry core contract deliberately injects a generic replacement event
because Balin's controller discarding their own hand cannot invoke opponent-only
Wilt-Leaf's replacement. Existing real Steal the Show scenario covers actual
opponent-discard-to-BF event routing. No new discard-prevention mechanic is added.

Mechanics: existing PermanentEntersYou include_source bypasses subtype only for
self; no entry matching changes. New private typed DiscardHandRememberCount instruction
uses the existing simultaneous discard/linked-entry context. RememberActual has no
payload or extra allocation and sets existing Resolution.moved_count only after the
batch commits (empty and nonempty overwrite previous results). DrawMoved reads it;
new private AmountValue::MovedCount reads the same exact saved result for damage.
Each independent resolution has its own field. Nested entry choices cannot execute
later program instructions until the entry batch completes. Clone/snapshot and
owned-byte accounting remain unchanged: no new fields/allocations, only an enum tag.

Validation: focused author family passed, consistent opt0 author scope; production
profiles and full release acceptance remain root-owned. Fixture names, complete
admission and exact Balin cost/types were preflighted against current catalogue.

Initial author receipt: `run-7ef1cf25694247b7ab5507911163698c` under the Balin cache.
Natural terminal exit101, 12passed/1failed, no timeout/orphans/OOM; guard42.879s,
peak2510458880 bytes. The failing assertion expected four pending packets before
existing multiplicity expansion (two base packets). Corrected it to check two base
packets, then four stack/ordering packets after prepare_triggers; no engine repair.
All inputs stayed fixed through terminal. The corrected helper preserves any live
trigger-ordering pending frame instead of redundantly resetting priority.

The result value is projected by the **existing public numeric**
Observation.group_move_count field. No key, type, enum vocabulary, or schema changes
are introduced. The instruction initializes0 before dispatching a batch; while a
linked entry is uncommitted it remains0, and once the batch commits it holds the
public number actually discarded. During a suspended Miracle draw both seats see
X, while drawn identities and reveal choices remain private. Declining the optional
block does not create a remembered result. Existing group-move users are unchanged.

Intermediate corrected receipt: run-3c2037b6d8ba4cbeb03380a78438ee7c, terminal0,
62passes/noignored across8 groups. Compile27.95s; guard187.505s including slot wait;
peak2528485376 bytes; no timeout/orphans/OOM. Final test-only additions assert
concurrent self+other mixed-controller entry and both public count projections at
the existing private Miracle suspension. No behavior repair after this pass.

## Final terminal evidence

Final receipt directory:
`<cache>/rustmage-authoring/hob-balin/run-ac6d1a1173344d3cb08c6d30856083b5`.
Persistent driver.py, entire outer.log, per-group native logs, results.json and
resource.json are preserved. Session18489 naturally returned0; resource terminal=true,
exit_code0, one Cargo job/test thread,6GiB cap, no memoryhigh/max/OOM events,
no timeout or orphaned processes. Inputs were fixed through natural termination.

63passed/0failed/0ignored:14 Balin (12card+2generic result),2 Steal the Show,
2discard follow-on,14storied,13trigger multiplicity,5Miracle,7connive,6cast rollback.
Test profile opt-level0 consistently uses the dedicated external Balin target;
compile28.08s; guard47.519959s including shared-slot wait; peak2485321728 bytes.
Reported native scenario execution totals0.89s across eight groups; Balin0.03s.
These are bounded author proof timings, not complete-game simulation throughput
or production profile performance. Root retains full release/performance acceptance.

Final source pins5417definitions/1565source blobs verified; catalogue version
55904a3cb4bef8df9d5aa25ac0dc6964ddcb5501e4f6bfdf2e7ec1f94afdab08.
Formatting and diff whitespace checks pass. No behavioral blockers remain in this
card; two independent nonauthor reviews and root integration acceptance remain.
Local generated coverage/catalogue and assurance/mod.rs registration are excluded
from the author commit and retained for reproducibility; root registers/regenerates.

Compiled input SHA256 (before this documentation-only receipt update):
- `src/card_definitions/hob/balin_loremaster.rs`: `e50c1bcc9d5de6249e841fbd5f858fd6587cbd6d08528cf5cde4be9da1cc1110`
- `src/card_definitions/hob/balin_loremaster.json`: `ad658a9d7ac7e6eb0725ec151ae5fa646db5362580d3b1846767cf8c43ec94de`
- `src/cards.rs`: `0c892355798af6edd83f08ac3b0e97197adb60fc040060b5e38255903a93a3b7`
- `src/engine_discard_events.rs`: `fcfc1e7ea2cbae1de78b3c4e788d40d41f7e8b66c42e79a823be273f9fcdbc0a`
- `src/engine_programs.rs`: `8d6b656e5994ad82c95ca903d9d1c7c1d7c2e1eba5e59b00f14e1e8c79c7b627`
- `src/engine_scalar_x.rs`: `804f2a21ffcdbc2bf5aba01e2674e774209ecbe720f616f021c85149487b62ed`
- `src/engine_stat_values.rs`: `3380d3313620ede82b5112faee5321304a9adc0ab9e15017420f2142226cfe57`
- `src/assurance/hob_balin.rs`: `4ca37bc30b2c26b73c5975170ba44f92bf99df41f06c2fb0e3706bc1cf8674be`

Receipt SHA256:

- `driver.py`: `4e9e005da87a5b5957ef65ff4fc7b3e8e0574371fe83d76733d4eed2f3d74468`
- `outer.log`: `bb993cb111a48e569348142e69eb8a94dbbeeab967b8371c403a78b0105931b3`
- `results.json`: `636878388a5491deb213acb137aaba80ac7209222934af2005f16c757a1bc923`
- `resource.json`: `da27f6b66931b608446ab1baf74ee65999a756e030874868273ec5dcb6e791c6`
