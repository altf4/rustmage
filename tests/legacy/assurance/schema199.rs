//! Schema 199 adds sparse Aetherdrift engine observations and continuations.
//! The prior-198 audit removes only derived entry-count metadata; meaningful
//! histories, permissions, delayed effects and opaque transactions are rejected.
use super::*;
use crate::cards as card_rules;
use serde::Serialize;

const ENTRY_COUNT: &str = "mount_or_vehicle_entries_this_turn";
// This slot was unused in schema 198. Keep the historical wire value explicit:
// the acquired row is removed by the exile cost before its packet resolves.
const NEW_EMBALM_ACTIVATION: u16 = 65_526;
const NEW_FIELDS: &[&str] = &[
    "after_main_combats",
    "delayed_begin_combat",
    "must_attack_this_combat",
    "granted_exhaust_uses",
    "declared_attack_history",
    "delayed_target_events",
    "exertions",
    "chosen_card_name",
    "paid_additional_cost",
    "can_finish_group",
    "additional_creature_types",
    "exhaust_activated_this_turn",
    "own_end_step",
    "to_hand",
    "entry_copy",
    "entry_exile",
    "embalm_abilities",
    "retained_activation",
    "retained_link_origin",
    "ability_origin",
    "embalm",
    "zombie",
];
const NEW_VARIANTS: &[&str] = &[
    "DeployPermanents",
    "FinishAdditionalCostGroup",
    "AnnounceGraveyardAdditionalCosts",
    "AnnounceGrantedMana",
    "AnnounceSelfGrantedMana",
    "ChooseEntryCopy",
    "ChooseEntryExile",
    "FinishEntryExile",
    "AnnounceRetainedCopyAbility",
    "AnnounceEmbalm",
    "GraveyardAdditionalCosts",
    "NthFromTop",
    "NonlandCardName",
    "NoAbilities",
    "BattlefieldIfManaValueAtMostElseHand",
    "FilteredCost",
    "NamedSearch",
    "SacrificeRange",
    "ExileGraveyardRange",
    "DiesOnce",
    "CombatDamageToPlayer",
    "ResolutionRemoval",
    "LookDiscardDraw",
    "DiscardRevealedKinds",
    "ExileRevealedKinds",
    "ArtifactSpellOrActivation",
    "ControlAndCombatLockWhileControlled",
    "card_name",
    "entry_copy",
    "entry_exile",
];

fn check(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::String(text) if NEW_VARIANTS.contains(&text.as_str()) => Err(format!(
            "new Aetherdrift variant {text} has no schema 198 representation"
        )),
        serde_json::Value::Array(rows) => rows.iter().try_for_each(check),
        serde_json::Value::Object(fields) => {
            // Counter receipts on the old acquired nonmana action are new.
            // Empty receipts are omitted by its real serializer; native
            // AnnounceActivate counter_payment is inherited and remains valid.
            if fields
                .get("kind")
                .is_some_and(|kind| kind == "AnnounceSelfGrantedAbility")
                && fields.contains_key("counter_payment")
            {
                return Err(
                    "acquired nonmana counter payment has no schema 198 representation".into(),
                );
            }
            // CardName was already an EntryChoiceValue. Only its new use as an
            // EntryChoiceKind is unrepresentable; old chosen-name values remain.
            if fields.get("kind").is_some_and(|kind| kind == "CardName") {
                return Err("new card-name decision has no schema 198 representation".into());
            }
            // EffectChoiceView already had inspected_cards in schema 198. The
            // new private hand view is distinguished by its actor/victim shape.
            if fields.contains_key("actor")
                && fields.contains_key("victim")
                && fields.contains_key("inspected_cards")
            {
                return Err("private hand inspection has no schema 198 representation".into());
            }
            if fields.get("ActivatedAbility").is_some_and(|kind| {
                kind.get("ability").and_then(serde_json::Value::as_u64)
                    == Some(u64::from(NEW_EMBALM_ACTIVATION))
            }) {
                return Err("acquired embalm activation has no schema 198 representation".into());
            }
            for (key, value) in fields {
                if NEW_FIELDS.contains(&key.as_str()) || NEW_VARIANTS.contains(&key.as_str()) {
                    return Err(format!(
                        "new Aetherdrift field {key} has no schema 198 representation"
                    ));
                }
                if key == ENTRY_COUNT {
                    if value.as_u64().is_none() {
                        return Err("derived entry count must be an unsigned integer".into());
                    }
                } else {
                    check(value)?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
fn wire_supported(value: &impl Serialize) -> bool {
    serde_json::to_value(value).is_ok_and(|value| check(&value).is_ok())
}

// The action spelling is inherited, but an intrinsic conditional graveyard
// permission is new. Preserve an independently sufficient old exact permission.
fn uses_conditional_graveyard_cast(game: &Game, action: &Action) -> bool {
    let card = match *action {
        Action::AnnounceCast { card }
        | Action::Cast { card, .. }
        | Action::BeginCast { card, .. }
        | Action::AnnounceAdventure { card }
        | Action::AnnounceOmen { card }
        | Action::AnnounceBack { card }
        | Action::AnnounceDisguise { card }
        | Action::AnnounceFlashSurcharge { card }
        | Action::AnnounceFree { card, .. }
        | Action::AnnounceSneak {
            card,
            permission: None,
        } => card,
        _ => return false,
    };
    let Some(player) = game.actor() else {
        return false;
    };
    if !game.has_graveyard_cast(player, card, card_rules::GraveyardCastKind::Cast) {
        return false;
    }
    !game
        .state
        .graveyard_cast_permissions
        .as_ref()
        .is_some_and(|rows| {
            rows.iter().any(|row| {
                row.player == player
                    && row.object == game.object_ref(card)
                    && matches!(
                        row.kind,
                        card_rules::GraveyardCastKind::Cast | card_rules::GraveyardCastKind::Play
                    )
                    && row.expires_turn >= game.state.turn
            })
        })
}

// A zero vector can now mean "choose the output after paying costs".
// Compare with the old pre-action domain on the actual activating object:
// borrowed definitions supply rules, while the borrower supplies quantities.
pub(super) fn prior_schema199_actions_supported(game: &Game, actions: &[Action]) -> bool {
    use rustmage_core::engine::ability_world::CharacteristicWorld;
    fn old_domain_contains_zero(game: &Game, source: CardId, recipe: ManaProduction) -> bool {
        // Avoid enumerating allocations: every vector in these domains has the
        // same total, and in_world resolves all characteristic-dependent recipes.
        match recipe.in_world(game, source) {
            ManaProduction::Fixed(mana) => mana == [0; 6],
            ManaProduction::Choice { mask, amount } => amount == 0 && mask & 63 != 0,
            ManaProduction::Combination { amount, .. } => amount == 0,
            _ => false,
        }
    }
    actions.iter().all(|action| {
        if !wire_supported(action)
            || uses_conditional_graveyard_cast(game, action)
            || (matches!(action, Action::ChooseMana { .. })
                && game
                    .state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.mana_output.is_some()))
        {
            return false;
        }
        let recipe = match *action {
            Action::AnnounceMana {
                card,
                ability,
                production,
                ..
            }
            | Action::ActivateMana {
                card,
                ability,
                production,
                ..
            } if production == [0; 6] => game.state.objects[card as usize]
                .card
                .activation(ability)
                .mana_production()
                .map(|recipe| (card, recipe)),
            Action::AnnounceBorrowedMana {
                card,
                exiled,
                ability,
                production,
                ..
            }
            | Action::ActivateBorrowedMana {
                card,
                exiled,
                ability,
                production,
                ..
            } if production == [0; 6] => game.state.objects[exiled.card as usize]
                .card
                .activation(ability)
                .mana_production()
                .map(|recipe| (card, recipe)),
            Action::AnnounceGraveyardBorrowedMana {
                card,
                graveyard,
                ability,
                production,
                ..
            }
            | Action::ActivateGraveyardBorrowedMana {
                card,
                graveyard,
                ability,
                production,
                ..
            } if production == [0; 6] => game.state.objects[graveyard.card as usize]
                .card
                .activation(ability)
                .mana_production()
                .map(|recipe| (card, recipe)),
            Action::TapGrantedMana {
                card,
                grantor,
                production,
            } if production == [0; 6] => CharacteristicWorld::granted_mana(game, card)
                .into_iter()
                .find(|grant| grant.source == grantor)
                .map(|grant| (card, grant.production)),
            _ => None,
        };
        recipe.is_none_or(|(source, recipe)| old_domain_contains_zero(game, source, recipe))
    })
}

/// Remove only a validated numeric metadata member, preserving every other byte
/// and its struct-field order. Value reserialization would reorder old members.
fn remove_entry_count(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' {
                i += 1;
                break;
            }
            i += 1;
        }
        let end = i;
        let mut colon = end;
        while bytes.get(colon).is_some_and(u8::is_ascii_whitespace) {
            colon += 1;
        }
        if bytes.get(colon) != Some(&b':')
            || serde_json::from_str::<String>(&input[start..end])
                .ok()
                .as_deref()
                != Some(ENTRY_COUNT)
        {
            continue;
        }
        let mut number = colon + 1;
        while bytes.get(number).is_some_and(u8::is_ascii_whitespace) {
            number += 1;
        }
        let mut after = number;
        while bytes.get(after).is_some_and(u8::is_ascii_digit) {
            after += 1;
        }
        let mut prior = start;
        while prior > 0 && bytes[prior - 1].is_ascii_whitespace() {
            prior -= 1;
        }
        let mut next = after;
        while bytes.get(next).is_some_and(u8::is_ascii_whitespace) {
            next += 1;
        }
        if prior > 0 && bytes[prior - 1] == b',' {
            spans.push((prior - 1, after));
        } else if bytes.get(next) == Some(&b',') {
            spans.push((start, next + 1));
        } else {
            spans.push((start, after));
        }
        i = after;
    }
    let mut out = String::with_capacity(input.len());
    let mut start = 0;
    for (from, to) in spans {
        out.push_str(&input[start..from]);
        start = to;
    }
    out.push_str(&input[start..]);
    out
}
pub(super) fn project_prior_schema199(input: &str) -> Result<String, String> {
    check(&serde_json::from_str(input).map_err(|error| error.to_string())?)?;
    Ok(remove_entry_count(input))
}

