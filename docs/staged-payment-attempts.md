# Staged payment attempts

RustMage's action contract is `staged_attempts_v1`, with observation schema **227**. This is the default contract for every game. There is no configuration flag or retained guaranteed-completable menu mode. The C ABI remains version 1 and the JSON response envelope remains schema 2; neither identifies the meaning of an action menu. Consumers must check `observation.action_contract` and `observation.schema`.

An action in `legal_actions()`, `LegalActionSpace.choices`, or `observation.actions` is an admissible **next choice**. It does not guarantee that later declarations and payments can finish. The caller chooses an execution; the engine validates its concrete steps against current state and existing declarations. The production admission, numeric-domain and payment paths do not explore alternate future payments, simulate mana abilities to discover future resources, or enumerate complete material subsets to filter the next prompt.

Timing, permissions, actors, source incarnations, target restrictions, selected-object constraints and concrete payments remain rules checks. Cheap current-state checks can reject an immediately unavailable option: an activation requiring two life is unavailable at one life, for example. An undeclared discard-or-life cost still admits its discard route. Declaration-dependent permissions remain important: a library permission requiring mana value four can admit a two-mana X spell whose valid X declarations start at two, independently of available mana.

## Stages and actions

`observation.attempt` identifies the active owned attempt and, for a nested mana ability, its `parent_attempt_id`. The attempt's stage is one of `declare`, `generate_mana`, `pay_costs`, or `recover`. Cost establishment and successful completion use the existing announcement machinery rather than separate extra priority passes.

| Phase | Caller actions | Engine behavior |
| --- | --- | --- |
| Begin | An offered `AnnounceCast`, casting-method variant, `AnnounceActivate`, or acquired/granted ability variant | Checks current admission rules and establishes an owned checkpoint. Existing direct actions remain concrete supplied executions. |
| Declare | Existing mode, optional-cost, X, target and distribution choices | Validates the supplied declaration. It does not prove that the payment can finish. |
| Establish cost | Finish required declarations and choose any required cost/reduction order | Determines and locks the actual total cost. Removing a reducer while paying does not recalculate the locked total. |
| Generate mana | Offered mana abilities, including their nested announcements, payments and mandatory effects | Executes the selected ability. It need not help its parent. Each nested attempt owns its own mana-window state. |
| End mana generation | `EndManaWindow`, or the existing ordered-payment transition such as `BeginOrderedCosts` | Closes this attempt's mana-generation window. The parent cannot reopen it during cost payment. |
| Pay | `ChooseCastCost`, material selections, contribution choices and concrete `PayCosts` allocations | Validates the supplied clause, ordering and joint payment. Whole-group costs commit atomically after their complete selection is valid. |
| Revise an unpaid selection | `ResetCostSelection`, only when offered | Clears the current provisional selection while preserving declarations, locked costs, paid clauses, executed effects and mana-window closure. |
| Complete | The final required concrete payment/continuation | Completes the cast or activation once and closes its attempt. `last_attempt_outcome.outcome` becomes `completed`. |
| Submit an unfinished execution | `SubmitAttempt`, only when offered | Returns an incomplete outcome or starts failed-action recovery from a current, concrete failure witness. It does not search for another payment or choose one automatically. |
| Recover | Offered `ChooseManaRollback { keep }` choices and any required continuations | Retains or reverses the permitted mana events and resumes the owning parent or ordinary game flow. |

The available controls are explicit in `attempt.can_end_mana_window`, `attempt.can_submit` and `attempt.can_reset_selection`, and in the action menu. They are withheld during mandatory entry/replacement/output continuations. A decision belonging to another player still belongs to that player; use `observation.actor`, not the attempted spell's controller, to dispatch the next request.

A menu return is not a Magic priority pass or a new state-based-action boundary. Triggers and required effects follow the existing rules engine. Likewise, the staged interface does not authorize interleaving mana generation with an already-started cost-payment stage.

### Numeric and material choices

