//! Additional admission interactions for gated FRA cards.
use super::*;

#[test]
fn refuge_sorcery_speed_and_mana_failure_do_not_consume_the_land() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let land = put(&mut g, player, "Room of Refuge", Zone::Hand);
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        act(&mut g, Action::PlayLand { card: land });
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::White),
            },
        );
        g.untap_objects(&[land], untap_rules::UntapCause::Effect);
        let activation = Action::AnnounceActivate {
            card: land,
            ability: 1,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::BeginCombat;
        reject(&mut g, activation.clone());
        assert!(!g.object_view(land).tapped);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        g.state.step = Step::Main1;
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
        reject(&mut g, activation);
        assert!(!g.object_view(land).tapped);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 0, 0, 4]);
        super::super::contracts::branch_contract(g);
    }
}

#[test]
fn surgeon_death_trigger_rechecks_control_and_target_incarnation() {
    for player in [Player::P0, Player::P1] {
        for change_control in [false, true] {
            let mut g = scenario(player);
            let target = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let surgeon = put(&mut g, player, "Graft Surgeon", Zone::Hand);
            cast(&mut g, surgeon, None);
            settle(&mut g);
            g.add_counter(surgeon, CounterKind::Stun, 2);
            g.move_object(surgeon, Zone::Graveyard);
            let selected = Action::ChooseTarget {
                target: Target::Card(target),
            };
            for _ in 0..48 {
                g.prepare_triggers();
                if g.legal_actions().contains(&selected) {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|action| matches!(action, Action::Pass | Action::PutTrigger { .. }))
                    .expect("death trigger targeting");
                act(&mut g, action);
            }
            act(&mut g, selected);
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            if change_control {
                let reference = g.object_ref(target);
                g.change_controller(reference, player.other());
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
                assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
                assert_eq!(g.counter_count(target, CounterKind::Stun), 0);
            }
        }
    }
}
