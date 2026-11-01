//! Literal Oracle expectations and lightweight cases for printed card features.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    kind: String,
    mana_cost: String,
    #[serde(default)]
    power: i128,
    #[serde(default)]
    toughness: i128,
    #[serde(default)]
    amount: i128,
    #[serde(default)]
    colors: Vec<String>,
}

#[test]
fn simple_card_families() {
    let cases: Vec<Case> = fixture_json("families.json");
    assert!(!cases.is_empty());
    for case in cases {
        println!("card family: {}", case.name);
        let mut g = game();
        let id = put(&mut g, Player::P0, &case.name, Zone::Hand);
        let target = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        if case.kind == "land" {
            assert!(
                !case.colors.is_empty(),
                "{} needs a printed mana outcome",
                case.name
            );
            act(&mut g, Action::PlayLand { card: id });
            for color in &case.colors {
                g.state.objects[id as usize].tapped = false;
                g.state.players[0].mana = [0; 6];
                let mut expected = [0; 6];
                expected["WUBRGC".find(color).unwrap()] = 1;
                choose(&mut g, |a| {
                    matches!(a, Action::TapMana { card } if *card == id)
                        || matches!(a, Action::ActivateMana { card, production, .. } if *card == id && *production == expected)
                });
                assert_eq!(g.state.players[0].mana, expected, "{} mana", case.name);
                assert!(g.object_view(id).tapped, "{} tap cost", case.name);
            }
            covered(&format!(
                "family/{}",
                Card::from_name(&case.name).unwrap().index()
            ));
            continue;
        }
        let targets = match case.kind.as_str() {
            "damage" => vec![Target::Player(Player::P1)],
            "destroy" | "bounce" => vec![Target::Card(target)],
            _ => vec![],
        };
        let mut printed_mana = [0; 6];
        for part in case.mana_cost.split('{').skip(1) {
            let symbol = part.trim_end_matches('}');
            if let Ok(amount) = symbol.parse::<u32>() {
                printed_mana[5] += amount;
            } else {
                let color = symbol.split('/').next().unwrap();
                printed_mana["WUBRGC".find(color).expect("fixed printed mana")] += 1;
            }
        }
        g.state.players[0].mana = printed_mana;
        cast(&mut g, id, &targets);
        assert_eq!(g.state.players[0].mana, [0; 6], "{} cost", case.name);
        if case.kind == "body_entry" {
            until(&mut g, |g| g.on_battlefield(id));
        } else {
            resolve(&mut g);
        }
        match case.kind.as_str() {
            "creature" | "body_entry" => {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield, "{}", case.name);
                assert_eq!(stats(&g, id), (case.power, case.toughness), "{}", case.name);
            }
            "damage" => assert_eq!(g.state.players[1].life, 20 - case.amount, "{}", case.name),
            "draw" => assert_eq!(
                g.hand_count(Player::P0),
                case.amount as usize,
                "{}",
                case.name
            ),
            "destroy" => assert_eq!(g.object_view(target).zone, Zone::Graveyard, "{}", case.name),
            "bounce" => assert_eq!(g.object_view(target).zone, Zone::Hand, "{}", case.name),
            "permanent" => assert_eq!(g.object_view(id).zone, Zone::Battlefield, "{}", case.name),
            other => panic!("unknown family: {other}"),
        }
        covered(&format!(
            "family/{}",
            Card::from_name(&case.name).unwrap().index()
        ));
    }
}

#[test]
fn smaug_entry_creates_tapped_treasures_for_opponents_artifacts() {
    let mut g = game();
    put(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
    for _ in 0..2 {
        put(&mut g, Player::P1, "Goldvein Pick", Zone::Battlefield);
    }
    let smaug = put(&mut g, Player::P0, "Smaug, Wicked Worm", Zone::Hand);
    cast(&mut g, smaug, &[]);
    resolve(&mut g);
    let treasures: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield && o.controller == Player::P0 && o.has_subtype("Treasure")
        })
        .collect();
    assert_eq!(treasures.len(), 2);
    assert!(treasures.iter().all(|o| o.tapped));
}

#[test]
fn smaug_treasure_funded_cast_draws_and_loses_life() {
    let mut g = game();
    put(&mut g, Player::P0, "Smaug, Wicked Worm", Zone::Battlefield);
    let treasure = put(
        &mut g,
        Player::P0,
        "@token/b60f6cd3b44b22bd289fe4e2",
        Zone::Battlefield,
    );
    g.state.players[0].mana = [0; 6];
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana {card, production: [0, 0, 0, 1, 0, 0], ..} if *card == treasure),
    );
    let shock = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, shock, &[Target::Player(Player::P1)]);
    resolve(&mut g);
    assert_eq!(g.hand_count(Player::P0), 1);
    assert_eq!(g.state.players[0].life, 19);
}
