//! Focused printed outcomes; shared engine invariants live in core tests.
//! Committed loyalty activation witnesses, independent of resolution.
use super::*;

#[test]
fn gideon_observes_only_opponents_committed_loyalty_abilities() {
    for actor in [Player::P0, Player::P1] {
        for enemy in [false, true] {
            let mut game = scenario(actor);
            game.create_tokens(actor, Card::JacePlaneswalkerToken, 1, false);
            let walker = game
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, object)| {
                    (object.zone == Zone::Battlefield
                        && object.controller == actor
                        && object.is_token()
                        && game.has_subtype(id as CardId, "Jace"))
                    .then_some(id as CardId)
                })
                .unwrap();
            game.state.objects[walker as usize].counters.loyalty = 5;
            let observer = if enemy { actor.other() } else { actor };
            let gideon = put(
                &mut game,
                observer,
                "Gideon the Oathless",
                Zone::Battlefield,
            );
            let life = game.state.players[actor.index()].life;
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: walker,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut game);
            let triggers = game
                .state
                .stack
                .iter()
                .filter(|item| {
                    item.source.is_some_and(|source| source.id == gideon)
                        && matches!(item.kind, StackKind::TriggeredAbility { .. })
                })
                .count();
            assert_eq!(triggers, usize::from(enemy));
            if enemy {
                game.move_object(gideon, Zone::Graveyard);
                settle(&mut game);
                assert_eq!(game.state.players[actor.index()].life, life - 1);
            }
        }
    }
}

#[test]
fn gideon_ward_retains_the_targeting_player_after_control_changes() {
    for payer in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut game = scenario(payer);
            let gideon = put(
                &mut game,
                payer.other(),
                "Gideon the Oathless",
                Zone::Battlefield,
            );
            let shock = put(&mut game, payer, "Shock", Zone::Hand);
            let secret = put(&mut game, payer, "Forest", Zone::Hand);
            cast(&mut game, shock, Some(Target::Card(gideon)));
            game.change_controller(game.object_ref(gideon), payer);
            game.resolve_top();
            assert!(
                matches!(game.state.pending, Pending::ResolutionMana { player } if player == payer)
            );
            let request = game.observe(payer).resolution_payment.unwrap();
            assert!(request.discard_only);
            assert_eq!(request.payer, payer);
            assert!(
                !game
                    .legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::PayResolutionMana { .. }))
            );
            assert!(
                !game
                    .observe(payer.other())
                    .objects
                    .iter()
                    .any(|o| o.id == secret)
            );

            if discard {
                act(&mut game, Action::Discard { card: secret });
            } else {
                act(&mut game, Action::DeclineResolutionMana);
            }
            settle(&mut game);
            assert_eq!(
                game.object_view(secret).zone,
                if discard { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(
                game.state.objects[gideon as usize].damage,
                if discard { 2 } else { 0 }
            );
            assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn gideon_ward_discard_is_not_an_opponents_discard_effect() {
    for payer in [Player::P0, Player::P1] {
        let mut game = scenario(payer);
        let gideon = put(
            &mut game,
            payer.other(),
            "Gideon the Oathless",
            Zone::Battlefield,
        );
        let shock = put(&mut game, payer, "Shock", Zone::Hand);
        let liege = put(&mut game, payer, "Wilt-Leaf Liege", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(gideon)));
        game.resolve_top();
        act(&mut game, Action::Discard { card: liege });
        assert_eq!(game.object_view(liege).zone, Zone::Graveyard);
    }
}
