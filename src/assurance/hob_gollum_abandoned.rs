//! Oracle outcomes for optional graveyard targeting and paid graveyard recovery.
use super::*;

#[test]
fn abandoned_gollum_paid_entry_optional_exile_and_all_target_illegal_fizzle() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Forest", Zone::Graveyard);
            let own = put(&mut g, p, "Forest", Zone::Graveyard);
            let gollum = put(&mut g, p, "Gollum the Abandoned", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
            let life = g.state.players[p.other().index()].life;
            cast(&mut g, gollum, None);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            if mode != 0 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            if mode == 2 {
                g.move_object(enemy, Zone::Hand);
                g.give_priority();
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if mode == 2 { 0 } else { 2 }
            );
            assert_eq!(
                g.object_view(enemy).zone,
                match mode {
                    0 => Zone::Graveyard,
                    1 => Zone::Exile,
                    _ => Zone::Hand,
                }
            );
            assert_eq!(stats(&g, gollum), (2, 2));
            assert!(g.keywords(gollum).contains(Keywords::CANT_BLOCK));
            g.blank_and_grant_haste(g.object_ref(gollum), g.object_ref(gollum));
            assert!(!g.keywords(gollum).contains(Keywords::CANT_BLOCK));
        }
    }
}

#[test]
fn abandoned_gollum_graveyard_recovery_pays_two_and_artifact_or_creature_as_sorcery() {
    for p in [Player::P0, Player::P1] {
        for fuel_name in ["Grizzly Bears", "Goldvein Pick"] {
            let mut g = scenario(p);
            let gollum = put(&mut g, p, "Gollum the Abandoned", Zone::Graveyard);
            let fuel = put(&mut g, p, fuel_name, Zone::Battlefield);
            let wrong = put(&mut g, p, "Pacifism", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), fuel_name, Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            let start = Action::AnnounceActivate {
                card: gollum,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.active = p.other();
            reject(&mut g, start.clone());
            g.state.active = p;
            act(&mut g, start);
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            for id in [wrong, enemy] {
                let object = g.object_ref(id);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            contracts::branch_contract(g.clone());
            let object = g.object_ref(fuel);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(gollum).zone, Zone::Hand);
        }
    }
}
