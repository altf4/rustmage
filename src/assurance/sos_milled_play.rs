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
    for p in [Player::P0, Player::P1] {
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
fn sos_milled_spell_normal_cost_timing_source_departure_and_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ark = put(&mut g, p, "Ark of Hunger", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Library);
        activate(&mut g, ark);
        g.move_object(ark, Zone::Graveyard);
        g.state.active = p.other();
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: shock });
        g.state.players[p.index()].mana[3] = 1;
        checked(&mut g, Action::AnnounceCast { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let cast = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell && s.source.is_some_and(|source| source.id == shock))
            .unwrap()
            .cast
            .unwrap();
        assert_eq!(cast.method, CastMethod::Normal);
        assert_eq!(cast.origin, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        finish(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert!(!g.has_graveyard_cast(p, shock, GraveyardCastKind::Play));
        assert_eq!(g.state.players[p.index()].life, 20);
        g.state.active = p;
        g.state.priority = p;
        let ark = put(&mut g, p, "Ark of Hunger", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Library);
        activate(&mut g, ark);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        g.state.active = p.other();
        reject(&mut g, Action::AnnounceCast { card: bear });
        g.state.active = p;
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Graveyard);
        finish(&mut g);
        reject(&mut g, Action::AnnounceCast { card: bear });
    }
}
#[test]
fn sos_milled_exile_replacement_remains_playable_and_expires_exactly() {
    for p in [Player::P0, Player::P1] {
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
#[test]
fn sos_milled_ark_graveyard_batch_is_one_damage_and_life_event_and_empty_mill() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ark = put(&mut g, p, "Ark of Hunger", Zone::Battlefield);
        let a = put(&mut g, p, "Island", Zone::Graveyard);
        let b = put(&mut g, p, "Forest", Zone::Graveyard);
        let events = g.move_objects_raw(&[a, b], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(g.state.players[p.index()].life_gain_events_this_turn, 1);
        while let Some(&id) = g.state.players[p.index()].library.last() {
            g.move_object(id, Zone::Exile);
        }
        activate(&mut g, ark);
        assert!(g.observe(p).graveyard_cast_permissions.is_empty());
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}
