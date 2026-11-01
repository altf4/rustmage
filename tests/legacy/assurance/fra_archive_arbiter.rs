use super::*;

fn arbiter(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Archive Arbiter").unwrap();
    id
}

fn entry_choice(g: &mut Game, p: Player) -> CardId {
    let id = arbiter(g, p);
    cast(g, id, None);
    for _ in 0..20 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseMode { .. }))
        {
            return id;
        }
        act(g, Action::Pass);
    }
    panic!("missing Archive Arbiter entry mode");
}

#[test]
fn archive_arbiter_entry_modes_target_legality_and_source_independence() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let enchantment = put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let life = g.state.players[p.index()].life;
            let source = entry_choice(&mut g, p);
            assert_eq!(stats(&g, source), (4, 4));
            assert!(g.keywords(source).contains(Keywords::FLYING));
            act(&mut g, Action::ChooseMode { mode });
            if mode == 0 {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(creature),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enchantment),
                    },
                );
            }
            super::contracts::branch_contract(g.clone());
            let snapshot = g.snapshot();
            g.move_object(source, Zone::Exile);
            settle(&mut g);
            assert_eq!(
                g.object_view(enchantment).zone,
                if mode == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.state.players[p.index()].life,
                life + if mode == 1 { 4 } else { 0 }
            );
            g.restore(&snapshot).unwrap();
            g.move_object(source, Zone::Exile);
            settle(&mut g);
            assert_eq!(
                g.object_view(enchantment).zone,
                if mode == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn archive_arbiter_destroy_respects_indestructible_and_target_invalidation() {
    for p in [Player::P0, Player::P1] {
        for invalid in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            entry_choice(&mut g, p);
            act(&mut g, Action::ChooseMode { mode: 0 });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if invalid {
                g.move_object(victim, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if invalid {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[path = "fra_modal_entry_extra.rs"]
mod modal_entry_extra;
