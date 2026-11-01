//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
fn cast_origin(
    g: &mut Game,
    p: Player,
    id: CardId,
    zone: Zone,
    flash: bool,
    target: Option<Target>,
) {
    if zone == Zone::Hand {
        cast(g, id, target);
        return;
    }
    if zone == Zone::Exile {
        g.grant_exile_play_permission(p, g.object_ref(id), false);
        act(g, Action::AnnounceExile { card: id });
    } else if flash {
        act(g, Action::AnnounceFlashback { card: id });
    } else {
        g.grant_graveyard_permission(p, g.object_ref(id), GraveyardCastKind::Cast);
        act(g, Action::AnnounceCast { card: id });
    }
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
#[test]
fn hob_graveyard_origin_plunder_hand_flashback_regular_graveyard_and_exile() {
    {
        let p = Player::P0;
        for (zone, flash, draws, destination) in [
            (Zone::Hand, false, 1, Zone::Graveyard),
            (Zone::Graveyard, true, 2, Zone::Exile),
            (Zone::Graveyard, false, 2, Zone::Graveyard),
            (Zone::Exile, false, 1, Zone::Graveyard),
        ] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Plunder the Trollshaws", zone);
            let mana_before: u32 = g.state.players[p.index()].mana.iter().copied().sum();
            cast_origin(&mut g, p, spell, zone, flash, None);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                draws,
                "{zone:?} flash {flash}"
            );
            assert_eq!(g.state.objects[spell as usize].zone, destination);
            let mana_after: u32 = g.state.players[p.index()].mana.iter().copied().sum();
            assert_eq!(mana_before - mana_after, if flash { 4 } else { 2 });
        }
    }
}
#[test]
fn hob_graveyard_origin_moment_paid_modes_and_controlled_only() {
    {
        let p = Player::P0;
        for (zone, flash) in [
            (Zone::Hand, false),
            (Zone::Graveyard, true),
            (Zone::Graveyard, false),
            (Zone::Exile, false),
        ] {
            let mut g = scenario(p);
            let a = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let b = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            let spell = put(&mut g, p, "Moment of Glory", zone);
            cast_origin(&mut g, p, spell, zone, flash, Some(Target::Card(a)));
            settle(&mut g);
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
            assert_eq!(
                g.counter_count(b, CounterKind::PlusOne),
                if zone == Zone::Graveyard { 1 } else { 0 }
            );
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
            assert_eq!(
                g.state.objects[spell as usize].zone,
                if flash { Zone::Exile } else { Zone::Graveyard }
            );
        }
    }
}

#[test]
fn hob_graveyard_origin_uncast_copy_ignores_inherited_origin() {
    for name in ["Plunder the Trollshaws", "Moment of Glory"] {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let a = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let b = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let spell = put(&mut g, p, name, Zone::Graveyard);
        cast_origin(
            &mut g,
            p,
            spell,
            Zone::Graveyard,
            true,
            if name == "Moment of Glory" {
                Some(Target::Card(a))
            } else {
                None
            },
        );
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        let object = g.object_ref(artifact);
        act(&mut g, Action::Sacrifice { object });
        // Targeted copies are published only after accepting their inherited targets.
        if g.legal_actions().contains(&Action::FinishStackTargets) {
            act(&mut g, Action::FinishStackTargets);
        }
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
        assert_eq!(copy.cast.unwrap().origin, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        let copy_id = copy.id;
        for _ in 0..20 {
            if !g.state.stack.iter().any(|s| s.id == copy_id) {
                break;
            }
            if g.legal_actions().contains(&Action::FinishStackTargets) {
                act(&mut g, Action::FinishStackTargets);
            } else {
                act(&mut g, Action::Pass);
            }
        }
        assert!(!g.state.stack.iter().any(|s| s.id == copy_id));
        if name == "Plunder the Trollshaws" {
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        } else {
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        }
        settle(&mut g);
        if name == "Plunder the Trollshaws" {
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        } else {
            assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
        }
    }
}
