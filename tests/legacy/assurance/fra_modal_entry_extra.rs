use super::*;

fn yuriko(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Yuriko, Hope from the Shadows").unwrap();
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
    panic!("missing Yuriko entry mode")
}

#[test]
fn arbiter_empty_target_pool_and_changed_target_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = entry_choice(&mut g, p);
        reject(&mut g, Action::ChooseMode { mode: 0 });
        let life = g.state.players[p.index()].life;
        act(&mut g, Action::ChooseMode { mode: 1 });
        let r = g.object_ref(source);
        g.change_controller(r, p.other());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 4);

        for replacement in ["Bear Cub", "Forest"] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
            entry_choice(&mut g, p);
            act(&mut g, Action::ChooseMode { mode: 0 });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            let r = g.object_ref(victim);
            g.apply_temporary_copy(
                r,
                CopyRecipe::new(Card::from_name(replacement).unwrap()),
                CopyDeadline::EndOfTurn,
            );
            let snapshot = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&snapshot).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn yuriko_surveil_short_library_and_trigger_controller() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1] {
            let mut g = scenario(p);
            let library = g.state.players[p.index()].library.clone();
            for &card in library.iter().skip(count) {
                g.move_object(card, Zone::Exile);
            }
            let source = yuriko(&mut g, p);
            act(&mut g, Action::ChooseMode { mode: 1 });
            let r = g.object_ref(source);
            g.change_controller(r, p.other());
            let snapshot = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&snapshot).unwrap();
                }
                for _ in 0..20 {
                    if matches!(g.state.pending, Pending::EffectLibrary { .. })
                        || (g.state.stack.is_empty() && g.state.resolving.is_none())
                    {
                        break;
                    }
                    act(&mut g, Action::Pass);
                }
                if count == 1 {
                    assert_eq!(g.actor(), Some(p));
                    assert!(g.observe(p).library_choice.is_some());
                    assert!(g.observe(p.other()).library_choice.is_none());
                    act(
                        &mut g,
                        Action::ArrangeLibrary {
                            card: library[0],
                            destination: LibraryDestination::Graveyard,
                        },
                    );
                }
                settle(&mut g);
                assert!(g.state.players[p.index()].library.is_empty());
                if count == 1 {
                    assert_eq!(g.object_view(library[0]).zone, Zone::Graveyard);
                }
            }
        }
    }
}
