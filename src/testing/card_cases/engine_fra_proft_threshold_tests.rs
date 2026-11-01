//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn proft_requires_seven_of_the_casters_graveyard_cards_to_cast() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let proft = put(&mut g, p, "Proft, Sinister Mastermind", Zone::Hand);
        for _ in 0..6 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        put(&mut g, p.other(), "Forest", Zone::Graveyard);
        reject(&mut g, Action::AnnounceCast { card: proft });
        put(&mut g, p, "Forest", Zone::Graveyard);

        act(&mut g, Action::AnnounceCast { card: proft });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(stats(&g, proft), (5, 5));
        assert!(g.keywords(proft).contains(Keywords::MENACE));
    }
}

#[test]
fn proft_graveyard_cast_excludes_the_card_itself_from_threshold() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let proft = put(&mut g, p, "Proft, Sinister Mastermind", Zone::Graveyard);
        for _ in 0..6 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        let object = g.object_ref(proft);
        g.state
            .graveyard_cast_permissions
            .get_or_insert_default()
            .push(GraveyardCastPermissionView {
                object,
                player: p,
                kind: GraveyardCastKind::Cast,
                expires_turn: g.state.turn,
            });
        reject(&mut g, Action::AnnounceCast { card: proft });
        put(&mut g, p, "Forest", Zone::Graveyard);
        act(&mut g, Action::AnnounceCast { card: proft });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(proft).zone, Zone::Battlefield);
    }
}

#[test]
fn proft_can_discard_itself_for_the_targeted_debuff_below_threshold() {
    {
        let p = Player::P0;
        for replacement in [false, true] {
            let mut g = scenario(p);
            if replacement {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let proft = put(&mut g, p, "Proft, Sinister Mastermind", Zone::Hand);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: proft,
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
            pay(&mut g);
            assert_eq!(
                g.object_view(proft).zone,
                if replacement {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );

            settle(&mut g);
            assert_eq!(stats(&g, target), (0, 2));
            g.finish_cleanup();
            assert_eq!(stats(&g, target), (3, 3));
        }
    }
}
