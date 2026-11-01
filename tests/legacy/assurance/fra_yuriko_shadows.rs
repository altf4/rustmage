use super::*;

fn entry_choice(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Yuriko, Hope from the Shadows").unwrap();
    g.state.active = p.other();
    g.state.priority = p;
    assert!(g.keywords(id).contains(Keywords::FLASH));
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
    panic!("missing Yuriko entry mode");
}

#[test]
fn yuriko_shadows_target_uses_current_own_graveyard_and_expires() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            put(&mut g, p, "Forest", Zone::Graveyard);
            put(&mut g, p, "Forest", Zone::Graveyard);
            put(&mut g, p.other(), "Forest", Zone::Graveyard);
            let source = entry_choice(&mut g, p);
            act(&mut g, Action::ChooseMode { mode: 0 });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            put(&mut g, p, "Forest", Zone::Graveyard);
            let snapshot = g.snapshot();
            super::contracts::branch_contract(g.clone());
            g.move_object(source, Zone::Exile);
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            settle(&mut g);
            if fizzle {
                assert_eq!(g.object_view(target).zone, Zone::Exile);
            } else {
                assert_eq!(stats(&g, target), (-1, 2));
                g.finish_cleanup();
                assert_eq!(stats(&g, target), (2, 2));
            }
            g.restore(&snapshot).unwrap();
            if fizzle {
                g.move_object(target, Zone::Exile);
            }
            settle(&mut g);
            if !fizzle {
                assert_eq!(stats(&g, target), (-1, 2));
            }
        }
    }
}

#[test]
fn yuriko_shadows_surveil_is_private_and_supports_both_destinations() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Forest", Zone::Library);
        let b = put(&mut g, p, "Forest", Zone::Library);
        entry_choice(&mut g, p);
        act(&mut g, Action::ChooseMode { mode: 1 });
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        super::contracts::branch_contract(g.clone());
        let before = g.snapshot();
        reject(
            &mut g,
            Action::ArrangeLibrary {
                card: 0,
                destination: LibraryDestination::Graveyard,
            },
        );
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: b,
                destination: LibraryDestination::Graveyard,
            },
        );
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: a,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&a));
        g.restore(&before).unwrap();
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: b,
                destination: LibraryDestination::Graveyard,
            },
        );
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: a,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    }
}
