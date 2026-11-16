//! Shared exact rules needed by Star Trek E–M cards.
mod support;

use rustmage as _;
use rustmage::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SACRIFICE_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const GOWRON_CONDITION: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SACRIFICE_CREATURE,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSacrificedKlingon { skip: 2 },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const MILES_HISTORY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::PermanentReturnedToYourHandThisTurn,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const EXPEDITION_PLAY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileUntilNonlandPlayThisTurn],
    ),
    ..CardRules::EMPTY
};
const MIRROR_COHORT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::CopyTargetPlayersCreaturesWithExceptions(
            CopyExceptions {
                add_keywords: Keywords::MENACE,
                ..CopyExceptions::NONE
            },
        )],
    ),
    ..CardRules::EMPTY
};
const DISTINCT_LANDS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};
const CHEKOV_EXILE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileOpponentTopChooseLandPlayWhileExiled(
            AmountValue::Stat(StatValue::DistinctNames(DISTINCT_LANDS)),
        )],
    }],
    ..CardRules::EMPTY
};
const ARTIFACT_DAMAGE: TriggerCondition = TriggerCondition::PlayerDamaged {
    subject: DamageSubject::Source,
    combat_only: true,
    opponent_only: false,
    one_or_more: false,
};
const HIVE_GRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledArtifactCreatureTrigger { ability: 0 }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&ARTIFACT_DAMAGE),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayControllerLife { amount: 1, skip: 2 },
            Instruction::LoseLife(PlayerSet::Controller, 1),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const ARTIFACT_ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..CardRules::EMPTY
};
const KAI_NAMED_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NonlandCardName),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentCastsOrActivatesChosenName,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::EventPlayer, 3)],
    }],
    ..CardRules::EMPTY
};
const NAMED_ACTIVATION: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget exhausted");
}

#[test]
fn sacrificed_klingon_uses_actual_predeparture_subtype_and_owns_index_restore() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GOWRON_CONDITION)]);
    for (sacrifice, qualifies) in [("General Chang, Cold Warrior", true), ("Bear Cub", false)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let material = if qualifies {
            spawn(
                &mut game,
                player,
                Card::from_name(sacrifice).unwrap(),
                Zone::Battlefield,
            )
        } else {
            put(&mut game, player, sacrifice, Zone::Battlefield)
        };
        put(&mut game, player, "Island", Zone::Library);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let action = Action::ChooseAdditionalCost {
            object: game.object_ref(material),
        };
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        pay(&mut game);
        drain(&mut game);
        assert_eq!(game.object_view(material).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(qualifies)
        );
        assert_eq!(
            game.state.players[player.index()].life,
            20 - i128::from(qualifies)
        );
    }
}

#[test]
fn returned_land_counts_for_owner_but_unrelated_departure_does_not() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MILES_HISTORY)]);
    for returned in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let permanent = put(&mut game, player, "Forest", Zone::Battlefield);
        put(&mut game, player, "Island", Zone::Library);
        game.move_object(
            permanent,
            if returned {
                Zone::Hand
            } else {
                Zone::Graveyard
            },
        );
        assert_eq!(
            game.state.players[player.index()]
                .turn_facts
                .permanent_returned_to_your_hand,
            returned
        );
        game.state.step = Step::Main2;
        game.give_priority();
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        drain(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(returned)
        );
    }
}

#[test]
fn first_exiled_nonland_gets_ordinary_play_only_for_this_turn() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &EXPEDITION_PLAY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first_nonland = put(&mut game, player, "Bear Cub", Zone::Library);
    let exiled_land = put(&mut game, player, "Forest", Zone::Library);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, spell, None);
    drain(&mut game);
    assert_eq!(game.object_view(exiled_land).zone, Zone::Exile);
    assert_eq!(game.object_view(first_nonland).zone, Zone::Exile);
    let permissions = game.observe(player).exile_permissions;
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].object, game.object_ref(first_nonland));
    assert!(!permissions[0].cast_only);
    assert_eq!(
        permissions[0].expiry,
        ExilePlayExpiry::Turn(game.state.turn)
    );
    reject(&mut game, Action::PlayLand { card: exiled_land });
    act(
        &mut game,
        Action::AnnounceExile {
            card: first_nonland,
        },
    );
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(first_nonland).zone, Zone::Battlefield);
}

