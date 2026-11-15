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
    nonland: true,
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
