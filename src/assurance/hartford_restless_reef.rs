use super::super::ability_world::CharacteristicWorld;
use super::*;

#[test]
fn restless_reef_entry_mana_animation_and_owned_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Restless Reef", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        assert!(g.state.objects[land as usize].tapped);
        reject(&mut g, Action::TapMana { card: land });
        for (ability, color) in [(0, 1), (1, 2)] {
            let mut branch = g.clone();
            branch.state.objects[land as usize].tapped = false;
            branch.state.players[p.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[color] = 1;
            choose(
                &mut branch,
                |a| matches!(a, Action::ActivateMana { card, production: got, .. } if *card == land && *got == production),
            );
            assert_eq!(
                branch.state.players[p.index()].mana,
                production,
                "mana ability {ability}"
            );
        }
        g.state.players[p.index()].mana = [0, 1, 1, 0, 0, 2];
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        let snapshot = g.snapshot();
        let mut branch = g.clone();
        settle(&mut g);
        settle(&mut branch);
        assert_eq!(g.snapshot(), branch.snapshot());
        branch.restore(&snapshot).unwrap();
        settle(&mut branch);
        assert_eq!(g.snapshot(), branch.snapshot());
        assert_eq!(stats(&g, land), (4, 4));
        assert!(g.is_creature(land));
        assert_ne!(g.kinds(land) & 2, 0, "animated Reef remains a land");
        assert!(g.keywords(land).contains(Keywords::DEATHTOUCH));
        assert_eq!(g.colors(land), 2 | 4);
        assert!(
            g.subtypes(land)
                .has(g.state.objects[land as usize].card, "Shark")
        );
        g.finish_cleanup();
        assert!(!g.is_creature(land));
        assert_eq!(g.colors(land), 0);
        assert!(!g.keywords(land).contains(Keywords::DEATHTOUCH));
    }
}

#[test]
fn restless_reef_attack_targets_either_player_and_mills_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        for target_player in [p, p.other()] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Restless Reef", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 1, 1, 0, 0, 2];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: land,
                    ability: 2,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            settle(&mut g);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: land });
            act(&mut g, Action::FinishAttackers);
            for _ in 0..16 {
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    .unwrap();
                act(&mut g, a);
            }
            assert!(g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p)
            }));
            assert!(g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other())
            }));
            let before = g.state.players[target_player.index()].library.len();
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(target_player),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            g.move_object(land, Zone::Hand);
            for _ in 0..24 {
                if g.state.players[target_player.index()].library.len() == before - 4 {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    .unwrap();
                act(&mut g, a);
            }
            assert_eq!(
                g.state.players[target_player.index()].library.len(),
                before - 4
            );
        }
    }
}

#[test]
fn restless_reef_stale_animation_does_not_bind_reentry() {
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Restless Reef", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 1, 1, 0, 0, 2];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: land,
            ability: 2,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    g.move_object(land, Zone::Hand);
    g.move_object(land, Zone::Battlefield);
    settle(&mut g);
    assert!(!g.is_creature(land));
    assert_eq!(g.colors(land), 0);
}

#[test]
fn restless_reef_animation_respects_color_timestamps() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Restless Reef", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 10, 10, 0, 10, 10];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        let spell = put(&mut g, p, "Fractalize", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseX { value: 2 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.colors(land), 2 | 16);
        assert_ne!(g.kinds(land) & 2, 0);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.colors(land), 2 | 4);
        assert_eq!(stats(&g, land), (4, 4));
        assert_ne!(g.kinds(land) & 2, 0);
    }
}
