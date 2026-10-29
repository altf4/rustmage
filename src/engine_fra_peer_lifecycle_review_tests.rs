//! Author follow-up for prepared Peer Review lifecycle and spell copies.
use super::*;

fn peer_announce(game: &mut Game, action: Action) {
    act(game, action);
    for _ in 0..16 {
        if game.state.announcing.is_none() {
            return;
        }
        choose(game, |a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. }
            )
        });
    }
    panic!("Peer Review announcement did not finish");
}

fn peer_finish(game: &mut Game) {
    for _ in 0..100 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
            )
        });
    }
    panic!("Peer Review resolution did not finish");
}

#[test]
fn prepared_peer_review_stack_copy_resolves_independently_without_repreparing() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(
            &mut game,
            player,
            "Fatehold Chronologist // Peer Review",
            Zone::Hand,
        );
        peer_announce(&mut game, Action::AnnounceCast { card: creature });
        peer_finish(&mut game);
        let permanent = game.object_ref(creature);
        let prepared = game
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|entry| entry.permanent == permanent)
            .unwrap()
            .spell_copy;
        peer_announce(
            &mut game,
            Action::AnnounceExile {
                card: prepared.card,
            },
        );
        assert!(!game.is_prepared(permanent));
        let original = game.state.stack.last().unwrap().clone();
        let copy = game.allocate_spell_copy(&original, player);
        let copied_object = copy.object;
        game.state.stack.push(copy);
        super::super::contracts::branch_contract(game.clone());
        peer_finish(&mut game);
        let cadets = game
            .state
            .objects
            .iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.card == Card::test_index(5298)
            })
            .count();
        assert_eq!(cadets, 2);
        assert!(!game.is_prepared(permanent));
        assert_eq!(game.object_view(prepared.card).zone, Zone::Outside);
        assert!(
            game.object_ref(copied_object.card) != copied_object
                || game.object_view(copied_object.card).zone == Zone::Outside
        );
    }
}

#[test]
fn preparing_an_already_prepared_creature_does_not_create_another_peer_review() {
    for player in [Player::P0, Player::P1] {
        for name in [
            "Fatehold Chronologist // Peer Review",
            "Semester Foreseer // Peer Review",
            "Prudent Fateseer // Peer Review",
        ] {
            let mut game = scenario(player);
            let creature = put(&mut game, player, name, Zone::Hand);
            peer_announce(&mut game, Action::AnnounceCast { card: creature });
            peer_finish(&mut game);
            let permanent = game.object_ref(creature);
            let before = game
                .state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .find(|entry| entry.permanent == permanent)
                .unwrap()
                .spell_copy;
            game.prepare_permanent(permanent);
            let entries: Vec<_> = game
                .state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .filter(|entry| entry.permanent == permanent)
                .collect();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].spell_copy, before);
            assert_eq!(game.object_view(before.card).zone, Zone::Exile);
            super::super::contracts::branch_contract(game);
        }
    }
}
