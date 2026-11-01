use super::*;
fn dwarves(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::from_name("@token/hob-dwarf").unwrap())
            .then_some(id as CardId)
        })
        .collect()
}
fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    g.state.players[p.index()].mana = [20; 6];
    let id = put(g, p, name, Zone::Hand);
    cast(g, id, target);
    settle(g);
    id
}
fn fili(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Fíli the Pathfinder", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
    cast(g, id, None);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("Fili decision did not arrive");
}
#[test]
fn hob_fili_paid_self_other_nontoken_and_token_exclusions_have_exact_red_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fili(&mut g, p);
        assert_eq!(stats(&g, source), (2, 2));
        assert_eq!(dwarves(&g, p).len(), 1);
        let token = dwarves(&g, p)[0];
        let v = g.object_view(token);
        assert_eq!((v.power, v.toughness, v.colors, v.kinds), (2, 2, 8, 1));
        assert!(g.has_subtype(token, "Dwarf"));
        assert!(!v.tapped && v.summoning_sick);
        paid(&mut g, p, "Axgard Cavalry", None);
        assert_eq!(dwarves(&g, p).len(), 2);
        paid(&mut g, p, "Bear Cub", None);
        assert_eq!(dwarves(&g, p).len(), 2);
        g.create_tokens(p, Card::from_name("@token/hob-dwarf").unwrap(), 1, false);
        settle(&mut g);
        assert_eq!(dwarves(&g, p).len(), 3, "token Dwarves do not recurse");
        let enemy = put(&mut g, p.other(), "Axgard Cavalry", Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(
            dwarves(&g, p).len(),
            3,
            "opponent's nontoken Dwarf is excluded"
        );
    }
}
#[test]
fn hob_fili_token_copy_self_triggers_despite_nontoken_other_filter() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let original = fili(&mut g, p);
        let count = dwarves(&g, p).len();
        let copy_spell = put(&mut g, p, "Self-Reflection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        cast(&mut g, copy_spell, Some(Target::Card(original)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        let copy = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card == Card::from_name("Fíli the Pathfinder").unwrap())
                .then_some(id as CardId)
            })
            .unwrap();
        let object = g.object_ref(copy);
        act(&mut g, Action::KeepLegend { object });
        settle(&mut g);
        assert_eq!(g.object_view(original).zone, Zone::Graveyard);
        assert_eq!(
            dwarves(&g, p).len(),
            count + 1,
            "only copy's self entry qualifies"
        );
    }
}
#[test]
fn hob_fili_live_story_bonus_suppression_restore_control_and_current_recipients() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fili(&mut g, p);
        let body = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(source)));
        let a = paid(&mut g, p, "Goldvein Pick", None);
        let b = paid(&mut g, p, "Lotus Ring", None);
        assert!(!g.state.players[p.index()].enduring_story);
        assert_eq!(stats(&g, body), (2, 2));
        paid(&mut g, p, "Disenchant", Some(Target::Card(aura)));
        assert!(g.state.players[p.index()].enduring_story);
        assert_eq!(stats(&g, source), (3, 3));
        assert_eq!(stats(&g, body), (3, 3));
        assert_eq!(stats(&g, enemy), (2, 2));
        for token in dwarves(&g, p) {
            assert_eq!(stats(&g, token), (3, 3));
        }
        g.move_objects_raw(&[a, b], Zone::Exile);
        assert_eq!(stats(&g, body), (3, 3), "story persists");
        let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(source)));
        assert_eq!(stats(&g, body), (2, 2), "anthem is suppressed");
        paid(&mut g, p, "Disenchant", Some(Target::Card(aura)));
        assert_eq!(stats(&g, body), (3, 3));
        g.record_control(g.object_ref(source), p.other(), None);
        assert_eq!(stats(&g, body), (2, 2));
        assert_eq!(stats(&g, enemy), (2, 2), "new controller has no story");
        let q = p.other();
        put(&mut g, q, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, q, "Lotus Ring", Zone::Battlefield);
        g.refresh_controllers();
        assert!(g.state.players[q.index()].enduring_story);
        assert_eq!(stats(&g, enemy), (3, 3));
        g.record_control(g.object_ref(body), q, None);
        assert_eq!(stats(&g, body), (3, 3), "recipient control is live");
        g.move_object(source, Zone::Exile);
        assert_eq!(stats(&g, body), (2, 2));
        assert_eq!(stats(&g, enemy), (2, 2));
    }
}
#[test]
fn hob_fili_legend_qualification_is_live_and_entry_packet_outlives_source_with_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fili(&mut g, p);
        let mut recipe = g.copiable_recipe(g.object_ref(source));
        recipe.overrides = Some(CopyOverrides {
            legendary: Some(false),
            ..CopyOverrides::NONE
        });
        g.set_copy_recipe(g.object_ref(source), recipe);
        g.refresh_controllers();
        paid(&mut g, p, "Goldvein Pick", None);
        paid(&mut g, p, "Lotus Ring", None);
        assert!(
            !g.state.players[p.index()].enduring_story,
            "nonlegendary source is no qualifier"
        );
        paid(&mut g, p, "Goldvein Pick", None);
        assert!(g.state.players[p.index()].enduring_story);
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = put(&mut g, p, "Fíli the Pathfinder", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
        cast(&mut g, source, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        g.move_object(source, Zone::Exile);
        super::contracts::branch_contract(g.clone());
        choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
        settle(&mut g);
        assert_eq!(
            dwarves(&g, p).len(),
            4,
            "replacement doubles twice, token entries do not recurse"
        );
    }
}