fn cast_filter_supported(filter: card_rules::CastFilter) -> bool {
    use card_rules::CastFilter;
    match filter {
        CastFilter::NotOwnedByCaster | CastFilter::CasterOffTurn { .. } => false,
        CastFilter::AnyOf(filters) => filters.iter().copied().all(cast_filter_supported),
        CastFilter::Colors { filter, .. }
        | CastFilter::ManaSymbols { filter, .. }
        | CastFilter::Targeted { filter, .. }
        | CastFilter::ManaValueMaximum { filter, .. }
        | CastFilter::ManaValueMinimum { filter, .. } => cast_filter_supported(*filter),
        _ => true,
    }
}
fn trigger_supported(condition: TriggerCondition) -> bool {
    match condition {
        TriggerCondition::ExhaustAbilityActivated
        | TriggerCondition::DelayedExhaustAbilityActivated
        | TriggerCondition::DelayedTargetEvent
        | TriggerCondition::CardsLeaveGraveyardKinds(_)
        | TriggerCondition::CardsExiledFromBattlefieldOrGraveyard
        | TriggerCondition::SelfCrewsOrSaddles
        | TriggerCondition::CreaturesAttackYou => false,
        TriggerCondition::AtStep { guard, .. } => guard_supported(guard),
        TriggerCondition::When { guard, event }
        | TriggerCondition::InterveningIf { guard, event } => {
            guard_supported(guard) && trigger_supported(*event)
        }
        TriggerCondition::InZone { event, .. }
        | TriggerCondition::Granted(event)
        | TriggerCondition::OncePerTurn(event)
        | TriggerCondition::DuringOpponentsTurn(event)
        | TriggerCondition::CastFromHand(event)
        | TriggerCondition::CastFromExile(event)
        | TriggerCondition::TreasureManaSpent(event)
        | TriggerCondition::ManaSpent { event, .. } => trigger_supported(*event),
        TriggerCondition::SpellCast { filter, .. } => cast_filter_supported(filter),
        _ => true,
    }
}
fn stat_supported(value: StatValue) -> bool {
    match value {
        StatValue::MountOrVehicleEntriesThisTurn
        | StatValue::SourceTotalCounters
        | StatValue::Parity { .. }
        | StatValue::ControllerSpeed
        | StatValue::OpponentControlsMoreLands => false,
        StatValue::Not(value)
        | StatValue::AtLeast { value, .. }
        | StatValue::GroupsOf { value, .. } => stat_supported(*value),
        StatValue::Sum(values) => values.iter().copied().all(stat_supported),
        _ => true,
    }
}
fn amount_supported(value: AmountValue) -> bool {
    match value {
        AmountValue::SourceTotalCounters
        | AmountValue::DiscardEventAmount
        | AmountValue::PaidAdditionalCostCount
        | AmountValue::PaidAdditionalCostSelectionCount => false,
        AmountValue::Multiple { value, .. } | AmountValue::Divided { value, .. } => {
            amount_supported(*value)
        }
        AmountValue::Sum(values) => values.iter().copied().all(amount_supported),
        AmountValue::Stat(value) => stat_supported(value),
        _ => true,
    }
}
fn guard_supported(guard: card_rules::StepGuard) -> bool {
    match guard {
        card_rules::StepGuard::YourMainPhase
        | card_rules::StepGuard::DeclaredAttackSubtypesAll(_) => false,
        card_rules::StepGuard::Expression { value, .. } => stat_supported(value),
        _ => true,
    }
}
fn target_supported(rule: TargetRule) -> bool {
    match rule {
        TargetRule::YourSpell | TargetRule::LinkedExileCreature { .. } => false,
        TargetRule::Union(rules) => rules.iter().copied().all(target_supported),
        TargetRule::Grouped { base, .. } => target_supported(*base),
        TargetRule::ManaValueExactly { base, amount, .. }
        | TargetRule::ManaValueAtMost { base, amount, .. } => {
            target_supported(*base) && amount_supported(amount)
        }
        TargetRule::Filtered { base, predicate } => {
            *predicate != TargetPredicate::NoAbilities && target_supported(*base)
        }
        _ => true,
    }
}
fn plan_supported(plan: TargetPlan) -> bool {
    match plan {
        TargetPlan::Single(spec) => target_supported(spec.rule),
        TargetPlan::Groups(groups) => groups.iter().all(|spec| target_supported(spec.rule)),
    }
}
fn resolved_stat_supported(value: card_rules::ResolvedStatValue) -> bool {
    match value {
        card_rules::ResolvedStatValue::SourcePower => false,
        card_rules::ResolvedStatValue::Stat(value)
        | card_rules::ResolvedStatValue::Threshold { value, .. } => stat_supported(value),
        _ => true,
    }
}
fn player_supported(player: card_rules::PlayerSet) -> bool {
    !matches!(player, card_rules::PlayerSet::SpeedBelow(_))
}
fn instruction_players_supported(instruction: Instruction) -> bool {
    match instruction {
        Instruction::EnsureGoblinArmyFor(player)
        | Instruction::LoseLifeTargetCardManaValue(player)
        | Instruction::LoseHalfLife(player)
        | Instruction::LoseHalfLifeRoundedUp(player)
        | Instruction::LoseGame(player)
        | Instruction::LoseGainedLife(player)
        | Instruction::DamageDiscardAmount(player)
        | Instruction::DiscardHand(player)
        | Instruction::DiscardHandRememberCount(player)
        | Instruction::ExileAttackingCreatures(player)
        | Instruction::RevealUntilLandToGraveyard(player)
        | Instruction::UntapPlayerCreatures(player)
        | Instruction::ExileGraveyard(player)
        | Instruction::DrawPowerOfTwoChosenX(player)
        | Instruction::DiscardScaled(player, _)
        | Instruction::AddPoison(player, _)
        | Instruction::Mill(player, _)
        | Instruction::DamagePlayers(player, _)
        | Instruction::EventObjectDamagePlayers(player, _)
        | Instruction::DamagePlayersScaled(player, _)
        | Instruction::Draw(player, _)
        | Instruction::DrawScaled(player, _)
        | Instruction::Discard(player, _)
        | Instruction::DiscardRandom(player, _)
        | Instruction::GainLife(player, _)
        | Instruction::GainLifeScaled(player, _)
        | Instruction::LoseLife(player, _)
        | Instruction::LoseLifeScaled(player, _)
        | Instruction::AmassGoblinsScaled { player, .. }
        | Instruction::CreateCopyTokenForPlayer { player, .. }
        | Instruction::MayPlayer { player, .. }
        | Instruction::GainControl {
            controller: player, ..
        }
        | Instruction::ExileTopLibrary {
            players: player, ..
        }
        | Instruction::DiscardAnyDraw {
            players: player, ..
        }
        | Instruction::ReturnGraveyardCreatures {
            players: player, ..
        }
        | Instruction::DiscardThenPutLand {
            players: player, ..
        }
        | Instruction::PutPlayerCountersWithSelf {
            players: player, ..
        }
        | Instruction::ChooseObjects {
            players: player, ..
        }
        | Instruction::CreateTokensFor {
            players: player, ..
        }
        | Instruction::PutPlayerCreatureCounters {
            players: player, ..
        }
        | Instruction::GrowPlayerCreatures {
            players: player, ..
        } => player_supported(player),
        Instruction::ReturnBattlefield { players, .. }
        | Instruction::DamageBattlefield { players, .. }
        | Instruction::DamageBattlefieldExceptSubtype { players, .. } => {
            players.is_none_or(player_supported)
        }
        // The other player-bearing variants are new instructions rejected below.
        _ => true,
    }
}
fn program_supported(program: &[Instruction]) -> bool {
    program.iter().all(|instruction| {
        instruction_players_supported(*instruction) && match *instruction {
            Instruction::DeployHandPermanents
            | Instruction::GrowFilteredScaled { .. }
            | Instruction::GrantEmbalmTargets
            | Instruction::CreateEmbalmedSourceCopy
            | Instruction::CopySourceFromLinkedExile { .. }
            | Instruction::ExertCost
            | Instruction::ExileLibraryTopFaceDownPlay { .. }
            | Instruction::ChooseCardName { .. }
            | Instruction::ExileTargetNamedCards { .. }
            | Instruction::DelayTargetEvent { .. }
            | Instruction::CopyNextExhaustAbility { .. }
            | Instruction::DiscardUpToDraw { .. }
            | Instruction::OpponentSacrificeIfAbleElseDiscard { .. }
            | Instruction::DestroyTargetsAtMostManaValue(..)
            | Instruction::DestroyTargetsThenSourceCountersIfOwnCreature { .. }
            | Instruction::AdditionalCombatsAfterCurrentMain(..)
            | Instruction::ScheduleBeginCombatThisTurn { .. }
            | Instruction::UntapCreaturesThatAttackedThisTurn
            | Instruction::UntapAttached
            | Instruction::MarkSaddled(..)
            | Instruction::IfSubjectSubtype { .. }
            | Instruction::DecreaseLeadingSpeed { .. }
            | Instruction::LookTargetHandMayDiscardThenDraw
            | Instruction::RevealTargetHandChoose { .. }
            | Instruction::AddManaChosenScaled { .. }
            | Instruction::IfSacrificedVehicle { .. }
            | Instruction::AnimateSubject { .. }
            | Instruction::SearchScaled { .. }
            | Instruction::SearchLibraryAndOrGraveyardToHand(..)
            | Instruction::CaptureEntryResult
            | Instruction::AnimateEntryResultIfSubtype { .. }
            | Instruction::PutEntryResultCountersIfKinds { .. }
            | Instruction::GrantEntryResultKeywords { .. }
            | Instruction::ScheduleEntryResultExile { .. }
            | Instruction::ReturnGraveyardWithDelayedSacrifice(..)
            | Instruction::AnimatePermanentGroup { .. }
            | Instruction::IfAmountParity { .. }
            | Instruction::UntapPermanentGroup(..)
            | Instruction::GrantCounterBearingCreatures(..)
            | Instruction::RemovePermanentGroupKeywords { .. }
            | Instruction::SetPermanentGroupBaseStats { .. }
            | Instruction::OwnerLibraryNthFromTop(..)
            | Instruction::GainControlAndPreventCombatWhileSourceControlled
            | Instruction::CreateTokensAttackThisCombat { .. } => false,
            Instruction::Search(spec) | Instruction::SearchDifferentNames(spec) => {
                wire_supported(&spec)
            }
            Instruction::ChooseHalf { kind } | Instruction::ChooseObjects { kind, .. } => {
                wire_supported(&kind)
            }
            Instruction::PutHandFiltered { filter, .. }
            | Instruction::PutHandAndAttachEquipment { filter }
            | Instruction::SearchHandAndOrLibrary(filter)
            | Instruction::SearchLibraryAndGraveyardWithXCounters { filter, .. } => {
                wire_supported(&filter)
            }
            Instruction::CopyTargetSpell { exceptions, .. }
            | Instruction::CreateCopyTokenWithExceptions { exceptions, .. }
            | Instruction::CreateCounteredCopyToken { exceptions, .. } => {
                wire_supported(&exceptions)
            }
            Instruction::SetResolvedBaseStats {
                power, toughness, ..
            } => {
                power.is_none_or(resolved_stat_supported)
                    && toughness.is_none_or(resolved_stat_supported)
            }
            Instruction::GrowResolved { value, .. } => resolved_stat_supported(value),
            Instruction::SelectLibraryTo(effect)
            | Instruction::SelectLibraryThenMayTransform(effect) => !matches!(
                effect.look,
                card_rules::LibraryLook::UntilKindsCount { .. }
            ) && match effect.look {
                card_rules::LibraryLook::Amount(value) => amount_supported(value),
                _ => true,
            } && effect
                .maximum_mana_value
                .is_none_or(amount_supported)
                && match effect.destination {
                    card_rules::LibraryPickDestination::BattlefieldIfManaValueAtMostElseHand(
                        value,
                    ) => amount_supported(value),
                    _ => true,
                },
            Instruction::EmpowerJaceScaled(value)
            | Instruction::AmassGoblinsScaled { amount: value, .. }
            | Instruction::ExileTopPlayScaled { amount: value, .. }
            | Instruction::ExileTopLibrary { amount: value, .. }
            | Instruction::PutHandPermanent { maximum: value, .. }
            | Instruction::DamageForExcess(value)
            | Instruction::ExileTargetLibraryPlayLife { amount: value }
            | Instruction::ReturnGraveyardManaValue { value, .. }
            | Instruction::DestroyManaValue { maximum: value, .. }
            | Instruction::DamagePlayersScaled(_, value)
            | Instruction::RemoveAbilitiesThenDestroyCreaturesManaAtMost(value)
            | Instruction::Proliferate(value)
            | Instruction::CreateTokensWithTemporaryKeywords { amount: value, .. }
            | Instruction::UpToTargets(value)
            | Instruction::IfAmountAtLeast { value, .. }
            | Instruction::PutLandCards { maximum: value, .. }
            | Instruction::DiscardScaled(_, value)
            | Instruction::AddCastEntryCounters { amount: value, .. }
            | Instruction::DrawThenDiscardIfDrew { amount: value, .. }
            | Instruction::SetCreatureForm { base: value, .. }
            | Instruction::MoveCountersFromSource { amount: value, .. }
            | Instruction::RevealTargetHandSubsetDiscard(value)
            | Instruction::DrawScaled(_, value)
            | Instruction::GainLifeScaled(_, value)
            | Instruction::LoseLifeScaled(_, value)
            | Instruction::DrainLifeScaled(value)
            | Instruction::LoseOpponentGainSame(value)
            | Instruction::DamageScaled(value)
            | Instruction::AddColorlessScaled(value)
            | Instruction::AddManaScaled { amount: value, .. }
            | Instruction::AddCombatRedScaled(value)
            | Instruction::Mobilize { amount: value, .. }
            | Instruction::CounterUnlessPayScaled { amount: value, .. }
            | Instruction::PutCountersScaled { amount: value, .. }
            | Instruction::PutPermanentCountersScaled { amount: value, .. }
            | Instruction::GrowScaled { amount: value, .. }
            | Instruction::GrowCreaturesScaled { amount: value, .. }
            | Instruction::GrowPermanentGroupScaled { amount: value, .. }
            | Instruction::CreateTokensScaled { amount: value, .. }
            | Instruction::CreateTokensWithCounters {
                counters: value, ..
            } => amount_supported(value),
            Instruction::Modal(modes)
            | Instruction::ModalOnce(modes)
            | Instruction::ModalOncePerTurn(modes)
            | Instruction::ModalUpToX { modes, .. } => modes
                .iter()
                .all(|mode| plan_program_supported(mode.targets, mode.program)),
            _ => true,
        }
    })
}
// Exchange formerly consumed only the active group. Preserving all roles is
// meaningful only when a selected declaration has more than one group; retain
// old single-occurrence exchanges (including a two-target Single specification).
// The destruction/reward opcode was already new to schema199 and remains
// rejected by program_supported independently of its group count.
fn plan_program_supported(plan: TargetPlan, program: &[Instruction]) -> bool {
    plan_supported(plan)
        && program_supported(program)
        && !(matches!(plan, TargetPlan::Groups(groups) if groups.len() > 1)
            && program
                .iter()
                .any(|instruction| matches!(instruction, Instruction::ExchangeControlTargets)))
}

