//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn mind_jace(g: &mut Game, owner: Player) -> CardId {
    g.create_tokens(owner, Card::JacePlaneswalkerToken, 1, false);
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.is_token()
                && o.zone == Zone::Battlefield
                && o.controller == owner
                && g.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .unwrap()
}

#[test]
fn mind_sculptor_counts_actual_additional_loyalty_counter_payment() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let way = put(&mut g, actor, "Way of the Mind Sculptor", Zone::Battlefield);
        let walker = mind_jace(&mut g, actor);
        g.state.objects[walker as usize].counters.loyalty = 5;
        // Synthetic typed cost models a -1 loyalty activation which also removes
        // one loyalty counter as an additional source cost.
        let mut definition = *Card::JacePlaneswalkerToken.activation(0);
        definition.cost.remove = Some((Some(CounterKind::Loyalty), 1));
        let mut item = g.activation_item(actor, walker, 0);
        g.pay_activation_definition(
            &mut item,
            [0; 6],
            Counters::of(CounterKind::Loyalty, 1),
            0,
            &definition,
        );
        assert_eq!(item.loyalty_counters_paid, 2);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
        g.committed_activation_observers(&item);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .any(|packet| packet.source.is_some_and(|source| source.id == way))
        );
    }
}

#[test]
fn combined_loyalty_and_source_counter_costs_never_offer_overcommitted_payments() {
    let mut cost = ActivationCost::ZERO;
    cost.loyalty = Some(-1);
    cost.remove = Some((Some(CounterKind::Loyalty), 1));
    let mut counters = Counters {
        loyalty: 1,
        ..Default::default()
    };
    let mut emitted = Vec::new();
    activations::counter_payments(cost, &counters, |payment| {
        emitted.push(payment);
        false
    });
    assert!(emitted.is_empty());
    counters.loyalty = 2;
    activations::counter_payments(cost, &counters, |payment| {
        emitted.push(payment);
        false
    });
    assert_eq!(emitted, vec![Counters::of(CounterKind::Loyalty, 1)]);
    cost.remove = Some((None, 1));
    counters.loyalty = 1;
    counters.plus_one = 1;
    emitted.clear();
    activations::counter_payments(cost, &counters, |payment| {
        emitted.push(payment);
        false
    });
    assert_eq!(emitted, vec![Counters::of(CounterKind::PlusOne, 1)]);
}
