//! Repeatable graveyard casting permissions with independently ordered costs.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRANTOR: CardRules = CardRules {
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
        program: &[Instruction::GrantCreatureGraveyardForageThisTurn {
            entry_counter: Some(CounterKind::Finality),
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const PRINTED_FORAGE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Forage,
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
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const PRINTED_BARGAIN: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const LIFE_RULE: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::CastFromGraveyardWithLifeDuringOwnTurn {
            filter: CastFilter::InstantSorcery,
            life: 1,
        },
    ],
}];

const LIFE_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: LIFE_RULE,
    extras: &CardExtras {
        static_ability_groups: LIFE_RULE,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn grant(game: &mut Game, player: Player) -> (CardId, u64) {
    let source = put(game, player, "Short Bow", Zone::Battlefield);
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == source),
    );
    settle(game);
    let row = game.observe(player).paid_graveyard_cast_permissions[0];
    assert!(row.repeat_with_forage);
    (source, row.id)
}

fn forage_cast(game: &Game, card: CardId, permission: u64, method: CastMethod) -> Action {
    Action::AnnounceGraveyardPaid {
        object: game.object_ref(card),
        permission,
        face: CastFace::Normal,
        method,
        flash_surcharge: false,
    }
}

fn life_cast(game: &Game, card: CardId) -> Action {
    Action::AnnounceGraveyardWithLife {
        object: game.object_ref(card),
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
        life: 1,
        entry_counter: None,
    }
}

fn choose_cost(game: &mut Game, cost: CastCostClause) {
    let index = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == cost && !row.paid)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause: index });
}

fn pay_forage_food(game: &mut Game, food: CardId) {
    act(
        game,
        Action::ChooseForageRoute {
            route: ForageRoute::Food,
        },
    );
    let object = game.object_ref(food);
    act(game, Action::ChooseForage { object });
}

#[test]
fn same_grant_pays_independently_and_marks_only_the_creature_that_actually_enters() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &GRANTOR)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let (source, permission) = grant(&mut game, player);
        let first = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
        let second = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
        let outsider = put(&mut game, player.other(), "Healer's Hawk", Zone::Graveyard);
        let noncreature = put(&mut game, player, "Disenchant", Zone::Graveyard);
        for card in [outsider, noncreature, first] {
            // No Food, and the two other cards in this graveyard cannot forage
            // together with the creature being announced.
            let action = forage_cast(&game, card, permission, CastMethod::Normal);
            reject(&mut game, action);
        }

        let food = put(&mut game, player, "Candy Trail", Zone::Battlefield);
        let action = forage_cast(&game, first, permission, CastMethod::Normal);
        let mut wrong_timing = game.clone();
        wrong_timing.state.step = Step::End;
        reject(&mut wrong_timing, action.clone());
        act(&mut game, action);
        act(&mut game, Action::BeginOrderedCosts);
        choose_cost(&mut game, CastCostClause::OriginForage);
        pay_forage_food(&mut game, food);
        choose_cost(&mut game, CastCostClause::Mana);
        pay(&mut game);
        let spell_id = game.state.stack.last().unwrap().id;
        let stack_object = game.object_ref(first);
        assert_eq!(game.counter_count(first, CounterKind::Finality), 0);
        assert!(
            game.observe(player)
                .cast_entry_counters
                .iter()
                .any(|row| row.spell == stack_object)
        );
        assert!(game.counter_stack_item(spell_id, false));
        game.move_object(first, Zone::Battlefield);
        assert_eq!(game.counter_count(first, CounterKind::Finality), 0);

        game.move_object(source, Zone::Graveyard);
        let action = forage_cast(&game, second, permission, CastMethod::Normal);
        act(&mut game, action);
        pay(&mut game);
        choose_cost(&mut game, CastCostClause::OriginForage);
        act(
            &mut game,
            Action::ChooseForageRoute {
                route: ForageRoute::ExileThree,
            },
        );
        for card in [source, noncreature, food] {
            let object = game.object_ref(card);
            act(&mut game, Action::ChooseForage { object });
        }
        settle(&mut game);
        assert_eq!(game.counter_count(second, CounterKind::Finality), 1);
        assert_eq!(
            game.observe(player).paid_graveyard_cast_permissions[0].id,
            permission
        );
        game.move_object(second, Zone::Graveyard);
        assert_eq!(game.object_view(second).zone, Zone::Exile);

        game.state.step = Step::End;
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(
            game.observe(player)
                .paid_graveyard_cast_permissions
                .is_empty()
        );
    }
}

