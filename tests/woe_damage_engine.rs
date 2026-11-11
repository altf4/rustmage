//! Creature-source prevention and exact single-creature spell damage observers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATURE_SHIELD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventCreatureDamageToControllerAndCreaturesThisTurn],
    ),
    ..CardRules::EMPTY
};
const PING: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
    program: &[Instruction::Damage(2)],
    sorcery: false,
    once_per_turn: false,
};
const DEALER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[
        PING,
        ActivatedAbility {
            program: &[
                Instruction::DamageCannotBePreventedThisTurn,
                Instruction::Damage(2),
            ],
            ..PING
        },
    ],
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSpellDamagesSoleCreatureTarget,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayersScaled(
            PlayerSet::Opponent,
            AmountValue::DamageEventAmount,
        )],
    }],
    ..CardRules::EMPTY
};
const DOUBLE_TARGETS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 2,
            max: 2,
        }),
        &[Instruction::Damage(2)],
    ),
    ..CardRules::EMPTY
};
const REPEATED_TARGET: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[Instruction::Damage(2)],
    ),
    ..CardRules::EMPTY
};
const CREATURE_POWER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::PermanentsDealPowerDamage {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            exclude_source: false,
        }],
    ),
    ..CardRules::EMPTY
};
const REDIRECT: CardRules = CardRules {
    effect: Effect::Creature(0, 10),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::RedirectControlledDamageToSource,
        )],
    }],
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::RedirectControlledDamageToSource],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn give_opportunity(game: &mut Game, player: Player) {
    if game.actor() != Some(player) {
        act(game, Action::Pass);
    }
    assert_eq!(game.actor(), Some(player));
}

fn ping(game: &mut Game, source: CardId, slot: u16, recipient: Target) {
    give_opportunity(game, game.object_view(source).controller);
    choose(game, |action| {
        matches!(action, Action::Activate { card, ability, target: Some(target), .. }
            if *card == source && *ability == slot && *target == recipient)
    });
}

fn play_shield(game: &mut Game, player: Player) -> CardId {
    give_opportunity(game, player);
    let source = put(game, player, "Opt", Zone::Hand);
    cast(game, source, None);
    settle(game);
    source
}

fn bounce(game: &mut Game, player: Player, recipient: CardId) {
    give_opportunity(game, player);
    let spell = put(game, player, "Unsummon", Zone::Hand);
    cast(game, spell, Some(Target::Card(recipient)));
}

#[test]
fn creature_source_shield_checks_live_recipients_and_last_known_creature_sources() {
    let _rules =
        fixture_rules::activate(&[(card("Opt"), &CREATURE_SHIELD), (Card::HillGiant, &DEALER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let dealer = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let shield_source = play_shield(&mut game, player);
        assert_eq!(game.object_view(shield_source).zone, Zone::Graveyard);
        let late = put(&mut game, player, "Gigantosaurus", Zone::Hand);
        cast(&mut game, late, None);
        settle(&mut game);
        let walker = put(
            &mut game,
            player,
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 9);

        ping(&mut game, dealer, 0, Target::Player(player));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20);

        ping(&mut game, dealer, 0, Target::Card(walker));
        settle(&mut game);
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 7);

        game.add_counter(late, CounterKind::Shield, 1);
        ping(&mut game, dealer, 0, Target::Card(late));
        pass_priority(&mut game);
        assert_eq!(game.actor(), Some(player));
        let shield = game.observe(opponent).creature_damage_shields[0];
        assert_eq!(
            shield.scope,
            CreatureDamageShieldScope::PlayerAndCreaturesFromCreatures
        );
        let action = Action::ChooseDamagePrevention {
            source: shield.source,
            prevention: DamagePreventionKind::CreatureSourcesThisTurn(shield.id),
        };
        let revision = game.revision();
        let checkpoint = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        assert_eq!(
            indexed.apply(opponent, action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(indexed.snapshot(), checkpoint);
        act(&mut game, action);
        indexed.apply_index(player, revision, index).unwrap();
        settle(&mut game);
        settle(&mut indexed);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(game.object_view(late).damage, 0);
        assert_eq!(game.counter_count(late, CounterKind::Shield), 1);
        indexed.restore(&checkpoint).unwrap();
        act(
            &mut indexed,
            Action::ChooseDamagePrevention {
                source: game.object_ref(late),
                prevention: DamagePreventionKind::ShieldCounter,
            },
        );
        settle(&mut indexed);
        assert_eq!(indexed.counter_count(late, CounterKind::Shield), 0);

        give_opportunity(&mut game, opponent);
        let shock = put(&mut game, opponent, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 18);

        let mut unpreventable = game.clone();
        ping(&mut unpreventable, dealer, 1, Target::Player(player));
        settle(&mut unpreventable);
        assert_eq!(unpreventable.state.players[player.index()].life, 16);

        ping(&mut game, dealer, 0, Target::Player(player));
        bounce(&mut game, player, dealer);
        settle(&mut game);
        assert_eq!(game.object_view(dealer).zone, Zone::Hand);
        assert_eq!(game.state.players[player.index()].life, 18);
    }
}

#[test]
fn creature_source_shield_prevents_combat_until_cleanup() {
    let _rules = fixture_rules::activate(&[(card("Opt"), &CREATURE_SHIELD)]);
    for attacker_controller in [Player::P0, Player::P1] {
        let defender = attacker_controller.other();
        let mut game = scenario(attacker_controller);
        let attacker = put(
            &mut game,
            attacker_controller,
            "Bear Cub",
            Zone::Battlefield,
        );
        play_shield(&mut game, defender);
        pass_priority(&mut game);
        pass_priority(&mut game);
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        pass_priority(&mut game);
        act(&mut game, Action::FinishBlockers);
        pass_priority(&mut game);
        assert_eq!(game.state.players[defender.index()].life, 20);
        assert_eq!(game.observe(defender).creature_damage_shields.len(), 1);
        game.state.step = Step::End;
        pass_priority(&mut game);
        assert!(game.observe(defender).creature_damage_shields.is_empty());
    }
}

fn settle_spell_copy(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishStackTargets | Action::PutTrigger { .. }
            )
        });
    }
    panic!("spell copy exceeded its action budget");
}

