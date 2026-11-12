//! Colored reductions and exact-incarnation counter-bound land types.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];

const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::InstantSorcery,
    amount: SpellReductionAmount::ColoredStat {
        color: ChoiceColor::Blue,
        value: StatValue::ControlledLandsWithCounter(CounterKind::Flood),
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::InstantSorcerySpellsCastThisTurn,
        minimum: None,
        maximum: Some(0),
    },
};

const FLOOD_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Flood,
                amount: 1,
            },
            Instruction::AddBasicLandTypeWhileCounter {
                subject: Subject::Targets,
                kind: BasicLandType::Island,
                counter: CounterKind::Flood,
            },
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    }],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const COUNTER_PROHIBITION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(2)],
    }],
    ..CardRules::EMPTY
};

fn flooded_lands(game: &mut Game, player: Player, count: usize) {
    for _ in 0..count {
        let id = put(game, player, "Mountain", Zone::Battlefield);
        game.add_counter(id, CounterKind::Flood, 2);
        game.state.objects[id as usize].tapped = true;
    }
}

fn cast_action(card: CardId, payment: Mana) -> Action {
    Action::Cast {
        card,
        target: None,
        payment,
    }
}

fn assert_payment(game: &Game, payment: Mana) {
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        payment
    );
}

fn enter_provider(game: &mut Game, player: Player, land: CardId) -> CardId {
    let source = put(game, player, "Hill Giant", Zone::Hand);
    cast(game, source, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    settle(game);
    source
}

#[test]
fn direct_and_announced_spells_reduce_blue_before_generic_and_count_lands_once() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &FLOOD_PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        for (name, lands, providers, payment) in [
            ("Opt", 1, 1, [0; 6]),
            ("Divination", 1, 2, GREEN),
            ("Dragon Fodder", 2, 1, RED),
        ] {
            let mut game = scenario(player);
            for _ in 0..providers {
                put(&mut game, player, "Hill Giant", Zone::Battlefield);
            }
            flooded_lands(&mut game, player, lands);
            flooded_lands(&mut game, player.other(), 3);
            let nonland = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            game.add_counter(nonland, CounterKind::Flood, 1);
            let spell = put(&mut game, player, name, Zone::Hand);
            game.state.players[player.index()].mana = payment;

            let direct = cast_action(spell, payment);
            let staged = Action::AnnounceCast { card: spell };
            assert!(game.legal_actions().contains(&direct), "{name}");
            assert!(game.legal_actions().contains(&staged), "{name}");
            let mut announced = game.clone();
            act(&mut game, direct);
            act(&mut announced, staged);
            act(&mut announced, Action::PayCosts { payment });
            for branch in [&game, &announced] {
                assert_payment(branch, payment);
                assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
                assert_eq!(
                    branch.state.players[player.index()].instant_sorcery_cast_this_turn,
                    1
                );
            }
        }
    }
}

#[test]
fn actual_casts_consume_the_discount_even_if_countered_or_before_the_provider_enters() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &FLOOD_PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        flooded_lands(&mut game, player, 1);
        let first = put(&mut game, player, "Opt", Zone::Hand);
        game.state.players[player.index()].mana = BLUE;
        act(&mut game, cast_action(first, BLUE));
        act(&mut game, Action::Pass);
        let negate = put(&mut game, player.other(), "Negate", Zone::Hand);
        cast(&mut game, negate, Some(Target::Spell(first)));
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let second = put(&mut game, player, "Opt", Zone::Hand);
        let free = cast_action(second, [0; 6]);
        reject(&mut game, free.clone());
        reject(&mut game, Action::AnnounceCast { card: second });

        game.state.step = Step::End;
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.state.active, player.other());
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(player));
        act(&mut game, free);
        assert_payment(&game, [0; 6]);
        assert_eq!(
            game.state.players[player.index()].instant_sorcery_cast_this_turn,
            1
        );
    }
}

