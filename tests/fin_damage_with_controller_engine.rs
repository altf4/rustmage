mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BOLT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::DamageWithControllerScaled {
            damage: 5,
            controller_damage: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Equipment"),
                basic: false,
                other: false,
                cards_only: false,
            })),
        }],
    ),
    ..CardRules::EMPTY
};
const BORROW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GainControlUntilEndTurn],
    ),
    ..CardRules::EMPTY
};
const SHIELD_EQUIPMENT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::SourceCounterSubtype {
                kind: CounterKind::Shield,
                minimum: 1,
                subtype: "Equipment",
                keywords: Keywords::NONE,
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::HealPreviousDamageToSelf,
            )],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::HealPreviousDamageToSelf],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn controller_damage_counts_equipment_and_uses_target_control_at_resolution() {
    let _rules = fixture_rules::activate(&[
        (Card::Shock, &BOLT),
        (Card::from_name("Opt").unwrap(), &BORROW),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let target = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.objects[target as usize].growth.toughness = 3;
        let equipped = put(&mut game, player, "Short Bow", Zone::Battlefield);
        assert!(game.attach_equipment(game.object_ref(equipped), game.object_ref(target)));
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        let removed = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        put(&mut game, opponent, "Short Bow", Zone::Battlefield);

        let bolt = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, bolt, Some(Target::Card(target)));
        act(&mut game, Action::Pass);
        let abrade = put(&mut game, opponent, "Abrade", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: abrade });
        act(&mut game, Action::ChooseMode { mode: 1 });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(removed),
            },
        );
        pay(&mut game);
        if game.actor() != Some(opponent) {
            act(&mut game, Action::Pass);
        }
        let borrow = put(&mut game, opponent, "Opt", Zone::Hand);
        cast(&mut game, borrow, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(game.object_view(removed).zone, Zone::Graveyard);
        assert_eq!(game.object_view(target).controller, opponent);
        assert_eq!(game.object_view(target).damage, 5);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.state.players[opponent.index()].life, 18);
    }
}

#[test]
fn amount_and_attribution_are_frozen_before_damage_prevention() {
    let _rules = fixture_rules::activate(&[
        (Card::Shock, &BOLT),
        (
            Card::from_name("Scorpion Sentinel").unwrap(),
            &SHIELD_EQUIPMENT,
        ),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player, "Scorpion Sentinel", Zone::Battlefield);
    let target_ref = game.object_ref(target);
    game.add_counter(target, CounterKind::Shield, 1);
    assert!(game.has_subtype(target, "Equipment"));
    let bolt = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, bolt, Some(Target::Card(target)));
    let source = game.object_ref(bolt);
    pass_priority(&mut game);

    let pending = game.observe(player).replacement_choice.unwrap();
    let event = pending.damage.unwrap();
    assert_eq!(event.source.reference, source);
    assert_eq!(event.source.controller, player);
    assert_eq!(event.target, Target::Card(target));
    assert_eq!(event.amount, 5);
    assert_eq!(game.state.players[player.index()].life, 20);
    act(
        &mut game,
        Action::ChooseDamagePrevention {
            source: target_ref,
            prevention: DamagePreventionKind::ShieldCounter,
        },
    );
    settle(&mut game);
    assert!(!game.has_subtype(target, "Equipment"));
    assert_eq!(game.counter_count(target, CounterKind::Shield), 0);
    assert_eq!(game.object_view(target).damage, 0);
    assert_eq!(game.state.players[player.index()].life, 19);
}

#[test]
fn zero_count_and_illegal_target_do_not_damage_a_player() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &BOLT)]);
    let player = Player::P0;
    for invalid in [false, true] {
        let mut game = scenario(player);
        let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        if invalid {
            put(&mut game, player, "Short Bow", Zone::Battlefield);
        }
        let bolt = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, bolt, Some(Target::Card(target)));
        if invalid {
            act(&mut game, Action::Pass);
            let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
            cast(&mut game, bounce, Some(Target::Card(target)));
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(target).zone,
            if invalid { Zone::Hand } else { Zone::Graveyard },
        );
        assert_eq!(game.state.players[player.other().index()].life, 20);
    }
}
