//! Additional noncombat-history consumer interactions.
use super::*;

#[test]
fn repriser_prowess_counts_only_its_controllers_noncreature_casts_and_expires() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let repriser = put(&mut g, player, "Grim Repriser", Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert_eq!(stats(&g, repriser), (2, 2));
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(player.other())));
        settle(&mut g);
        assert_eq!(stats(&g, repriser), (3, 3));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.finish_cleanup();
        assert_eq!(stats(&g, repriser), (2, 2));
        g.restore(&saved).unwrap();
        g.finish_cleanup();
        assert_eq!(stats(&g, repriser), (2, 2));
    }
}

#[test]
fn repriser_return_does_not_find_a_new_graveyard_incarnation() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let repriser = put(&mut g, player, "Grim Repriser", Zone::Graveyard);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.apply_damage_batch(&[DamageEvent {
            source: g.damage_source(source),
            target: Target::Player(player.other()),
            amount: 1,
            combat: false,
            preventable: true,
        }]);
        let old = g.object_ref(repriser);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: repriser,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        g.move_object(repriser, Zone::Exile);
        g.move_object(repriser, Zone::Graveyard);
        assert_ne!(g.object_ref(repriser), old);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(repriser).zone, Zone::Graveyard);
            assert_eq!(g.counter_count(repriser, CounterKind::Finality), 0);
        }
    }
}

#[test]
fn command_stage_uses_opponents_last_turn_damage_not_this_turn_or_controllers_damage() {
    for player in [Player::P0, Player::P1] {
        for last_recipient in [player, player.other()] {
            for active in [player, player.other()] {
                let mut g = scenario(player);
                let spell = put(&mut g, player, "Command the Stage", Zone::Graveyard);
                let dealer = put(&mut g, player, "Bear Cub", Zone::Battlefield);
                g.apply_damage_batch(&[DamageEvent {
                    source: g.damage_source(dealer),
                    target: Target::Player(last_recipient),
                    amount: 1,
                    combat: false,
                    preventable: true,
                }]);
                g.finish_cleanup();
                // Current-turn damage must not stand in for the previous turn.
                g.apply_damage_batch(&[DamageEvent {
                    source: g.damage_source(dealer),
                    target: Target::Player(player.other()),
                    amount: 1,
                    combat: false,
                    preventable: true,
                }]);
                g.state.active = active;
                g.begin_step(Step::Upkeep);
                g.prepare_triggers();
                let saved = g.snapshot();
                super::super::contracts::branch_contract(g.clone());
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&saved).unwrap();
                    }
                    while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                        choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
                    }
                    settle(&mut g);
                    assert_eq!(
                        g.object_view(spell).zone,
                        if last_recipient == player.other() {
                            Zone::Hand
                        } else {
                            Zone::Graveyard
                        }
                    );
                }
            }
        }
    }
}
