//! Noncombat damage floors and turn history use the source at the damage event.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const RED: u8 = 1 << 3;
const BLUE: u8 = 1 << 1;
const FLOOR: ReplacementSpec = ReplacementSpec::NoncombatDamageToOpponentAtLeastSourcePower {
    color: ChoiceColor::Red,
};
const FLOOR_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, FLOOR)],
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: FLOOR_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FLOOR_GROUPS,
        replacement_effects: &[FLOOR],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const RED_DAMAGE: StatValue = StatValue::NoncombatDamageDealtByColorThisTurn(ChoiceColor::Red);
const HISTORY_GATE: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: RED_DAMAGE,
            minimum: 4,
        },
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn damage(game: &Game, player: Player, source: CardId, color: ChoiceColor) -> i128 {
    let count = game.stat_value(
        StatValue::NoncombatDamageDealtByColorThisTurn(color),
        game.object_ref(source),
        player,
    );
    let observed = game.observe(player.other()).players[player.index()]
        .noncombat_damage_dealt_by_color_this_turn;
    assert_eq!(observed[color.index()], u128::try_from(count).unwrap());
    count
}

fn red_damage(game: &Game, player: Player, source: CardId) -> i128 {
    damage(game, player, source, ChoiceColor::Red)
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn shock(game: &mut Game, caster: Player, target: Target) {
    let spell = put(game, caster, "Shock", Zone::Hand);
    cast(game, spell, Some(target));
}

fn announce_ability(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn affected_player_orders_floor_and_doubler_with_owned_replay() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut root = scenario(player);
        let floor = put(&mut root, player, "Hill Giant", Zone::Battlefield);
        let doubler = put(&mut root, player, "Twinflame Tyrant", Zone::Battlefield);
        shock(&mut root, player, Target::Player(opponent));
        pass_priority(&mut root);

        assert_eq!(root.actor(), Some(opponent));
        assert_eq!(
            root.observe(opponent)
                .replacement_choice
                .unwrap()
                .damage
                .unwrap()
                .amount,
            2
        );
        assert_eq!(red_damage(&root, player, floor), 0);
        let choose_floor = Action::ChooseReplacement {
            source: root.object_ref(floor),
            ability: 0,
        };
        let choose_doubler = Action::ChooseReplacement {
            source: root.object_ref(doubler),
            ability: 0,
        };
        let revision = root.revision();
        let actions = root.legal_actions();
        assert!(actions.contains(&choose_doubler));
        let index = actions.iter().position(|a| *a == choose_floor).unwrap();
        let checkpoint = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        let wrong_slot = Action::ChooseReplacement {
            source: raw.object_ref(floor),
            ability: 1,
        };
        reject(&mut raw, wrong_slot);
        act(&mut raw, choose_floor);
        indexed.apply_index(opponent, revision, index).unwrap();
        settle(&mut raw);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[opponent.index()].life, STARTING_LIFE - 8);
        assert_eq!(red_damage(&raw, player, floor), 8);
        assert_eq!(red_damage(&raw, opponent, floor), 0);

        indexed.restore(&checkpoint).unwrap();
        assert_eq!(red_damage(&indexed, player, floor), 0);
        assert_eq!(
            indexed.apply_index(opponent, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, choose_doubler);
        settle(&mut indexed);
        assert_eq!(
            indexed.state.players[opponent.index()].life,
            STARTING_LIFE - 4
        );
        assert_eq!(red_damage(&indexed, player, floor), 4);
        assert_eq!(red_damage(&raw, player, floor), 8);
    }
}

#[test]
fn floor_uses_current_power_and_excludes_self_damage_and_combat() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        for power in [-2i128, 0, 1, 2, 5] {
            let mut game = scenario(player);
            let floor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
            shock(&mut game, player, Target::Player(opponent));
            game.state.objects[floor as usize].growth.power = power - 4;
            settle(&mut game);
            let dealt = power.max(2);
            assert_eq!(
                game.state.players[opponent.index()].life,
                STARTING_LIFE - dealt
            );
            assert_eq!(red_damage(&game, player, floor), dealt);
        }

        let mut game = scenario(player);
        let floor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let attacker = put(&mut game, player, "Fanatical Firebrand", Zone::Battlefield);
        shock(&mut game, player, Target::Player(player));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 2);
        assert_eq!(red_damage(&game, player, floor), 2);

        pass_priority(&mut game);
        pass_priority(&mut game);
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        pass_priority(&mut game);
        act(&mut game, Action::FinishBlockers);
        pass_priority(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE - 1);
        assert_eq!(red_damage(&game, player, floor), 2);
    }
}

