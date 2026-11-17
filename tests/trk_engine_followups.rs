//! Rules contracts for Star Trek engine dependencies discovered during authorship.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OPPONENT: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: true,
}));
const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const ATTACK_COMPANY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantAttackAlone],
    }],
    ..CardRules::EMPTY
};
const TAPPED_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::CreateTappedCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const PRIVATE_EXILE_DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        OPPONENT,
        &[Instruction::LookTargetHandMayExileNonlandThenVictimDraw],
    ),
    ..CardRules::EMPTY
};
const PRIVATE_EXILE_TAX: CardRules = CardRules {
    effect: Effect::Spell(
        OPPONENT,
        &[Instruction::LookTargetHandMayExileNonlandWithPlaySurcharge(
            std::num::NonZeroU32::new(1).unwrap(),
        )],
    ),
    ..CardRules::EMPTY
};
const OWN_TURN_SPACECRAFT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledSubtypeKeywordsOnYourTurn {
            subtype: "Spacecraft",
            keywords: Keywords::HEXPROOF,
        }],
    }],
    ..CardRules::EMPTY
};
const COUNTER_BEARING_BONUS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PutCounterBearingControlledCreatureCounters {
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const LASTING_BASE: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE,
        &[Instruction::SetBaseStatsPermanent {
            subject: Subject::Targets,
            power: Some(1),
            toughness: Some(1),
        }],
    ),
    ..CardRules::EMPTY
};
const DRAW_ONE_THRESHOLD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DrawRemembered(AmountValue::Fixed(1)),
            Instruction::IfRememberedDrawsBelow {
                minimum: 2,
                skip: 1,
            },
            Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Controller,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const DRAW_TWO_THRESHOLD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DrawRemembered(AmountValue::Fixed(2)),
            Instruction::IfRememberedDrawsBelow {
                minimum: 2,
                skip: 1,
            },
            Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Controller,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn advance_to_choice(game: &mut Game) {
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::Pass));
    }
    panic!("private hand choice did not open");
}

fn hand_size(game: &Game, owner: Player) -> usize {
    game.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Hand && o.owner == owner)
        .count()
}

#[test]
fn pet_sehlat_attacks_with_company_but_blocks_alone() {
    let _rules = fixture_rules::activate(&[(Card::DrakeToken, &ATTACK_COMPANY)]);
    let mut attack_game = scenario(Player::P0);
    let pet = spawn(
        &mut attack_game,
        Player::P0,
        Card::DrakeToken,
        Zone::Battlefield,
    );
    attack_game.state.step = Step::Attackers;
    attack_game.state.pending = Pending::Attackers;
    admit(&attack_game, Action::Attack { card: pet });
    let companion = put(
        &mut attack_game,
        Player::P0,
        "Grizzly Bears",
        Zone::Battlefield,
    );
    act(&mut attack_game, Action::Attack { card: pet });
    reject(&mut attack_game, Action::FinishAttackers);
    act(&mut attack_game, Action::Attack { card: companion });
    act(&mut attack_game, Action::FinishAttackers);

    let mut block_game = scenario(Player::P0);
    let attacker = put(
        &mut block_game,
        Player::P0,
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let blocker = spawn(
        &mut block_game,
        Player::P1,
        Card::DrakeToken,
        Zone::Battlefield,
    );
    block_game.state.step = Step::Attackers;
    block_game.state.pending = Pending::Attackers;
    act(&mut block_game, Action::Attack { card: attacker });
    act(&mut block_game, Action::FinishAttackers);
    act(&mut block_game, Action::Pass);
    act(&mut block_game, Action::Pass);
    assert_eq!(block_game.state.pending, Pending::Blockers);
    act(&mut block_game, Action::Block { blocker, attacker });
    act(&mut block_game, Action::FinishBlockers);
    assert_eq!(block_game.state.blocks, [(blocker, attacker)]);
}

#[test]
fn tapped_copy_uses_token_replacements_and_enters_tapped() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &TAPPED_COPY)]);
    let mut game = scenario(Player::P0);
    let target = spawn(&mut game, Player::P0, Card::DrakeToken, Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    settle(&mut game);
    let copies: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (id as CardId != target
                && object.zone == Zone::Battlefield
                && object.controller == Player::P0
                && object.is_token())
            .then_some(id as CardId)
        })
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(game.object_view(copies[0]).tapped);
    assert!(!game.object_view(target).tapped);
    assert_eq!(stats(&game, copies[0]), stats(&game, target));
}

#[test]
fn private_exile_draws_only_after_a_nonland_is_exiled_and_branches_cleanly() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &PRIVATE_EXILE_DRAW)]);
    let mut game = scenario(Player::P0);
    let victim = Player::P1;
    let nonland = put(&mut game, victim, "Grizzly Bears", Zone::Hand);
    let land = put(&mut game, victim, "Forest", Zone::Hand);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    let initial_hand = hand_size(&game, victim);
    cast(&mut game, spell, Some(Target::Player(victim)));
    advance_to_choice(&mut game);
    let view = game.observe(Player::P0).hand_choice.unwrap();
    assert_eq!(view.candidates, [game.object_ref(nonland)]);
    assert_eq!(view.inspected_cards.unwrap().len(), 2);
    assert!(game.observe(victim).hand_choice.is_none());
    let land_ref = game.object_ref(land);
    reject(
        &mut game,
        Action::ChooseHandCard {
            object: Some(land_ref),
        },
    );
    let checkpoint = game.snapshot();
    let action = Action::ChooseHandCard {
        object: Some(game.object_ref(nonland)),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let revision = game.revision();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(nonland).zone, Zone::Exile);
    assert_eq!(hand_size(&game, victim), initial_hand);

    let mut declined = game.clone();
    declined.restore(&checkpoint).unwrap();
    act(&mut declined, Action::ChooseHandCard { object: None });
    assert_eq!(declined.object_view(nonland).zone, Zone::Hand);
    assert_eq!(hand_size(&declined, victim), initial_hand);
}

