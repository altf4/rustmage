use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn robbery_exiles_opponent_cards_privately_and_grants_persistent_play() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p.other(), "Island", Zone::Library);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Library);
        let robbery = put(&mut g, p, "Outrageous Robbery", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: robbery });
        checked(&mut g, Action::ChooseX { value: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        settle(&mut g);
        for card in [land, creature] {
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            assert!(g.can_play_exiled(p, card));
            assert!(!g.can_view_hideaway(p.other(), g.object_ref(card)));
            assert!(g.can_view_hideaway(p, g.object_ref(card)));
            assert!(g.observe(p).objects.iter().any(|object| object.id == card));
            assert!(
                !g.observe(p.other())
                    .objects
                    .iter()
                    .any(|object| object.id == card)
            );
            let permission = g
                .state
                .exile_permissions
                .iter()
                .find(|a| a.object == g.object_ref(card))
                .unwrap();
            assert_eq!(permission.expiry, ExilePlayExpiry::WhileExiled);
            assert!(permission.spend_any_mana);
        }
        let snapshot = g.snapshot();
        let mut child = g.clone();
        child.restore(&snapshot).unwrap();
        assert_eq!(child.snapshot(), snapshot);
        g.state.turn += 4;
        assert!(g.can_play_exiled(p, creature));
        let mut cast = g.clone();
        cast.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
        checked(&mut cast, Action::AnnounceExile { card: creature });
        pay(&mut cast);
        settle(&mut cast);
        assert_eq!(cast.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(cast.object_view(creature).controller, p);
        g.move_object(creature, Zone::Graveyard);
        g.move_object(creature, Zone::Exile);
        assert!(!g.can_play_exiled(p, creature));
    }
}

#[test]
fn web_watch_exiles_two_and_produces_only_exile_spell_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Island", Zone::Library);
        let creature = put(&mut g, p, "Bear Cub", Zone::Library);
        let watch = put(&mut g, p, "Interdimensional Web Watch", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: watch });
        pay(&mut g);
        settle(&mut g);
        for card in [land, creature] {
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            assert!(g.can_play_exiled(p, card));
        }
        let mut expiry = g.clone();
        expiry.finish_cleanup();
        assert!(expiry.can_play_exiled(p, land));
        expiry.finish_cleanup();
        assert!(expiry.can_play_exiled(p, land));
        expiry.finish_cleanup();
        assert!(!expiry.can_play_exiled(p, land));
        let mut off_turn = g.clone();
        off_turn.state.active = p.other();
        off_turn.state.priority = p;
        assert!(
            !off_turn
                .legal_actions()
                .contains(&Action::AnnounceExile { card: creature })
        );
        g.state.players[p.index()].mana = [0; 6];
        let mana_action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production, .. } if *card == watch && *production == [0, 1, 0, 0, 1, 0])).expect("mixed color mana");
        checked(&mut g, mana_action);
        let lots = g.mana_lots(p);
        assert!(
            lots.iter()
                .any(|lot| lot.quality.restriction == ManaRestrictionView::CastFromExile)
        );
        let hand = put(&mut g, p, "Bear Cub", Zone::Hand);
        let exile_purpose =
            g.spell_mana_purpose_for_card(p, creature, Card::from_name("Bear Cub").unwrap());
        let hand_purpose =
            g.spell_mana_purpose_for_card(p, hand, Card::from_name("Bear Cub").unwrap());
        let lot = &g.mana_lots(p)[0];
        assert!(lot.quality.eligible(exile_purpose));
        assert!(!lot.quality.eligible(hand_purpose));
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: hand })
        );
        checked(&mut g, Action::AnnounceExile { card: creature });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    }
}