// The old noninstant capture path admitted a prepared-copy consumer, and
// repeated-all-spells packets already retained any spell. The newly broadened
// single-copy consumer needs schema199 only when neither old route supplied the
// same amount-one packet. Keep mixed prepared programs eligible.
fn prior_noninstant_cast_copy_supported(
    condition: TriggerCondition,
    program: &[Instruction],
    prepared: bool,
) -> bool {
    !program.iter().any(|instruction| {
        matches!(
            instruction,
            Instruction::CopyCastSpell {
                prior_spells: false
            }
        )
    }) || (prepared
        && program
            .iter()
            .any(|instruction| matches!(instruction, Instruction::CopyPreparedCastSpell)))
        || condition
            == TriggerCondition::Granted(&TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: card_rules::CastFilter::Any,
                ordinal: None,
            })
}
fn packet_supported(item: &SpellView) -> bool {
    if matches!(
        item.kind,
        StackKind::ActivatedAbility {
            ability: NEW_EMBALM_ACTIVATION
        }
    ) || !wire_supported(item)
    {
        return false;
    }
    let trigger = match item.kind {
        StackKind::TriggeredAbility { ability } => item.source.and_then(|_| {
            item.trigger_definition()
                .triggers()
                .get(usize::from(ability))
        }),
        _ => None,
    };
    // Some inherited event packets use a legacy synthetic trigger index, without
    // a typed TriggeredAbility. Never index their absent modes/program entry.
    let legacy_trigger =
        matches!(item.kind, StackKind::TriggeredAbility { .. }) && trigger.is_none();
    (legacy_trigger
        || item
            .program()
            .is_none_or(|(plan, program)| plan_program_supported(plan, program)))
        && trigger.is_none_or(|trigger| trigger_supported(trigger.condition))
        && trigger.is_none_or(|trigger| {
            item.copy_event.as_ref().is_none_or(|event| {
                event.spell.kind != StackKind::Spell
                    || event.spell.source.is_none_or(|source| {
                        card_rules::CastFilter::InstantSorcery.matches(source.card)
                            || item.program().is_none_or(|(_, program)| {
                                prior_noninstant_cast_copy_supported(
                                    trigger.condition,
                                    program,
                                    source.card.is_prepared_face(),
                                )
                            })
                    })
            })
        })
        && match item.kind {
            StackKind::ActivatedAbility { ability } => item.source.is_none_or(|source| {
                source
                    .card
                    .activation(ability)
                    .cost
                    .additional
                    .is_none_or(|spec| wire_supported(&spec.kind))
            }),
            _ => true,
        }
        && item
            .copy_event
            .as_ref()
            .is_none_or(|event| packet_supported(&event.spell))
}

