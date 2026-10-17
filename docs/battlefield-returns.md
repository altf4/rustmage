# Simultaneous non-targeted battlefield returns

Issue #306 implements complete Aetherize and River’s Rebuke on actual main
8108a6f. Prototype coverage is 1,888/5,164 Standard identities, 3,276 gated;
observation schema86 is unchanged. This is an incomplete candidate, not full
Standard support.

The immutable ReturnBattlefield instruction selects a live battlefield group at
resolution and freezes it before any simultaneous departure. Aetherize selects
current attacking creatures. River’s Rebuke targets a player, then selects that
player’s current nonland permanents. The group members are not targets, so their
hexproof, ward and indestructible do not prevent return. Existing owned zone
moves return stolen objects to their owners, clear combat/attachment bindings,
remove departed tokens and dispatch source-linked returns. The latter cannot
recapture a creature that was in exile when the group was frozen. No mutable
state field or per-object allocation is added.

A generic whole-clause compiler also supports the complete all-creatures return
instruction, while compound unimplemented consumers remain gated. Every new
caller authenticates untouched raw rules before historical parenthetical erasure.
The pinned audit includes two complete Java card records and their explicit
direct imports (20 records); it is neither an execution whitelist nor a claim
that transitive Java behavior was ported.

Initial focused validation passes eleven compiler/provenance methods. The exact
main comparison preserves 5,329 of 5,331 definitions plus every helper, color,
alias, recipe and flag; only the two complete bodies change. Four new paid native
groups pass both seats in 0.02 seconds, exercising legal attacks, ward/hexproof,
foreign-owned recipients, exact owner/controller returns, tokens, simultaneous
Aura departures versus later Aura state actions, real Banishing Light departure
and return, empty groups, sorcery timing, underpayment and per-action owned
raw/index branching after parent drop, restoration and both-player observations.

Initial diagnostics are retained: the first native build failed on two test
references to a nonexistent PlayerState.hand field; the next run passed three
groups and correctly rejected a fixture asking Banishing Light to exile its own
controller’s creature. Correcting that fixture to an opponent-controlled,
caster-owned creature makes the intended source-return case legal. Neither
failure required a production runtime change. All terminal prototype targets
were cleaned immediately with shared-library paths/hashes preserved.

Frozen baseline `dc3ec3511c7ce545fedf1f334a5dc311c143abb0`, on the stated
8108a6f parent, passes 1,785 native tests, all-target Clippy and formatting,
452 unique compiler methods (830.079 seconds), and 269 unique public API methods
(1,483.921 seconds), without failures, errors or skips. Four focused public
body/seat cells pass in 18.136 seconds. The fixed, unretried 100-game natural
Standard workload passes in 302.306 seconds with 82,844 owned frames and
165,688 denial checks. All 826 frozen execution files and the immutable library
hash remain unchanged for every gate.

The baseline optimized library is
`returns306-dc3ec35-3f71-schema86-020003f66419.so`, SHA-256
`020003f66419d89669d15681413004a44b61ac6bd4674f32d9bdb09a20725531`.
Its catalogue fingerprint is
`3f7109af57d830b6395d73d987868854f50c4473635b61b6fa5c8e6cdbe955e0`.
Both independent preliminary complete-batch reviews report no findings; their
novel paid interaction checks and raw-rule mutation controls are retained.

Composition with the newer main parent, fresh integration checks, and two
independent approvals at the final published head remain required. The broad
baseline results above do not claim validation of that future composition.
