//! Split second remains a live spell ability on stack and while resolving.
use super::*;

#[test]
fn samut_grant_is_visible_and_tracks_spell_copy_controller_and_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        let original = g.state.stack.last().unwrap().clone();
        // The source snapshot includes real continuous grants at cast time.
        assert!(original.source_keywords.contains(Keywords::SPLIT_SECOND));
        let copy = g.allocate_spell_copy(&original, p.other());
        // The other player's copy is a new source and does not inherit that grant.
        assert!(!copy.source_keywords.contains(Keywords::SPLIT_SECOND));
        let copy_id = copy.id;
        let copy_card = copy.source.unwrap().id;
        g.state.stack.push(copy);
        let visible = |g: &Game, viewer: Player, id: u64| {
            g.observe(viewer)
                .stack
                .iter()
                .find(|v| v.id == id)
                .unwrap()
                .split_second
        };
        for viewer in [p, p.other()] {
            assert!(visible(&g, viewer, original.id));
            let observed = g.observe(viewer);
            assert!(
                observed
                    .stack
                    .iter()
                    .find(|v| v.id == original.id)
                    .unwrap()
                    .source_keywords
                    .contains(Keywords::SPLIT_SECOND)
            );
            assert!(!visible(&g, viewer, copy_id));
        }
        assert!(g.keywords(shock).contains(Keywords::SPLIT_SECOND));
        assert!(!g.keywords(copy_card).contains(Keywords::SPLIT_SECOND));
        assert!(
            g.object_view(shock)
                .keywords
                .contains(Keywords::SPLIT_SECOND)
        );
        assert!(
            !g.object_view(copy_card)
                .keywords
                .contains(Keywords::SPLIT_SECOND)
        );
        super::super::contracts::branch_contract(g.clone());
        let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(provider));
        assert!(!visible(&g, p, original.id));
        assert!(!g.keywords(shock).contains(Keywords::SPLIT_SECOND));
        assert!(!g.nonmana_actions_prohibited());
        g.move_object(aura, Zone::Graveyard);
        assert!(visible(&g, p, original.id));
        g.change_controller(g.object_ref(provider), p.other());
        assert!(!visible(&g, p, original.id));
        assert!(visible(&g, p, copy_id));
        assert!(
            !g.object_view(shock)
                .keywords
                .contains(Keywords::SPLIT_SECOND)
        );
        assert!(
            g.object_view(copy_card)
                .keywords
                .contains(Keywords::SPLIT_SECOND)
        );
        g.move_object(provider, Zone::Graveyard);
        assert!(!visible(&g, p, original.id));
        assert!(!visible(&g, p, copy_id));
        assert!(!g.nonmana_actions_prohibited());
    }
}

#[test]
fn resolving_split_second_spell_still_blocks_its_own_free_casts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let library_spell = put(&mut g, p, "Hill Giant", Zone::Library);
        let capstone = put(&mut g, p, "Improvisation Capstone", Zone::Hand);
        cast(&mut g, capstone, None);
        g.resolve_top();
        assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
        assert_eq!(g.object_view(library_spell).zone, Zone::Exile);
        let resolving = g.observe(p).resolving.unwrap();
        assert_eq!(resolving.card, capstone);
        assert!(resolving.split_second);
        assert!(resolving.source_keywords.contains(Keywords::SPLIT_SECOND));
        assert!(g.nonmana_actions_prohibited());
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::CastExiledDuringResolution { .. }))
        );
        assert!(g.legal_actions().contains(&Action::FinishResolutionCasts));
        g.move_object(provider, Zone::Graveyard);
        assert!(!g.nonmana_actions_prohibited());
        assert!(g.legal_actions().iter().any(|a| matches!(a, Action::CastExiledDuringResolution { object, .. } if object.card == library_spell)));
        g.move_object(capstone, Zone::Exile);
        assert!(!g.observe(p).resolving.unwrap().split_second);
    }
}

#[test]
fn announcing_spell_projects_current_split_second_for_both_viewers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        act(&mut g, Action::AnnounceCast { card: shock });
        for viewer in [p, p.other()] {
            let spell = g.observe(viewer).announcing.unwrap();
            assert!(spell.split_second);
            assert!(spell.source_keywords.contains(Keywords::SPLIT_SECOND));
        }
        g.move_object(provider, Zone::Graveyard);
        for viewer in [p, p.other()] {
            assert!(!g.observe(viewer).announcing.unwrap().split_second);
        }
    }
}