/// Opaque transactions can own nonpublic result captures, frozen authorization,
/// copied entry decisions, mana ancestors or rollback checkpoints. Do not infer
/// eligibility from a matching public view after erasing those continuations.
pub(super) fn prior_schema199_state_supported(state: &State) -> bool {
    if matches!(
        state.pending,
        Pending::EntryCopy { .. } | Pending::EntryExile { .. } | Pending::CardName { .. }
    ) || state
        .embalm_abilities
        .as_ref()
        .is_some_and(|rows| !rows.is_empty())
        || state
            .players
            .iter()
            .any(|player| player.turn_facts.exhaust_activated_this_turn)
        || state
            .exertions
            .as_ref()
            .is_some_and(|rows| !rows.is_empty())
        || state
            .granted_exhaust_uses
            .as_ref()
            .is_some_and(|rows| !rows.is_empty())
        || state.declared_attack_history.is_some()
        || state.delayed_target_events.is_some()
        || state.combat_rules.as_ref().is_some_and(|rules| {
            rules.after_main.is_some()
                || rules.begin_combat_this_turn.is_some()
                || !rules.must_attack_this_combat.is_empty()
        })
        || state
            .library_top
            .as_ref()
            .is_some_and(|top| top.private_known_top.is_some())
        || state.resolving.is_some()
        || state.announcing.is_some()
        || state.activation_payment.is_some()
        || state.mana_announcement_stack.is_some()
        || state.cast_rollback.is_some()
        || state.replacements.is_some()
        || state.entry_batch.is_some()
        || state
            .zone_rules
            .as_ref()
            .is_some_and(|rules| rules.has_checkpoint())
        || state
            .entry_values
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.origin.is_some()))
        || state.saddle_payment.is_some()
        || state
            .crew
            .as_ref()
            .is_some_and(|crew| crew.payment.is_some())
        || state.linked_choices.as_ref().is_some_and(|choices| {
            choices.has_opaque_interaction()
                || choices.links.iter().any(|row| row.ability_origin.is_some())
        })
        || state.roles.as_ref().is_some_and(|roles| {
            roles
                .source_linked_effects
                .iter()
                .any(|row| !wire_supported(row))
                || roles.keyword_effects.iter().any(|row| !wire_supported(row))
        })
        || state
            .delayed_returns
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| !wire_supported(&row.view)))
        || state
            .copy_recipes
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| !wire_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.original_recipe
                    .as_ref()
                    .is_some_and(|recipe| !wire_supported(recipe))
                    || row.layers.iter().any(|layer| !wire_supported(&layer.value))
            })
        })
        || state
            .hits
            .iter()
            .any(|hit| !wire_supported(&hit.source.facts))
        || state
            .damage_choices
            .iter()
            .any(|hit| !wire_supported(&hit.source.facts))
    {
        return false;
    }
    if state.mana_lots.as_ref().is_some_and(|mana| {
        mana.trace.is_some()
            || mana.replay.is_some()
            || mana.choice.is_some()
            || mana.prepared.is_some()
            || mana
                .pools
                .iter()
                .flatten()
                .any(|lot| !wire_supported(&lot.quality))
            || mana
                .effect_quality
                .as_ref()
                .is_some_and(|quality| !wire_supported(quality))
            || mana
                .announcement_quality
                .as_ref()
                .is_some_and(|(_, quality)| !wire_supported(quality))
    }) {
        return false;
    }
    state
        .stack
        .iter()
        .chain(&state.pending_triggers)
        .chain(&state.ordering_triggers)
        .chain(
            state
                .delayed_artifact_copies
                .as_deref()
                .into_iter()
                .flatten(),
        )
        .chain(
            state
                .delayed_attack_this_turn
                .as_deref()
                .into_iter()
                .flatten(),
        )
        .chain(
            state
                .delayed_opponent_attack_until_next_turn
                .as_deref()
                .into_iter()
                .flatten(),
        )
        .chain(state.delayed_main_mana.as_deref().into_iter().flatten())
        .chain(state.delayed_upkeep_tokens.as_deref().into_iter().flatten())
        .chain(
            state
                .stack_copy_data
                .as_ref()
                .into_iter()
                .flat_map(|copies| copies.packets()),
        )
        .chain(
            state
                .paradigm
                .as_ref()
                .into_iter()
                .flat_map(|paradigm| paradigm.packets()),
        )
        .chain(
            state
                .delayed_returns
                .as_deref()
                .into_iter()
                .flatten()
                .map(|row| &row.item),
        )
        .chain(
            state
                .earthbend_returns
                .as_deref()
                .into_iter()
                .flatten()
                .map(|row| &row.item),
        )
        .chain(
            state
                .manifest
                .as_ref()
                .into_iter()
                .flat_map(|manifest| manifest.deaths.iter().map(|death| &death.source)),
        )
        .all(packet_supported)
}

