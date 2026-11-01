use super::*;

// Gated card definitions are exercised directly, without opening them to decks.
fn draft(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let id = put(g, owner, "Bear Cub", zone);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

#[test]
fn rattler_paid_evasion_rejects_real_block_and_failed_activation_is_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = draft(&mut g, p, "Eardrum Rattler", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, activation.clone());
        assert!(!g.object_view(source).tapped);
        g.state.players[p.index()].mana[5] = 1;
        act(&mut g, activation);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        let before = g.snapshot();
        super::contracts::branch_contract(g.clone());
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        settle(&mut g);
        g.restore(&before).unwrap();
        pay(&mut g);
        settle(&mut g);
        let life = g.state.players[p.other().index()].life;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..8 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::Blockers);
        reject(&mut g, Action::Block { blocker, attacker });
        act(&mut g, Action::FinishBlockers);
        for _ in 0..32 {
            if g.state.step == Step::Main2 {
                break;
            }
            if g.state.pending == Pending::Priority {
                act(&mut g, Action::Pass);
            } else {
                choose(&mut g, |a| !matches!(a, Action::Concede));
            }
        }
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.object_view(blocker).zone, Zone::Battlefield);
    }
}

#[test]
fn dark_matter_mills_only_remaining_cards_and_respects_exile_replacement() {
    for p in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut g = scenario(p);
            while g.state.players[p.index()].library.len() > 2 {
                let id = *g.state.players[p.index()].library.last().unwrap();
                g.move_object(id, Zone::Exile);
            }
            if replacement {
                put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
            }
            let creature = draft(&mut g, p, "Dark Matter Manipulator", Zone::Hand);
            cast(&mut g, creature, None);
            super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), 0);
            let in_graveyard = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
                .count();
            assert_eq!(in_graveyard, if replacement { 0 } else { 2 });
            assert_eq!(stats(&g, creature), (1, 2));
            assert!(g.outcome().is_none());
        }
    }
}

#[test]
fn dark_matter_live_six_seven_and_thirteen_fourteen_boundaries_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = draft(&mut g, p, "Dark Matter Manipulator", Zone::Battlefield);
        g.register_battlefield_sources(creature);
        for _ in 0..6 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        assert_eq!(stats(&g, creature), (1, 2));
        let seventh = put(&mut g, p, "Forest", Zone::Library);
        let before = g.snapshot();
        g.move_object(seventh, Zone::Graveyard);
        assert_eq!(stats(&g, creature), (3, 2));
        super::contracts::branch_contract(g.clone());
        g.restore(&before).unwrap();
        assert_eq!(stats(&g, creature), (1, 2));
        g.move_object(seventh, Zone::Graveyard);
        for _ in 0..6 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        assert_eq!(stats(&g, creature), (3, 2));
        let fourteenth = put(&mut g, p, "Forest", Zone::Library);
        g.move_object(fourteenth, Zone::Graveyard);
        assert_eq!(stats(&g, creature), (5, 2));
        g.move_object(fourteenth, Zone::Exile);
        assert_eq!(stats(&g, creature), (3, 2));
    }
}
