//! Smoke policies for tests and benchmarks; not a training implementation.
use crate::cards::{CounterKind, Effect, Instruction, PlayerSet};
use crate::{Action, Game, StackKind, Target, Zone};

/// Deterministic simple policy: keep, play lands, cast, attack, block, tap, pass.
/// Uses only the acting player's observation and the offered action list.
pub fn smoke_index(game: &Game, actions: &[Action]) -> usize {
    let actor = game.actor().expect("nonterminal game");
    let observation = game.observe(actor);
    // Fundable declarations are offered before any speculative mana activation.
    // Preserve spare sources for later steps once those declarations are exhausted.
    let tap_for_payment = observation.announcing.is_some()
        || observation.plot_payment.is_some()
        || observation.payment_request.is_some();
    let resolution_mana_score = |payment: crate::Mana, production: crate::Mana| {
        let request = observation.resolution_payment.as_ref().unwrap();
        let pool = observation.players[actor.index()].mana;
        let deficit = |pool: crate::Mana| {
            let colored: u64 = pool
                .iter()
                .zip(request.cost.colored)
                .map(|(&pool, need)| u64::from(need.saturating_sub(pool)))
                .sum();
            let spare: u64 = pool
                .iter()
                .zip(request.cost.colored)
                .map(|(&pool, need)| u64::from(pool.saturating_sub(need)))
                .sum();
            colored + u64::from(request.cost.generic).saturating_sub(spare)
        };
        let after = std::array::from_fn(|i| pool[i] - payment[i] + production[i]);
        if deficit(after) < deficit(pool) {
            70
        } else {
            -2
        }
    };
    let target_score = |program: &[Instruction], target: Target| -> i128 {
        let beneficial = program
            .iter()
            .find_map(|instruction| match instruction {
                Instruction::PutCounters { kind, .. }
                | Instruction::DistributeCounters { kind, .. }
                | Instruction::DoubleCounterKind { kind, .. } => {
                    Some(*kind == CounterKind::PlusOne)
                }
                Instruction::DoubleCounters => Some(true),
                Instruction::GrantKeywords { .. } => Some(true),
                Instruction::Connive { .. }
                | Instruction::CreateCopyToken { .. }
                | Instruction::CreateCopyTokenWithExceptions { .. } => Some(true),
                Instruction::Grow {
                    power, toughness, ..
                }
                | Instruction::GrowScaled {
                    power, toughness, ..
                } => Some(*power >= 0 && *toughness >= 0),
                Instruction::Untap
                | Instruction::BlinkUntilEndStep { .. }
                | Instruction::BlinkGroupTappedUntilEndStep
                | Instruction::ReturnToBattlefield
                | Instruction::ReturnToBattlefieldWithRole { .. }
                | Instruction::ReturnToBattlefieldTapped
                | Instruction::AttachSource => Some(true),
                Instruction::Draw(PlayerSet::Targeted, _)
                | Instruction::DrawScaled(PlayerSet::Targeted, _)
                | Instruction::GainLife(PlayerSet::Targeted, _) => Some(true),
                Instruction::ReturnToHand => Some(match target {
                    Target::Card(id) => observation
                        .objects
                        .iter()
                        .any(|o| o.id == id && o.zone == Zone::Graveyard),
                    _ => false,
                }),
                Instruction::Exile | Instruction::ExileUntilSourceLeaves => Some(false),
                Instruction::Counter
                | Instruction::Destroy
                | Instruction::Damage(_)
                | Instruction::Tap
                | Instruction::LoseLife(PlayerSet::Targeted, _)
                | Instruction::Discard(PlayerSet::Targeted, _)
                | Instruction::Mill(PlayerSet::Targeted, _) => Some(false),
                _ => None,
            })
            .unwrap_or(false);
        let own = match target {
            Target::Player(player) => player == actor,
            Target::Card(id) => observation
                .objects
                .iter()
                .any(|o| o.id == id && o.controller == actor),
            Target::Stack(id) => observation
                .stack
                .iter()
                .any(|item| item.id == id && item.controller == actor),
            Target::Spell(id) => observation.stack.iter().any(|item| {
                item.kind == StackKind::Spell && item.card == id && item.controller == actor
            }),
        };
        if own == beneficial { 70 } else { -10 }
    };
    let aura_score = |card: crate::Card, target: Target| {
        let aura = card.aura().unwrap();
        let beneficial = !card.controls_attached()
            && aura.bonus.stats.power >= 0
            && aura.bonus.stats.toughness >= 0
            && !aura.bonus.keywords.contains(crate::Keywords::DEFENDER)
            && !aura.bonus.keywords.contains(crate::Keywords::CANT_BLOCK);
        let own = matches!(target, Target::Card(id) if observation.objects.iter().any(|o|o.id==id && o.controller==actor));
        if own == beneficial { 70 } else { -10 }
    };
    let score = |action: &Action| -> i128 {
        // A smoke policy need not spend the whole turn re-equipping, including
        // legal zero-cost activations. The engine still offers those actions.
        if let Action::Activate { card, ability, .. }
        | Action::BeginActivate { card, ability, .. }
        | Action::AnnounceActivate { card, ability, .. } = action
            && let Some(object) = observation.objects.iter().find(|o| o.id == *card)
            && object.card.activations()[*ability as usize]
                .program
                .contains(&Instruction::AttachSource)
            && object.used_abilities & (1u64 << ability) != 0
        {
            return -10;
        }
        match action {
            Action::AssignLibraryPile { pile, .. } => {
                let piles = observation.library_piles.as_ref().unwrap();
                // A balanced split is a generic deterministic baseline, using only
                // the chooser's visible counts rather than library/RNG state.
                let down = piles.face_down_count <= piles.face_up.len();
                i128::from((*pile == crate::LibraryPile::FaceDown) == down)
            }
            Action::ChooseLibraryPile { pile } => {
                let piles = observation.library_piles.as_ref().unwrap();
                // The opponent gives away the smaller pile without seeing the
                // hidden identities. Both empty and face-down piles are legal.
                -i128::try_from(if *pile == crate::LibraryPile::FaceDown {
                    piles.face_down_count
                } else {
                    piles.face_up.len()
                })
                .unwrap()
            }
            Action::KeepPermanentType { object } => {
                let permanent = observation
                    .objects
                    .iter()
                    .find(|o| o.reference == *object)
                    .unwrap();
                permanent.power.max(0) + permanent.toughness.max(0)
            }
            Action::ChooseExplore { graveyard, .. } => {
                if *graveyard {
                    500
                } else {
                    400
                }
            }
            Action::AnnounceAdventure { .. } | Action::AnnounceOmen { .. } => 30,
            Action::ChooseManaLotSpend { amount } => 120 + i128::from(*amount != 0),
            Action::ChooseEntryCopy { object: None } => 0,
            Action::ChooseEntryCopy {
                object: Some(object),
            } => observation
                .objects
                .iter()
                .find(|candidate| candidate.reference == *object)
                .map_or(-1, |candidate| {
                    if candidate.card.as_enters_copy_creature() {
                        -1
                    } else {
                        120 + candidate.power.max(0) + candidate.toughness.max(0)
                    }
                }),
            Action::ChooseEntryTimestamp { .. } => 120,
            Action::ChooseReplacement { .. } => 120,
            Action::ChooseCreatedToken { .. } => 120,
            Action::ChooseEndure { put_counters } => {
                if *put_counters {
                    40
                } else {
                    35
                }
            }
            Action::BeginConvoke => 40,
            Action::ChooseConvoker { .. } => 50,
            Action::ChooseMobilizeDefender { .. } | Action::ChooseHandDefender { .. } => 50,
            Action::PutHandCreature { .. }
            | Action::PutHandLand { .. }
            | Action::SelectLand { .. } => 50,
            Action::FinishLandSelection | Action::FinishHandSelection => -1,
            Action::DeclineHandEntry => -1,
            Action::ChooseSaddler { .. } | Action::ChooseCrewer { .. } => 50,
            Action::FinishSaddle | Action::FinishCrew => 100,
            Action::ChooseSneakCostOrder { .. } => 50,
            Action::ChooseSneakAdditionalCost { .. } => 60,
            Action::ChooseCounterCostOrder { after_printed } => {
                if *after_printed {
                    2
                } else {
                    3
                }
            }
            Action::ChooseX { value } => 80 + i128::from((*value).min(20)),
            Action::ChooseGift { beneficiary } => {
                if beneficiary.is_some() {
                    40
                } else {
                    0
                }
            }
            Action::ChooseOffspring { pay } | Action::ChooseKicker { pay } => {
                if *pay {
                    100
                } else {
                    0
                }
            }
            Action::ChooseCollectEvidence { collect } => {
                if *collect {
                    60
                } else {
                    0
                }
            }
            Action::ChooseEvidence { .. } => 50,
            Action::FinishEvidence => 80,
            Action::DeclineEvidence => -10,
            Action::ChooseBeholdType { creature_type } => {
                if creature_type.is_some() {
                    60
                } else {
                    0
                }
            }
            Action::ChooseOptionalAdditionalCost { object } => {
                if object.is_some() {
                    60
                } else {
                    0
                }
            }
            Action::ChooseBargain { sacrifice } => {
                if sacrifice.is_some() {
                    60
                } else {
                    0
                }
            }
            Action::ChooseConspire { pay, .. } => {
                if *pay {
                    70
                } else {
                    0
                }
            }
            Action::PayConspire { .. } | Action::PayConspirePrintedCosts => 60,
            Action::ChooseAdditionalCost { .. } | Action::ChooseCreatureCounterCost { .. } => 60,
            Action::ChooseForageRoute { .. } | Action::ChooseForage { .. } => 60,
            Action::DeclineForage => -10,
            Action::ChooseConniver { .. } => 60,
            Action::PayResolutionMana { .. } | Action::PayResolutionLife => 100,
            Action::DeclineResolutionMana | Action::DeclineResolutionLife => -1,
            Action::ChooseAuraAttachment { object } => aura_score(
                observation.aura_entry.as_ref().unwrap().card,
                Target::Card(object.card),
            ),
            Action::SelectMilled { .. } | Action::ChooseRevealedDiscard { .. } => 60,
            Action::FinishMilledSelection => 0,
            Action::RecoverMilled { object } => {
                if object.is_some() {
                    60
                } else {
                    0
                }
            }
            Action::Concede => -1000,
            Action::ChooseZoneReplacement { .. } => 50,
            Action::Sacrifice { .. }
            | Action::Blight { .. }
            | Action::ChooseCreature { .. }
            | Action::BottomHandCard { .. }
            | Action::ChooseOutsideGameCard { .. }
            | Action::ExileHandCard { .. }
            | Action::RevealHandCard { .. }
            | Action::ExileGraveyardCard { .. }
            | Action::TapForEffect { .. }
            | Action::ChooseCounterRemoval { .. } => 50,
            Action::ProliferatePlayer { player } => {
                if *player != actor {
                    60
                } else {
                    -1
                }
            }
            Action::ProliferatePermanent { object } => observation
                .objects
                .iter()
                .find(|o| o.id == object.card)
                .map_or(-1, |o| {
                    let value = CounterKind::ALL
                        .into_iter()
                        .filter(|&kind| o.counters.get(kind) > 0)
                        .map(|kind| {
                            let beneficial =
                                !matches!(kind, CounterKind::MinusOne | CounterKind::Stun);
                            if beneficial == (o.controller == actor) {
                                1i128
                            } else {
                                -1
                            }
                        })
                        .sum::<i128>();
                    if value > 0 { 60 } else { -1 }
                }),
            Action::FinishProliferate => 0,
            Action::DeclineEffectPayment => -1,
            Action::AnnounceFree {
                flash_surcharge: false,
                ..
            } => 81,
            Action::AnnounceLinkedFree { .. }
            | Action::AnnounceFree { .. }
            | Action::AnnounceGrantedFlashback { .. }
            | Action::AnnounceCounterGraveyard { .. }
            | Action::AnnounceLinkedExileCounterCost { .. }
            | Action::AnnounceCast { .. }
            | Action::AnnounceFlashSurcharge { .. }
            | Action::AnnounceWarp { .. }
            | Action::AnnounceSneak { .. }
            | Action::AnnounceGraveyardSpell { .. }
            | Action::AnnounceLibrarySpell { .. }
            | Action::AnnouncePlotted { .. }
            | Action::AnnounceExile { .. }
            | Action::AnnounceFlashback { .. } => 79,
            Action::AnnouncePlot { .. } => 40,
            Action::AnnounceActivate { .. } | Action::AnnounceAttachedAbility { .. } => 34,
            Action::AnnounceMana { .. } => -2,
            Action::PayCosts { .. } | Action::PayPlot { .. } => 100,
            Action::ChooseSpellCost { .. } | Action::ChooseActivationCost { .. } => 50,
            Action::ChooseMode { .. } => 50,
            Action::ChooseSearch { .. } | Action::ChooseSearchBattlefield { .. } => 60,
            Action::FinishSearch => 0,
            Action::ChooseEntryValue {
                value: crate::EntryChoiceValue::CreatureType(typ),
            } => observation
                .objects
                .iter()
                .filter(|object| {
                    (match object.zone {
                        Zone::Hand => object.owner == actor,
                        Zone::Battlefield => object.controller == actor,
                        _ => false,
                    }) && object.has_subtype(typ.name())
                })
                .count() as i128,
            Action::ChooseEntryValue {
                value: crate::EntryChoiceValue::CardName(name),
            } => {
                let looked = observation
                    .entry_value
                    .as_ref()
                    .and_then(|choice| choice.opponent_hand.as_ref());
                observation
                    .objects
                    .iter()
                    .chain(looked.into_iter().flatten())
                    .filter(|object| {
                        name.matches(object.name())
                            && object
                                .card
                                .activations()
                                .iter()
                                .any(|ability| ability.mana_production().is_none())
                    })
                    .map(|object| {
                        if object.controller == actor {
                            -20
                        } else if object.zone == Zone::Battlefield {
                            12
                        } else {
                            8
                        }
                    })
                    .sum()
            }
            Action::ChooseColor { color } => {
                let existing = observation
                    .objects
                    .iter()
                    .filter(|object| object.zone == Zone::Battlefield && object.controller == actor)
                    .fold(0u8, |mask, object| mask | object.colors);
                if existing & (1 << color.index()) == 0 {
                    10
                } else {
                    0
                }
            }
            Action::ChooseCreatureType {
                creature_type: kind,
            } => observation
                .objects
                .iter()
                .filter(|object| {
                    ((object.zone == Zone::Battlefield && object.controller == actor)
                        || (object.zone == Zone::Graveyard && object.owner == actor))
                        && object.has_subtype(kind.name())
                })
                .count() as i128,
            Action::ChooseEntryValue { .. } => 0,
            Action::ChooseEntryLife { pay } => {
                // Avoid voluntary lethal payment; the example still exercises
                // both branches as life falls during games.
                if *pay
                    && observation.players[actor.index()].life
                        > i128::from(observation.entry_life.as_ref().unwrap().cost) + 4
                {
                    80
                } else if !*pay {
                    70
                } else {
                    -2
                }
            }
            Action::KeepRole { .. } | Action::KeepLegend { .. } => 100,
            Action::Keep | Action::FinishOpeningHand => 100,
            Action::BeginOnBattlefield { .. } => 110,
            Action::Mulligan => -100,
            Action::PlayLand { .. }
            | Action::PlayGraveyardPermanent {
                permanent_type: crate::PermanentType::Land,
                ..
            } => 90,
            Action::PlayGraveyardPermanent { .. } => 79,
            Action::CastExiledDuringResolution { .. } => 81,
            Action::FinishResolutionCasts => -1,
            Action::BeginCast { .. } => 80,
            // Once a complete target set is legal, commit it instead of
            // revisiting optional edits (which may otherwise oscillate).
            Action::ChooseSpellCopy { copy } => {
                if *copy {
                    100
                } else {
                    10
                }
            }
            Action::FinishStackTargets => 100,
            Action::AllocateTargetAmount { amount } => 100 + i128::from(*amount),
            Action::RestoreStackTarget { .. } => -1,
            Action::SetStackTarget { index, target } => observation
                .retarget
                .as_ref()
                .and_then(|r| {
                    r.item.program().map(|(_, p)| {
                        target_score(p, *target)
                            - target_score(p, r.item.targets[*index as usize].target)
                    })
                })
                .unwrap_or(0),
            Action::ChooseTarget { target } => {
                let item = observation.announcing.as_ref().unwrap();
                if item.kind == StackKind::Spell && item.source_card.aura().is_some() {
                    return aura_score(item.source_card, *target);
                }
                let program = item.program().map(|(_, program)| program).unwrap_or(&[]);
                target_score(program, *target)
            }
            Action::FinishTargets => 0,
            Action::ChooseMana { production } => mana_score(&observation, *production),
            Action::TapGrantedMana { production, .. } => {
                if observation.resolution_payment.is_some() {
                    return resolution_mana_score([0; 6], *production);
                }
                if tap_for_payment {
                    20 + mana_score(&observation, *production)
                } else {
                    -5
                }
            }
            Action::ActivateManaWithTapCost {
                production,
                payment,
                ..
            }
            | Action::ActivateMana {
                production,
                payment,
                ..
            } => {
                if observation.resolution_payment.is_some() {
                    return resolution_mana_score(*payment, *production);
                }
                if tap_for_payment {
                    20 + mana_score(&observation, *production)
                } else {
                    -5
                }
            }
            Action::BeginActivate { .. } => 35,
            Action::Activate {
                card,
                ability,
                target,
                ..
            } => {
                let object = observation
                    .objects
                    .iter()
                    .find(|object| object.id == *card)
                    .unwrap();
                let program = object.card.activations()[*ability as usize].program;
                target
                    .map(|target| target_score(program, target) - 25)
                    .unwrap_or(35)
            }
            Action::Cast { card, target, .. } => {
                let definition = observation
                    .objects
                    .iter()
                    .find(|o| o.id == *card)
                    .map(|o| o.card)
                    .or_else(|| {
                        observation
                            .known_library_top
                            .iter()
                            .flatten()
                            .find(|top| top.object.card == *card)
                            .map(|top| top.card)
                    })
                    .expect("legal cast source is visible to the acting player");
                if definition.aura().is_some() {
                    return target.map(|t| aura_score(definition, t)).unwrap_or(-10);
                }
                if definition.is_creature() || matches!(definition.effect(), Effect::Permanent) {
                    80
                } else {
                    let program = match definition.effect() {
                        Effect::Spell(_, program) => program,
                        _ => &[],
                    };
                    target
                        .map(|target| target_score(program, target))
                        .unwrap_or(70)
                }
            }
            Action::Attack { .. } | Action::AttackPlaneswalker { .. } | Action::Block { .. } => 60,
            Action::AssignDamage { amount, .. } => 50 + *amount,
            Action::Bottom { .. } | Action::Discard { .. } => 50,
            Action::PutTrigger { .. } => 50,
            Action::ChooseLibraryDestination { .. } => 50,
            Action::SelectLibrary { .. } => 60,
            Action::FinishLibrarySelection
            | Action::OrderLibraryBottom { .. }
            | Action::MillLookedCard { .. } => 50,
            Action::ArrangeLibrary { destination, .. } => {
                if *destination == crate::LibraryDestination::Top {
                    50
                } else {
                    40
                }
            }
            Action::ChooseOptional { accept } => {
                if *accept {
                    50
                } else {
                    0
                }
            }
            Action::TapMana { card } => {
                if observation.resolution_payment.is_some() {
                    let mut production = [0; 6];
                    production[observation
                        .objects
                        .iter()
                        .find(|o| o.id == *card)
                        .unwrap()
                        .card
                        .mana_color()
                        .unwrap()] = 1;
                    return resolution_mana_score([0; 6], production);
                }
                if tap_for_payment { 10 } else { -20 }
            }
            Action::ChooseEntryReplacement { source, intrinsic } => {
                let effect = observation
                    .entry_replacement
                    .as_ref()
                    .unwrap()
                    .effects
                    .iter()
                    .find(|effect| effect.source == *source && effect.intrinsic == *intrinsic)
                    .unwrap();
                if effect.kind == crate::EntryTapKind::Untapped {
                    5
                } else {
                    10
                }
            }
            Action::Pass | Action::FinishAttackers | Action::FinishBlockers => 0,
        }
    };
    actions
        .iter()
        .enumerate()
        .max_by_key(|(_, a)| score(a))
        .map(|(i, _)| i)
        .expect("legal action")
}

fn mana_score(observation: &crate::Observation, production: crate::Mana) -> i128 {
    let pool = observation.players[observation.viewer.index()].mana;
    observation
        .objects
        .iter()
        .filter(|object| object.owner == observation.viewer && object.zone == Zone::Hand)
        .map(|object| {
            let (required, _) = object.card.cost();
            (0..6)
                .map(|index| {
                    required[index]
                        .saturating_sub(pool[index])
                        .min(production[index]) as i128
                })
                .sum()
        })
        .max()
        .unwrap_or(0)
}