#[test]
fn prior_198_preserves_order_and_only_projects_the_derived_entry_count() {
    let old = r#"[{"schema":198,"players":[{"life":20,"mana":[0,0,0,0,0,0]}],"old":true}]"#;
    assert_eq!(project_prior_schema199(old).unwrap(), old);
    let current = r#"[{"schema":198,"players":[{"life":20,"mount_or_vehicle_entries_this_turn":17,"mana":[0,0,0,0,0,0]}],"old":true}]"#;
    assert_eq!(project_prior_schema199(current).unwrap(), old);
    for (current, old) in [
        (
            r#"{"mount_or_vehicle_entries_this_turn":1,"life":20}"#,
            r#"{"life":20}"#,
        ),
        (
            r#"{"life":20,"mount_or_vehicle_entries_this_turn":18446744073709551615}"#,
            r#"{"life":20}"#,
        ),
        (r#"{"mount_or_vehicle_entries_this_turn":0}"#, "{}"),
    ] {
        assert_eq!(project_prior_schema199(current).unwrap(), old);
    }
    for bad in ["-1", "true", "1.5", "null", "18446744073709551616"] {
        assert!(project_prior_schema199(&format!(r#"{{"{ENTRY_COUNT}":{bad}}}"#)).is_err());
    }
    // These members already existed and must not be mistaken for new contexts.
    let inherited = r#"{"value":{"CardName":"Forest"},"kind":{"CoverUpSearch":{"owner":"P1","name":"Forest"}},"inspected_cards":[],"old":true}"#;
    assert_eq!(project_prior_schema199(inherited).unwrap(), inherited);
}

#[test]
fn prior_198_rejects_every_new_public_wire_family() {
    for field in NEW_FIELDS {
        assert!(
            project_prior_schema199(&format!(r#"{{"{field}":null}}"#)).is_err(),
            "{field}"
        );
    }
    for variant in NEW_VARIANTS {
        for value in [
            format!(r#"{{"action":"{variant}"}}"#),
            format!(r#"{{"kind":{{"{variant}":{{}}}}}}"#),
        ] {
            assert!(project_prior_schema199(&value).is_err(), "{value}");
        }
    }
    assert!(project_prior_schema199(r#"{"kind":"CardName"}"#).is_err());
    assert!(
        project_prior_schema199(r#"{"actor":"P0","victim":"P1","inspected_cards":[]}"#).is_err()
    );
}

#[test]
fn prior_198_allows_derived_history_in_real_views_without_erasing_exhaust_or_exert() {
    let mut game = scenario(Player::P0);
    let old = serde_json::to_string(&[game.observe(Player::P0), game.observe(Player::P1)]).unwrap();
    game.state.players[1].mount_or_vehicle_entries_this_turn = 7;
    assert!(prior_schema199_state_supported(&game.state));
    let current =
        serde_json::to_string(&[game.observe(Player::P0), game.observe(Player::P1)]).unwrap();
    assert_eq!(project_prior_schema199(&current).unwrap(), old);
    let saved = game.snapshot();
    game.state.players[0].turn_facts.exhaust_activated_this_turn = true;
    assert!(!prior_schema199_state_supported(&game.state));
    let mut branch = game.clone();
    drop(game);
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&saved).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
    let card = put(&mut branch, Player::P0, "Grizzly Bears", Zone::Battlefield);
    branch.exert_source(branch.object_ref(card), Player::P0);
    assert!(!prior_schema199_state_supported(&branch.state));
}

#[test]
fn prior_198_rejects_private_top_knowledge_and_source_independent_delayed_history() {
    let mut game = scenario(Player::P0);
    let saved = game.snapshot();
    let top = game.top_library_card(Player::P1).unwrap();
    game.remember_private_library_top(Player::P1, Player::P0, game.object_ref(top));
    assert!(!prior_schema199_state_supported(&game.state));
    game.restore(&saved).unwrap();
    game.state.declared_attack_history =
        Some(Box::new(declared_attack_history::DeclaredAttackHistory {
            queried: vec!["Pirate", "Vehicle"],
            rows: Vec::new(),
        }));
    assert!(!prior_schema199_state_supported(&game.state));
    game.restore(&saved).unwrap();
    game.state.combat_rules.get_or_insert_default().after_main = Some(MainPhaseCombatView {
        remaining: 1,
        resume: Step::End,
    });
    assert!(!prior_schema199_state_supported(&game.state));
}

#[test]
fn prior_198_checks_owned_packet_facts_even_outside_public_stack() {
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Shock", Zone::Hand);
    cast(&mut game, card, Some(Target::Player(Player::P1)));
    assert!(prior_schema199_state_supported(&game.state));
    let mut packet = game.state.stack.pop().unwrap();
    packet.paid_additional_cost = Some(Box::new(PaidAdditionalCostView {
        count: 1,
        selected: 1,
        sacrificed_vehicle: true,
    }));
    game.state.delayed_artifact_copies = Some(Box::new(vec![packet]));
    assert!(!prior_schema199_state_supported(&game.state));
}

#[test]
fn prior_198_rejects_copied_choice_origins_after_the_entry_phase_ends() {
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.set_entry_value(
        game.object_ref(card),
        EntryChoiceValue::CardName(CardName::from_name("Forest").unwrap()),
    );
    assert!(prior_schema199_state_supported(&game.state));
    let saved = game.snapshot();
    game.state.entry_values.as_mut().unwrap()[0].origin = Some(17);
    let mut branch = game.clone();
    drop(game);
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&saved).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
}

#[test]
fn prior_198_preserves_neutral_copy_bytes_and_rejects_final_copy_properties() {
    let recipe = CopyRecipe::new(Card::GrizzlyBears);
    for bytes in [
        serde_json::to_string(&recipe).unwrap(),
        serde_json::to_string(&CopyTypes::default()).unwrap(),
    ] {
        assert_eq!(project_prior_schema199(&bytes).unwrap(), bytes);
        for field in ["embalm", "zombie", "retained_activation"] {
            assert!(!bytes.contains(&format!("\"{field}\"")));
        }
    }
    for exceptions in [
        CopyExceptions {
            embalm: true,
            ..CopyExceptions::NONE
        },
        CopyExceptions {
            retained_activation: Some(card_rules::RetainedCopyActivation {
                definition: Card::GrizzlyBears,
                ability: 0,
            }),
            ..CopyExceptions::NONE
        },
    ] {
        assert!(!wire_supported(&recipe.with_exceptions(exceptions)));
    }
    assert!(!wire_supported(&CopyTypes {
        zombie: true,
        ..CopyTypes::default()
    }));
}

#[test]
fn prior_198_rejects_final_actions_and_paid_embalm_slot_without_live_acquisition() {
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let source = game.object_ref(card);
    for action in [
        Action::ChooseEntryExile { object: source },
        Action::FinishEntryExile,
        Action::AnnounceRetainedCopyAbility { source },
        Action::AnnounceEmbalm {
            source,
            acquisition: 7,
        },
    ] {
        assert!(!wire_supported(&action), "{action:?}");
    }
    assert_eq!(card_rules::activation_slots::EMBALM, NEW_EMBALM_ACTIVATION);
    let packet = game.activation_packet(Player::P0, card, NEW_EMBALM_ACTIVATION);
    game.move_object(card, Zone::Exile);
    assert!(game.state.embalm_abilities.is_none());
    assert!(prior_schema199_state_supported(&game.state));
    assert!(!wire_supported(&packet));
    assert!(!packet_supported(&packet));
    let clean = game.snapshot();
    // Paid packets can outlive the graveyard row in every retained packet family.
    game.state.stack.push(packet.clone());
    assert!(!prior_schema199_state_supported(&game.state));
    game.restore(&clean).unwrap();
    game.state.pending_triggers.push(packet.clone());
    assert!(!prior_schema199_state_supported(&game.state));
    game.restore(&clean).unwrap();
    game.state.delayed_artifact_copies = Some(Box::new(vec![packet]));
    let mut branch = game.clone();
    drop(game);
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&clean).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
}

#[test]
fn prior_198_rejects_owned_embalm_history_even_when_no_current_view_exposes_it() {
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let clean = game.snapshot();
    let object = game.object_ref(card);
    let turn = game.state.turn;
    game.state
        .embalm_abilities
        .get_or_insert_default()
        .push(EmbalmAbilityView {
            object,
            acquisition: 11,
            expires_turn: turn,
        });
    for viewer in [Player::P0, Player::P1] {
        let view = game.observe(viewer);
        assert!(view.embalm_abilities.is_some());
        assert!(!wire_supported(&view));
    }
    assert!(!prior_schema199_state_supported(&game.state));
    game.state.turn = turn.checked_add(1).unwrap();
    assert!(game.observe(Player::P0).embalm_abilities.is_none());
    let mut branch = game.clone();
    drop(game);
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&clean).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
}

#[test]
fn prior_198_rejects_idle_link_origins_and_lower_layer_retained_origins() {
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let exiled = put(&mut game, Player::P0, "Hill Giant", Zone::Exile);
    let source = game.object_ref(card);
    let exiled = game.object_ref(exiled);
    game.state
        .linked_choices
        .get_or_insert_default()
        .links
        .push(linked_choices::LinkedExileView {
            source,
            exiled,
            ability_origin: None,
        });
    assert!(prior_schema199_state_supported(&game.state));
    let old = serde_json::to_string(&game.observe(Player::P0)).unwrap();
    assert_eq!(project_prior_schema199(&old).unwrap(), old);
    let linked = game.snapshot();
    game.state.linked_choices.as_mut().unwrap().links[0].ability_origin = Some(19);
    assert!(!prior_schema199_state_supported(&game.state));
    game.restore(&linked).unwrap();
    game.apply_temporary_copy(
        source,
        CopyRecipe::new(Card::from_name("Savannah Lions").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    game.apply_temporary_copy(
        source,
        CopyRecipe::new(Card::HillGiant),
        CopyDeadline::EndOfTurn,
    );
    assert!(prior_schema199_state_supported(&game.state));
    let plain = game.snapshot();
    // Some(None) is meaningful: the retained template belongs to the printed acquisition.
    game.state.temporary_copies.as_mut().unwrap()[0].layers[0]
        .value
        .retained_link_origin = Some(temporary_copy::RetainedActivationOrigin {
        ability_origin: None,
    });
    let mut branch = game.clone();
    drop(game);
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&plain).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
    branch.state.temporary_copies.as_mut().unwrap()[0].original_recipe = Some(
        CopyRecipe::new(Card::GrizzlyBears).with_exceptions(CopyExceptions {
            embalm: true,
            ..CopyExceptions::NONE
        }),
    );
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&plain).unwrap();
    branch.set_copy_recipe(
        source,
        CopyRecipe::new(Card::HillGiant).with_exceptions(CopyExceptions {
            embalm: true,
            ..CopyExceptions::NONE
        }),
    );
    assert!(!prior_schema199_state_supported(&branch.state));
}

#[test]
fn prior_198_rejects_active_and_suspended_mana_output_receipts_after_owned_restore() {
    use card_rules::authored::CardRules;
    const RULES: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::TapSourcePowerChoice {
                mask: 31,
            })],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let clean = game.snapshot();
    let item = game.activation_packet(Player::P0, card, 0);
    let mut a = game.payment_announcement(item, Counters::default(), Some([0; 6]), None);
    let output = a
        .mana_output
        .as_mut()
        .expect("dynamic mana recipe retained");
    assert!(output.choice.is_none());
    output.resume = Some(Box::new(Pending::Priority));
    game.state.announcing = Some(a);
    game.state.pending = Pending::EffectMana {
        player: Player::P0,
        production: ManaProduction::any_color(2),
        recipients: [true, false],
    };
    let active = game.snapshot();
    let mut branch = game.clone();
    drop(game);
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&clean).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
    branch.restore(&active).unwrap();
    let a = branch.state.announcing.take().unwrap();
    branch.state.mana_announcement_stack =
        Some(Box::new(vec![mana_announcements::ManaAnnouncement {
            announcement: a,
            pending: Pending::PayCosts { player: Player::P0 },
            quality: None,
        }]));
    branch.state.pending = Pending::Priority;
    assert!(branch.state.announcing.is_none());
    let suspended = branch.snapshot();
    assert!(!prior_schema199_state_supported(&branch.state));
    branch.restore(&clean).unwrap();
    branch.restore(&suspended).unwrap();
    assert!(!prior_schema199_state_supported(&branch.state));
}

#[test]
fn prior_198_rejects_new_semantics_nested_in_inherited_typed_continuations() {
    assert!(!target_supported(TargetRule::Union(&[
        TargetRule::Grouped {
            base: &TargetRule::LinkedExileCreature {
                ability_origin: Some(None)
            },
            constraint: TargetGroupConstraint::DifferentControllers,
        }
    ])));
    for program in [
        &[Instruction::GrantEmbalmTargets][..],
        &[Instruction::CreateEmbalmedSourceCopy][..],
        &[Instruction::CopySourceFromLinkedExile { base: (0, 0) }][..],
        &[Instruction::Draw(PlayerSet::SpeedBelow(4), 1)][..],
        &[Instruction::DamagePlayersScaled(
            PlayerSet::Controller,
            AmountValue::DiscardEventAmount,
        )][..],
        &[Instruction::GrowResolved {
            subject: Subject::Source,
            value: ResolvedStatValue::SourcePower,
            power: 1,
            toughness: 1,
        }][..],
        &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::FilteredCost {
                action: card_rules::CostAction::Sacrifice,
                any: &[card_rules::CostMatch::Kinds(16)],
            },
            players: PlayerSet::Controller,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            amount: 1,
            optional_skip: None,
        }][..],
    ] {
        assert!(!program_supported(program), "{program:?}");
    }
    assert!(program_supported(&[Instruction::Draw(
        PlayerSet::Controller,
        1
    )]));
}

#[test]
fn prior_198_keeps_inherited_null_cast_link_origins_eligible_after_owned_restore() {
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let exiled = put(&mut game, Player::P1, "Hill Giant", Zone::Exile);
    let source = game.object_ref(source);
    let exiled = game.object_ref(exiled);
    let clean = game.snapshot();
    // Both this field and ExileRevealedLinked's acquisition binding existed at
    // 73092e3. The new public LinkedExileView field does not make its owned
    // counterpart a new semantic state.
    game.state
        .linked_choices
        .get_or_insert_default()
        .null_cast_links
        .push(linked_choices::NullCastLink {
            source,
            exiled,
            ability_origin: Some(23),
        });
    assert!(prior_schema199_state_supported(&game.state));
    let linked = game.snapshot();
    let mut branch = game.clone();
    drop(game);
    assert!(prior_schema199_state_supported(&branch.state));
    assert_eq!(
        branch
            .state
            .linked_choices
            .as_ref()
            .unwrap()
            .null_cast_links[0]
            .ability_origin,
        Some(23)
    );
    branch.restore(&clean).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
    branch.restore(&linked).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
    assert_eq!(
        branch
            .state
            .linked_choices
            .as_ref()
            .unwrap()
            .null_cast_links[0]
            .ability_origin,
        Some(23)
    );
}

#[test]
fn prior_198_rejects_only_new_nonempty_acquired_nonmana_counter_receipts() {
    let source = ObjectRef {
        card: 17,
        generation: 2,
    };
    let empty = Action::AnnounceSelfGrantedAbility {
        source,
        grant_timestamp: 31,
        counter_payment: Counters::default(),
    };
    let inherited = serde_json::to_string(&empty).unwrap();
    assert!(!inherited.contains("counter_payment"));
    assert_eq!(project_prior_schema199(&inherited).unwrap(), inherited);
    let acquired = Action::AnnounceSelfGrantedAbility {
        source,
        grant_timestamp: 31,
        counter_payment: Counters::of(CounterKind::Charge, 1),
    };
    assert!(project_prior_schema199(&serde_json::to_string(&acquired).unwrap()).is_err());
    let native = Action::AnnounceActivate {
        card: source.card,
        ability: 0,
        counter_payment: Counters::of(CounterKind::PlusOne, 1),
    };
    let native = serde_json::to_string(&native).unwrap();
    assert_eq!(project_prior_schema199(&native).unwrap(), native);
}

#[test]
fn prior_198_source_ledger_proves_each_new_late_fixed_output_capability() {
    let audit: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/aetherdrift-prior198-trace-audit.json"
    ))
    .unwrap();
    let rows = audit["cards"].as_array().unwrap();
    assert_eq!(rows.len(), 42);
    let mut late = 0;
    let mut duplicate = 0;
    for row in rows {
        if row["classification"] == "duplicate_borrowed_mana_actions" {
            assert_eq!(row["card"], "Territory Forge");
            duplicate += 1;
            continue;
        }
        let name = row["mana_definition"].as_str().unwrap();
        let definition = Card::from_name(name).unwrap();
        let ability = u16::try_from(row["ability"].as_u64().unwrap()).unwrap();
        let recipe = definition
            .activation(ability)
            .mana_production()
            .unwrap()
            .base();
        let (mask, amount) = match recipe {
            ManaProduction::Choice { mask, amount }
            | ManaProduction::Combination { mask, amount } => (mask, amount),
            other => panic!("{} unexpected domain {other:?}", row["card"]),
        };
        assert_eq!(u64::from(mask), row["recipe"]["mask"].as_u64().unwrap());
        assert_eq!(u64::from(amount), row["recipe"]["amount"].as_u64().unwrap());
        assert!(amount > 0);
        let mut game = scenario(Player::P0);
        let card = if row["classification"] == "new_late_fixed_choice_route_from_created_treasure" {
            let id = game.state.objects.len() as CardId;
            game.create_tokens(Player::P0, definition, 1, false);
            assert_eq!(game.state.objects[id as usize].card, definition);
            id
        } else {
            put(&mut game, Player::P0, name, Zone::Battlefield)
        };
        let zero = Action::AnnounceMana {
            card,
            ability,
            counter_payment: Counters::default(),
            production: [0; 6],
        };
        let actions = game.legal_actions();
        assert!(
            actions.contains(&zero),
            "{} must expose the new late route",
            row["card"]
        );
        assert!(
            !prior_schema199_actions_supported(&game, &[zero]),
            "{}",
            row["card"]
        );
        let mut concrete = None;
        recipe.visit(|mana| {
            concrete = Some(mana);
            true
        });
        let concrete = Action::AnnounceMana {
            card,
            ability,
            counter_payment: Counters::default(),
            production: concrete.unwrap(),
        };
        assert!(
            actions.contains(&concrete),
            "{} retains concrete output",
            row["card"]
        );
        assert!(
            prior_schema199_actions_supported(&game, &[concrete]),
            "{}",
            row["card"]
        );
        late += 1;
    }
    assert_eq!((late, duplicate), (41, 1));
}