#[test]
fn x_and_flashback_use_the_total_cost_of_the_chosen_route() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &FLOOD_PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        flooded_lands(&mut game, player, 3);
        let x_spell = put(&mut game, player, "Mind Spring", Zone::Hand);
        let flashback = put(&mut game, player, "Think Twice", Zone::Graveyard);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 0];

        let mut from_graveyard = game.clone();
        act(
            &mut from_graveyard,
            Action::AnnounceFlashback { card: flashback },
        );
        act(&mut from_graveyard, Action::PayCosts { payment: [0; 6] });
        assert_payment(&from_graveyard, [0; 6]);
        settle(&mut from_graveyard);
        assert_eq!(from_graveyard.object_view(flashback).zone, Zone::Exile);

        act(&mut game, Action::AnnounceCast { card: x_spell });
        reject(&mut game, Action::ChooseX { value: 4 });
        act(&mut game, Action::ChooseX { value: 3 });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_payment(&game, [0, 0, 0, 0, 2, 0]);
        assert_eq!(game.state.stack.last().unwrap().mana_value(), 5);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
    }
}

#[test]
fn island_ability_persists_only_while_the_same_land_retains_its_counters() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &FLOOD_PROVIDER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Mountain", Zone::Battlefield);
    let original = game.object_ref(land);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let source = enter_provider(&mut game, player, land);
    assert_eq!(game.counter_count(land, CounterKind::Flood), 2);
    game.move_object(source, Zone::Graveyard);
    assert!(game.object_view(land).has_subtype("Island"));
    assert!(game.object_view(land).has_subtype("Mountain"));
    game.state.players[player.index()].mana = [0; 6];

    let before = game.snapshot();
    let mut projection = FundingState::from_game(
        &game,
        &FundingGoal {
            actor: player,
            required: BLUE,
            generic: 0,
            reserve: None,
            source: None,
        },
    );
    assert!(projection.remove_cost_counter(original, CounterKind::Flood, 2));
    assert!(
        projection
            .subtypes(land)
            .has(projection.definition(land), "Island")
    );
    assert!(projection.remove_cost_counter(original, CounterKind::Flood, 2));
    assert!(
        !projection
            .subtypes(land)
            .has(projection.definition(land), "Island")
    );
    assert!(projection.visit_mana_source_counter_placement(
        original,
        Some((CounterKind::Flood, 1)),
        false,
        &mut |projected| {
            assert!(projected.counter_count(land, CounterKind::Flood) > 0);
            assert!(
                !projected
                    .subtypes(land)
                    .has(projected.definition(land), "Island")
            );
            true
        }
    ));
    assert_eq!(game.snapshot(), before);

    game.pay_counters(land, &Counters::of(CounterKind::Flood, 1));
    let tap_blue = Action::TapMana { card: land };
    let tap_added_red = Action::TapGrantedMana {
        card: land,
        grantor: original,
        production: RED,
    };
    assert!(game.legal_actions().contains(&tap_added_red));
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == tap_blue)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, tap_blue.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[player.index()].mana, BLUE);

    game.pay_counters(land, &Counters::of(CounterKind::Flood, 1));
    game.add_counter(land, CounterKind::Flood, 1);
    game.state.objects[land as usize].tapped = false;
    reject(&mut game, tap_added_red);
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 1, 0, 0]);
    assert!(!game.object_view(land).has_subtype("Island"));

    indexed.restore(&before).unwrap();
    indexed.move_object(land, Zone::Exile);
    indexed.move_object(land, Zone::Battlefield);
    indexed.add_counter(land, CounterKind::Flood, 1);
    assert_ne!(indexed.object_ref(land), original);
    assert!(!indexed.object_view(land).has_subtype("Island"));
}

#[test]
fn prevented_counter_placement_can_only_start_a_duration_if_one_is_already_present() {
    let lock = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &FLOOD_PROVIDER),
        (lock, &COUNTER_PROHIBITION),
    ]);
    let player = Player::P0;
    for prior in [0, 1] {
        let mut game = scenario(player);
        let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
        game.add_counter(land, CounterKind::Flood, prior);
        spawn(&mut game, player, lock, Zone::Battlefield);
        enter_provider(&mut game, player, land);
        assert_eq!(game.counter_count(land, CounterKind::Flood), prior);
        assert!(game.object_view(land).has_subtype("Forest"));
        assert_eq!(game.object_view(land).has_subtype("Island"), prior > 0);
    }
}