`x_choice.intervals` describes allowed declarations under rules and representation constraints. It is not a maximum-affordable-X calculation. Submit a concrete `ChooseX { value }`; large numeric domains use bounded representatives in the indexed menu, so use the raw action API for other represented values. Representation/capacity failures remain transactional API errors rather than proof that a different payment sequence is illegal.

`mana_payment` retains its concrete allocation constraints. A six-vector must satisfy the actual colored, colorless, generic and available-mana requirements; restricted mana and provenance choices remain enforced. The removal of future funding proofs does not permit overspending the pool or paying the wrong color.

`special_mana_payments` supplies the same complete allocation domains for face-up, Room unlock, plot, foretell and resolution payments. Each row names its action (`pay_face_up`, `pay_room_unlock`, `pay_plot`, `pay_foretell` or `pay_resolution_mana`) and `domain`; resolution can offer multiple printed mana alternatives. The field is omitted when empty and is visible only to the actor and players sharing their information. Domains require the current source, selected life/material components and eligible mana to be valid. Menus contain at most 64 representatives per domain; raw actions accept every allocation contained in the domain, including allocations absent from the menu. No domain searches future mana or material choices.

Material menus expose individual current candidates. Their partial selections may strand another unpaid clause. Fixed cardinalities, distinct names, craft subtype assignments, weighted requirements and source identities are checked against the supplied set. The engine does not enumerate all unused-object combinations to decide whether the prefix should be offered.

Conspire illustrates the distinction. `ChooseConspireObject { object }` stores a provisional first member without tapping it. The next prompt offers `PayConspire { objects }` pairs extending that member; the pair taps together when committed. A raw, fully supplied `PayConspire` pair can also be validated directly. Initial pair enumeration is not required.

## Submission, recovery and cancellation

`SubmitAttempt` is not a general cancel or undo command. It is offered after this attempt's mana window is closed, and for an immediately witnessed unsatisfiable target declaration. Submission in a mana/contribution prompt seals the supplied contribution selection; it does not add another convoker, improviser, delver or mana allocation on the caller's behalf.

The result is reported in `last_attempt_outcome`, with the attempt identity, player, source/ability where applicable, original turn, outcome and optional reason:

| Outcome | Meaning |
| --- | --- |
| `completed` | All actual obligations completed through the normal payment path. |
| `incomplete` | Required choices remain and no current failure was certified. The attempt stays open. This is not permission to abandon it. |
| `recovering` | A failed concrete execution has entered a rollback continuation. Further permitted retention/reversal decisions remain. |
| `failed` | Recovery of that attempt finished. A suspended parent may now be active. |

Failure reasons are bounded: `unsatisfied_mana`, `unsatisfied_material`, `unsatisfied_declaration`, or `incomplete_payment`. A mana shortfall is checked against the sealed supplied contributions and actual eligible pool. Material/declaration failure checks use current candidates or simple count/weight bounds; they do not prove that all other cost orders or future executions are impossible. An unknown result remains incomplete.

There is no generic spell/activation draft-cancellation action in this contract. A caller can edit its own unsubmitted proposal before announcing it. After the engine accepts an announcement, use the offered continuation and submission controls. Existing specialized combat/special-action abort controls retain their own scopes. Successful completion closes the attempt immediately and cannot be undone through attempt controls.

`ResetCostSelection` edits only unpaid provisional choices at an owned payment boundary. Depending on the active clause, it clears the selected additional-cost objects/counters/life route, evidence or forage materials, teammates, first conspire member, or mana contributors. Evidence thresholds and selected forage-route requirements remain. It also unseals an explicitly submitted contribution selection so the caller can revise it. It neither reverses a committed clause nor restores spent mana, sacrifices, draws or other effects.

Failed-action recovery is distinct from restoring a generic snapshot. The checkpoint and mana-event receipts preserve the existing rules for irreversible library changes, retained mana events, dependencies, object incarnations, public knowledge and randomness. Retaining an event reuses its recorded outcome; it does not draw again, reroll randomness or dispatch the same event twice. A nested child's failure reverses that child's owned execution and eligible child events; it does not discard the suspended parent's announcement. Completed or retained child receipts become the parent's responsibility when appropriate.