#[test]
fn live_source_color_and_control_are_checked_when_damage_is_dealt() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        for (colors, controller, dealt) in [(BLUE, player, 2), (RED, opponent, 2), (RED, player, 4)]
        {
            let mut game = scenario(player);
            let floor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
            let dealer = put(&mut game, player, "Guttersnipe", Zone::Battlefield);
            let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
            let murder = put(&mut game, player, "Murder", Zone::Hand);
            cast(&mut game, murder, Some(Target::Card(creature)));
            let source = game.object_ref(dealer);
            if controller != player {
                game.record_control(source, controller, None);
            }
            game.set_color_effects(&[dealer], colors, AnimationDuration::Permanent);
            settle(&mut game);
            assert_eq!(
                game.state.players[opponent.index()].life,
                STARTING_LIFE - dealt
            );

            game.record_control(source, controller.other(), None);
            game.set_color_effects(&[dealer], 0, AnimationDuration::Permanent);
            assert_eq!(
                red_damage(&game, controller, floor),
                if colors == RED { dealt } else { 0 }
            );
            assert_eq!(red_damage(&game, controller.other(), floor), 0);
        }
    }
}

#[test]
fn departed_source_uses_its_last_color_and_controller_after_reentry() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        for (colors, dealt) in [(BLUE, 1), (RED | BLUE, 4)] {
            let mut game = scenario(player);
            let floor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
            let dealer = put(
                &mut game,
                opponent,
                "Fanatical Firebrand",
                Zone::Battlefield,
            );
            let old = game.object_ref(dealer);
            game.record_control(old, player, None);
            game.set_color_effects(&[dealer], colors, AnimationDuration::Permanent);
            act(&mut game, announce_ability(dealer));
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Player(opponent),
                },
            );
            pay(&mut game);
            assert_eq!(game.object_view(dealer).zone, Zone::Graveyard);
            game.move_object(dealer, Zone::Battlefield);
            assert_ne!(game.object_ref(dealer), old);
            assert_eq!(game.object_view(dealer).controller, opponent);
            game.set_color_effects(&[dealer], 0, AnimationDuration::Permanent);
            settle(&mut game);

            assert_eq!(
                game.state.players[opponent.index()].life,
                STARTING_LIFE - dealt
            );
            assert_eq!(damage(&game, player, floor, ChoiceColor::Blue), dealt);
            assert_eq!(
                red_damage(&game, player, floor),
                if colors & RED != 0 { dealt } else { 0 }
            );
            assert_eq!(red_damage(&game, opponent, floor), 0);
        }
    }
}

#[test]
fn prevention_can_precede_or_follow_the_floor_and_excludes_the_hit_from_history() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(opponent);
        let floor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let light = put(&mut game, opponent, "Morningtide's Light", Zone::Hand);
        cast(&mut game, light, None);
        settle(&mut game);
        act(&mut game, Action::Pass);
        shock(&mut game, player, Target::Player(opponent));
        pass_priority(&mut game);
        assert_eq!(game.actor(), Some(opponent));

        let mut prevented_first = game.clone();
        choose(&mut prevented_first, |action| {
            matches!(action, Action::ChooseDamagePrevention { .. })
        });
        let choose_floor = Action::ChooseReplacement {
            source: game.object_ref(floor),
            ability: 0,
        };
        act(&mut game, choose_floor);
        for mut branch in [game, prevented_first] {
            settle(&mut branch);
            assert_eq!(branch.state.players[opponent.index()].life, STARTING_LIFE);
            assert_eq!(red_damage(&branch, player, floor), 0);
        }
    }
}

#[test]
fn finalized_nonplayer_damage_from_multiple_sources_unlocks_the_turn_guard() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PROVIDER),
        (named("Promising Vein"), &HISTORY_GATE),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let land = put(&mut game, player, "Promising Vein", Zone::Battlefield);
        let floor = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let creature = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let walker = put(&mut game, opponent, "Jace Reawakened", Zone::Battlefield);
        game.add_counter(walker, CounterKind::Loyalty, 6);
        let battle = spawn(
            &mut game,
            opponent,
            named("@fixture/siege-front"),
            Zone::Battlefield,
        );
        game.add_counter(battle, CounterKind::Defense, 6);
        game.set_battle_protector(game.object_ref(battle), player);
        game.add_counter(floor, CounterKind::Shield, 1);
        let pyroclasm = put(&mut game, player, "Pyroclasm", Zone::Hand);
        let activate = announce_ability(land);
        reject(&mut game, activate.clone());

        cast(&mut game, pyroclasm, None);
        settle(&mut game);
        assert_eq!(game.object_view(creature).damage, 2);
        assert_eq!(game.object_view(floor).damage, 0);
        assert_eq!(game.counter_count(floor, CounterKind::Shield), 0);
        assert_eq!(red_damage(&game, player, land), 2);
        reject(&mut game, activate.clone());

        shock(&mut game, player, Target::Card(battle));
        settle(&mut game);
        assert_eq!(game.counter_count(battle, CounterKind::Defense), 4);
        assert_eq!(red_damage(&game, player, land), 4);
        assert!(game.legal_actions().contains(&activate));

        shock(&mut game, player, Target::Card(walker));
        settle(&mut game);
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 4);
        assert_eq!(red_damage(&game, player, land), 6);
        assert_eq!(red_damage(&game, opponent, land), 0);
        act(&mut game, activate);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 1);

        game.state.step = Step::End;
        pass_priority(&mut game);
        assert_eq!(game.state.active, opponent);
        assert_eq!(red_damage(&game, player, land), 0);
        assert_eq!(red_damage(&game, opponent, land), 0);
    }
}