#[test]
fn targeted_opponent_creature_cohort_is_frozen_and_all_copies_gain_menace() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &MIRROR_COHORT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let officer = spawn(
        &mut game,
        player.other(),
        Card::from_catalogue_id(6008),
        Zone::Battlefield,
    );
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    let first_new = game.state.objects.len();
    cast(&mut game, spell, Some(Target::Player(player.other())));
    drain(&mut game);
    let copies: Vec<_> = (first_new..game.state.objects.len())
        .map(|id| id as CardId)
        .filter(|&id| game.object_view(id).zone == Zone::Battlefield)
        .collect();
    assert_eq!(copies.len(), 2);
    assert_eq!(stats(&game, own), (2, 2));
    assert_eq!(stats(&game, other), (2, 2));
    assert_eq!(stats(&game, officer), (1, 1));
    for copy in copies {
        assert_eq!(game.object_view(copy).controller, player);
        assert!(game.keywords(copy).contains(Keywords::MENACE));
    }
}

#[test]
fn opponent_top_exile_selects_only_their_land_and_grants_enduring_exact_play() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CHEKOV_EXILE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Island", Zone::Battlefield);
    let untouched = put(&mut game, player.other(), "Island", Zone::Library);
    let chosen = put(&mut game, player.other(), "Forest", Zone::Library);
    let unchosen = put(&mut game, player.other(), "Bear Cub", Zone::Library);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, source, None);
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::MilledSelection { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.pending, Pending::MilledSelection { player });
    assert_eq!(game.object_view(untouched).zone, Zone::Library);
    assert_eq!(game.object_view(unchosen).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).milled_choice.unwrap().candidates,
        [game.object_ref(chosen)]
    );
    assert!(game.observe(player.other()).milled_choice.is_none());
    let unchosen_ref = game.object_ref(unchosen);
    reject(
        &mut game,
        Action::SelectMilled {
            object: unchosen_ref,
        },
    );
    let select = Action::SelectMilled {
        object: game.object_ref(chosen),
    };
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == select)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, select);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    drain(&mut game);
    let permissions = game.observe(player).exile_permissions;
    assert_eq!(permissions.len(), 1);
    assert_eq!(permissions[0].object, game.object_ref(chosen));
    assert_eq!(permissions[0].expiry, ExilePlayExpiry::WhileExiled);
    reject(&mut game, Action::PlayLand { card: unchosen });
    act(&mut game, Action::PlayLand { card: chosen });
    assert_eq!(game.object_view(chosen).zone, Zone::Battlefield);
    assert_eq!(game.object_view(chosen).controller, player);
    assert_eq!(game.object_view(chosen).owner, player.other());
}

#[test]
fn only_artifact_creatures_acquire_the_paid_combat_damage_draw_trigger() {
    let artifact = Card::from_name("Interstellar Effigy").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &HIVE_GRANT),
        (artifact, &ARTIFACT_ATTACKER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let artifact_attacker = spawn(&mut game, player, artifact, Zone::Battlefield);
    let ordinary_attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, player, "Island", Zone::Library);
    game.state.objects[provider as usize].summoning_sick = false;
    game.state.objects[artifact_attacker as usize].summoning_sick = false;
    game.state.objects[ordinary_attacker as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(
        &mut game,
        Action::Attack {
            card: artifact_attacker,
        },
    );
    act(
        &mut game,
        Action::Attack {
            card: ordinary_attacker,
        },
    );
    act(&mut game, Action::FinishAttackers);
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::EffectOptional { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    assert!(matches!(game.state.pending, Pending::EffectOptional { player: p, .. } if p == player));
    let accepted = game.clone();
    act(&mut game, Action::ChooseOptional { accept: false });
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    let mut accepted = accepted;
    act(&mut accepted, Action::ChooseOptional { accept: true });
    drain(&mut accepted);
    assert_eq!(accepted.state.players[player.index()].life, 19);
    assert_eq!(
        accepted.state.players[player.index()].cards_drawn_this_turn,
        1
    );
}

#[test]
fn chosen_name_observes_opponent_cast_and_activation_and_freezes_actor() {
    let bow = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &KAI_NAMED_OBSERVER),
        (bow, &NAMED_ACTIVATION),
    ]);
    for (name, activate) in [("Opt", false), ("Short Bow", true)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let kai = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        cast(&mut game, kai, None);
        for _ in 0..8 {
            if matches!(game.state.pending, Pending::EntryValue { .. }) {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let chosen_name = CardName::from_name(name).unwrap();
        reject(
            &mut game,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CardName(CardName::from_name("Forest").unwrap()),
            },
        );
        let action = Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(chosen_name),
        };
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        drain(&mut game);
        let opposing_source = if activate {
            spawn(&mut game, player.other(), bow, Zone::Battlefield)
        } else {
            put(&mut game, player.other(), "Opt", Zone::Hand)
        };
        act(&mut game, Action::Pass);
        if activate {
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: opposing_source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut game);
        } else {
            cast(&mut game, opposing_source, None);
        }
        for _ in 0..16 {
            if game.state.players[player.other().index()].life == 17 {
                break;
            }
            choose(&mut game, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.state.players[player.other().index()].life, 17);
        assert_eq!(game.object_view(kai).zone, Zone::Battlefield);
    }
}

