//! Discarding a card is a separate cost route for one equip ability.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FLAIL: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[
            Instruction::EquipDiscardAlternative,
            Instruction::AttachSource,
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const FIRST_EQUIP_FREE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FirstEquipZeroCost {
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
const TARGET_DISCOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::EquipTargetReduction(3)],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn routes(game: &Game, player: Player) -> Vec<ActivationCostRoute> {
    game.observe(player)
        .cost_choices
        .into_iter()
        .map(|choice| choice.activation_route.unwrap())
        .collect()
}

fn choose_route(game: &mut Game, player: Player, route: ActivationCostRoute) {
    let cost = game
        .observe(player)
        .cost_choices
        .into_iter()
        .find(|choice| choice.activation_route == Some(route))
        .unwrap()
        .index;
    act(game, Action::ChooseActivationCost { cost });
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn discard_route_owns_the_hand_choice_and_commits_as_one_activation() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &FLAIL)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let gear = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let foreign_hand = put(&mut game, other, "Opt", Zone::Hand);
        reject(&mut game, activate(gear));

        let first = put(&mut game, player, "Shock", Zone::Hand);
        let second = put(&mut game, player, "Opt", Zone::Hand);
        let first = game.object_ref(first);
        let second = game.object_ref(second);
        let foreign_hand = game.object_ref(foreign_hand);
        act(&mut game, activate(gear));
        assert_eq!(routes(&game, player), [ActivationCostRoute::DiscardCard]);
        assert!(routes(&game, other).is_empty());
        choose_route(&mut game, player, ActivationCostRoute::DiscardCard);
        reject(&mut game, target(enemy));
        act(&mut game, target(bearer));
        let cost = game.observe(player).additional_cost.unwrap();
        assert_eq!(cost.kind, EffectChoiceKind::Discard);
        assert_eq!(cost.candidates, [first, second]);
        assert!(game.observe(other).additional_cost.is_none());
        reject(
            &mut game,
            Action::ChooseAdditionalCost {
                object: foreign_hand,
            },
        );

        let mut second_choice = game.clone();
        act(
            &mut second_choice,
            Action::ChooseAdditionalCost { object: second },
        );
        act(&mut game, Action::ChooseAdditionalCost { object: first });
        assert_eq!(
            serde_json::to_value(game.observe(other)).unwrap(),
            serde_json::to_value(second_choice.observe(other)).unwrap()
        );
        assert_eq!(game.object_view(first.card).zone, Zone::Hand);
        assert!(!game.state.players[player.index()].equip_activated_this_turn);
        reject(&mut game, Action::TapMana { card: land });

        let pay = Action::PayCosts { payment: [0; 6] };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == pay)
            .unwrap();
        let saved = game.snapshot();
        let mut raw = game.clone();
        let mut indexed = game;
        act(&mut raw, pay);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for branch in [&mut raw, &mut indexed] {
            assert_eq!(branch.object_view(first.card).zone, Zone::Graveyard);
            assert_eq!(branch.object_view(second.card).zone, Zone::Hand);
            assert!(branch.state.players[player.index()].equip_activated_this_turn);
            assert_eq!(
                branch.state.stack.last().unwrap().kind,
                StackKind::ActivatedAbility { ability: 0 }
            );
            assert_eq!(branch.object_view(gear).attached_to, None);
            settle(branch);
            assert_eq!(
                branch.object_view(gear).attached_to,
                Some(branch.object_ref(bearer))
            );
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        assert_eq!(indexed.object_view(first.card).zone, Zone::Hand);
    }
}

#[test]
fn equal_mana_totals_keep_their_own_routes_and_use_the_same_equip_history() {
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &FLAIL),
        (named("Bear Cub"), &FIRST_EQUIP_FREE),
        (named("Grizzly Bears"), &TARGET_DISCOUNT),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        root.state.players[player.index()].mana = [0; 6];
        let gear = put(&mut root, player, "Short Bow", Zone::Battlefield);
        let normal = put(&mut root, player, "Bear Cub", Zone::Battlefield);
        let discounted = put(&mut root, player, "Grizzly Bears", Zone::Battlefield);
        let hand = put(&mut root, player, "Shock", Zone::Hand);
        act(&mut root, activate(gear));
        assert_eq!(
            routes(&root, player),
            [
                ActivationCostRoute::Printed,
                ActivationCostRoute::FirstEquipZero,
                ActivationCostRoute::DiscardCard,
            ]
        );

        let mut printed = root.clone();
        choose_route(&mut printed, player, ActivationCostRoute::Printed);
        reject(&mut printed, target(normal));
        act(&mut printed, target(discounted));
        assert_eq!(printed.observe(player).payment_request.unwrap().generic, 0);
        assert!(printed.observe(player).additional_cost.is_none());
        pay(&mut printed);
        settle(&mut printed);
        assert_eq!(printed.object_view(hand).zone, Zone::Hand);
        assert_eq!(
            printed.object_view(gear).attached_to,
            Some(printed.object_ref(discounted))
        );
        act(&mut printed, activate(gear));
        assert_eq!(
            routes(&printed, player),
            [
                ActivationCostRoute::Printed,
                ActivationCostRoute::DiscardCard
            ],
        );

        let mut free = root.clone();
        choose_route(&mut free, player, ActivationCostRoute::FirstEquipZero);
        act(&mut free, target(normal));
        assert!(free.observe(player).additional_cost.is_none());
        pay(&mut free);
        settle(&mut free);
        assert_eq!(free.object_view(hand).zone, Zone::Hand);
        assert_eq!(
            free.object_view(gear).attached_to,
            Some(free.object_ref(normal))
        );

        choose_route(&mut root, player, ActivationCostRoute::DiscardCard);
        act(&mut root, target(normal));
        let object = root.object_ref(hand);
        act(&mut root, Action::ChooseAdditionalCost { object });
        assert_eq!(root.observe(player).payment_request.unwrap().generic, 0);
        pay(&mut root);
        settle(&mut root);
        assert_eq!(root.object_view(hand).zone, Zone::Graveyard);
        act(&mut root, activate(gear));
        assert_eq!(routes(&root, player), [ActivationCostRoute::Printed]);
    }
}

#[test]
fn a_named_increase_adds_mana_to_the_discard_route() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &FLAIL)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let gear = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let first = put(&mut game, player, "Forest", Zone::Battlefield);
        let hand = put(&mut game, player, "Shock", Zone::Hand);
        let tax = put(
            &mut game,
            player.other(),
            "Skyseer's Chariot",
            Zone::Battlefield,
        );
        game.set_entry_value(
            game.object_ref(tax),
            EntryChoiceValue::CardName(rustmage_core::CardName::from_name("Short Bow").unwrap()),
        );
        reject(&mut game, activate(gear));
        let second = put(&mut game, player, "Forest", Zone::Battlefield);
        act(&mut game, activate(gear));
        assert_eq!(routes(&game, player), [ActivationCostRoute::DiscardCard]);
        choose_route(&mut game, player, ActivationCostRoute::DiscardCard);
        act(&mut game, target(bearer));
        let object = game.object_ref(hand);
        act(&mut game, Action::ChooseAdditionalCost { object });
        assert_eq!(game.observe(player).payment_request.unwrap().generic, 2);
        act(&mut game, Action::TapMana { card: first });
        reject(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        act(&mut game, Action::TapMana { card: second });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(hand).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(gear).attached_to,
            Some(game.object_ref(bearer))
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}
