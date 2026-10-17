# Role semantic kernels — issue 295 / draft PR 297

The complete bodies are Return Triumphant, Twisted Sewer-Witch, Syr Armont,
the Redeemer and Asinine Antics. The same generic predicates also complete
A Tale for the Ages and Mystical Tether. Both additional admissions have explicit
paid tests and complete source pins. Full Standard remains incomplete.

Return Triumphant stores an owned completion on its entry transaction. Only a
successful battlefield result creates a Young Hero Role, attached to that exact
new incarnation. Skipped moves, replacement destinations and a target that left
and reentered do not attach. The completion captures the Role creator separately
from the returned creature's controller, preserving owner-controlled returns.

Mass Roles freeze actual matching creature references. Rat matching includes
Changeling and actual creature control. One replacement event stores a group for
each recipient; doubling preserves bindings, while added Frogs remain unbound.
Stale recipients never shift later group indices. Roles enter together and use
the existing per-controller timestamp/SBA rules. Public `role_attachments`
exposes these frozen group bindings to both viewers, including unbound additions.
Ordinary token events allocate no recipient vector. Counter privacy is unchanged.

Syr Armont and A Tale use an exact attached-Aura predicate. The recipient must
currently be on the battlefield with an exact attached battlefield Aura. Its
controller determines the lord's recipient set, independently of Aura control.
Equipment does not count. An Aura remains attached until state-based actions
remove an illegal attachment; the predicate does not remove it prematurely.

`AnnounceFlashSurcharge` and `CastInfo.flash_surcharge` declare the optional
timing cost independently of casting method. Normal timing retains the cheaper
route. The extra cost precedes reductions, uses the caster's resources and
respects exact foreign-exile permissions. It does not grant a Flash keyword.
Mystical Tether retains the existing linked exile/return semantics. Unimplemented
combinations of Warp and paid flash remain gated. Observation schema 87 includes
the cast declaration and Role replacement bindings.

The audit-only [source manifest](role-semantic-admissions.json) pins all six full
bodies and resolvable direct imports: 201 records / 99 unique complete Java files
at `492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Shared Role factory sources are
included. This does not claim all transitive Java dependencies were rewritten.
No provenance inventory or name grants runtime permission.

The new quoted creature-token wording authenticates original raw text for every
generic caller, including spells, permanent triggers, activations, whole Sagas
and both Adventure/Omen faces. Parenthetical erasure detects the route only;
unknown, nested, infix and copied reminders cannot authorize a body. Own compiler
controls cover 314 clean and 1,530 adverse cases. The two independent reviewers
add 48/288 and 24/168 clean/adverse controls, plus paid cross-layer, foreign-return
and resolution-time recipient/control/permission-loss probes.

The final candidate completes 1,892/5,164 pinned Standard identities (3,272 remain
gated), schema87. Against actual main `8108a6f`, exactly these six definitions and
four required helper entries change; all 5,325 other definitions, 878 existing
helpers, 5,331 colors, 94 recipes, one emblem, three flags and 5,437 aliases remain
identical. The reverse comparison against fully tested `c5af333` changes only the
seven separately reviewed parent target-exclusion definitions, retaining every
one of the six authored programs and all 882 existing helper entries unchanged.
No unsupported partial body was admitted.

Full gates at frozen composition `c5af333` pass 1,784 native tests, all-target
Clippy, all 448 unique compiler/provenance methods (555.847 seconds, no skips),
and all 268 unique public API methods (1469.336 seconds, no skips). All 819 tracked
inputs remain identical through those runs. Its 100 fixed naturally terminal
Standard games pass in 263.838 seconds, with 84,835 owned frames and 169,670
wrong-actor/stale-revision denials. Its exact library SHA256 is
`c19a7a43fd584752759a86efc12bda44ff63c9e647f4b03f4730588cb681cdce`, schema87,
fingerprint `77c138c7c7ec750f0bdc30e74a7c74d6791480af9a47ab8cca89f22ee8f9ec3d`.
The complete paid sixteen-case policy, including live opposing hexproof at the
actual Asinine cast, passes within that fresh API suite.

The actual-main composition `3e681b6` separately passes 1,790 native tests
(1,778 unit plus 12 integration), all-target Clippy with warnings rejected,
formatting, deterministic generation and 36 affected compiler/provenance/raw
methods (29.583 seconds). All 825 tracked inputs remain identical through its
fresh native/library/public gates. Four fresh public methods pass all 42
naturally terminal Standard body/mode/seat cases: the sixteen six-body cases
in 45.842 seconds and all 26 parent target-exclusion cases in 63.146 seconds.

The final immutable library is
`/home/altf4/.cache/rustmage-immutable/roles295-3e681b6-aaf3-schema87-5ba70b459597.so`,
SHA256 `5ba70b459597fda7f99649187cd7d358a757d772fe92c7f1db6e54fe74e8830b`,
schema87, fingerprint
`aaf3add17ed8e6e8421a93e7beaa4314a35ff3244378a9384a323b010cc09bdc`.
Both independent reviewers rechecked this final production; one also reran the
paid Tale/Syr/Asinine layered interaction in both seats, including actual removal
and state-based deaths, on the final source. Review scopes exclude authored
ancestors and cover all six additions. Final published-head confirmations are
still the merge gate.

These final composition checks and strict equivalence carry the earlier full
suite evidence forward with its stated source scope; the 448/268 suites and
100-game workload are not described as fresh runs of the later composition.

Earlier attempts remain diagnostics. The quoted-token generic raw bypass found
at `3ace7b8` was corrected at `fb56027`; genuine generated programs stayed equal
except fingerprint. That checkpoint's full compiler run passed 440 methods and
failed one obsolete assertion that assumed uncounterable was the last helper.
Its public run was explicitly canceled after old exact CastInfo expectations
omitted the new false surcharge field. Both fixtures were corrected before the
complete passing `c5af333` runs. Earlier policy and missing example-field failures
are not counted as passes.

Every applied public action checks owned raw/index branches after parent
destruction, restore, both observations, wrong-actor rejection and stale-revision
rejection. Completed targets are cleaned of non-SO files immediately, retaining
every shared library at its exact original path and hash. Full Standard remains
incomplete. Two independent NONAUTHOR final published-head approvals are required
before merge.
