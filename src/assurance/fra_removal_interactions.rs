use super::*;

fn draft(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let id = put(g, owner, "Bear Cub", zone);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    if zone == Zone::Battlefield {
        g.register_battlefield_sources(id);
    }
    id
}

#[test]
fn prophesied_end_rechecks_hexproof_and_draws_when_destruction_is_replaced() {
    for p in [Player::P0, Player::P1] {
        for invalid in [false, true] {
            let mut g = scenario(p);
            if !invalid {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = draft(&mut g, p, "Prophesied End", Zone::Hand);
            let before = g.state.players[p.other().index()].library.len();
            cast(&mut g, spell, Some(Target::Card(victim)));
            if invalid {
                g.state.objects[victim as usize].gained_keywords = Keywords::HEXPROOF;
            }
            let checkpoint = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if invalid {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(
                g.state.players[p.other().index()].library.len(),
                before - usize::from(!invalid)
            );
            g.restore(&checkpoint).unwrap();
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].library.len(),
                before - usize::from(!invalid)
            );
        }
    }
}

#[test]
fn budding_insurgent_requires_sorcery_timing_and_rejects_invalid_cost_transactionally() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = draft(&mut g, p, "Budding Insurgent", Zone::Battlefield);
        let valid = put(&mut g, p.other(), "Phyrexian Arena", Zone::Battlefield);
        let invalid = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.active = p.other();
        reject(&mut g, activation.clone());
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.state.active = p;
        let stack_spell = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, stack_spell, Some(Target::Player(p.other())));
        reject(&mut g, activation.clone());
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        settle(&mut g);
        act(&mut g, activation);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(invalid),
            },
        );
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(valid),
            },
        );
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PayCosts { .. }))
        {
            pay(&mut g);
        }
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(valid).zone, Zone::Graveyard);
    }
}

#[test]
fn budding_insurgent_checks_legendary_enchantment_on_resolution_and_replacement() {
    for p in [Player::P0, Player::P1] {
        for legendary in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            let source = draft(&mut g, p, "Budding Insurgent", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Phyrexian Arena", Zone::Battlefield);
            let before = g.state.players[p.index()].library.len();
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
            {
                pay(&mut g);
            }
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            if legendary {
                // Change the target fixture’s characteristics without a zone change.
                g.state.objects[target as usize].card =
                    Card::from_name("Way of the Mentor").unwrap();
            }
            assert_eq!(g.object_view(target).is_legendary(), legendary);
            let checkpoint = g.snapshot();
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from(legendary)
            );
            g.restore(&checkpoint).unwrap();
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from(legendary)
            );
        }
    }
}