const GUL_DRAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DrainLifeScaled(
            AmountValue::OpponentLifeLostThisTurn,
        )],
    ),
    ..CardRules::EMPTY
};

#[test]
fn gul_drain_uses_gross_prior_opponent_loss_and_gains_actual_drain() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &GUL_DRAIN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.other().index()].life = 16;
    game.state.players[player.other().index()].life_lost_this_turn = 4;
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 12);
    assert_eq!(game.state.players[player.index()].life, 24);
    assert_eq!(
        game.state.players[player.other().index()].life_lost_this_turn,
        8
    );
}

const SPOCK_DRAW: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::DoubleDraw],
    }],
    ..CardRules::EMPTY
};

#[test]
fn unconditional_draw_replacement_applies_to_each_draw_and_owned_restore() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPOCK_DRAW)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let checkpoint = game.snapshot();
    let mut branch = game.clone();
    branch.draw_many(player, 2);
    assert_eq!(
        branch.state.players[player.index()].cards_drawn_this_turn,
        4
    );
    assert_eq!(
        branch
            .state
            .objects
            .iter()
            .filter(|object| object.owner == player && object.zone == Zone::Hand)
            .count(),
        4
    );
    branch.restore(&checkpoint).unwrap();
    assert_eq!(branch.snapshot(), checkpoint);
    game.draw_many(player.other(), 1);
    assert_eq!(
        game.state.players[player.other().index()].cards_drawn_this_turn,
        1
    );
}

#[test]
fn copied_spock_provider_doubles_draws_and_restores_when_copy_ends() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPOCK_DRAW)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let copy = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let source = game.object_ref(copy);
    let baseline = game.snapshot();
    game.set_copy_recipe(source, CopyRecipe::new(Card::GrizzlyBears));
    let copied = game.snapshot();
    game.draw_many(player, 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    game.restore(&copied).unwrap();
    assert_eq!(game.snapshot(), copied);
    game.clear_copy_recipe(source);
    game.draw_many(player, 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.restore(&baseline).unwrap();
    assert_eq!(game.snapshot(), baseline);
}

const GEORDI_DOUBLE: ReplacementSpec = ReplacementSpec::ControlledArtifactCountersPlaced {
    kind: CounterKind::Charge,
    factor: 2,
};
const GEORDI: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, GEORDI_DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[GEORDI_DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PLACE_CHARGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::Charge,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn geordi_doubles_only_own_charge_placements_on_owned_artifacts() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GEORDI), (opt, &PLACE_CHARGE)]);
    for (placer, recipient, expected) in [
        (Player::P0, "Short Bow", 2),
        (Player::P0, "Bear Cub", 1),
        (Player::P1, "Short Bow", 1),
    ] {
        let mut game = scenario(Player::P0);
        put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut game, Player::P0, recipient, Zone::Battlefield);
        let spell = spawn(&mut game, placer, opt, Zone::Hand);
        if placer == Player::P1 {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, spell, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(game.counter_count(target, CounterKind::Charge), expected);
    }
}

#[test]
fn copied_geordi_provider_replaces_charge_placement_and_clearing_copy_removes_it() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GEORDI), (opt, &PLACE_CHARGE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let copy = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let target = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let copy_ref = game.object_ref(copy);
    let baseline = game.snapshot();
    game.set_copy_recipe(copy_ref, CopyRecipe::new(Card::GrizzlyBears));
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.counter_count(target, CounterKind::Charge), 2);
    game.restore(&baseline).unwrap();
    assert_eq!(game.snapshot(), baseline);
    game.clear_copy_recipe(copy_ref);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.counter_count(target, CounterKind::Charge), 1);
}

const LIFE_LOCK: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantLoseFromNonpositiveLife],
    }],
    ..CardRules::EMPTY
};
const DOUBLE_LIFE: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::DoubleControllerLifeTotal]),
    ..CardRules::EMPTY
};

#[test]
fn life_total_doubling_handles_positive_and_negative_totals() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LIFE_LOCK), (opt, &DOUBLE_LIFE)]);
    for (starting, expected) in [(7, 14), (-3, -6)] {
        let player = Player::P0;
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.state.players[player.index()].life = starting;
        let spell = spawn(&mut game, player, opt, Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, expected);
        assert_eq!(game.outcome(), None);
    }
}

