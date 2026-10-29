//! Sphinx's real resolution payment with synthetic replacement providers.
use super::*;

#[test]
fn sphinx_competing_exile_replacements_replay_atomically() {
    use zone_replacements::{TEST_PREVENT_ZONE_MOVE, TEST_REDIRECT_EXILE_TO_BATTLEFIELD};
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) {
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(None));
    }}
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards: Vec<_> = (0..4).map(|_| {
            let id = put(&mut g, player, "Shock", Zone::Graveyard);
            g.state.objects[id as usize].card = Card::test_index(4016);
            id
        }).collect();
        let source = put(&mut g, player, "Shock", Zone::Hand);
        g.state.objects[source as usize].card = Card::test_index(4016);
        cast(&mut g, source, None);
        for _ in 0..96 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) { break; }
            choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
        let reference = g.object_ref(cards[0]);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(Some(reference)));
        for &id in &cards[..3] {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        let last = g.object_ref(cards[3]);
        act(&mut g, Action::SelectResolutionExile { object: last });
        assert!(matches!(g.state.pending, Pending::ZoneReplacement { player: p } if p == player));
        let pending = g.snapshot();
        let views = [player, player.other()].map(|p| serde_json::to_string(&g.observe(p)).unwrap());
        for outcome in [ZoneReplacementKind::Prevent, ZoneReplacementKind::Battlefield] {
            g.restore(&pending).unwrap();
            assert_eq!([player, player.other()].map(|p| serde_json::to_string(&g.observe(p)).unwrap()), views);
            let action = Action::ChooseZoneReplacement { effect: outcome };
            let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            assert!(raw.apply(player.other(), action.clone()).is_err());
            assert_eq!(raw.snapshot(), pending);
            raw.apply(player, action).unwrap();
            indexed.apply_index(player, g.revision(), index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.object_view(source).zone, Zone::Exile);
            for &id in &cards[1..] { assert_eq!(raw.object_view(id).zone, Zone::Exile); }
            assert_eq!(raw.object_view(cards[0]).zone, if outcome == ZoneReplacementKind::Prevent { Zone::Graveyard } else { Zone::Battlefield });
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(None));
    }
}

fn sphinx_card(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Shock", zone);
    g.state.objects[id as usize].card = Card::test_index(4016);
    id
}
fn wait_for_sphinx_choice(g: &mut Game) {
    for _ in 0..96 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("missing Sphinx choice: {:?}", g.state.pending);
}

#[test]
fn sphinx_prevented_component_still_pays_and_queues_one_departure_observer() {
    use zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None)); } }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let observer = put(&mut g, player, "Owlin Historian", Zone::Battlefield);
        let cards: Vec<_> = (0..4).map(|_| sphinx_card(&mut g, player, Zone::Graveyard)).collect();
        let source = sphinx_card(&mut g, player, Zone::Hand);
        cast(&mut g, source, None);
        wait_for_sphinx_choice(&mut g);
        let before = g.snapshot();
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(cards[0]))));
        for _ in 0..2 {
            for &id in &cards {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            assert_eq!(g.object_view(cards[0]).zone, Zone::Graveyard);
            assert!(cards[1..].iter().all(|&id| g.object_view(id).zone == Zone::Exile));
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(g.state.pending_triggers.iter().filter(|item| item.card == observer).count(), 1);
            assert!(g.state.resolving.is_some(), "observer waits for Sphinx's search");
            g.restore(&before).unwrap();
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn sphinx_copy_created_through_legal_cast_exiles_then_ceases_after_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards: Vec<_> = (0..4).map(|_| sphinx_card(&mut g, player, Zone::Graveyard)).collect();
        let sphinx = put(&mut g, player, "Curator of Destinies", Zone::Library);
        put(&mut g, player, "Island", Zone::Library);
        put(&mut g, player, "Island", Zone::Library);
        let source = sphinx_card(&mut g, player, Zone::Hand);
        cast(&mut g, source, None);
        let sparks = put(&mut g, player, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(&mut g, Action::ChooseTarget { target: Target::Spell(source) });
        pay(&mut g);
        wait_for_sphinx_choice(&mut g);
        let copy = g.state.resolving.as_ref().unwrap().item.object;
        assert!(g.state.resolving.as_ref().unwrap().item.is_copy);
        let before = g.snapshot();
        for _ in 0..2 {
            let action = Action::SelectResolutionExile { object: g.object_ref(cards[0]) };
            assert!(g.apply(player.other(), action.clone()).is_err());
            assert_eq!(g.snapshot(), before);
            act(&mut g, action);
            for &id in &cards[1..] {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            // Requires the separately drafted copy-card lifecycle foundation.
            assert_eq!(g.object_view(copy.card).zone, Zone::Exile);
            act(&mut g, Action::ChooseSearch { card: sphinx });
            act(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
            g.restore(&before).unwrap();
        }
    }
}
