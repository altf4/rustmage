//! Independent follow-up interactions for gated FRA artifacts and search.
use super::*;

#[test]
fn aerid_counts_current_controlled_artifacts_after_its_new_token_enters() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let aerid = put(&mut g, p, "Aerid Konstrari", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: aerid,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        // These enter after announcement; only the friendly one counts.
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(stats(&g, aerid), (7, 4));
            g.finish_cleanup();
            assert_eq!(stats(&g, aerid), (5, 4));
        }
    }
}

#[test]
fn aerid_activation_creates_heartwood_even_when_source_changes_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let aerid = put(&mut g, p, "Aerid Konstrari", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: aerid,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        let old = g.object_ref(aerid);
        // Exile avoids Aerid's independent death trigger.
        g.move_object(aerid, Zone::Exile);
        g.move_object(aerid, Zone::Battlefield);
        assert_ne!(g.object_ref(aerid), old);
        // Reentry also creates a Heartwood, independently of the old activation.
        settle(&mut g);
        let heartwoods = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card::test_index(5438)
            })
            .count();
        assert_eq!(heartwoods, 2);
        assert_eq!(stats(&g, aerid), (5, 4));
    }
}

#[test]
fn demon_empty_library_still_discards_a_random_hand_card_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let library = g.state.players[p.index()].library.clone();
        for card in library {
            g.move_object(card, Zone::Exile);
        }
        let hand = put(&mut g, p, "Forest", Zone::Hand);
        let demon = put(&mut g, p, "Curse-Marred Demon", Zone::Hand);
        cast(&mut g, demon, None);
        // Resolve the creature spell, leaving its entry ability on the stack.
        for _ in 0..32 {
            if g.object_view(demon).zone == Zone::Battlefield {
                break;
            }
            act(&mut g, Action::Pass);
        }
        g.move_object(demon, Zone::Exile);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            for _ in 0..48 {
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.resolving.is_none()
                {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|action| {
                        matches!(
                            action,
                            Action::Pass | Action::PutTrigger { .. } | Action::FinishSearch
                        )
                    })
                    .expect("empty search progression");
                act(&mut g, action);
            }
            assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            assert!(g.state.players[p.index()].library.is_empty());
        }
    }
}
