# FRA funding: production termination audit

This is a source audit of candidate `1bc88a2b`, not a general funding termination
proof or an Emrakul admission. The supported production catalogue and the
projected payment engine both change over time; a future provider that violates
an assumption below invalidates the argument. Test-only replacement providers
are deliberately distinguished from authored card behavior.

## Evidence

- `ActivatedAbility::mana_production` in `src/cards.rs` accepts only untargeted
  one-instruction mana programs (or the same followed by a reflexive trigger)
  and excludes loyalty costs. A textual scan of direct `ActivatedAbility`
  initializers with `Instruction::AddMana` found nontap, once-per-turn
  definitions and a loyalty definition. The captured intrinsic abilities
  in `src/cards.rs` all tap. This scan is not an executable invariant and does
  not prove that every future or indirectly constructed definition is covered.
- `FundingState::moves` permits granted and intrinsic mana only from an untapped
  resource, and `apply` taps those resources. Ordinary mana activations pay their
  printed costs, mark once-use slots, and consume any required counters or life.
  Borrowed activations preserve their donor's mana cost, once-use and exhaust
  rules; `engine_borrowed_activations.rs` assigns a usage slot per source
  incarnation and ability acquisition. Thranduil, the Elvenking and Territory
  Forge provide the authored graveyard and exile borrowing mechanisms,
  respectively. A loss and later reacquisition of a graveyard ability can create
  a fresh slot, so the initial donor count alone is not a bound.
- The only static fragments used to prevent sacrifice or prevent a battlefield
  permanent's graveyard move have no direct authored definitions at this head.
  Synthetic test providers are not evidence of a supported production card.
  `cost_departures` in `engine_cast_cost_planning.rs` moves mana-cost objects
  to graveyard, exile or library, or leaves them in place if prevention applies.
  The real discard-to-battlefield replacement in `engine_zone_replacements.rs`
  requires an opponent-caused discard from hand and does not apply to a
  battlefield mana-cost departure. This does not rule out future providers.
- Projected funding adds resources by referencing objects already present in the
  game's object array. `visit_linked_returns` reads immutable existing exile
  links and records each returned exiled incarnation as consumed; the projected
  payment path does not create new links. The `tapped = false` assignments in
  funding are departure cleanup or projected entry-tap outcomes. A linked
  return can therefore reset tap, counters and use state, but consumes an
  existing link. This scan did not find object creation in the production
  funding files; entry effects and indirect helpers still require a full audit.
- Source-bound temporary-copy refresh can restore a previous definition and
  its use bits. It only selects a different existing layer when its source's
  exact battlefield incarnation is no longer present; a new incarnation is
  not the same source. Graveyard borrowing refresh can also re-acquire an
  ability when a live source, donor, control or ability origin changes.
- `complete_funding_continuation` takes the final frozen parent frame and
  recurses with the remaining, strictly shorter frame slice. It does not append
  frames during projected continuation. This limits continuation depth; it
  does not alone bound each nested reachability search.

## Candidate finite hierarchy and limits

For a fixed production game, let N be its existing object count and L the
number of initial exact return links. If every projected battlefield return
consumes one of those links, the number of battlefield incarnations and
subsequent departures is finite. Within one incarnation, tap-based mana
activations consume an untapped opportunity. Nontap production must consume a
once/exhaust slot, counters, life, or an irreversible cost object. A return,
copy-layer expiry, control change, or changed graveyard-borrowing acquisition
can reset or replace these opportunities, so those transitions must be counted
as separate epochs. Control and ability refreshes are deterministic functions
of the projected world; the inspected characteristic and borrowing paths do not
read the mana pool directly. Storied acquisition is monotone. The frozen nested
continuation chain strictly shortens when an enclosing payment is completed.

This suggests a finite hierarchy based on remaining initial returns, remaining
object departures, bounded copy-layer expiries and counter/life expenditure,
then tap and once-use opportunities within each resulting acquisition epoch.
It is not yet a verified bound: the catalogue scan is textual, and a complete
proof must enumerate indirect/generated ability definitions and show that every
control, ability-origin or borrowing re-acquisition transition is caused by a
previously bounded event. It must also check all entry and cost helpers, actual
nested-announcement legality, and the practical search bound. Tests using a
synthetic prevention provider intentionally violate the production assumption.
Keep Emrakul gated until a defensible termination guarantee and the rest of its
rules obligations are established; do not use this note as such a guarantee.