#[test]
fn prior_198_rejects_a_real_late_output_start_after_its_costs_complete() {
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Prophetic Prism", Zone::Battlefield);
    let start = Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production: [0; 6],
    };
    assert!(game.legal_actions().contains(&start));
    assert!(!prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&start)
    ));
    assert!(prior_schema199_state_supported(&game.state));
    act(&mut game, start.clone());
    let payment = game
        .legal_actions()
        .into_iter()
        .find_map(|a| match a {
            Action::PayCosts { payment } => Some(payment),
            _ => None,
        })
        .unwrap();
    let before = game.state.players[0].mana;
    act(&mut game, Action::PayCosts { payment });
    assert!(game.state.objects[card as usize].tapped);
    assert_eq!(
        game.state.players[0].mana,
        std::array::from_fn(|i| before[i] - payment[i])
    );
    assert!(matches!(game.state.pending, Pending::EffectMana { .. }));
    assert!(!prior_schema199_state_supported(&game.state));
    let late = game.snapshot();
    let mut branch = game.clone();
    drop(game);
    let production = [0, 0, 0, 1, 0, 0];
    let choice = Action::ChooseMana { production };
    assert!(branch.legal_actions().contains(&choice));
    assert!(!prior_schema199_actions_supported(
        &branch,
        std::slice::from_ref(&choice)
    ));
    act(&mut branch, choice);
    assert_eq!(
        branch.state.players[0].mana,
        std::array::from_fn(|i| before[i] - payment[i] + production[i])
    );
    assert!(branch.state.announcing.is_none());
    assert!(prior_schema199_state_supported(&branch.state));
    // An idle result alone cannot certify a journal which selected the new route.
    assert!(!prior_schema199_actions_supported(&branch, &[start]));
    branch.restore(&late).unwrap();
    assert!(!prior_schema199_state_supported(&branch.state));
}

#[test]
fn prior_198_preserves_real_zero_power_mana_and_resolves_borrowed_donor_templates() {
    use card_rules::authored::CardRules;
    const ZERO_POWER: CardRules = CardRules {
        effect: Effect::Creature(0, 2),
        activated: &[
            ActivatedAbility {
                exhaust: false,
                guard: ActivationGuard::Always,
                origin: ActivationOrigin::Battlefield,
                cycling: false,
                cost: ActivationCost::ZERO,
                targets: TargetPlan::NONE,
                program: &[Instruction::AddMana(ManaProduction::SourcePower {
                    color: 4,
                })],
                sorcery: false,
                once_per_turn: false,
            },
            ActivatedAbility {
                exhaust: false,
                guard: ActivationGuard::Always,
                origin: ActivationOrigin::Battlefield,
                cycling: false,
                cost: ActivationCost::ZERO,
                targets: TargetPlan::NONE,
                program: &[Instruction::AddMana(ManaProduction::Choice {
                    mask: 1,
                    amount: 0,
                })],
                sorcery: false,
                once_per_turn: false,
            },
        ],
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[(Card::GrizzlyBears, &ZERO_POWER)]);
    let mut game = scenario(Player::P0);
    let zero_power = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let genuine = Action::AnnounceMana {
        card: zero_power,
        ability: 0,
        counter_payment: Counters::default(),
        production: [0; 6],
    };
    assert!(game.legal_actions().contains(&genuine));
    assert!(prior_schema199_actions_supported(&game, &[genuine]));
    let fixed_zero = Action::AnnounceMana {
        card: zero_power,
        ability: 1,
        counter_payment: Counters::default(),
        production: [0; 6],
    };
    assert!(game.legal_actions().contains(&fixed_zero));
    assert!(prior_schema199_actions_supported(&game, &[fixed_zero]));
    let source = put(&mut game, Player::P0, "Territory Forge", Zone::Battlefield);
    let donor = put(&mut game, Player::P0, "Prophetic Prism", Zone::Exile);
    let exiled = game.object_ref(donor);
    let grave = put(&mut game, Player::P0, "Gravestone Strider", Zone::Graveyard);
    let graveyard = game.object_ref(grave);
    for action in [
        Action::AnnounceBorrowedMana {
            card: source,
            exiled,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0; 6],
        },
        Action::ActivateBorrowedMana {
            card: source,
            exiled,
            ability: 0,
            counter_payment: Counters::default(),
            payment: [1, 0, 0, 0, 0, 0],
            production: [0; 6],
        },
        Action::AnnounceGraveyardBorrowedMana {
            card: source,
            graveyard,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0; 6],
        },
        Action::ActivateGraveyardBorrowedMana {
            card: source,
            graveyard,
            ability: 0,
            counter_payment: Counters::default(),
            payment: [1, 0, 0, 0, 0, 0],
            production: [0; 6],
        },
    ] {
        assert!(!prior_schema199_actions_supported(&game, &[action]));
    }
    let inherited = Action::AnnounceBorrowedMana {
        card: source,
        exiled,
        ability: 0,
        counter_payment: Counters::default(),
        production: [1, 0, 0, 0, 0, 0],
    };
    assert!(prior_schema199_actions_supported(&game, &[inherited]));
}

