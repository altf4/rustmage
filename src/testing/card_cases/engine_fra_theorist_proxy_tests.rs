//! Focused printed outcomes; shared engine invariants live in core tests.
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

#[test]
fn proxy_entry_empowers_jace_and_has_flash() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let proxy = put(&mut g, p, "Theorist's Proxy", Zone::Hand);

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
fn proxy_copy_is_not_a_cast_and_does_not_inherit_the_protection() {
    {
        let p = Player::P0;
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
