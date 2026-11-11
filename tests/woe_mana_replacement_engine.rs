//! Replacement of original tap mana and preservation of its exact payment route.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ZERO: Mana = [0; 6];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const BLACK: Mana = [0, 0, 1, 0, 0, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];

const MULTIPLIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledBasicLandManaMultiplier(3)],
    }],
    ..CardRules::EMPTY
};
const TRIGGER_BONUS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledLandTapManaBonus(GREEN)],
    }],
    ..CardRules::EMPTY
};
const TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const LATE_LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..TAP.cost
        },
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Choice {
                mask: (1 << ChoiceColor::Black.index()) | (1 << ChoiceColor::Red.index()),
                amount: 1,
            },
            quality: &ManaQuality {
                restriction: ManaRestriction::InstantSorcery,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        ..TAP
    }],
    ..CardRules::EMPTY
};
const RED_GREEN: Mana = [0, 0, 0, 1, 1, 0];
const COMBINED_LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(ManaProduction::Fixed(RED_GREEN))],
        ..TAP
    }],
    ..CardRules::EMPTY
};
const SPLIT_LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(ManaProduction::Split(ManaSplit {
            first: ChoiceColor::Blue,
            second: ChoiceColor::Red,
            total: 4,
        }))],
        ..TAP
    }],
    ..CardRules::EMPTY
};

fn provider() -> Card {
    Card::from_name("A Realm Reborn").unwrap()
}

fn start_late_mana(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        payment: ZERO,
        production: ZERO,
    }
}