#[test]
fn private_taxed_exile_preserves_victim_hand_privacy() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &PRIVATE_EXILE_TAX)]);
    let mut game = scenario(Player::P0);
    let card = put(&mut game, Player::P1, "Grizzly Bears", Zone::Hand);
    let land = put(&mut game, Player::P1, "Forest", Zone::Hand);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(Player::P1)));
    advance_to_choice(&mut game);
    assert!(game.observe(Player::P1).hand_choice.is_none());
    assert_eq!(
        game.observe(Player::P0)
            .hand_choice
            .unwrap()
            .inspected_cards
            .unwrap()
            .len(),
        2
    );
    let land_ref = game.object_ref(land);
    reject(
        &mut game,
        Action::ChooseHandCard {
            object: Some(land_ref),
        },
    );
    let card_ref = game.object_ref(card);
    act(
        &mut game,
        Action::ChooseHandCard {
            object: Some(card_ref),
        },
    );
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert!(!game.state.exile_permissions.is_empty());
}

#[test]
fn counter_bearing_permanent_query_and_placement_include_the_right_objects() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &COUNTER_BEARING_BONUS)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let land = put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 1);
    game.add_counter(land, CounterKind::Stun, 1);
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    assert_eq!(
        CharacteristicWorld::value(&game, StatValue::CounterBearingPermanents, source),
        2
    );
    assert_eq!(
        CharacteristicWorld::value(&game, StatValue::CounterBearingCreatures, source),
        1
    );
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.object_view(source).counters.plus_one, 2);
    assert_eq!(game.object_view(other).counters.plus_one, 0);
    assert_eq!(game.object_view(enemy).counters.plus_one, 1);
    assert_eq!(
        CharacteristicWorld::value(&game, StatValue::CounterBearingPermanents, source),
        2
    );
}

#[test]
fn spacecraft_hexproof_tracks_turn_controller_and_live_subtype() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &OWN_TURN_SPACECRAFT)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let craft = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Class-F Shuttlecraft").unwrap(),
        Zone::Battlefield,
    );
    assert_eq!(
        CharacteristicWorld::value(&game, StatValue::SourceNoncreature, craft),
        1
    );
    assert!(game.keywords(craft).contains(Keywords::HEXPROOF));
    game.state.active = Player::P1;
    assert!(!game.keywords(craft).contains(Keywords::HEXPROOF));
    game.state.active = Player::P0;
    let craft_ref = game.object_ref(craft);
    game.record_control(craft_ref, Player::P1, None);
    assert!(!game.keywords(craft).contains(Keywords::HEXPROOF));
    assert_eq!(
        CharacteristicWorld::value(&game, StatValue::SourceNoncreature, source),
        0
    );
}

#[test]
fn attacked_source_guard_uses_the_exact_declared_incarnation() {
    let mut game = scenario(Player::P0);
    let attacker = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let exact = game.object_ref(attacker);
    assert!(!game.step_guard(StepGuard::SourceAttackedThisTurn, Player::P0, exact, false));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    assert!(game.step_guard(StepGuard::SourceAttackedThisTurn, Player::P0, exact, true));
    game.move_object(attacker, Zone::Exile);
    game.move_object(attacker, Zone::Battlefield);
    assert!(!game.step_guard(
        StepGuard::SourceAttackedThisTurn,
        Player::P0,
        game.object_ref(attacker),
        false,
    ));
}

#[test]
fn equipment_entry_guard_remembers_actual_entry_after_departure() {
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let source_ref = game.object_ref(source);
    let guard = StepGuard::EquipmentOrSpacecraftEnteredThisTurn;
    assert!(!game.step_guard(guard, Player::P0, source_ref, false));
    let equipment = put(&mut game, Player::P0, "Short Bow", Zone::Hand);
    cast(&mut game, equipment, None);
    settle(&mut game);
    assert!(game.step_guard(guard, Player::P0, source_ref, false));
    game.move_object(equipment, Zone::Graveyard);
    assert!(game.step_guard(guard, Player::P0, source_ref, false));
    assert!(!game.step_guard(guard, Player::P1, source_ref, false));
}

#[test]
fn lasting_base_stats_survive_cleanup_but_not_a_new_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &LASTING_BASE)]);
    let mut game = scenario(Player::P0);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (1, 1));
    game.expire_base_stats();
    assert_eq!(stats(&game, creature), (1, 1));
    game.move_object(creature, Zone::Exile);
    game.move_object(creature, Zone::Battlefield);
    assert_eq!(stats(&game, creature), (2, 2));
}

#[test]
fn remembered_draw_threshold_uses_completed_draw_count() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &DRAW_ONE_THRESHOLD),
        (Card::from_name("Shock").unwrap(), &DRAW_TWO_THRESHOLD),
    ]);
    let mut game = scenario(Player::P0);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let one = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, one, None);
    settle(&mut game);
    assert_eq!(game.object_view(creature).counters.plus_one, 0);
    let two = put(&mut game, Player::P0, "Shock", Zone::Hand);
    cast(&mut game, two, None);
    settle(&mut game);
    assert_eq!(game.object_view(creature).counters.plus_one, 2);
}