#[test]
fn prior_198_distinguishes_new_dynamic_native_starts_from_genuine_zero_domains() {
    use card_rules::authored::CardRules;
    const BASE: ActivatedAbility = ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[],
        sorcery: false,
        once_per_turn: false,
    };
    const POWER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::TapSourcePowerChoice {
                mask: 31,
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const DISTINCT: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::DistinctCreaturePowers)],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[
        (Card::GrizzlyBears, &POWER),
        (Card::from_name("Prophetic Prism").unwrap(), &DISTINCT),
    ]);
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let zero = Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production: [0; 6],
    };
    assert!(game.legal_actions().contains(&zero));
    assert!(!prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&zero)
    ));
    game.state.objects[card as usize].growth.power = -2;
    assert!(game.legal_actions().contains(&zero));
    assert!(prior_schema199_actions_supported(&game, &[zero]));

    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Prophetic Prism", Zone::Battlefield);
    let zero = Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production: [0; 6],
    };
    assert!(game.legal_actions().contains(&zero));
    assert!(prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&zero)
    ));
    // A zero-power creature still contributes one distinct power; opposing
    // creatures never contribute to this source controller's domain.
    put(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
    assert!(prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&zero)
    ));
    put(&mut game, Player::P0, "Aegis Turtle", Zone::Battlefield);
    assert!(game.legal_actions().contains(&zero));
    assert!(!prior_schema199_actions_supported(&game, &[zero]));
}

#[test]
fn prior_198_dynamic_borrowed_domains_sample_the_borrower_not_the_donor() {
    use card_rules::authored::CardRules;
    const ABILITIES: &[ActivatedAbility] = &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::TapSourcePowerChoice {
            mask: 31,
        })],
        sorcery: false,
        once_per_turn: false,
    }];
    const POSITIVE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: ABILITIES,
        ..CardRules::EMPTY
    };
    const ZERO: CardRules = CardRules {
        effect: Effect::Creature(0, 5),
        activated: ABILITIES,
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[
        (Card::GrizzlyBears, &POSITIVE),
        (Card::from_name("Aegis Turtle").unwrap(), &ZERO),
    ]);
    for donor_name in ["Grizzly Bears", "Aegis Turtle"] {
        let mut game = scenario(Player::P0);
        let source = put(&mut game, Player::P0, "Territory Forge", Zone::Battlefield);
        let donor = put(&mut game, Player::P1, donor_name, Zone::Exile);
        let source_ref = game.object_ref(source);
        let exiled = game.object_ref(donor);
        game.state
            .borrowed_links
            .get_or_insert_default()
            .push(borrowed_activations::Link {
                source: source_ref,
                origin: None,
                exiled,
                first_slot: 0,
                width: 1,
                once_per_turn: false,
            });
        game.change_object_form(
            source_ref,
            card_rules::AnimationSpec {
                artifact: true,
                creature_types: card_rules::AnimationCreatureTypes::Unchanged,
                base: Some((0, 3)),
                keywords: Keywords::NONE,
                duration: card_rules::AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        let zero = Action::AnnounceBorrowedMana {
            card: source,
            exiled,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0; 6],
        };
        assert!(game.legal_actions().contains(&zero), "{donor_name}");
        assert!(
            prior_schema199_actions_supported(&game, std::slice::from_ref(&zero)),
            "zero borrower, {donor_name}"
        );
        game.state.objects[source as usize].growth.power = 3;
        assert!(game.legal_actions().contains(&zero), "{donor_name}");
        assert!(
            !prior_schema199_actions_supported(&game, &[zero]),
            "positive borrower, {donor_name}"
        );
    }
}

#[test]
fn prior_198_keeps_legacy_resolving_spell_mana_choices_eligible() {
    use card_rules::authored::CardRules;
    const MANA_SPELL: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::AddMana(ManaProduction::Choice {
                mask: 31,
                amount: 1,
            })],
        ),
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[(Card::Shock, &MANA_SPELL)]);
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P0, "Shock", Zone::Hand);
    cast(&mut game, card, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(game.state.resolving.is_some());
    assert!(game.state.announcing.is_none());
    assert!(matches!(game.state.pending, Pending::EffectMana { .. }));
    let choice = Action::ChooseMana {
        production: [0, 0, 0, 0, 1, 0],
    };
    assert!(game.legal_actions().contains(&choice));
    assert!(prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&choice)
    ));
    act(&mut game, choice);
    assert!(game.state.resolving.is_none());
    assert!(prior_schema199_state_supported(&game.state));
}

#[test]
fn prior_198_rejects_positive_revealed_hand_selection_but_preserves_old_discard() {
    use rustmage_core::engine::hand_choices::HandChoiceKind;
    for kind in [
        HandChoiceKind::DiscardRevealedKinds { kinds: 4 | 8 },
        HandChoiceKind::ExileRevealedKinds { kinds: 1 | 16 },
    ] {
        assert!(!wire_supported(&kind));
    }
    for exile in [false, true] {
        assert!(!program_supported(&[Instruction::RevealTargetHandChoose {
            kinds: 1 | 16,
            exile,
        }]));
    }
    assert!(wire_supported(&HandChoiceKind::DiscardRevealed));
    assert!(program_supported(&[Instruction::RevealHandDiscard {
        excluded_kinds: 2
    }]));
    let old = r#"{"kind":"DiscardRevealed","actor":"P0","victim":"P1","required":1,"selected":[],"candidates":[],"optional":false}"#;
    assert_eq!(project_prior_schema199(old).unwrap(), old);
}

#[test]
fn prior_198_rejects_attached_untap_without_rejecting_inherited_attached_tap() {
    assert!(!program_supported(&[Instruction::UntapAttached]));
    assert!(program_supported(&[Instruction::TapAttached]));
}

#[test]
fn prior_198_distinguishes_conditional_self_cast_from_inherited_graveyard_authority() {
    use card_rules::authored::{CardExtras, CardRules};
    const GROUPS: &[card_rules::StaticAbilityGroup] = &[card_rules::StaticAbilityGroup {
        line: 0,
        first_layer: card_rules::AbilityLayer::Rule,
        fragments: &[card_rules::StaticAbilityFragment::SelfGraveyardCastIf(
            StatValue::AtLeast {
                value: &StatValue::ControllerSpeed,
                minimum: 4,
            },
        )],
    }];
    const RULES: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: GROUPS,
        extras: &CardExtras {
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let id = put(&mut game, p, "Grizzly Bears", Zone::Graveyard);
    let action = Action::AnnounceCast { card: id };
    assert!(
        prior_schema199_actions_supported(&game, std::slice::from_ref(&action)),
        "false intrinsic predicate adds no route"
    );
    game.state.players[p.index()].speed = 4;
    assert!(game.legal_actions().contains(&action));
    assert!(!prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&action)
    ));
    let exact = game.object_ref(id);
    game.grant_graveyard_permission(p, exact, card_rules::GraveyardCastKind::Flashback);
    assert!(
        !prior_schema199_actions_supported(&game, std::slice::from_ref(&action)),
        "flashback alone does not authorize this normal action"
    );
    game.grant_graveyard_permission(p, exact, card_rules::GraveyardCastKind::Cast);
    assert!(prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&action)
    ));
    game.move_object(id, Zone::Exile);
    game.move_object(id, Zone::Graveyard);
    assert_ne!(exact, game.object_ref(id));
    assert!(
        !prior_schema199_actions_supported(&game, std::slice::from_ref(&action)),
        "stale permission cannot mask new self-casting semantics"
    );
    let exact = game.object_ref(id);
    game.grant_graveyard_permission(p, exact, card_rules::GraveyardCastKind::Play);
    assert!(prior_schema199_actions_supported(
        &game,
        std::slice::from_ref(&action)
    ));
    game.move_object(id, Zone::Hand);
    assert!(game.legal_actions().contains(&action));
    assert!(prior_schema199_actions_supported(&game, &[action]));
    assert!(!program_supported(&[Instruction::IfSubjectSubtype {
        subject: Subject::Targets,
        subtype: "Vehicle",
        skip: 1,
    }]));
}

#[test]
fn prior_198_rejects_the_new_filtered_signed_growth_program() {
    let instruction = Instruction::GrowFilteredScaled {
        filter: card_rules::CountFilter {
            zone: card_rules::CountZone::Battlefield,
            relation: -1,
            kind: 1,
            exclude: 16,
            subtype: None,
            basic: false,
            other: true,
            cards_only: false,
        },
        amount: AmountValue::Fixed(1),
        power: -1,
        toughness: -1,
    };
    assert!(!program_supported(&[instruction]));
    assert!(program_supported(&[Instruction::Grow {
        subject: Subject::Source,
        power: 1,
        toughness: 1
    }]));
}

#[test]
fn prior_198_rejects_new_private_deployment_mode_even_when_opponent_cannot_see_kind() {
    use card_rules::authored::CardRules;
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::DeployHandPermanents],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    assert!(!wire_supported(&HandChoiceKind::DeployPermanents));
    assert!(wire_supported(&HandChoiceKind::DeployCreatures));
    assert!(!program_supported(&[Instruction::DeployHandPermanents]));
    assert!(program_supported(&[Instruction::DeployHandCreaturesEach]));
    let _scope = card_rules::fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let clean = game.snapshot();
    let item = game.activation_packet(Player::P0, source, 0);
    game.start_controller_hand_permanent_deployment(&item);
    assert!(game.observe(Player::P1).hand_choice.is_none());
    assert!(
        wire_supported(&game.observe(Player::P1)),
        "the old-shaped public count does not reveal the private new mode"
    );
    assert!(!prior_schema199_state_supported(&game.state));
    let selected = game.snapshot();
    let mut child = game.clone();
    drop(game);
    assert!(!prior_schema199_state_supported(&child.state));
    child.restore(&clean).unwrap();
    assert!(prior_schema199_state_supported(&child.state));
    child.restore(&selected).unwrap();
    assert!(!prior_schema199_state_supported(&child.state));
}

