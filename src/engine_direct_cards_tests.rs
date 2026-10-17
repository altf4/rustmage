//! Actual paid casts of every authored combination, using shared owned replay checks.
use super::*;

fn direct_cast(g: &mut Game, id: CardId, mode: u16, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    reject(g, Action::ChooseMode { mode: u16::MAX });
    checked(g, Action::ChooseMode { mode });
    reject(g, Action::ChooseMode { mode });
    for &target in targets {
        if !g.legal_actions().contains(&Action::ChooseTarget { target }) {
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(g, Action::FinishTargets);
        } else {
            reject(g, Action::PayCosts { payment: [0; 6] });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(g, pay);
        }
    }
    panic!("unfinished authored cast");
}
#[test]
fn every_complete_authored_mode_combination_is_paid_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (name, count) in [
            ("Explosive Derailment", 3),
            ("Jailbreak Scheme", 3),
            ("Requisition Raid", 7),
            ("Rustler Rampage", 3),
        ] {
            for mode in 0..count {
                let mut g = setup(p);
                let q = p.other();
                let mask = mode + 1;
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = named(&mut g, q, "Bear Cub", Zone::Battlefield);
                let extra = named(&mut g, q, "Bear Cub", Zone::Battlefield);
                let artifact = named(&mut g, q, "Hullcarver", Zone::Battlefield);
                let enchantment = named(&mut g, q, "Anthem of Champions", Zone::Battlefield);
                if name == "Rustler Rampage" {
                    for id in [own, enemy, extra] {
                        g.state.objects[id as usize].tapped = true;
                    }
                    g.state.objects[enemy as usize].counters.stun = 1;
                }
                let id = named(&mut g, p, name, Zone::Hand);
                let mut targets = Vec::new();
                match name {
                    "Explosive Derailment" | "Jailbreak Scheme" => {
                        if mask & 1 != 0 {
                            targets.push(Target::Card(enemy));
                        }
                        if mask & 2 != 0 {
                            targets.push(Target::Card(artifact));
                        }
                    }
                    "Requisition Raid" => {
                        if mask & 1 != 0 {
                            targets.push(Target::Card(artifact));
                        }
                        if mask & 2 != 0 {
                            targets.push(Target::Card(enchantment));
                        }
                        if mask & 4 != 0 {
                            targets.push(Target::Player(q));
                        }
                    }
                    _ => {
                        if mask & 1 != 0 {
                            targets.push(Target::Player(q));
                        }
                        if mask & 2 != 0 {
                            targets.push(Target::Card(own));
                        }
                    }
                }
                direct_cast(&mut g, id, mode, &targets);
                let item = g.state.stack.last().unwrap();
                assert_eq!(item.mana_value(), 1);
                let extra_cost = match name {
                    "Explosive Derailment" => 2 * mask.count_ones(),
                    "Jailbreak Scheme" => {
                        if mask == 1 {
                            3
                        } else if mask == 2 {
                            2
                        } else {
                            5
                        }
                    }
                    _ => mask.count_ones(),
                };
                assert_eq!(
                    mana_sum(item.cast.unwrap().payment),
                    u64::from(extra_cost + 1)
                );
                settle(&mut g);
                match name {
                    "Explosive Derailment" => {
                        assert_eq!(g.on_battlefield(enemy), mask & 1 == 0);
                        assert_eq!(g.on_battlefield(artifact), mask & 2 == 0);
                    }
                    "Jailbreak Scheme" => {
                        assert_eq!(
                            g.state.objects[enemy as usize].counters.plus_one,
                            u64::from(mask & 1 != 0)
                        );
                        assert_eq!(
                            g.keywords(enemy).contains(Keywords::CANT_BE_BLOCKED),
                            mask & 1 != 0
                        );
                        assert_eq!(
                            g.state.objects[artifact as usize].zone,
                            if mask & 2 != 0 {
                                Zone::Library
                            } else {
                                Zone::Battlefield
                            }
                        );
                        g.finish_cleanup();
                        assert!(!g.keywords(enemy).contains(Keywords::CANT_BE_BLOCKED));
                    }
                    "Requisition Raid" => {
                        assert_eq!(g.on_battlefield(artifact), mask & 1 == 0);
                        assert_eq!(g.on_battlefield(enchantment), mask & 2 == 0);
                        for body in [enemy, extra] {
                            assert_eq!(
                                g.state.objects[body as usize].counters.plus_one,
                                u64::from(mask & 4 != 0)
                            );
                        }
                        assert_eq!(g.state.objects[own as usize].counters.plus_one, 0);
                    }
                    _ => {
                        assert!(g.state.objects[own as usize].tapped);
                        assert!(g.state.objects[enemy as usize].tapped);
                        assert_eq!(
                            g.state.objects[enemy as usize].counters.stun,
                            u64::from(mask & 1 == 0)
                        );
                        assert_eq!(g.state.objects[extra as usize].tapped, mask & 1 == 0);
                        assert_eq!(
                            g.keywords(own).contains(Keywords::DOUBLE_STRIKE),
                            mask & 2 != 0
                        );
                        g.finish_cleanup();
                        assert!(!g.keywords(own).contains(Keywords::DOUBLE_STRIKE));
                    }
                }
            }
        }
    }
}
#[test]
fn free_base_does_not_waive_spree_additions_and_compact_cast_cannot_bypass_modes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let q = p.other();
        let bear = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, q, "Gilded Lotus", Zone::Battlefield);
        let id = named(&mut g, p, "Explosive Derailment", Zone::Exile);
        let object = g.object_ref(id);
        let turn = g.state.turn;
        g.state
            .plot
            .get_or_insert_with(Default::default)
            .cards
            .push(PlottedCard {
                object,
                owner: p,
                plotted_turn: turn - 1,
            });
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnouncePlotted { card: id });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::AnnouncePlotted { card: id });
        assert_eq!(g.observe(p).mode_choices.len(), 3);
        assert!(g.observe(q).mode_choices.is_empty());
        checked(&mut g, Action::ChooseMode { mode: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.required, [0; 6]);
        assert_eq!(request.generic, 4);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert!(!g.on_battlefield(bear));
        assert!(!g.on_battlefield(artifact));
        let id = named(&mut g, p, "Rustler Rampage", Zone::Hand);
        reject(
            &mut g,
            Action::Cast {
                card: id,
                target: Some(Target::Card(bear)),
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
    }
}
