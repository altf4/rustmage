//! Additional Eye and Relic timing and legality checks.
use super::*;

fn reach(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!(
        "conditional-artifact choice not reached: {:?}",
        g.state.pending
    );
}

#[test]
fn eye_with_empty_library_still_checks_existing_seven_card_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let eye = put(&mut g, p, "Eye of Jace", Zone::Battlefield);
        let library = g.state.players[p.index()].library.clone();
        for card in library {
            g.move_object(card, Zone::Exile);
        }
        for _ in 0..7 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(eye).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(g.state.players[p.other().index()].life, 18);
        }
    }
}

#[test]
fn relic_paid_reflexive_fizzles_after_target_changes_control_or_incarnation() {
    for p in [Player::P0, Player::P1] {
        for control_change in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let relic = put(&mut g, p, "Lich's Relic", Zone::Hand);
            cast(&mut g, relic, None);
            reach(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            reach(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            if control_change {
                let reference = g.object_ref(target);
                g.change_controller(reference, p);
            } else {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn relic_equip_cannot_be_announced_at_instant_speed_or_without_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let relic = put(&mut g, p, "Lich's Relic", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let action = Action::AnnounceActivate {
            card: relic,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::BeginCombat;
        reject(&mut g, action.clone());
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, action);
        assert_eq!(g.object_view(relic).attached_to, None);
        assert_eq!(stats(&g, target), (2, 2));
    }
}
