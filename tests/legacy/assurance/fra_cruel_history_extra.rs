use super::*;

#[test]
fn cruel_calculations_draws_from_paid_real_mill_and_respects_exile_replacement() {
    for p in [Player::P0, Player::P1] {
        for target in [p, p.other()] {
            for replace in [false, true] {
                let mut g = scenario(p);
                if replace {
                    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
                }
                let crow = put(&mut g, target, "Crow of Dark Tidings", Zone::Hand);
                g.state.active = target;
                g.state.priority = target;
                paid(&mut g, crow, None, &[]);
                finish(&mut g);
                let count = if replace { 0 } else { 2 };
                for viewer in [Player::P0, Player::P1] {
                    assert_eq!(
                        g.observe(viewer).players[target.index()]
                            .cards_library_to_graveyard_this_turn,
                        count
                    );
                }
                g.state.active = p;
                g.state.priority = p;
                g.state.step = Step::Main1;
                let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
                g.state.objects[spell as usize].card =
                    Card::from_name("Cruel Calculations").unwrap();
                let before = g.state.players[p.index()].library.len();
                paid(&mut g, spell, None, &[Target::Player(target)]);
                let saved = g.snapshot();
                super::super::contracts::branch_contract(g.clone());
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&saved).unwrap();
                    }
                    finish(&mut g);
                    assert_eq!(
                        g.state.players[p.index()].library.len(),
                        before - count as usize
                    );
                }
            }
        }
    }
}

#[test]
fn cruel_calculations_history_survives_irreversible_mana_rollback() {
    for p in [Player::P0, Player::P1] {
        let (mut g, silver, enemy, prison) = setup(p);
        let crow = put(&mut g, p, "Crow of Dark Tidings", Zone::Hand);
        paid(&mut g, crow, None, &[]);
        finish(&mut g);
        assert_eq!(
            g.observe(p.other()).players[p.index()].cards_library_to_graveyard_this_turn,
            2
        );
        g.move_object(crow, Zone::Exile);
        let colossus = put(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(colossus),
            },
        );
        pay(&mut g);
        finish(&mut g);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        propose(&mut g, spell, prison, enemy);
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: colossus,
                grantor,
                production: [0, 0, 3, 0, 0, 0],
            },
        );
        let library = g.state.players[p.index()].library.clone();
        fail(&mut g, silver);
        assert!(g.observe(p).mana_rollback.unwrap().irreversible);
        reject(&mut g, Action::ChooseManaRollback { keep: false });
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            checked(&mut g, Action::ChooseManaRollback { keep: true });
            assert_eq!(g.object_view(colossus).zone, Zone::Library);
            assert_eq!(g.state.players[p.index()].library, library);
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    g.observe(viewer).players[p.index()].cards_library_to_graveyard_this_turn,
                    2
                );
            }
        }
    }
}
