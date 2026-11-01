//! The Aetherspark's complete printed behavior through ordinary actions.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn aetherspark_full_card() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spark = put(&mut game, player, "The Aetherspark", Zone::Hand);
        paid_cast(&mut game, spark, None, &[]);
        finish(&mut game);
        assert_eq!(game.counter_count(spark, CounterKind::Loyalty), 4);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        checked(
            &mut game,
            Action::AnnounceActivate {
                card: spark,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        finish(&mut game);
        assert_eq!(
            game.object_view(spark).attached_to,
            Some(game.object_ref(bear))
        );
        assert_eq!(game.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(spark, CounterKind::Loyalty), 5);

        let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.active = player.other();
        game.state.priority = player.other();
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        let attack_walker = Action::AttackPlaneswalker {
            card: opponent,
            defender: game.object_ref(spark),
        };
        reject(&mut game, attack_walker);
        game.state.active = player;
        game.state.priority = player;
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        checked(&mut game, Action::Attack { card: bear });
        checked(&mut game, Action::FinishAttackers);
        for _ in 0..8 {
            if game.state.pending == Pending::Blockers {
                break;
            }
            checked(&mut game, Action::Pass);
        }
        assert_eq!(game.state.pending, Pending::Blockers);
        checked(&mut game, Action::FinishBlockers);
        for _ in 0..20 {
            if game
                .state
                .pending_triggers
                .iter()
                .chain(game.state.stack.iter())
                .any(|item| item.creature_combat_event.is_some())
            {
                break;
            }
            if let Some(range) = game.damage_assignment_range() {
                checked(
                    &mut game,
                    Action::AssignDamage {
                        target: range.target,
                        amount: range.minimum,
                    },
                );
            } else {
                checked(&mut game, Action::Pass);
            }
        }
        let packet = game
            .state
            .pending_triggers
            .iter()
            .chain(game.state.stack.iter())
            .find(|item| item.creature_combat_event.is_some())
            .expect("equipped combat trigger");
        assert_eq!(packet.creature_combat_event.as_ref().unwrap().amount, 3);
        let saved = game.snapshot();
        finish(&mut game);
        assert_eq!(game.counter_count(spark, CounterKind::Loyalty), 8);
        game.restore(&saved).unwrap();
        finish(&mut game);
        assert_eq!(game.counter_count(spark, CounterKind::Loyalty), 8);

        let mut draw = scenario(player);
        let spark = put(&mut draw, player, "The Aetherspark", Zone::Battlefield);
        draw.state.objects[spark as usize].counters.loyalty = 5;
        checked(
            &mut draw,
            Action::AnnounceActivate {
                card: spark,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut draw);
        finish(&mut draw);
        assert_eq!(draw.hand_count(player), 2);

        let mut mana = scenario(player);
        let spark = put(&mut mana, player, "The Aetherspark", Zone::Battlefield);
        mana.state.objects[spark as usize].counters.loyalty = 10;
        mana.state.players[player.index()].mana = [0; 6];
        checked(
            &mut mana,
            Action::AnnounceActivate {
                card: spark,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut mana);
        mana.resolve_top();
        checked(
            &mut mana,
            Action::ChooseMana {
                production: [0, 0, 0, 10, 0, 0],
            },
        );
        finish(&mut mana);
        assert_eq!(mana.state.players[player.index()].mana, [0, 0, 0, 10, 0, 0]);
    }
}
