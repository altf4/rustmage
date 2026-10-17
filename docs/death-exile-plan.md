# Temporary would-die exile replacements — issue313

This batch adds eleven complete bodies on actual parent
`5d103e7a3172e4742687d3f77f3ed814264b1532`: Feed the Flames, Bot Bashing Time,
Elspeth’s Smite, Narset’s Rebuke, Obliterating Bolt, Scorching Dragonfire, Suplex,
Agate Assault, Essence Burn, Combustion Technique and Pinecone Strike.
Coverage is **1,914/5,164 Standard identities**, with **3,250 gated**, forty
Adventure pairs, thirteen Omen pairs, 94 token recipes and one emblem. The strict
parent comparison changes exactly these eleven definitions, preserves the other
5,320 definitions and all 888 helper arms, colors, aliases, flags and recipes.
Full Standard support and the complete training/search goal remain unfinished.

The [whole eighteen-card audit](death-exile-admissions.json) contains complete
metadata, 242 source records and 88 complete unique Java files pinned at
`492369c7844b8d76479f0393ff9eb6b0b0cf30d7`. Every blob matches the checkout;
every explicit class import resolves. Wildcard imports are recorded without a
claim of transitive Java migration. The audit records original inventory parent
c023 separately from the delivered program comparison parent5d. It cannot grant
admission. No partial actual card, name whitelist or erased unknown clause is used.

The engine substitutes direct exile for an exact battlefield-to-graveyard move
before death/graveyard observers. Sparse owned rows bind the exact incarnation,
historical source/controller and current-turn expiry; no object field is added.
Source departure does not revoke the effect. Recipient departure clears it, and
cleanup clears it before state actions. Tokens visit exile and then follow ordinary
token state actions; no exile-play permission is created. Replaced deaths suppress
normal dies triggers and temporary death grants. Duplicate source order is tested
as idempotent because all currently admitted destination replacements require exile.
This does not implement a choice UI for future conflicting destination replacements.
Public owned observations use schema92; clone, child survival after parent drop,
restore, indexed/raw actions and illegal-action atomicity cover the sparse state.

Generic compiler lowering preserves entire damage/replacement groups and all
independent modes. It adds typed black/green creature-or-planeswalker unions,
artifact-token targeting and constant-plus-Lesson-count damage. Every new caller,
including callers without the exile clause, authenticates untouched raw metadata
through source-name and normalization stages. Renamed/reserved-name positives and
unknown, infix, nested, stolen-reminder, malformed-cost/type and incomplete-body
negatives cover these routes.

Betrayer’s Bargain, Faunsbane Troll, Fiery Annihilation, Gnashing of Teeth,
Malicious Eclipse, Torch the Tower and Wilt in the Heat remain gated. Their
complete dependencies are tracked in [issue323](https://github.com/altf4/rustmage/issues/323).
In particular, this exact-target replacement is unconditional on damage completion;
it does not implement Torch the Tower’s actual-damage binding or a new prevention
kernel. Zero-damage registration is tested as a primitive boundary, not as a claim
that a new prevention card is supported.

## Validation and source scope

Frozen execution source is `667c502738208622f40a536ee1296c4da8b7a59f`.
The optimized library is
`<cache>/rustmage-immutable/death313-667c502-ba88-schema92-6a8611dcf214.so`,
SHA256 `6a8611dcf214ed40f59c2cf11f204cc4ee1cb00d2c726f530d15ee3b283e4f1d`,
catalogue fingerprint
`ba88114f740c2281de8da35f1861924ea17d94170f57565bc78c03199e9f95fe`.
All 857 tracked execution inputs and that library were hash-checked before and
after the broad gates. Final assertion/audit/documentation corrections preserve
all production, generator, card data, generated catalogue and public policy bytes.

- Full native: **1,836** tests (1,824 unit +12 integration), all-target Clippy with
  warnings denied and formatting pass; combined gate35.588s.
- Full unique public API: **276 methods**, no failures/errors/skips,1877.512s.
  Default-library callers and decoded-NUL subprocesses use the same pinned library.
- Thirty natural paid Standard cases: all fifteen full body/mode vectors in both
  seats,52.446s. Native acceptance separately pays all sixty body/mode/seat/local
  or foreign-origin combinations, plus actual planeswalker targets, caster-relative
  Lesson counts, nonlethal damage followed by Murder/death-grant suppression,
  partial/stale groups, counters, simultaneous movement, tokens and cleanup.
- Fixed unretried **100 natural Standard games**:170.996s,73,375 owned frames,
  146,750 wrong-actor/stale-revision denials; every game finishes naturally.
- Full unique compiler: **483 methods,480 passes and three failures**, no errors
  or skips,707.358s. The failures are inherited PR309 assertions: Mouser Mark III
  and Technodrome still expected gated, and the combat canonical fixture compared
  its old parent directly to later additions. Corrections assert both cards are
  supported and compare the historical five-card delivery to its pinned final
  catalogue, then preserve those five definitions and six helper arms in current
  output. This failed full gate remains historical, not a passing final claim.

The fresh corrected affected suite passes **276 methods** (264 complete catalogue,
five combat and seven death-exile methods), no failures/errors/skips,537.037s.
This includes all three previously failing assertions, strict current-parent
preservation, generic raw controls and complete pinned Java provenance. Generator
check passes with the unchanged fingerprint. The final six-file delta is limited
to two test files, the audit baseline label and three Markdown records; all
production, generator, card data, catalogue and public policy inputs remain identical.
 Earlier native compile and
fixture diagnostics remain recorded; paid native claims begin at f17ef2c and the
full frozen validation above uses667c502. Two independent NONAUTHOR reviews inspect
the whole eleven-card batch and final published head before merge.
