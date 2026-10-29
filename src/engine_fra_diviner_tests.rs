use super::*;

fn diviner_finish(g: &mut Game) {
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Diviner resolution did not finish");
}
fn diviner_prepare(g: &mut Game, p: Player) -> (CardId, ObjectRef) {
    let source = put(g, p, "Diviner of Victory // Unwind History", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    diviner_finish(g);
    let permanent = g.object_ref(source);
    assert!(g.is_prepared(permanent));
    let copy = g.state.preparations.as_ref().unwrap().iter().find(|row| row.permanent == permanent).unwrap().spell_copy;
    assert_eq!(g.state.objects[copy.card as usize].card, Card::test_index(5458));
    (source, copy)
}
fn diviner_wait_library(g: &mut Game) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectLibrary { .. }) { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("surveil did not start");
}

#[test]
fn diviner_prepared_spell_bounces_then_surveils_and_triggers_itself() {
    for p in [Player::P0, Player::P1] {
        for destination in [LibraryDestination::Top, LibraryDestination::Graveyard] {
            let mut g = scenario(p);
            let (source, copy) = diviner_prepare(&mut g, p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let expensive = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let top = *g.state.players[p.index()].library.last().unwrap();
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
            act(&mut g, Action::AnnounceExile { card: copy.card });
            reject(&mut g, Action::ChooseTarget { target: Target::Card(expensive) });
            reject(&mut g, Action::ChooseTarget { target: Target::Card(own) });
            act(&mut g, Action::ChooseTarget { target: Target::Card(victim) });
            pay(&mut g);
            assert!(!g.is_prepared(g.object_ref(source)));
            diviner_wait_library(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Hand);
            assert!(g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            super::contracts::branch_contract(g.clone());
            choose(&mut g, |a| matches!(a, Action::ArrangeLibrary { destination: d, .. } if *d == destination));
            diviner_finish(&mut g);
            assert_eq!(stats(&g, source), (2, 2));
            assert_eq!(g.object_view(top).zone, if destination == LibraryDestination::Top { Zone::Library } else { Zone::Graveyard });
            g.finish_cleanup();
            assert_eq!(stats(&g, source), (1, 1));
        }
    }
}

#[test]
fn diviner_fizzled_prepared_copy_is_consumed_without_surveilling() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (source, copy) = diviner_prepare(&mut g, p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceExile { card: copy.card });
        act(&mut g, Action::ChooseTarget { target: Target::Card(victim) });
        pay(&mut g);
        g.change_controller(g.object_ref(victim), p);
        let before = g.state.players[p.index()].library.clone();
        diviner_finish(&mut g);
        assert!(!g.is_prepared(g.object_ref(source)));
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].library, before);
        assert_eq!(stats(&g, source), (1, 1));
    }
}

#[test]
fn diviner_grows_for_own_scry_but_not_opponents_scry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (source, _) = diviner_prepare(&mut g, p);
        let spell = put(&mut g, p, "Plunge into Winter", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        diviner_wait_library(&mut g);
        choose(&mut g, |a| matches!(a, Action::ArrangeLibrary { destination: LibraryDestination::Top, .. }));
        diviner_finish(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (1, 1));
        let opposing_spell = put(&mut g, p.other(), "Plunge into Winter", Zone::Hand);
        g.state.players[p.other().index()].mana = [1, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: opposing_spell });
        pay(&mut g);
        diviner_wait_library(&mut g);
        choose(&mut g, |a| matches!(a, Action::ArrangeLibrary { destination: LibraryDestination::Top, .. }));
        diviner_finish(&mut g);
        assert_eq!(stats(&g, source), (1, 1));
    }
}