A failed attempt does not blacklist its card or ability. Another permitted sequence can succeed. The engine does not impose a retry quota.

## Observations and ownership

Read the focused `attempt` view together with the existing fields that explain the current choice: `cast_costs`, `announcing_cost`, `additional_cost`, contribution selections, `x_choice`, `mana_payment`, mana restrictions/lots, `mana_announcements` and `mana_rollback`. The attempt view includes its source/ability, parent context, stage, locked cost, mana-window state, sealed-contribution state and available controls. It is a player-facing view, not an exposed checkpoint.

Views are filtered by the observing player's information access. Provisional private choices are not public merely because they are stored in an owned attempt. Cards drawn during an unfinished announcement follow the deferred-draw rules: their identities/characteristics remain hidden until the appropriate completion boundary, with opaque handles exposed through `attempt_draws`. An opaque drawn object can still be used for an unrestricted discard, but cannot be inspected or treated as a known typed/named card to satisfy a characteristic-dependent cost. Retained public knowledge remains known after recovery.

Cloning and snapshots own active announcements, parent frames, provisional selections and rollback continuations. A clone can continue after the original game is dropped. Native `Game::restore` restores rules state while advancing the external revision. JSON `step` and `act` both require the current revision; `apply_index` does so in native Rust. An index from before recovery or restore remains stale. Attempt IDs are branch/session context, not globally unique identifiers across independent clones.

Malformed requests, wrong actors, stale protocol revisions, invalid immediate choices and capacity failures return an error without mutating game state or revision. An accepted `SubmitAttempt` is a real transition, even when its outcome is incomplete or failed: it advances the revision and returns an observation, with no JSON `error`. Python `act`/`step` therefore return these outcomes normally; they raise `EngineError`/`CapacityError` only for rejected requests.

The Python constructor verifies `ACTION_CONTRACT == "staged_attempts_v1"` and the observation schema from `data/observation-schema.txt` against the loaded native library. ABI compatibility alone is insufficient. Policies, encoders, datasets and receipts must also record the action contract; an old guaranteed-completable-menu policy is not behaviorally compatible merely because tensor dimensions match.

## Caller strategy

A simple caller can use present resources to prioritize sensible attempts, without requiring its estimates to be complete:

1. Choose an admissible source/method and provide its declarations. Use a deliberate X value rather than treating the largest indexed representative as an affordability recommendation.
2. Generate mana through concrete offered abilities, recursively finishing any child decisions. Then close the mana window.
3. Choose a cost order, supply complete material groups/contributions, and pay the exact allocation. Track which actions have already changed the game.
4. At an offered submission boundary, submit a stranded concrete sequence. On `incomplete`, supply the missing choices; do not loop on the same command expecting an automatic cancellation.
5. Resolve recovery choices until the failed attempt closes or its parent resumes. Use fresh observations and revisions throughout.
6. Record the semantic sequence and attempt outcome. Avoid repeating an identical unsuccessful sequence from the same information state; try a different sequence or a different ordinary action when control returns.

Any retry/decision cap belongs to the caller's documented protocol. Count primitive decisions separately from completed game actions and completed attempts. Additional microsteps and retries are part of performance costs, not free work to omit from throughput reports.

The repository smoke policy uses a deliberately limited present-resource heuristic. Its preferred X value estimates current pool plus untapped lands minus fixed mana value, saturates at zero and is capped at 20; it favors an offered value closest to that preference. This is a policy preference; the engine's numeric domain has no such gameplay cap. After the acting player's failed attempt on the current turn, the smoke policy skips further paid proposals from priority menus for that turn. The engine retains a private latest outcome per player so alternating priority does not erase retry history; it is not a per-card restriction or a gameplay blacklist. After a failed mana child, it prefers paying with the current pool or closing the parent mana window; this prevents cycles among several unfunded children. The smoke caller also uses current candidate counts to avoid incomplete menace/company declarations and repeated cancel/reselect loops. These are policy preferences; the engine continues to offer the partial choices. Mandatory continuations still need to finish. Record these choices in new benchmark protocols rather than silently applying them to historical receipts.

