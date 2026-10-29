use super::*;

fn identity_activate(g: &mut Game, source: CardId, target: CardId) {
    act(g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
    act(g, Action::ChooseTarget { target: Target::Card(target) });
    pay(g);
}
fn identity_reveal(g: &mut Game) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("reveal did not begin");
}

#[test]
fn identity_echo_exiles_controlled_permanent_and_publicly_reveals_to_first_match() {
    for p in [Player::P0, Player::P1] {
        for matching in ["Bear Cub", "Ajani, Caller of the Pride"] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Identity Echo", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.change_controller(g.object_ref(target), p);
            let deeper = put(&mut g, p, "Shock", Zone::Library);
            let found = put(&mut g, p, matching, Zone::Library);
            let first = put(&mut g, p, "Forest", Zone::Library);
            let second = put(&mut g, p, "Mountain", Zone::Library);
            identity_activate(&mut g, source, target);
            identity_reveal(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            for viewer in [p, p.other()] {
                let frame = g.observe(viewer).library_selection.unwrap();
                assert!(frame.public);
                assert_eq!(frame.remaining.iter().map(|o| o.id).collect::<Vec<_>>(), vec![second, first, found]);
                assert_eq!(frame.candidates, vec![g.object_ref(found)]);
            }
            reject(&mut g, Action::SelectLibrary { card: first });
            super::contracts::branch_contract(g.clone());
            act(&mut g, Action::SelectLibrary { card: found });
            act(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            assert_eq!(g.object_view(found).zone, Zone::Battlefield);
            assert_eq!(g.object_view(found).controller, p);
            assert_eq!(g.state.players[p.index()].library.last(), Some(&deeper));
            let mut bottom = g.state.players[p.index()].library[..2].to_vec();
            bottom.sort_unstable();
            let mut expected = vec![first, second]; expected.sort_unstable();
            assert_eq!(bottom, expected);
        }
    }
}

#[test]
fn identity_echo_rejects_opponents_and_stops_when_target_is_illegal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Identity Echo", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        reject(&mut g, Action::ChooseTarget { target: Target::Card(opponent) });
        act(&mut g, Action::ChooseTarget { target: Target::Card(own) });
        pay(&mut g);
        g.change_controller(g.object_ref(own), p.other());
        let library = g.state.players[p.index()].library.clone();
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].library, library);
    }
}

#[test]
fn identity_echo_empty_result_still_reveals_and_randomizes_the_rest() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Identity Echo", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let before = g.state.players[p.index()].library.clone();
        identity_activate(&mut g, source, target);
        identity_reveal(&mut g);
        let view = g.observe(p.other()).library_selection.unwrap();
        assert!(view.public);
        assert_eq!(view.remaining.len(), before.len());
        assert!(view.candidates.is_empty());
        act(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        let mut after = g.state.players[p.index()].library.clone(); after.sort_unstable();
        let mut expected = before; expected.sort_unstable();
        assert_eq!(after, expected);
    }
}
