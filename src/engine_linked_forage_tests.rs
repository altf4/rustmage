//! Forage routes keep their intent while mana costs release new witnesses.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn returned_witness(p: Player, name: &str) -> (Game, CardId, CardId, CardId, CardId) {
    let mut g = setup(p);
    let food = name == "Candy Trail";
    let source = named(
        &mut g,
        if food { p.other() } else { p },
        if food { "Banishing Light" } else { "Henchbots" },
        Zone::Battlefield,
    );
    let treasure = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|i| i.name == "Treasure Token"))
        .unwrap();
    // The paid Henchbots/Goatnap/Mirrorform line is covered by linked_funding_tests.
    // This physical source retains its link through copying and becomes a real
    // graveyard card when its copied Treasure ability sacrifices it.
    g.copy_permanents_exact(&[g.object_ref(source)], CopyRecipe::new(treasure));
    assert!(!g.object_view(source).is_token());
    let witness = named(&mut g, p, name, Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(witness),
        return_zone: Zone::Battlefield,
    });
    if food {
        // The opponent's Mirrorform can copy their Banishing Light into
        // Treasure. A later Confiscate, outside that frozen recipient set,
        // lets us sacrifice it and return our originally exiled Food.
        let confiscate = named(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, confiscate, None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.object_view(source).controller, p);
    }
    g.state.players[p.index()].mana = [0; 6];
    let swamp = named(&mut g, p, "Swamp", Zone::Battlefield);
    let spell = named(&mut g, p, "Feed the Cycle", Zone::Hand);
    (g, source, witness, swamp, spell)
}

fn pay_route(p: Player, route: ForageRoute) {
    let (mut g, source, witness, swamp, spell) = returned_witness(
        p,
        if route == ForageRoute::Food {
            "Candy Trail"
        } else {
            "Llanowar Elves"
        },
    );
    let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let mut graveyard = Vec::new();
    if route == ForageRoute::ExileThree {
        graveyard.push(named(&mut g, p, "Forest", Zone::Graveyard));
        graveyard.push(named(&mut g, p, "Island", Zone::Graveyard));
    }
    checked(&mut g, Action::AnnounceCast { card: spell });
    assert!(matches!(g.state.pending, Pending::ForageMethod { .. }));
    assert!(!g.legal_actions().contains(&Action::ChooseForageRoute {
        route: ForageRoute::Black,
    }));
    assert!(!g.legal_actions().contains(&Action::ChooseForageRoute {
        route: if route == ForageRoute::Food {
            ForageRoute::ExileThree
        } else {
            ForageRoute::Food
        },
    }));
    checked(&mut g, Action::ChooseForageRoute { route });
    // Route intent precedes targets; mana cannot be activated in this phase.
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    assert!(matches!(g.state.pending, Pending::Forage { .. }));
    let locked = g.state.announcing.as_ref().unwrap().total_cost;
    assert!(locked.is_some());
    assert!(g.observe(p.other()).forage.is_none());
    let activation = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a, Action::ActivateMana { card, production, .. }
                if *card == source && *production == [1, 0, 0, 0, 0, 0]
            )
        })
        .expect("linked source can supply mana before forage witnesses are chosen");
    checked(&mut g, activation);
    assert_eq!(g.object_view(witness).zone, Zone::Battlefield);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.announcing.as_ref().unwrap().total_cost, locked);
    if route == ForageRoute::Food {
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseForage { object });
    } else {
        graveyard.push(source);
        for &id in &graveyard {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseForage { object });
        }
    }
    checked(&mut g, Action::TapMana { card: swamp });
    checked(
        &mut g,
        Action::PayCosts {
            payment: [1, 0, 1, 0, 0, 0],
        },
    );
    assert!(g.state.stack.iter().any(|s| s.card == spell));
    if route == ForageRoute::Food {
        assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
    } else {
        for id in graveyard {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
    }
    settle(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn linked_forage_mana_sacrifice_provides_third_graveyard_card_both_seats() {
    for p in [Player::P0, Player::P1] {
        pay_route(p, ForageRoute::ExileThree);
    }
}

#[test]
fn linked_forage_mana_return_provides_food_both_seats() {
    for p in [Player::P0, Player::P1] {
        pay_route(p, ForageRoute::Food);
    }
}
