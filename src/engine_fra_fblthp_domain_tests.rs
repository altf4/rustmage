use super::*;

#[test]
fn fblthp_domain_tracks_distinct_controlled_basic_types_in_all_zones() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Fblthp, Knows the Way", Zone::Hand);
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        put(&mut g, p, "Forest", Zone::Battlefield);
        let island = put(&mut g, p, "Island", Zone::Battlefield);
        put(&mut g, p.other(), "Swamp", Zone::Battlefield);
        assert_eq!(g.stats(source).power, 2);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.stats(source).power, 2);
        g.move_object(source, Zone::Battlefield);
        assert_eq!(stats(&g, source), (2, 2));
        g.move_object(island, Zone::Graveyard);
        assert_eq!(stats(&g, source), (1, 2));
        g.move_object(forest, Zone::Graveyard);
        assert_eq!(stats(&g, source), (1, 2));
        super::contracts::branch_contract(g.clone());
    }
}

#[test]
fn fblthp_paid_x_limits_search_to_distinct_basic_land_names() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = scenario(p);
            let first = g.state.players[p.index()].library[0];
            let duplicate = g.state.players[p.index()].library[1];
            let island = put(&mut g, p, "Island", Zone::Library);
            let swamp = put(&mut g, p, "Swamp", Zone::Library);
            let invalid = put(&mut g, p, "Bear Cub", Zone::Library);
            let source = put(&mut g, p, "Fblthp, Knows the Way", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: source });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::EffectSearch { .. }) { break; }
                g.prepare_triggers();
                choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
            assert!(matches!(g.state.pending, Pending::EffectSearch { .. }));
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: invalid });
            if x == 0 {
                reject(&mut g, Action::ChooseSearch { card: first });
            } else {
                act(&mut g, Action::ChooseSearch { card: first });
                reject(&mut g, Action::ChooseSearch { card: duplicate });
                act(&mut g, Action::ChooseSearch { card: island });
                reject(&mut g, Action::ChooseSearch { card: swamp });
            }
            super::contracts::branch_contract(g.clone());
            act(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.object_view(first).zone, if x == 0 { Zone::Library } else { Zone::Hand });
            assert_eq!(g.object_view(island).zone, if x == 0 { Zone::Library } else { Zone::Hand });
            assert_eq!(g.object_view(swamp).zone, Zone::Library);
        }
    }
}
