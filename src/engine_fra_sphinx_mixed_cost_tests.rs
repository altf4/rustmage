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
