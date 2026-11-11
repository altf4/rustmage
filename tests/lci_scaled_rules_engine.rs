//! Live resolution quantities and exact target history.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CONTROLLED_CAVES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Cave"),
    basic: false,
    other: false,
    cards_only: false,
};
const GRAVE_CAVES: CountFilter = CountFilter {
    zone: CountZone::Graveyard,
    cards_only: true,
    ..CONTROLLED_CAVES
};
const ANY_PERMANENT: BattlefieldFilter = BattlefieldFilter {
    kinds: 0,
    relation: -1,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};
const CAVE_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamageBattlefieldScaled {
            filters: &[BattlefieldFilter {
                kinds: 1 | 64,
                ..ANY_PERMANENT
            }],
            amount: AmountValue::Stat(StatValue::Sum(&[
                StatValue::Count(CONTROLLED_CAVES),
                StatValue::Count(GRAVE_CAVES),
            ])),
        }],
    ),
    ..CardRules::EMPTY
};
const ANIMATE_CAVE: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: StatValue::Sum(&[
                StatValue::Count(CountFilter {
                    other: true,
                    ..CONTROLLED_CAVES
                }),
                StatValue::Count(GRAVE_CAVES),
            ]),
            minimum: 3,
        },
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AnimateSource(AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Elemental"]),
            base: Some((3, 3)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const BATTLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::IfTargetEnteredThisTurn { group: 0, skip: 1 },
            Instruction::PutCountersOnTargetGroup {
                group: 0,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Fight {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const DESTROY_EXACT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DestroyManaValueExactly {
            kinds: 16,
            amount: AmountValue::ChosenX,
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GREATEST_POWER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::DamageScaled(
            AmountValue::GreatestPermanentPower {
                filter: BattlefieldFilter {
                    relation: 0,
                    subtype: Some("Dinosaur"),
                    ..ANY_PERMANENT
                },
                minimum: 2,
            },
        )],
    ),
    ..CardRules::EMPTY
};
const REMOVE_BORE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::SourceCounterAtLeast {
            kind: CounterKind::Bore,
            minimum: 3,
        },
        cost: ActivationCost::ZERO,
        program: &[Instruction::RemoveAllSourceCounters(CounterKind::Bore)],
        ..ANIMATE_CAVE.activated[0]
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

#[test]
fn current_battlefield_and_graveyard_caves_drive_both_threshold_and_damage() {
    let _rules = fixture_rules::activate(&[
        (named("Opt"), &CAVE_DAMAGE),
        (named("Cavernous Maw"), &ANIMATE_CAVE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let cave = spawn(&mut game, player, named("Cavernous Maw"), Zone::Battlefield);
        let other = spawn(
            &mut game,
            player,
            named("Hidden Courtyard"),
            Zone::Battlefield,
        );
        spawn(
            &mut game,
            player,
            named("Pit of Offerings"),
            Zone::Graveyard,
        );
        spawn(
            &mut game,
            opponent,
            named("Hidden Necropolis"),
            Zone::Battlefield,
        );
        spawn(
            &mut game,
            opponent,
            named("Pit of Offerings"),
            Zone::Graveyard,
        );
        assert!(game.has_subtype(cave, "Cave"));
        reject(&mut game, activate(cave));
        let last = spawn(
            &mut game,
            player,
            named("Hidden Necropolis"),
            Zone::Graveyard,
        );
        act(&mut game, activate(cave));
        pay(&mut game);
        game.move_object(last, Zone::Exile);
        settle(&mut game);
        let animated = game.object_view(cave);
        assert!(animated.has_subtype("Cave") && animated.has_subtype("Elemental"));
        assert_eq!(animated.kinds, 1 | 2);
        assert_eq!(stats(&game, cave), (3, 3));

        let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let artifact = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
        let walker = put(
            &mut game,
            opponent,
            "Chandra, Flameshaper",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 6);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        game.move_object(other, Zone::Exile);
        settle(&mut game);
        assert_eq!(game.object_view(creature).damage, 2);
        assert_eq!(game.object_view(cave).damage, 2);
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 4);
        assert_eq!(game.object_view(artifact).damage, 0);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE);
    }
}

#[test]
fn entry_condition_checks_the_surviving_exact_target_before_fight() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &BATTLE)]);
    for (fresh, remove_foe, remove_own, counters, damage) in [
        (true, false, false, 1, 3),
        (false, false, false, 0, 2),
        (true, true, false, 1, 0),
        (true, false, true, 0, 0),
    ] {
        let player = Player::P0;
        let mut game = scenario(player);
        let own = put(
            &mut game,
            player,
            "Grizzly Bears",
            if fresh { Zone::Hand } else { Zone::Battlefield },
        );
        if fresh {
            cast(&mut game, own, None);
            settle(&mut game);
        } else {
            game.state.entered_this_turn = None;
        }
        let foe = put(&mut game, player.other(), "Serra Angel", Zone::Battlefield);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        announce(&mut game, spell, &[Target::Card(own), Target::Card(foe)]);
        if remove_foe {
            game.move_object(foe, Zone::Exile);
        }
        if remove_own {
            game.move_object(own, Zone::Exile);
            game.move_object(own, Zone::Battlefield);
        }
        settle(&mut game);
        assert_eq!(game.object_view(foe).damage, damage);
        if damage == 0 {
            assert_eq!(game.counter_count(own, CounterKind::PlusOne), counters);
        } else {
            assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn exact_mana_value_destroys_qualifying_artifacts_on_both_sides() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &DESTROY_EXACT)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let low = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let one = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let two = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let other_kind = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(one).zone, Zone::Graveyard);
    assert_eq!(game.object_view(two).zone, Zone::Graveyard);
    assert_eq!(game.object_view(low).zone, Zone::Battlefield);
    assert_eq!(game.object_view(other_kind).zone, Zone::Battlefield);
}

#[test]
fn greatest_power_uses_the_current_controlled_subtype_and_minimum() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &GREATEST_POWER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let dinosaur = put(
        &mut game,
        player,
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let target = put(
        &mut game,
        player.other(),
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    let checkpoint = game.snapshot();
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    game.restore(&checkpoint).unwrap();
    game.move_object(dinosaur, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(target).damage, 2);
}

#[test]
fn removal_empties_only_the_named_counters_on_the_exact_source() {
    let _rules = fixture_rules::activate(&[(named("Swiftfoot Boots"), &REMOVE_BORE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    game.add_counter(source, CounterKind::Bore, 5);
    game.add_counter(source, CounterKind::Time, 2);
    act(&mut game, activate(source));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let checkpoint = game.snapshot();
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::Bore), 0);
    assert_eq!(game.counter_count(source, CounterKind::Time), 2);
    game.restore(&checkpoint).unwrap();
    game.move_object(source, Zone::Exile);
    game.move_object(source, Zone::Battlefield);
    game.add_counter(source, CounterKind::Bore, 5);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::Bore), 5);
}