#[test]
fn prior_198_rejects_new_noninstant_single_copy_capture_but_preserves_old_capture_routes() {
    use card_rules::authored::CardRules;
    const COPY: TriggeredAbility = TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: card_rules::CastFilter::Any,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    };
    const MODAL: TriggeredAbility = TriggeredAbility {
        program: &[Instruction::Modal(&[SpellMode {
            text: "Copy the triggering spell.",
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCastSpell {
                prior_spells: false,
            }],
        }])],
        ..COPY
    };
    const RULES: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[COPY, MODAL],
        ..CardRules::EMPTY
    };
    let _scope =
        card_rules::fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &RULES)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    let bear = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut game, bear, None);
    let packet = game
        .state
        .pending_triggers
        .iter()
        .chain(&game.state.ordering_triggers)
        .chain(&game.state.stack)
        .find(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { ability: 0 })
                && item.source.is_some_and(|s| s.id == source)
        })
        .unwrap()
        .clone();
    assert!(packet.copy_event.is_some());
    assert!(!packet_supported(&packet));
    assert!(!prior_schema199_state_supported(&game.state));
    let mut modal = packet.clone();
    modal.kind = StackKind::TriggeredAbility { ability: 1 };
    modal.mode = Some(0);
    assert!(
        !packet_supported(&modal),
        "selected mode programs require the same capture guard"
    );
    modal.copy_event = None;
    assert!(
        packet_supported(&modal),
        "an inherited modal program without a new capture stays eligible"
    );
    let mut uncaptured = packet.clone();
    uncaptured.copy_event = None;
    assert!(
        packet_supported(&uncaptured),
        "the inherited program alone is unchanged"
    );
    let mut nested = packet.clone();
    nested.copy_event.as_mut().unwrap().spell = packet.clone();
    assert!(
        !packet_supported(&nested),
        "an inner new capture must not escape the owned packet guard"
    );
    let mut instant = packet.clone();
    instant
        .copy_event
        .as_mut()
        .unwrap()
        .spell
        .source
        .as_mut()
        .unwrap()
        .card = Card::from_name("Shock").unwrap();
    assert!(
        packet_supported(&instant),
        "inherited instant/sorcery captures stay eligible"
    );
    assert!(!prior_noninstant_cast_copy_supported(
        COPY.condition,
        COPY.program,
        false
    ));
    assert!(
        !prior_noninstant_cast_copy_supported(COPY.condition, COPY.program, true),
        "prepared alone did not admit the old false arm"
    );
    // No currently admitted permanent prepared face exists. Explicitly test
    // the historical classifier inputs instead of masquerading an instant face
    // as noninstant metadata and accidentally taking the I/S fast path above.
    const MIXED_PROGRAMS: &[&[Instruction]] = &[
        &[
            Instruction::CopyCastSpell {
                prior_spells: false,
            },
            Instruction::CopyPreparedCastSpell,
        ],
        &[
            Instruction::CopyPreparedCastSpell,
            Instruction::CopyCastSpell {
                prior_spells: false,
            },
        ],
    ];
    for &program in MIXED_PROGRAMS {
        let mixed = TriggeredAbility { program, ..COPY };
        assert!(prior_noninstant_cast_copy_supported(
            mixed.condition,
            mixed.program,
            true
        ));
        assert!(!prior_noninstant_cast_copy_supported(
            mixed.condition,
            mixed.program,
            false
        ));
    }
    let prepared = TriggeredAbility {
        program: &[Instruction::CopyPreparedCastSpell],
        ..COPY
    };
    assert!(prior_noninstant_cast_copy_supported(
        prepared.condition,
        prepared.program,
        true
    ));
    let repeated = TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: card_rules::CastFilter::Any,
            ordinal: None,
        }),
        ..COPY
    };
    assert!(prior_noninstant_cast_copy_supported(
        repeated.condition,
        repeated.program,
        false
    ));
    assert!(prior_noninstant_cast_copy_supported(
        repeated.condition,
        repeated.program,
        true
    ));
}

#[test]
fn prior_198_rejects_multigroup_aggregate_exchange_but_preserves_single_occurrence_and_unrelated_groups()
 {
    use card_rules::authored::CardRules;
    const fn role(relation: i8) -> TargetRule {
        TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 17,
            relation,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }
    }
    const ROLES: TargetPlan =
        TargetPlan::Groups(&[TargetSpec::one(role(0)), TargetSpec::one(role(1))]);
    const SINGLE: TargetPlan = TargetPlan::Single(TargetSpec {
        rule: TargetRule::Grouped {
            base: &role(-1),
            constraint: TargetGroupConstraint::DifferentControllers,
        },
        min: 2,
        max: 2,
    });
    const EXCHANGE: &[Instruction] = &[Instruction::ExchangeControlTargets];
    const fn ability(targets: TargetPlan, program: &'static [Instruction]) -> ActivatedAbility {
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets,
            program,
            sorcery: false,
            once_per_turn: false,
        }
    }
    const RULES: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[
            ability(ROLES, EXCHANGE),
            ability(SINGLE, EXCHANGE),
            ability(
                TargetPlan::Groups(&[TargetSpec {
                    rule: role(-1),
                    min: 2,
                    max: 2,
                }]),
                EXCHANGE,
            ),
            ability(ROLES, &[Instruction::SelectGroup(1), Instruction::Destroy]),
            ability(
                TargetPlan::NONE,
                &[Instruction::Modal(&[
                    SpellMode {
                        text: "Exchange one occurrence.",
                        targets: SINGLE,
                        program: EXCHANGE,
                    },
                    SpellMode {
                        text: "Exchange independent roles.",
                        targets: ROLES,
                        program: EXCHANGE,
                    },
                ])],
            ),
        ],
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert!(prior_schema199_state_supported(&game.state));
    let new = game.activation_packet(Player::P0, source, 0);
    assert!(
        !packet_supported(&new),
        "a new multigroup declaration is meaningful even before choices"
    );
    for index in [1, 2, 3] {
        assert!(
            packet_supported(&game.activation_packet(Player::P0, source, index)),
            "inherited declaration {index}"
        );
    }
    let mut modal = game.activation_packet(Player::P0, source, 4);
    modal.mode = Some(0);
    assert!(
        packet_supported(&modal),
        "use the selected mode's one-group declaration"
    );
    modal.mode = Some(1);
    assert!(
        !packet_supported(&modal),
        "use the selected mode's independent roles"
    );
    assert!(
        !program_supported(&[Instruction::Modal(&[SpellMode {
            text: "New aggregate declaration.",
            targets: ROLES,
            program: EXCHANGE
        },])]),
        "nested declared modes must not hide new semantics"
    );
    assert!(program_supported(&[Instruction::Modal(&[SpellMode {
        text: "Inherited single declaration.",
        targets: SINGLE,
        program: EXCHANGE
    },])]));
    // Every owned packet family calls the same predicate; do not erase groups.
    let clean = game.snapshot();
    game.state.stack.push(new.clone());
    assert!(!prior_schema199_state_supported(&game.state));
    let saved = game.snapshot();
    let mut branch = game.clone();
    branch.restore(&clean).unwrap();
    assert!(prior_schema199_state_supported(&branch.state));
    branch.restore(&saved).unwrap();
    assert!(!prior_schema199_state_supported(&branch.state));
    game.state.stack.clear();
    game.state.delayed_artifact_copies = Some(Box::new(vec![new.clone()]));
    assert!(!prior_schema199_state_supported(&game.state));
    let mut nested = game.activation_packet(Player::P0, source, 1);
    nested.copy_event = Some(Box::new(CopyEvent {
        spell: new,
        amount: 1,
    }));
    assert!(
        !packet_supported(&nested),
        "recursively owned captures retain their own declaration guard"
    );
    assert!(
        !plan_program_supported(
            SINGLE,
            &[Instruction::DestroyTargetsThenSourceCountersIfOwnCreature {
                kind: CounterKind::PlusOne,
                amount: 2,
            }]
        ),
        "the reward opcode itself remains schema199-only even for one group"
    );
}

#[test]
fn prior_198_keeps_real_burglars_plot_single_group_capture_eligible() {
    let mut game = scenario(Player::P0);
    let own = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = put(
        &mut game,
        Player::P0,
        "Bilbo, Luckwearer // Burglar's Plot",
        Zone::Hand,
    );
    act(&mut game, Action::AnnounceAdventure { card: spell });
    for card in [own, enemy] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    choose(&mut game, |action| {
        matches!(action, Action::PayCosts { .. })
    });
    let packet = game.state.stack.last().unwrap();
    assert!(matches!(packet.program().unwrap().0, TargetPlan::Single(_)));
    assert!(packet_supported(packet));
    assert!(prior_schema199_state_supported(&game.state));
}

#[test]
fn half_life_round_up_retains_the_existing_player_set_compatibility_domain() {
    use card_rules::PlayerSet;
    for player in [PlayerSet::Controller, PlayerSet::Opponent, PlayerSet::Each] {
        assert!(instruction_players_supported(
            Instruction::LoseHalfLifeRoundedUp(player)
        ));
    }
    assert!(!instruction_players_supported(
        Instruction::LoseHalfLifeRoundedUp(PlayerSet::SpeedBelow(4))
    ));
    assert!(!instruction_players_supported(Instruction::LoseHalfLife(
        PlayerSet::SpeedBelow(4)
    )));
}
