# Named as-entry modes

This shared rule implements a permanent's mandatory bounded printed choice,
including the clan choices used by TDM's five Siege enchantments. It does not
admit those card definitions or prove their remaining printed effects.

Set `CardExtras::entry_choice` to
`EntryChoiceKind::NamedModes(&["Jeskai", "Temur"])`. The order defines stable
source-local indices. Submit `Action::ChooseEntryValue` with
`EntryChoiceValue::NamedMode(index)`. The ordinary entry transaction offers the
choice before the permanent commits, validates its bounds and retains the owned
choice through branches, rollback and restore. Definition checks reject empty
option lists, empty labels and invalid conditional-provider slots or indices.
No label is parsed or used to select behavior.

The existing `EntryChoiceView.kind` publishes the option labels. Visible
`ObjectView.entry_choice` publishes the saved index. A choice belongs to an
exact incarnation and ability acquisition: changing control retains it; a true
zone change clears it; entering as a copy makes a fresh choice. An in-place copy
gets neither another permanent's chosen value nor a new as-entry prompt. A
previous choice remains dormant under that later copy, and becomes active again
if the original ability acquisition returns.

Use these typed providers:

- Put `StaticAbilityFragment::EntryMode(index)` in an immutable static ability
  group to qualify the whole group. The checked allowlist covers
  `StaticModifier`, `KeywordGrant`, `ControlledPermanentKeywords`,
  `PlayLandFromGraveyard` and `SourceEntryModeActivation` fragments. Groups retain
  their ordinary ability-removal and timestamp semantics. Other fragment families
  are rejected until their consumers explicitly integrate group availability and
  have focused rules evidence; the marker cannot silently qualify a raw legacy
  fragment consumer.
- Map `(trigger_slot, mode_index)` in `CardExtras::entry_mode_triggers` for a
  conditional printed trigger. The event freezes the source's active mode before
  simultaneous departures clear live state. The chosen mode is not an
  intervening-if condition: an already queued trigger survives later source
  departure, copying or control changes.
- Use `StaticAbilityFragment::SourceEntryModeActivation { mode, ability }` to
  provide a conditional activated template to its own source. This uses the
  existing layer-six granted-template path, retains ordinary costs/timing/use
  identity, and supplies both actual and speculative mana queries. It does not
  expose an inactive template merely because the card has that array slot.

The mode witness in `ObjectView` is internal and skipped by serialization. The
new `NamedModes` and `NamedMode` enum variants extend the public wire vocabulary;
the set integrator owns the observation-schema bump and prior-schema rejection
checks. Existing chosen values and ordinary no-removal fast paths are preserved.

## Focused evidence

`tests/tdm_entry_modes.rs` uses immutable fixture definitions on existing admitted
identities, keeping all TDM behavior and admission files unchanged. Its scenarios
assert the following independently specified outcomes:

| Rule | Observable result |
| --- | --- |
| Mandatory bounded choice | Entry pauses on the stack, both seats see labels, invalid indices/value kinds and priority passes fail transactionally. |
| Static and triggered modes | Only the chosen own-creature bonus/keywords or end-step draw applies; opponents do not receive the bonus. |
| Conditional activation | Mode zero offers and pays for a life-gain activation; mode one alone offers a tap-green mana ability and funds a green payment. |
| Control changes | The saved choice persists while the permanent's new controller supplies recipients and activation ownership. |
| Graveyard land permission | Only its selected mode permits own-graveyard land play, with ordinary timing/land allowance and immediate source-departure removal. |
| Copying and blink | In-place copies inherit no choice, dormant original choices return with their acquisition, true reentry prompts again, entering copies choose independently. |
| Departure packets | A simultaneous departure uses the conditional trigger present in the frozen source, rather than a cleared live choice. |
| Suppression | Ability removal suppresses the selected providers without clearing the choice; removing suppression restores the same chosen providers. |
| Funding projection | Each prospective entry choice produces the matching live bonus and mana template, without mutating its parent game. |
| Owned contracts | Raw/index children agree, siblings choose independently after parent disposal, wrong-actor and stale-revision failures preserve state, restore reproduces the outcome. |

Run the target through `scripts/run_native_tests.py --test-target tdm_entry_modes`
with the workstream's stable external Cargo target. Validation results belong in
persistent cache reports; a pending or focused pass is not full-set acceptance.
