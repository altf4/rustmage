//! Focused printed outcomes; shared engine invariants live in core tests.
//! Whole-card paid hybrid Aura and exact graveyard return outcomes.
use super::*;
const FIXTURES: &[&str] = &[
    "Eagle's Rescue",
    "Forest",
    "Llanowar Elves",
    "Bear Cub",
    "Giant Growth",
    "Unsummon",
    "Rest in Peace",
    "Elvish Archivist",
    "Shock",
    "@token/18e92c0b45eb6124d717a226",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("eagle decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn activation(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn mana(route: u16) -> Mana {
    match route {
        0 => [0, 2, 0, 0, 0, 2],
        1 => [1, 1, 0, 0, 0, 2],
        2 => [2, 0, 0, 0, 0, 2],
        _ => panic!("hybrid route"),
    }
}

fn return_aura(g: &mut Game, p: Player, source: CardId, target: CardId, route: u16) {
    g.state.players[p.index()].mana = mana(route);
    act(g, activation(source));
    act(g, Action::ChooseActivationCost { cost: route });
    bind(g, Target::Card(target));
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn eagles_rescue_cast_all_hybrids_enchants_any_creature_with_live_two_two_flying() {
    {
        let p = Player::P0;
        for route in 0..3 {
            let mut g = scenario(p);
            let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Hand);
            let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
            g.state.players[p.index()].mana = mana(route);
            act(&mut g, Action::AnnounceCast { card: source });
            act(&mut g, Action::ChooseSpellCost { cost: route });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(forest),
                },
            );
            bind(&mut g, Target::Card(target));
            pay(&mut g);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(source).attached_to,
                Some(g.object_ref(target))
            );
            assert_eq!(stats(&g, target), (4, 4));
            assert!(g.keywords(target).contains(Keywords::FLYING));
            assert!(!g.keywords(source).contains(Keywords::FLYING));
            reject(&mut g, activation(source));
            g.record_control(g.object_ref(target), p, None);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(stats(&g, target), (4, 4));
            g.move_object(source, Zone::Hand);
            assert_eq!(stats(&g, target), (2, 2));
            assert!(!g.keywords(target).contains(Keywords::FLYING));
        }
    }
}
#[test]
fn eagles_rescue_graveyard_all_hybrids_return_attached_and_power_can_then_exceed_one() {
    {
        let p = Player::P0;
        for route in 0..3 {
            let mut g = scenario(p);
            let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
            let before = g.object_ref(source);
            return_aura(&mut g, p, source, target, route);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            finish(&mut g);
            assert_ne!(g.object_ref(source), before);
            assert_eq!(
                g.object_view(source).attached_to,
                Some(g.object_ref(target))
            );
            assert_eq!(stats(&g, target), (3, 3));
            assert!(g.keywords(target).contains(Keywords::FLYING));
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            g.record_control(g.object_ref(target), p.other(), None);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(stats(&g, target), (3, 3));
        }
    }
}

#[test]
fn eagles_rescue_attachment_exists_before_actual_enchantment_entry_observer_freezes_source_power() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let observer = fixture(&mut g, p, "Elvish Archivist", Zone::Battlefield);
        let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
        let draw_before = g.state.players[p.index()].cards_drawn_this_turn;
        return_aura(&mut g, p, source, observer, 0);
        until(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
        assert_eq!(stats(&g, observer), (2, 3));
        assert!(g.keywords(observer).contains(Keywords::FLYING));
        assert!(!matches!(g.state.pending, Pending::AuraEntry { .. }));
        let trigger = g.state.stack.last().expect("single trigger auto-stacks");
        assert_eq!(trigger.source.unwrap().id, observer);
        assert_eq!(trigger.source.unwrap().facts.power, 2);
        finish(&mut g);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            draw_before + 1
        );
    }
}