const OWN_LAND: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 2,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const ERICA_LAND_RETURN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CaptureEntryResult,
                Instruction::PutHandLand { tapped: false },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::CapturedEntryResultCount,
                    minimum: 1,
                    skip: 1,
                },
                Instruction::ScheduleStep {
                    step: TriggerStep::End,
                    relation: CastRelation::Any,
                    ability: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseControlledPermanentToHand(OWN_LAND)],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn optional_hand_land_schedules_return_only_after_accepting_entry() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ERICA_LAND_RETURN)]);
    struct ResetPrevention;
    impl Drop for ResetPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _reset = ResetPrevention;
    for (accept, prevented) in [(false, false), (true, false), (true, true)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let erica = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let prior = put(&mut game, player, "Forest", Zone::Battlefield);
        let from_hand = put(&mut game, player, "Island", Zone::Hand);
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| {
            rule.set(prevented.then(|| game.object_ref(from_hand)));
        });
        game.tap_objects(&[erica]);
        for _ in 0..8 {
            if game.legal_actions().iter().any(|action| {
                matches!(
                    action,
                    Action::PutHandLand { .. } | Action::DeclineHandEntry
                )
            }) {
                break;
            }
            let actions = game.legal_actions();
            let action = actions
                .iter()
                .find(|action| matches!(action, Action::Pass | Action::PutTrigger { .. }))
                .unwrap_or_else(|| panic!("hand entry wait: {:?}: {actions:?}", game.state.pending))
                .clone();
            act(&mut game, action);
        }
        if accept {
            let object = game.object_ref(from_hand);
            act(&mut game, Action::PutHandLand { object });
        } else {
            act(&mut game, Action::DeclineHandEntry);
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(from_hand).zone,
            if accept && !prevented {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        game.begin_step(Step::End);
        game.prepare_triggers();
        if accept && !prevented {
            let object = game.object_ref(prior);
            for _ in 0..8 {
                if game
                    .legal_actions()
                    .contains(&Action::ChooseEffectObject { object })
                {
                    break;
                }
                let actions = game.legal_actions();
                let action = actions
                    .iter()
                    .find(|action| matches!(action, Action::Pass | Action::PutTrigger { .. }))
                    .unwrap_or_else(|| {
                        panic!("delayed return wait: {:?}: {actions:?}", game.state.pending)
                    })
                    .clone();
                act(&mut game, action);
            }
            act(&mut game, Action::ChooseEffectObject { object });
            if game.legal_actions().contains(&Action::FinishEffectChoice) {
                act(&mut game, Action::FinishEffectChoice);
            }
            settle(&mut game);
            assert_eq!(game.object_view(prior).zone, Zone::Hand);
        } else {
            settle(&mut game);
            assert_eq!(game.object_view(prior).zone, Zone::Battlefield);
        }
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

const CHARGE_SUNBURST: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        x_symbols: 1,
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ColorsSpentKind(CounterKind::Charge),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn sunburst_places_charge_counters_for_distinct_paid_colors() {
    let explosives = Card::from_name("Engineered Explosives").unwrap();
    let _rules = fixture_rules::activate(&[(explosives, &CHARGE_SUNBURST)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, explosives, Zone::Hand);
    game.state.players[player.index()].mana = [1, 1, 0, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: source });
    act(&mut game, Action::ChooseX { value: 2 });
    act(
        &mut game,
        Action::PayCosts {
            payment: [1, 1, 0, 0, 0, 0],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(source, CounterKind::Charge), 2);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
}

const REPLACE_TYPES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ChangeCreatureTypesUntilEndTurn {
            subject: Subject::Targets,
            types: &["Doctor"],
            additional: false,
        }],
    ),
    ..CardRules::EMPTY
};
const ADD_TYPES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ChangeCreatureTypesUntilEndTurn {
            subject: Subject::Targets,
            types: &["Warrior"],
            additional: true,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn temporary_creature_type_replacement_and_addition_expire_with_owned_restore() {
    let opt = Card::from_name("Opt").unwrap();
    let shock = Card::from_name("Shock").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &REPLACE_TYPES), (shock, &ADD_TYPES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let replace = spawn(&mut game, player, opt, Zone::Hand);
    let add = spawn(&mut game, player, shock, Zone::Hand);
    let checkpoint = game.snapshot();
    cast(&mut game, replace, Some(Target::Card(creature)));
    settle(&mut game);
    assert!(game.object_view(creature).has_subtype("Doctor"));
    assert!(!game.object_view(creature).has_subtype("Bear"));
    cast(&mut game, add, Some(Target::Card(creature)));
    settle(&mut game);
    assert!(game.object_view(creature).has_subtype("Doctor"));
    assert!(game.object_view(creature).has_subtype("Warrior"));
    let active = game.snapshot();
    let mut branch = game.clone();
    branch.state.step = Step::Cleanup;
    branch.finish_cleanup();
    assert!(branch.object_view(creature).has_subtype("Bear"));
    assert!(!branch.object_view(creature).has_subtype("Doctor"));
    assert!(!branch.object_view(creature).has_subtype("Warrior"));
    branch.restore(&active).unwrap();
    assert_eq!(branch.snapshot(), active);
    game.restore(&checkpoint).unwrap();
    assert_eq!(game.snapshot(), checkpoint);
}
