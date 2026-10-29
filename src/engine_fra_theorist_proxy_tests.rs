use super::*;

fn grant_proxy(g: &mut Game, p: Player) -> CardId {
    let proxy = put(g, p, "Theorist's Proxy", Zone::Battlefield);
    act(
        g,
        Action::AnnounceActivate {
            card: proxy,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    assert_eq!(g.object_view(proxy).zone, Zone::Graveyard);
    settle(g);
    assert!(
        g.observe(p.other()).players[p.index()]
            .turn_cast_grants
            .unwrap()
            .next_spell_uncounterable
    );
    proxy
}

fn opposing_negate(g: &mut Game, p: Player, spell: CardId) {
    act(g, Action::Pass);
    assert_eq!(g.actor(), Some(p.other()));
    let negate = put(g, p.other(), "Negate", Zone::Hand);
    cast(g, negate, Some(Target::Spell(spell)));
    settle(g);
}

#[test]
fn proxy_entry_empowers_jace_and_has_flash() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let proxy = put(&mut g, p, "Theorist's Proxy", Zone::Hand);
        assert!(g.keywords(proxy).contains(Keywords::FLASH));
        cast(&mut g, proxy, None);
        settle(&mut g);
        let jace = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card::JacePlaneswalkerToken)
                    .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 3);
    }
}

#[test]
fn proxy_protects_next_successfully_cast_spell_from_actual_counter_and_restores() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        grant_proxy(&mut g, p);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: shock });
        assert!(
            g.observe(p).players[p.index()]
                .turn_cast_grants
                .unwrap()
                .next_spell_uncounterable
        );
        g.state.players[p.index()].mana = [8; 6];
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&before).unwrap();
            }
            let life = g.state.players[p.other().index()].life;
            cast(&mut g, shock, Some(Target::Player(p.other())));
            let id = g.state.stack.last().unwrap().id;
            assert!(g.observe(p.other()).uncounterable_spell_ids.contains(&id));
            assert!(
                !g.observe(p).players[p.index()]
                    .turn_cast_grants
                    .is_some_and(|grants| grants.next_spell_uncounterable)
            );
            opposing_negate(&mut g, p, shock);
            assert_eq!(g.state.players[p.other().index()].life, life - 2);
            assert!(!g.observe(p).uncounterable_spell_ids.contains(&id));
            let next = put(&mut g, p, "Shock", Zone::Hand);
            let after = g.state.players[p.other().index()].life;
            cast(&mut g, next, Some(Target::Player(p.other())));
            opposing_negate(&mut g, p, next);
            assert_eq!(g.state.players[p.other().index()].life, after);
        }
    }
}

#[test]
fn proxy_copy_is_not_a_cast_and_does_not_inherit_the_protection() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        grant_proxy(&mut g, p);
        g.state.priority = p.other();
        let enemy = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, enemy, Some(Target::Player(p)));
        let template = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&template, p);
        let id = copy.id;
        g.push_stack_item(copy);
        assert!(
            g.observe(p).players[p.index()]
                .turn_cast_grants
                .unwrap()
                .next_spell_uncounterable
        );
        assert!(!g.observe(p).uncounterable_spell_ids.contains(&id));
        assert!(g.counter_stack_item(id, false));
        assert!(
            g.observe(p).players[p.index()]
                .turn_cast_grants
                .unwrap()
                .next_spell_uncounterable
        );
    }
}

#[test]
fn proxy_pending_protection_expires_at_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        grant_proxy(&mut g, p);
        g.finish_cleanup();
        assert!(
            !g.observe(p).players[p.index()]
                .turn_cast_grants
                .is_some_and(|grants| grants.next_spell_uncounterable)
        );
        // Cleanup advances the turn and empties both players' mana pools.
        g.state.priority = p;
        g.state.players[p.index()].mana = [8; 6];
        g.state.players[p.other().index()].mana = [8; 6];
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        let life = g.state.players[p.other().index()].life;
        cast(&mut g, shock, Some(Target::Player(p.other())));
        opposing_negate(&mut g, p, shock);
        assert_eq!(g.state.players[p.other().index()].life, life);
    }
}

#[test]
fn proxy_protection_stays_on_the_cast_spell_not_its_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        grant_proxy(&mut g, p);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let copy_id = copy.id;
        g.push_stack_item(copy);
        assert!(g.observe(p).uncounterable_spell_ids.contains(&original.id));
        assert!(!g.observe(p).uncounterable_spell_ids.contains(&copy_id));
        assert!(g.counter_stack_item(copy_id, false));
        assert!(!g.counter_stack_item(original.id, false));
    }
}
