# Schema 191 prepared-copy migration evidence

The published schema-190 baseline in `src/assurance/card_traces.json` contains 2,929 card traces (SHA-256 `a0fb3511c0f0c840713b6fc39c3b56429e8fc5eb74c572ec2a3ab0b8ccc81fe5`). The schema-191 candidate at `8708adbd` was compared with that baseline using the preserved reports below. The baseline was not rewritten.

| Audit | Exact whole-trace matches | Unrepresentable | Report SHA-256 |
| --- | ---: | ---: | --- |
| Strict schema-191 to 190 projection | 2,889 | 40 | `e83b8a45a529736ffe87e2390d22c3bad4ef04bcd0399a4d8fac868580c6fa48` |
| Opt-in prepared-copy observation-only audit | 2,929 | 0 | `d65882fc9fe9779001322be3287746f9911278d6d2a0f2bcc753c0dd6cbb61a0` |

The saved reports are `~/.cache/fra-mixed-191-fixed.previous-schema.json` and `~/.cache/fra-mixed-191-fixed.prior-prepared-observations-only.json`. A direct comparison of each report's trace entries against the checked-in baseline produced these counts. The strict exceptions report an unrepresentable owned Sphinx or zone-cost state; the corresponding observation-only cases qualify through the narrow prepared-copy audit. The opt-in report labels its scope as final observation bytes only, excluding owned-state, action, rollback and continuation compatibility. Trace observations are represented by historical FNV64 digests; matching digests are evidence for these sampled traces, not a proof of all possible observations or continuations. The strict owned-state guard remains in force.

This distinction matters because the live prepared-copy lifecycle changed. `prepared_copy_can_produce_a_real_stack_to_graveyard_event_before_sba` checks a non-card copy's stack-to-graveyard event, its subsequent cessation at state-based actions, and snapshot restoration. `refute_counters_prepared_copy_before_its_own_discard_choice` exercises a legal Refute interaction: the copy is visible in the graveyard during the suspended discard choice, then ceases after the spell finishes; it also checks both players, restored branches and stale revision rejection. These tests cover particular interactions, not a general prior-schema owned-state migration. The observation-only comparison does not erase the changed intermediate behavior or establish backward compatibility of serialized game state.