#[test]
fn projected_payment_respects_capacity_of_replaced_and_trigger_mana_together() {
    let bonus = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[
        (provider(), &MULTIPLIER),
        (bonus, &TRIGGER_BONUS),
        (Card::Forest, &COMBINED_LAND),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, provider(), Zone::Battlefield);
    spawn(&mut game, player, bonus, Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, u32::MAX - 3, 0];
    let announce = Action::AnnounceCast { card: shock };
    reject(&mut game, announce.clone());

    game.state.players[player.index()].mana[4] -= 1;
    act(&mut game, announce);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    act(
        &mut game,
        Action::ActivateMana {
            card: forest,
            ability: 0,
            counter_payment: Counters::default(),
            payment: ZERO,
            production: RED_GREEN,
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].mana,
        [0, 0, 0, 2, u32::MAX, 0]
    );
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn live_providers_stack_for_the_actors_basic_lands_before_tap_triggers() {
    let bonus = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[(provider(), &MULTIPLIER), (bonus, &TRIGGER_BONUS)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let first = spawn(&mut game, player, provider(), Zone::Battlefield);
    spawn(&mut game, player, provider(), Zone::Battlefield);
    spawn(&mut game, player.other(), provider(), Zone::Battlefield);
    spawn(&mut game, player, bonus, Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let second_forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let dual = put(&mut game, player, "Lush Portico", Zone::Battlefield);
    let opposing_forest = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    game.state.players[player.index()].mana = ZERO;
    game.state.players[player.other().index()].mana = ZERO;

    let mut overflow = game.clone();
    overflow.state.players[player.index()].mana[4] = u32::MAX - 8;
    let before = overflow.snapshot();
    let revision = overflow.revision();
    assert!(matches!(
        overflow.apply(player, Action::TapMana { card: forest }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(overflow.snapshot(), before);
    assert_eq!(overflow.revision(), revision);

    act(&mut game, Action::TapMana { card: forest });
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 10, 0]);
    assert!(game.has_subtype(dual, "Forest"));
    act(&mut game, Action::TapMana { card: dual });
    assert_eq!(game.state.players[player.index()].mana, [1, 0, 0, 0, 11, 0]);

    game.move_object(first, Zone::Graveyard);
    act(
        &mut game,
        Action::TapMana {
            card: second_forest,
        },
    );
    assert_eq!(game.state.players[player.index()].mana, [1, 0, 0, 0, 15, 0]);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::TapMana {
            card: opposing_forest,
        },
    );
    assert_eq!(
        game.state.players[player.other().index()].mana,
        [0, 0, 0, 0, 3, 0]
    );
    assert!(game.state.stack.is_empty());
}

#[test]
fn an_intrinsic_tap_can_fund_a_spell_and_records_the_scaled_mana() {
    let _rules = fixture_rules::activate(&[(provider(), &MULTIPLIER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let multiplier = spawn(&mut game, player, provider(), Zone::Battlefield);
        let forest = put(&mut game, player, "Forest", Zone::Battlefield);
        let ranger = put(&mut game, player, "Beast-Kin Ranger", Zone::Hand);
        game.state.players[player.index()].mana = ZERO;
        let announce = Action::AnnounceCast { card: ranger };

        let mut without_provider = game.clone();
        without_provider.move_object(multiplier, Zone::Graveyard);
        reject(&mut without_provider, announce.clone());
        assert!(game.legal_actions().contains(&announce));

        // An opposing permanent linked in exile enables the owned cast receipt.
        let banishing_light = put(&mut game, player, "Banishing Light", Zone::Battlefield);
        let exiled = put(&mut game, player.other(), "Bear Cub", Zone::Exile);
        game.state.exile_links.push(ExileLink {
            ability_origin: None,
            source: game.object_ref(banishing_light),
            exiled: game.object_ref(exiled),
            return_zone: Zone::Battlefield,
        });

        act(&mut game, announce);
        act(&mut game, Action::TapMana { card: forest });
        let mana = [0, 0, 0, 0, 3, 0];
        assert_eq!(game.state.players[player.index()].mana, mana);
        let event = &game.state.cast_rollback.as_ref().unwrap().events[0];
        assert_eq!(event.production, mana);
        assert_eq!(event.tap_bonus, ZERO);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(ranger).zone, Zone::Battlefield);
        assert_eq!(game.state.players[player.index()].mana, ZERO);
    }
}

#[test]
fn late_output_scales_the_chosen_type_from_the_departed_land_and_keeps_its_quality() {
    let _rules = fixture_rules::activate(&[(provider(), &MULTIPLIER), (Card::Forest, &LATE_LAND)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let multiplier = spawn(&mut game, player, provider(), Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let bat = put(&mut game, player, "Barrow Naughty", Zone::Hand);
    let victim = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    game.state.players[player.index()].mana = ZERO;
    let announce_murder = Action::AnnounceCast { card: murder };
    let announce_bat = Action::AnnounceCast { card: bat };
    let choose_black = Action::ChooseMana { production: BLACK };

    let mut without_provider = game.clone();
    without_provider.move_object(multiplier, Zone::Graveyard);
    reject(&mut without_provider, announce_murder.clone());
    reject(&mut game, announce_bat.clone());

    let mut uncommitted = game.clone();
    act(&mut uncommitted, start_late_mana(forest));
    act(&mut uncommitted, choose_black.clone());
    assert_eq!(
        uncommitted.state.players[player.index()].mana,
        [0, 0, 3, 0, 0, 0]
    );
    reject(&mut uncommitted, announce_bat);

    act(&mut game, announce_murder);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    let exact_source = game.object_ref(forest);
    act(&mut game, start_late_mana(forest));
    assert_ne!(game.object_ref(forest), exact_source);
    assert_eq!(game.object_view(forest).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).mana_choices, [BLACK]);
    assert!(game.observe(player.other()).mana_choices.is_empty());
    reject(&mut game, Action::ChooseMana { production: RED });
    reject(
        &mut game,
        Action::ChooseMana {
            production: [0, 0, 3, 0, 0, 0],
        },
    );

    let snapshot = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choose_black)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, choose_black.clone());
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, choose_black);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 3, 0, 0, 0]);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, ZERO);
}

#[test]
fn split_choices_use_original_allocations_with_capacity_for_the_scaled_output() {
    let _rules = fixture_rules::activate(&[(provider(), &MULTIPLIER), (Card::Forest, &SPLIT_LAND)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, provider(), Zone::Battlefield);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, u32::MAX - 6, 0, u32::MAX - 9, 0, 0];

    act(&mut game, start_late_mana(forest));
    let domain = game.mana_split_choice().unwrap();
    assert_eq!(
        (domain.total, domain.minimum_first, domain.maximum_first),
        (4, 1, 2)
    );
    reject(
        &mut game,
        Action::ChooseMana {
            production: [0, 0, 0, 4, 0, 0],
        },
    );
    act(
        &mut game,
        Action::ChooseMana {
            production: [0, 1, 0, 3, 0, 0],
        },
    );
    assert_eq!(
        game.state.players[player.index()].mana,
        [0, u32::MAX - 3, 0, u32::MAX, 0, 0]
    );
}
