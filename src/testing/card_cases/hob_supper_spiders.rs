//! Focused printed outcomes; shared engine invariants live in core tests.
//! Exact graveyard history and simultaneous, source-independent Food returns.
use super::*;
const FIXTURES: &[&str] = &[
    "Supper for Spiders",
    "Forest",
    "Bear Cub",
    "Shock",
    "Llanowar Elves",
    "Elvish Archivist",
    "Lifecreed Duo",
    "Adaptive Automaton",
    "Rest in Peace",
    "Witness Protection",
    "Fíli the Pathfinder",
    "Giada, Font of Hope",
    "Serra Angel",
    "Negate",
    "@token/hob-dwarf",
    "Giant's Boulder",
    "Authority of the Consuls",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn finish(g: &mut Game) {
    for _ in 0..96 {
        if quiet(g) || g.outcome().is_some() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("Supper resolution budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player) -> CardId {
    assert_eq!(g.actor(), Some(p));
    let s = fixture(g, p, "Supper for Spiders", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
    act(g, Action::AnnounceCast { card: s });
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
    );
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    s
}
fn shock(g: &mut Game, p: Player, victim: CardId) {
    let s = fixture(g, p, "Shock", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(g, s, Some(Target::Card(victim)));
    finish(g);
    if g.object_view(victim).is_token() {
        assert_ne!(g.object_view(victim).zone, Zone::Battlefield);
    } else {
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}
fn dead(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = fixture(g, p.other(), name, Zone::Battlefield);
    shock(g, p, id);
    id
}
fn food(g: &Game, id: CardId, p: Player) {
    let o = g.object_view(id);
    assert_eq!(
        (o.zone, o.controller, o.owner, o.kinds),
        (Zone::Battlefield, p, p.other(), 16)
    );
    assert!(o.has_subtype("Food"));
    for typ in [
        "Bear",
        "Elf",
        "Druid",
        "Dwarf",
        "Angel",
        "Human",
        "Construct",
    ] {
        assert!(!o.has_subtype(typ), "{typ}");
    }
    let row = g
        .observe(p)
        .self_granted_abilities
        .unwrap()
        .into_iter()
        .find(|r| r.recipient == o.reference)
        .unwrap();
    assert_eq!(row.kind, SelfGrantedAbilityKind::Activated { ability: 0 });
    assert_eq!(row.expires_turn, None);
    assert!(row.available);
}
fn activate_food(g: &mut Game, id: CardId) {
    let source = g.object_ref(id);
    choose(
        g,
        |a| matches!(a,Action::AnnounceSelfGrantedAbility{source:r,..}if *r==source),
    );
    pay(g);
}
#[test]
fn supper_paid_instant_both_seats_returns_all_and_only_current_opponent_battlefield_history() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let a = dead(&mut g, p, "Bear Cub");
        let b = dead(&mut g, p, "Llanowar Elves");
        let milled = fixture(&mut g, p.other(), "Bear Cub", Zone::Library);
        g.move_object(milled, Zone::Graveyard);
        let discarded = fixture(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.move_object(discarded, Zone::Graveyard);
        let own = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        shock(&mut g, p, own);
        let artifact = fixture(&mut g, p.other(), "Giant's Boulder", Zone::Battlefield);
        g.move_object(artifact, Zone::Graveyard);
        let token = dead(&mut g, p, "@token/hob-dwarf");
        let hidden = fixture(&mut g, p.other(), "Negate", Zone::Hand);
        g.state.step = Step::End; // instant-speed casting, no target permission involved
        let spell = paid(&mut g, p);

        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        food(&g, a, p);
        food(&g, b, p);
        assert_eq!(g.object_view(a).card, Card::from_name("Bear Cub").unwrap());
        assert_eq!(
            g.object_view(b).colors,
            Card::from_name("Llanowar Elves").unwrap().colors()
        );
        for id in [milled, discarded, own, artifact] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_ne!(g.object_view(token).zone, Zone::Battlefield);
        for viewer in [p, p.other()] {
            let o = g.observe(viewer);
            assert!(o.objects.iter().any(|v| v.reference.card == a));
            assert!(o.objects.iter().all(|v| v.zone != Zone::Library));
            if viewer == p {
                assert!(o.objects.iter().all(|v| v.reference.card != hidden));
            }
        }
    }
}
#[test]
fn supper_food_cost_is_two_tap_sacrifice_gains_three_and_noncreature_can_repeat_this_turn() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let a = dead(&mut g, p, "Bear Cub");
        paid(&mut g, p);
        finish(&mut g);
        food(&g, a, p);
        assert!(g.object_view(a).summoning_sick);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions().iter().any(
                |x| matches!(x,Action::AnnounceSelfGrantedAbility{source,..}if source.card==a)
            )
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let old = g.object_ref(a);
        activate_food(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_ne!(g.object_ref(a), old);
        assert_eq!(g.object_view(a).owner, p.other());
        assert_eq!(g.object_view(a).kinds, 1);
        assert!(!g.object_view(a).has_subtype("Food"));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 23);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        paid(&mut g, p);
        finish(&mut g);
        food(&g, a, p);
        assert!(
            g.observe(p)
                .self_granted_abilities
                .unwrap()
                .iter()
                .all(|r| r.recipient == g.object_ref(a))
        );
    }
}
#[test]
fn supper_retains_self_etb_printed_mana_and_legendary_abilities_without_copying_the_food_form() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let elf = dead(&mut g, p, "Llanowar Elves");
        let fili = dead(&mut g, p, "Fíli the Pathfinder");
        paid(&mut g, p);
        finish(&mut g);
        food(&g, elf, p);
        food(&g, fili, p);
        assert!(g.object_view(fili).card.is_legendary());
        // Self entry still triggers even though the incoming permanent is not a Dwarf.
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card.identity() == "@token/hob-dwarf")
                .count(),
            1
        );
        let recipe = g.copiable_recipe(g.object_ref(elf));
        assert_eq!(recipe.kinds(), 1);
        assert!(recipe.definition.has_subtype("Elf"));
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::TapMana { card: elf });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        g.state.objects[elf as usize].tapped = false;
        // Its retained mana ability is usable immediately to fund a real spell.
        let s = fixture(&mut g, p, "Supper for Spiders", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: s });
        act(&mut g, Action::TapMana { card: elf });
        pay(&mut g);
        finish(&mut g);
        assert!(g.object_view(elf).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn supper_simultaneous_foods_observe_artifacts_not_creatures_and_keep_self_observers() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let archivist = dead(&mut g, p, "Elvish Archivist");
        let duo = dead(&mut g, p, "Lifecreed Duo");
        paid(&mut g, p);
        finish(&mut g);
        food(&g, archivist, p);
        food(&g, duo, p);
        assert_eq!(g.counter_count(archivist, CounterKind::PlusOne), 2);
        assert_eq!(
            g.state.players[p.index()].life,
            20,
            "neither Food is a creature entry"
        );
    }
}

#[test]
fn supper_as_entry_food_replaces_angel_subtype_and_creature_tapped_entry_replacements() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        fixture(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        fixture(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let angel = fixture(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        g.move_object(angel, Zone::Graveyard);
        paid(&mut g, p);
        finish(&mut g);
        food(&g, angel, p);
        assert_eq!(g.counter_count(angel, CounterKind::PlusOne), 0);
        assert!(!g.object_view(angel).tapped);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