## Coverage and verification

The shared machinery covers spell announcements and ordinary, granted, acquired, inherited and borrowed activations, including nested mana abilities and craft. Alternate/additional costs reuse the same typed rules and concrete payment routines. Combat taxes, face-up payments, plot/foretell, room unlocks and resolution-time payment windows use their existing action families and current-state funding helpers; their specialized controls should not be mistaken for generic spell cancellation. This change does not implement unsupported card mechanics. In particular, bestow is not implemented by the matched source baseline, so no bestow-specific permission coverage is claimed.

The old future-payment projection routines retained for white-box regression fixtures are test-support code, not a production fallback. Deterministic previews needed to validate an actually selected replacement/entry event remain production rules machinery. Funding/admission search counters and rollback-retention work are separate measurements: selective mana rollback may still inspect dependency/retention choices, and its rules constraints must remain intact.

Use deterministic successful and failed sequences to verify final state, payments, triggers, mana provenance, visibility and RNG, plus raw/index agreement and active-attempt clone/restore. `tests/staged_declaration_choices.rs` covers unfunded craft admission, provisional material choices, declaration-dependent library X permission, linear conspire selection and fixed-life checks. The affordability captures must be replayed by semantic actions with identity checks under this contract; historical indices name a different menu.

Repository validation uses warm build outputs and the guarded runner:

```sh
python3 scripts/link_catalogue.py
python3 scripts/resource_guard.py -- cargo test --test staged_declaration_choices
python3 scripts/validate.py --release --python smoke
```

### Benchmark protocol

Use a new protocol/receipt for this action contract. Freeze the source commit and dirty diff, native-library hash, observation schema, caller/policy and encoder versions, decks, seed/schedule, worker topology, temperature, timeouts, decision limits and retry strategy. Keep all consuming inputs fixed until the jobs terminate. Preserve old receipts and binaries at their original paths; do not overwrite the historical index traces.

First measure deterministic engine fixtures with both successful and failed scripted executions. Record observation/menu and apply latency, payment-search entry/expansion counters, separate rollback work, memory, primitive decisions, and complete-attempt time including recovery. Cover raw and indexed actions, numeric-domain generation and post-step processing. Zero production future-payment search is a structural/counter assertion, not merely a wall-time improvement.

Replay Enigma and Grove's Bounty by semantic identity-checked actions. Enigma's original problem was already fixed before this contract, so that trace alone is insufficient evidence. The handoff identifies remaining cases 279, 374, 706, 763, 781, 877 and 977 in the preserved affordability comparison receipts; those receipts do not contain full executable traces for every case. Capture their actual requests or create minimized equivalent fixtures before claiming coverage.

The repository's small native benchmark can provide a repeatable smoke-policy sample:

```sh
python3 scripts/resource_guard.py -- cargo build --release --bin benchmark
python3 scripts/benchmark.py --games 10000 --repeats 5 --output ~/.cache/rustmage-staged-native-benchmark.json
```

Its fixed freeform deck and native caller exclude model inference, tensor encoding, JSON and Python. Report that scope with its host/compiler metadata and samples; it does not measure the complete Standard pool or collection pipeline. Add the contract, schema, library hash and smoke-policy rules above to the experiment's provenance.

Only after engine correctness and timing pass, compare collection with a caller and policy that understand this contract. For the handoff's last panel, use 48 workers and two inference engines per side, with temperature 0.5 where supported. Report completed games per minute, timeouts, engine CPU, inference calls and wait time, primitive decisions per game, failures/retries and peak memory. A changed action-count limit or retry strategy defines a new protocol. An old policy on changed menus is not a controlled strength comparison. Faster action generation alone does not establish faster completed-game throughput, and no throughput target is presumed achieved.
