//! Focused printed outcomes; shared engine invariants live in core tests.
//! Exact public milled-card permissions through paid legal actions.
use super::big_helpers::{checked, settle as finish};
use super::*;
use crate::engine::untap_rules::UntapCause;
fn activate(g: &mut Game, ark: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: ark,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    finish(g);
}
#[test]
fn sos_milled_land_play_respects_allowance_and_cast_prohibitions() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let ark = put(&mut g, p, "Ark of Hunger", Zone::Battlefield);
        let land = put(&mut g, p, "Island", Zone::Library);
        activate(&mut g, ark);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert!(g.has_graveyard_cast(p, land, GraveyardCastKind::Play));
        reject(&mut g, Action::AnnounceCast { card: land });
        g.state.casting_prohibited = true;
        checked(&mut g, Action::PlayLand { card: land });
        finish(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        g.untap_objects(&[ark], UntapCause::Effect);
        let second = put(&mut g, p, "Island", Zone::Library);
        activate(&mut g, ark);
        reject(&mut g, Action::PlayLand { card: second });
        g.finish_cleanup();
        assert!(!g.has_graveyard_cast(p, second, GraveyardCastKind::Play));
    }
}

#[test]
fn sos_milled_exile_replacement_remains_playable_and_expires_exactly() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let ark = put(&mut g, p, "Ark of Hunger", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Library);
        activate(&mut g, ark);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        assert!(g.can_play_exiled(p, land));
        g.move_object(ark, Zone::Graveyard);
        checked(&mut g, Action::PlayLand { card: land });
        finish(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, 20);
        let ark = put(&mut g, p, "Ark of Hunger", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Library);
        activate(&mut g, ark);
        assert!(g.can_play_exiled(p, shock));
        g.finish_cleanup();
        assert!(!g.can_play_exiled(p, shock));
    }
}
