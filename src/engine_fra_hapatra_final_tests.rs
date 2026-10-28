use super::*;

fn choose_entry_target(g: &mut Game, p: Player, name: &str, victim: CardId) {
    let source = put(g, p, name, Zone::Hand);
    cast(g, source, None);
    let mut selected = false;
    for _ in 0..48 {
        g.prepare_triggers();
        let actions = g.legal_actions();
        let choice = Action::ChooseTarget {
            target: Target::Card(victim),
        };
        if !selected && actions.contains(&choice) {
            act(g, choice);
            selected = true;
        } else if actions.contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
            assert!(selected);
            return;
        } else if selected && g.state.pending == Pending::Priority {
            return;
        } else if let Some(action) = actions
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
        {
            act(g, action);
        } else {
            panic!("unexpected entry target choice {:?}", g.state.pending);
        }
    }
    panic!("entry target was not selected")
}

#[test]
fn hapatra_frost_untapped_target_legality_and_unfunded_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Hapatra, the Desert Frost", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        reject(&mut g, activation.clone());
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut g, activation);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        let snapshot = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&snapshot).unwrap();
            }
            settle(&mut g);
            assert!(!g.object_view(own).tapped);
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn hapatra_entry_does_not_affect_reentered_target() {
    for p in [Player::P0, Player::P1] {
        for name in ["Hapatra, the Desert Fang", "Hapatra, the Desert Frost"] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
            put(&mut g, p, "Bear Cub", Zone::Graveyard);
            choose_entry_target(&mut g, p, name, victim);
            let old = g.object_ref(victim);
            g.move_object(victim, Zone::Exile);
            g.move_object(victim, Zone::Battlefield);
            assert_ne!(g.object_ref(victim), old);
            let snapshot = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&snapshot).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 0);
                assert_eq!(g.counter_count(victim, CounterKind::Stun), 0);
                assert!(!g.object_view(victim).tapped);
            }
        }
    }
}

#[test]
fn hapatra_fang_counts_graveyard_owner_not_previous_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let foreign = put(
            &mut g,
            p.other(),
            "Ghalta, Primal Hunger",
            Zone::Battlefield,
        );
        let reference = g.object_ref(foreign);
        g.change_controller(reference, p);
        g.move_object(foreign, Zone::Graveyard);
        assert_eq!(g.object_view(foreign).owner, p.other());
        put(&mut g, p, "Bear Cub", Zone::Graveyard);
        choose_entry_target(&mut g, p, "Hapatra, the Desert Fang", victim);
        let snapshot = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&snapshot).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 2);
        }
    }
}
