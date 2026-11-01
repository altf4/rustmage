use super::*;

fn wolves(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Wolf"))
            .then_some(id as CardId)
        })
        .collect()
}
fn upkeep(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_step(Step::Upkeep);
    g.prepare_triggers();
    settle(g);
}
fn attackers(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}

#[test]
fn hob_warg_company_paid_body_trample_and_controller_only_upkeep_wolf_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Chief Warg's Company", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 1, 1];
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, source), (5, 3));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        assert!(g.has_subtype(source, "Wolf"));
        assert!(wolves(&g, p).is_empty(), "no entry token");
        upkeep(&mut g, p.other());
        assert!(wolves(&g, p).is_empty());
        for count in [1, 2] {
            upkeep(&mut g, p);
            let tokens = wolves(&g, p);
            assert_eq!(tokens.len(), count);
            for token in tokens {
                let view = g.object_view(token);
                assert_eq!(
                    (view.power, view.toughness, view.colors, view.kinds),
                    (2, 2, 16, 1)
                );
                assert!(!view.tapped);
                assert!(view.summoning_sick);
                assert_eq!(view.owner, p);
                assert_eq!(view.card.identity(), "@token/hob-wolf");
                assert_eq!(view.card.info().deck_limit, None);
            }
            assert!(wolves(&g, p.other()).is_empty());
        }
    }
}
#[test]
fn hob_warg_company_attack_requires_two_other_controlled_wolves_not_two_attackers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Chief Warg's Company", Zone::Battlefield);
        for _ in 0..2 {
            put(&mut g, p.other(), "@token/hob-wolf", Zone::Battlefield);
        }
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        attackers(&mut g, p);
        reject(&mut g, Action::Attack { card: source });
        let one = put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        reject(&mut g, Action::Attack { card: source });
        let two = put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        assert!(g.state.attackers.iter().any(|a| a.card == source));
        assert!(
            !g.state
                .attackers
                .iter()
                .any(|a| a.card == one || a.card == two)
        );
        assert!(g.object_view(source).tapped);
        assert!(!g.object_view(one).tapped && !g.object_view(two).tapped);
    }
}
#[test]
fn hob_warg_company_final_declaration_rechecks_transactionally_and_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Chief Warg's Company", Zone::Battlefield);
        put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        let departing = put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        attackers(&mut g, p);
        act(&mut g, Action::Attack { card: source });
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut committed = g.clone();
        let index = committed
            .legal_actions()
            .iter()
            .position(|a| *a == Action::FinishAttackers)
            .unwrap();
        committed
            .apply_index(p, committed.revision(), index)
            .unwrap();
        assert!(committed.state.attackers.iter().any(|a| a.card == source));
        // Fixture invalidates an already selected declaration before its commit.
        g.move_object(departing, Zone::Exile);
        reject(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(source).tapped);
        assert!(g.state.attackers.is_empty());
        g.restore(&saved).unwrap();
        act(&mut g, Action::FinishAttackers);
        assert_eq!(g.snapshot(), committed.snapshot());
    }
}
