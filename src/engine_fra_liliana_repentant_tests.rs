use super::*;

fn finish_liliana(g: &mut Game) {
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Liliana resolution did not finish");
}
fn liliana_activate(g: &mut Game, source: CardId, target: CardId) {
    act(g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
    act(g, Action::ChooseTarget { target: Target::Card(target) });
    pay(g);
}

#[test]
fn liliana_observes_each_other_friendly_creature_or_planeswalker_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Liliana the Repentant", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let observers = g.death_trigger_sources();
        let events = g.move_objects_raw(&[creature, walker, enemy], Zone::Battlefield);
        g.finish_zone_events(&events, &observers);
        assert_eq!(g.state.pending_triggers.len(), 2);
        g.move_object(source, Zone::Exile);
        super::contracts::branch_contract(g.clone());
        finish_liliana(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), 4);
        assert_eq!(g.state.players[p.other().index()].library.len(), 8);
    }
}

#[test]
fn liliana_exhaust_returns_card_then_counters_and_triggers_mill() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Liliana the Repentant", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let action = Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() };
        act(&mut g, action.clone());
        reject(&mut g, Action::ChooseTarget { target: Target::Card(enemy) });
        act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
        pay(&mut g);
        super::contracts::branch_contract(g.clone());
        finish_liliana(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(target).controller, p);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].library.len(), 6);
        put(&mut g, p, "Hill Giant", Zone::Graveyard);
        reject(&mut g, action);
    }
}

#[test]
fn liliana_exhaust_is_spent_on_fizzle_and_resets_on_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Liliana the Repentant", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        liliana_activate(&mut g, source, target);
        g.move_object(target, Zone::Exile);
        finish_liliana(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        let replacement = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let action = Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() };
        reject(&mut g, action.clone());
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        finish_liliana(&mut g);
        act(&mut g, action);
        act(&mut g, Action::ChooseTarget { target: Target::Card(replacement) });
        pay(&mut g);
        finish_liliana(&mut g);
        assert_eq!(g.object_view(replacement).zone, Zone::Battlefield);
    }
}
