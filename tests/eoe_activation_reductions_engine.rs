//! Actual and projected generic reductions share live characteristics and a locked total.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TARGETED: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        generic: 3,
        colored: [1, 0, 0, 0, 0, 0],
        mana_component: true,
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: one(TargetRule::Creature),
    program: &[Instruction::Damage(1)],
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    exhaust: false,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
};
const COUNTER_DISCOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[TARGETED],
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: StatValue::AtLeast {
                    value: &StatValue::ControlledCreaturesWithCounter(CounterKind::PlusOne),
                    minimum: 1,
                },
                multiplier: 2,
            },
        )],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MANA: Mana = [1, 0, 0, 0, 0, 1];
const COUNTER_FUNDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(MANA))],
        ..TARGETED
    }],
    ..CardRules::EMPTY
};
const REDUCED_FUNDER: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        ..COUNTER_FUNDER.activated[0]
    }],
    ..COUNTER_DISCOUNT
};

const NAMED_LANDS: StatValue = StatValue::DistinctNames(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
const LAND_DISCOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 10,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        ..TARGETED
    }],
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: NAMED_LANDS,
                multiplier: 1,
            },
        )],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ATTACK_DISCOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 6,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ..TARGETED
    }],
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: StatValue::AtLeast {
                    value: &StatValue::DeclaredSubtypeAttackersThisTurn("Spacecraft"),
                    minimum: 1,
                },
                multiplier: 3,
            },
        )],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn positive_creature_counter_reduction_stays_locked_when_its_funder_leaves() {
    let _rules = fixture_rules::activate(&[
        (Card::GoblinPiker, &COUNTER_DISCOUNT),
        (Card::from_name("Bear Cub").unwrap(), &COUNTER_FUNDER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let funder = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let noncreature = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[noncreature as usize].counters.plus_one = 1;
    game.state.objects[opponent as usize].counters.plus_one = 1;
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, activate(source));
    game.state.objects[funder as usize].counters.plus_one = 2;

    let before = game.snapshot();
    let exact_funder = game.object_ref(funder);
    let world = FundingState::from_game(
        &game,
        &FundingGoal {
            actor: player,
            required: [1, 0, 0, 0, 0, 0],
            generic: 1,
            reserve: None,
            source: Some((
                game.object_ref(source),
                0,
                Counters::default(),
                Card::GoblinPiker,
                None,
            )),
        },
    );
    assert_eq!(
        world.activation_generic_discount(source, Card::GoblinPiker, 0),
        2
    );
    assert!(
        world.cost_departures(&[exact_funder], Zone::Graveyard, &mut |next| {
            assert_eq!(
                next.activation_generic_discount(source, Card::GoblinPiker, 0),
                0
            );
            true
        })
    );
    assert_eq!(game.snapshot(), before);

    act(&mut game, activate(source));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 1);
    act(
        &mut game,
        Action::ActivateMana {
            card: funder,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: MANA,
        },
    );
    assert_eq!(game.object_view(funder).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 1);
    pay(&mut game);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn differently_named_lands_count_once_and_only_for_the_controller() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &LAND_DISCOUNT)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    for name in ["Forest", "Forest", "Island"] {
        let land = put(&mut game, player, name, Zone::Battlefield);
        game.state.objects[land as usize].tapped = true;
    }
    put(&mut game, player.other(), "Plains", Zone::Battlefield);
    let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 7];
    reject(&mut game, activate(source));
    game.state.players[player.index()].mana[5] = 8;
    act(&mut game, activate(source));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 8);
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
}

#[test]
fn projected_funding_keeps_non_tap_mana_abilities_with_live_reductions() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GoblinPiker, &COUNTER_DISCOUNT),
        (bear, &REDUCED_FUNDER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let funder = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[funder as usize].counters.plus_one = 1;
    game.state.players[player.index()].mana = [0; 6];

    act(&mut game, activate(source));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    choose(&mut game, |action| match action {
        Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } => *card == funder,
        _ => false,
    });
    if game.object_view(funder).zone == Zone::Battlefield {
        pay(&mut game);
    }
    assert_eq!(game.object_view(funder).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 1);
    pay(&mut game);
    assert!(game.object_view(source).tapped);
}

#[test]
fn spacecraft_discount_reads_actual_declarations_before_the_ability_enters() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &ATTACK_DISCOUNT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let ship_ref = game.object_ref(ship);
    game.activation_cost_counters(ship_ref, CounterKind::Charge, 6, player);
    let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: ship });
    act(&mut game, Action::FinishAttackers);
    game.move_object(ship, Zone::Exile);

    let source = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut game, activate(source));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 3);
    pay(&mut game);
}
