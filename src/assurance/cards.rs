use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    kind: String,
    mana_value: u32,
    #[serde(default)]
    power: i128,
    #[serde(default)]
    toughness: i128,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    amount: i128,
}
fn keyword(name: &str) -> Keywords {
    match name {
        "flying" => Keywords::FLYING,
        "vigilance" => Keywords::VIGILANCE,
        "lifelink" => Keywords::LIFELINK,
        "deathtouch" => Keywords::DEATHTOUCH,
        "trample" => Keywords::TRAMPLE,
        "reach" => Keywords::REACH,
        "haste" => Keywords::HASTE,
        "defender" => Keywords::DEFENDER,
        "first strike" => Keywords::FIRST_STRIKE,
        "double strike" => Keywords::DOUBLE_STRIKE,
        "menace" => Keywords::MENACE,
        "flash" => Keywords::FLASH,
        "hexproof" => Keywords::HEXPROOF,
        "indestructible" => Keywords::INDESTRUCTIBLE,
        _ => panic!("unreviewed keyword: {name}"),
    }
}

#[test]
fn oracle_whole_card_outcomes_and_printed_costs() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("oracle_cases.json")).unwrap();
    assert_eq!(
        cases.len(),
        54,
        "review additions and coverage count together"
    );
    for c in cases {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, &c.name, Zone::Hand);
            let before_mana: u32 = g.state.players[p.index()].mana.iter().sum();
            let before_hand = g.hand_count(p);
            let target = match c.kind.as_str() {
                "damage" => Some(Target::Player(p.other())),
                "destroy" | "bounce" | "pump" => Some(Target::Card(creature)),
                _ => None,
            };
            cast(&mut g, spell, target);
            assert_eq!(
                before_mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                c.mana_value,
                "{} cost",
                c.name
            );
            settle(&mut g);
            match c.kind.as_str() {
                "body" => {
                    assert_eq!(g.object_view(spell).zone, Zone::Battlefield, "{}", c.name);
                    assert_eq!(stats(&g, spell), (c.power, c.toughness), "{}", c.name);
                    let expected = c
                        .keywords
                        .iter()
                        .fold(Keywords::NONE, |k, s| k.union(keyword(s)));
                    assert_eq!(g.keywords(spell), expected, "{}", c.name);
                    assert!(g.object_view(spell).summoning_sick);
                }
                "damage" => assert_eq!(
                    g.state.players[p.other().index()].life,
                    20 - c.amount,
                    "{}",
                    c.name
                ),
                "draw" => assert_eq!(
                    g.hand_count(p),
                    before_hand - 1 + c.amount as usize,
                    "{}",
                    c.name
                ),
                "destroy" => {
                    assert_eq!(g.object_view(creature).zone, Zone::Graveyard, "{}", c.name)
                }
                "bounce" => assert_eq!(g.object_view(creature).zone, Zone::Hand, "{}", c.name),
                "pump" => {
                    assert_eq!(
                        stats(&g, creature),
                        (2 + c.power, 2 + c.toughness),
                        "{}",
                        c.name
                    );
                    g.finish_cleanup();
                    assert_eq!(stats(&g, creature), (2, 2));
                }
                _ => unreachable!(),
            }
            if c.kind != "body" {
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn every_physical_definition_has_the_correct_admission_and_private_opening_hand() {
    let coverage: serde_json::Value =
        serde_json::from_str(include_str!("../../data/standard/coverage.json")).unwrap();
    let mut supported = 0;
    for row in coverage["cards"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let card = Card::from_name(name).unwrap();
        let complete = row["implemented"].as_bool().unwrap();
        assert_eq!(card.is_supported(), complete, "{name}");
        let result = Game::new(Config {
            decks: [vec![card; 7], vec![Card::Forest; 7]],
            seed: 1,
            starting_player: Player::P0,
            deck_rules: DeckRules::Freeform,
        });
        if complete {
            supported += 1;
            let g = result.unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(
                g.observe(Player::P0)
                    .objects
                    .iter()
                    .filter(|o| o.card == card)
                    .count(),
                7,
                "{name}"
            );
            assert!(
                g.observe(Player::P1)
                    .objects
                    .iter()
                    .all(|o| o.owner != Player::P0),
                "{name} hidden hand"
            );
        } else {
            assert!(result.is_err(), "unimplemented {name} admitted");
        }
    }
    assert_eq!(supported, coverage["implemented_total"].as_u64().unwrap());
}

#[test]
fn basic_lands_supply_exact_colors_once_per_untap() {
    for (name, color) in [
        ("Plains", 0),
        ("Island", 1),
        ("Swamp", 2),
        ("Mountain", 3),
        ("Forest", 4),
    ] {
        let mut g = scenario(Player::P0);
        g.state.players[0].mana = [0; 6];
        let land = put(&mut g, Player::P0, name, Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        act(&mut g, Action::TapMana { card: land });
        let mut expected = [0; 6];
        expected[color] = 1;
        assert_eq!(g.state.players[0].mana, expected);
        assert!(g.object_view(land).tapped);
        reject(&mut g, Action::TapMana { card: land });
        let next = put(&mut g, Player::P0, name, Zone::Hand);
        reject(&mut g, Action::PlayLand { card: next });
    }
}
