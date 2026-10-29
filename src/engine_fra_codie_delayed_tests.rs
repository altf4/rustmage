use super::*;

fn arm_delayed(game: &mut Game, player: Player) {
    let way = put(game, player, "Way of the Cryomancer", Zone::Hand);
    cast(game, way, None);
    settle(game);
    let walker = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && game.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .unwrap();
    let source = game.object_ref(walker);
    let grantor = game.object_ref(way);
    act(
        game,
        Action::AnnouncePlaneswalkerGrant {
            source,
            grantor,
            ability: 0,
        },
    );
    pay(game);
    settle(game);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
}

fn finish_delayed(game: &mut Game) {
    for _ in 0..128 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishStackTargets
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
            )
        });
    }
    panic!("copy triggers did not finish");
}

#[test]
fn codie_and_delayed_trigger_each_copy_a_prepared_sorcery() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        arm_delayed(&mut g, p);
        let codie = put(&mut g, p, "Codie, Ravenous Codex", Zone::Battlefield);
        let source = put(
            &mut g,
            p,
            "Fatehold Chronologist // Peer Review",
            Zone::Battlefield,
        );
        let permanent = g.object_ref(source);
        g.prepare_permanent(permanent);
        let copy = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|row| row.permanent == permanent)
            .unwrap()
            .spell_copy;
        act(&mut g, Action::AnnounceExile { card: copy.card });
        choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
        pay(&mut g);
        assert!(g.observe(p).delayed_spell_copies.is_empty());
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(g.state.ordering_triggers.iter())
                .chain(g.state.stack.iter())
                .any(|item| item.source.is_some_and(|source| source.id == codie) && item.kind != StackKind::Spell)
        );
        let snapshot = g.snapshot();
        for _ in 0..2 {
            finish_delayed(&mut g);
            let cadets = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::test_index(5298)
                })
                .count();
            assert_eq!(cadets, 3);
            g.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn synthetic_noninstant_prepared_capture_keeps_delayed_instant_copy_armed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        arm_delayed(&mut g, p);
        let codie = put(&mut g, p, "Codie, Ravenous Codex", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None);
        let spell_index = g
            .state
            .stack
            .iter()
            .position(|s| s.source.is_some_and(|source| source.id == bear) && s.kind == StackKind::Spell)
            .unwrap();
        let first = g.state.pending_triggers.len();
        let source = g.object_view(codie);
        assert!(g.queue_trigger(&source, 0));
        g.test_capture_noninstant_prepared_cast(spell_index, first);
        assert!(g.state.pending_triggers[first].copy_event.is_some());
        assert_eq!(g.observe(p).delayed_spell_copies.len(), 1);
        // Do not resolve the synthetic packet as a real cast: it deliberately
        // combines a prepared identity with a non-instant type not in the catalogue.
        g.state.pending_triggers.truncate(first);
        finish_delayed(&mut g);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        assert!(g.observe(p).delayed_spell_copies.is_empty());
    }
}