#[test]
fn adventure_damage_and_its_spell_copy_each_trigger_with_their_own_damage() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &OBSERVER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let teach = put(&mut game, player, "Teach by Example", Zone::Hand);
        cast(&mut game, teach, None);
        settle(&mut game);
        let adventure = put(&mut game, player, "Frolicking Familiar", Zone::Hand);
        act(&mut game, Action::AnnounceAdventure { card: adventure });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut game);
        settle_spell_copy(&mut game);
        assert_eq!(game.object_view(adventure).zone, Zone::Exile);
        assert_eq!(game.object_view(target).damage, 2);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.state.players[opponent.index()].life, 18);
    }
}

#[test]
fn single_target_trigger_freezes_modified_damage_and_ignores_prevented_damage() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &OBSERVER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let doubler = put(&mut game, player, "Twinflame Tyrant", Zone::Battlefield);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        let mut prevented = game.clone();
        prevented.add_counter(target, CounterKind::Shield, 1);
        cast(&mut prevented, shock, Some(Target::Card(target)));
        // Choose the shield before doubling to prevent the whole event.
        for _ in 0..12 {
            if prevented.state.pending == Pending::Priority && prevented.state.stack.is_empty() {
                break;
            }
            choose(&mut prevented, |action| {
                matches!(action, Action::ChooseDamagePrevention { .. } | Action::Pass)
            });
        }
        assert!(prevented.state.stack.is_empty());
        assert_eq!(prevented.object_view(target).damage, 0);
        assert_eq!(prevented.state.players[opponent.index()].life, 20);
        assert_eq!(prevented.counter_count(target, CounterKind::Shield), 0);

        cast(&mut game, shock, Some(Target::Card(target)));
        pass_priority(&mut game);
        assert_eq!(game.object_view(target).damage, 4);
        game.move_object(doubler, Zone::Hand);
        bounce(&mut game, opponent, observer);
        settle(&mut game);
        assert_eq!(game.object_view(observer).zone, Zone::Hand);
        assert_eq!(game.state.players[opponent.index()].life, 16);
    }
}

#[test]
fn original_distinct_targets_and_damage_source_are_required() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &OBSERVER),
        (Card::Divination, &DOUBLE_TARGETS),
        (card("Opt"), &REPEATED_TARGET),
        (card("Kellan's Lightblades"), &CREATURE_POWER),
        (Card::GrizzlyBears, &REDIRECT),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let second = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let multi = put(&mut game, player, "Divination", Zone::Hand);
        announce(
            &mut game,
            multi,
            &[Target::Card(target), Target::Card(second)],
        );
        bounce(&mut game, opponent, second);
        settle(&mut game);
        assert_eq!(game.object_view(target).damage, 2);
        assert_eq!(game.state.players[opponent.index()].life, 20);

        let by_creature = put(&mut game, player, "Kellan's Lightblades", Zone::Hand);
        cast(&mut game, by_creature, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(game.object_view(target).damage, 5);
        assert_eq!(game.state.players[opponent.index()].life, 20);

        let repeated = put(&mut game, player, "Opt", Zone::Hand);
        announce(
            &mut game,
            repeated,
            &[Target::Card(target), Target::Card(target)],
        );
        settle(&mut game);
        assert_eq!(game.object_view(target).damage, 7);
        assert_eq!(game.state.players[opponent.index()].life, 18);

        let redirector = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(game.object_view(target).damage, 7);
        assert_eq!(game.object_view(redirector).damage, 2);
        assert_eq!(game.state.players[opponent.index()].life, 18);
    }
}