#[test]
fn the_origin_and_printed_forage_costs_keep_separate_choices_in_either_order() {
    let spell_card = named("Healer's Hawk");
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &GRANTOR),
        (spell_card, &PRINTED_FORAGE),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let (_, permission) = grant(&mut game, player);
    let spell = spawn(&mut game, player, spell_card, Zone::Graveyard);
    let first = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let action = forage_cast(&game, spell, permission, CastMethod::Normal);
    reject(&mut game, action.clone());
    let second = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    act(&mut game, action);
    act(
        &mut game,
        Action::ChooseForageRoute {
            route: ForageRoute::Food,
        },
    );
    act(&mut game, Action::BeginOrderedCosts);

    let before = game.snapshot();
    let mut printed_first = game.clone();
    for (branch, first_cost, second_cost) in [
        (
            &mut game,
            CastCostClause::OriginForage,
            CastCostClause::Additional,
        ),
        (
            &mut printed_first,
            CastCostClause::Additional,
            CastCostClause::OriginForage,
        ),
    ] {
        choose_cost(branch, first_cost);
        if first_cost == CastCostClause::OriginForage {
            act(
                branch,
                Action::ChooseForageRoute {
                    route: ForageRoute::Food,
                },
            );
        }
        let object = branch.object_ref(first);
        let pick = Action::ChooseForage { object };
        let index = branch
            .legal_actions()
            .iter()
            .position(|a| *a == pick)
            .unwrap();
        let mut indexed = branch.clone();
        indexed
            .apply_index(player, branch.revision(), index)
            .unwrap();
        act(branch, pick);
        assert_eq!(branch.snapshot(), indexed.snapshot());

        choose_cost(branch, second_cost);
        if second_cost == CastCostClause::OriginForage {
            act(
                branch,
                Action::ChooseForageRoute {
                    route: ForageRoute::Food,
                },
            );
        }
        reject(branch, Action::ChooseForage { object });
        assert!(branch.observe(player.other()).forage.is_none());
        let object = branch.object_ref(second);
        act(branch, Action::ChooseForage { object });
        choose_cost(branch, CastCostClause::Mana);
        pay(branch);
        settle(branch);
        assert_eq!(branch.counter_count(spell, CounterKind::Finality), 1);
        assert_eq!(branch.object_view(first).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(second).zone, Zone::Graveyard);
    }
    printed_first.restore(&before).unwrap();
    assert_eq!(printed_first.snapshot(), before);
}

#[test]
fn the_creature_alternative_of_a_noncreature_card_can_pay_forage() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &GRANTOR)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let (_, permission) = grant(&mut game, player);
    let weapon = spawn(
        &mut game,
        player,
        named("Concealed Weapon"),
        Zone::Graveyard,
    );
    let food = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let normal = forage_cast(&game, weapon, permission, CastMethod::Normal);
    reject(&mut game, normal);
    let disguise = forage_cast(&game, weapon, permission, CastMethod::Disguise);
    act(&mut game, disguise);
    pay(&mut game);
    choose_cost(&mut game, CastCostClause::OriginForage);
    pay_forage_food(&mut game, food);
    settle(&mut game);
    assert!(game.is_face_down(game.object_ref(weapon)));
    assert_eq!(game.counter_count(weapon, CounterKind::Finality), 1);
}

#[test]
fn the_life_route_is_live_on_its_controllers_turn_and_preserves_printed_costs() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &LIFE_PROVIDER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let instant = put(&mut game, player, "Thrill of Possibility", Zone::Graveyard);
    let sorcery = put(&mut game, player, "Divination", Zone::Graveyard);
    let opponent_spell = put(
        &mut game,
        player.other(),
        "Thrill of Possibility",
        Zone::Graveyard,
    );
    let own_action = life_cast(&game, instant);
    let opposing_action = life_cast(&game, opponent_spell);
    reject(&mut game, opposing_action);
    reject(&mut game, own_action.clone());
    let discard = put(&mut game, player, "Forest", Zone::Hand);

    game.state.active = player.other();
    game.state.step = Step::End;
    reject(&mut game, own_action.clone());
    game.state.active = player;
    let sorcery_action = life_cast(&game, sorcery);
    reject(&mut game, sorcery_action.clone());

    let mut gone = game.clone();
    gone.move_object(provider, Zone::Exile);
    reject(&mut gone, own_action.clone());
    let mut controlled_by_opponent = game.clone();
    controlled_by_opponent.state.objects[provider as usize].controller = player.other();
    reject(&mut controlled_by_opponent, own_action.clone());

    act(&mut game, own_action);
    act(&mut game, Action::BeginOrderedCosts);
    choose_cost(&mut game, CastCostClause::Life(1));
    choose_cost(&mut game, CastCostClause::Additional);
    let object = game.object_ref(discard);
    act(&mut game, Action::ChooseAdditionalCost { object });
    choose_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);
    assert!(game.observe(player).cast_entry_counters.is_empty());
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 19);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.object_view(instant).zone, Zone::Graveyard);

    game.state.step = Step::Main2;
    act(&mut game, sorcery_action);
    pay(&mut game);
    choose_cost(&mut game, CastCostClause::Life(1));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
}

#[test]
fn a_bargain_declaration_reserves_food_independently_of_the_origin_cost() {
    let spell_card = named("Healer's Hawk");
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &GRANTOR),
        (spell_card, &PRINTED_BARGAIN),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let (_, permission) = grant(&mut game, player);
    let spell = spawn(&mut game, player, spell_card, Zone::Graveyard);
    let bargain_food = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let action = forage_cast(&game, spell, permission, CastMethod::Normal);
    let bargain = Action::ChooseBargain {
        sacrifice: Some(game.object_ref(bargain_food)),
    };

    let mut one_food = game.clone();
    act(&mut one_food, action.clone());
    reject(&mut one_food, bargain.clone());
    act(&mut one_food, Action::ChooseBargain { sacrifice: None });

    let origin_food = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    act(&mut game, action);
    act(&mut game, bargain);
    pay(&mut game);
    choose_cost(&mut game, CastCostClause::OriginForage);
    act(
        &mut game,
        Action::ChooseForageRoute {
            route: ForageRoute::Food,
        },
    );
    let object = game.object_ref(bargain_food);
    reject(&mut game, Action::ChooseForage { object });
    let object = game.object_ref(origin_food);
    act(&mut game, Action::ChooseForage { object });
    choose_cost(&mut game, CastCostClause::Additional);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().bargain,
        Some(true)
    );
    settle(&mut game);
    assert_eq!(game.counter_count(spell, CounterKind::Finality), 1);
    assert_eq!(game.object_view(origin_food).zone, Zone::Graveyard);
    assert_eq!(game.object_view(bargain_food).zone, Zone::Graveyard);
}
