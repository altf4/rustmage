use rustmage::{Card, Config, DeckRules, Game, Player};

fn config(deck: Vec<Card>) -> Config {
    Config {
        decks: [deck.clone(), deck],
        seed: 1,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    }
}

#[test]
fn every_pinned_standard_identity_has_a_native_catalogue_entry() {
    let snapshot: serde_json::Value =
        serde_json::from_str(include_str!("../data/standard/cards.json")).unwrap();
    let cards = snapshot["cards"].as_array().unwrap();
    assert_eq!(
        cards.len(),
        snapshot["card_count"].as_u64().unwrap() as usize
    );
    assert_eq!(
        Card::catalogue()
            .filter(|card| card.is_standard_legal())
            .count(),
        cards.len()
    );
    for record in cards {
        let name = record["name"].as_str().unwrap();
        let card = Card::from_name(name).expect(name);
        assert_eq!(card.name(), name);
        assert_eq!(card.info().oracle_id, record["oracle_id"].as_str());
        assert!(card.is_standard_legal());
        assert_eq!(
            serde_json::from_value::<Card>(record["name"].clone()).unwrap(),
            card
        );
    }
}

#[test]
fn coverage_manifest_matches_actual_executable_admission() {
    let coverage: serde_json::Value =
        serde_json::from_str(include_str!("../data/standard/coverage.json")).unwrap();
    assert_eq!(
        coverage["catalogue_version"].as_str().unwrap(),
        rustmage::CATALOGUE_VERSION
    );
    for row in coverage["cards"].as_array().unwrap() {
        let card = Card::from_name(row["name"].as_str().unwrap()).unwrap();
        assert_eq!(card.is_supported(), row["implemented"].as_bool().unwrap());
        assert_eq!(
            card.is_standard_legal(),
            row["standard_legal"].as_bool().unwrap()
        );
        let mut cfg = config(vec![card; 60]);
        cfg.deck_rules = DeckRules::Freeform;
        assert_eq!(
            Game::new(cfg).is_ok(),
            card.is_supported(),
            "{}",
            card.name()
        );
    }
}

#[test]
fn standard_checks_pool_and_semantics_independently() {
    let mut deck = vec![Card::Plains; 56];
    deck.extend([Card::from_name("Savannah Lions").unwrap(); 4]);
    assert!(Game::new(config(deck.clone())).is_ok());
    deck[0] = Card::from_name("Lightning Bolt").unwrap();
    assert!(
        Game::new(config(deck.clone()))
            .unwrap_err()
            .to_string()
            .contains("not Standard legal")
    );
    deck[0] = Card::from_name("Hare Apparent").unwrap();
    assert!(
        Game::new(config(deck))
            .unwrap_err()
            .to_string()
            .contains("rules not implemented")
    );
    let mut banned = vec![Card::Forest; 59];
    banned.push(Card::from_name("Heartfire Hero").unwrap());
    assert!(
        Game::new(config(banned))
            .unwrap_err()
            .to_string()
            .contains("not Standard legal")
    );
}

#[test]
fn source_spelling_aliases_do_not_replace_authoritative_names() {
    assert_eq!(Card::from_name("LightningBolt"), Some(Card::LightningBolt));
    assert!(Card::from_name("Araña, Heart of the Spider").is_some());
    assert!(
        Card::from_name("Araña, Heart of the Spider")
            .unwrap()
            .is_standard_legal()
    );
    assert!(Card::from_name("This Card Does Not Exist").is_none());
}

#[test]
fn token_factories_are_not_submitted_cards_in_any_deck_mode() {
    let tokens: Vec<_> = Card::catalogue().filter(|card| card.is_token()).collect();
    let coverage: serde_json::Value =
        serde_json::from_str(include_str!("../data/standard/coverage.json")).unwrap();
    assert_eq!(
        tokens.len(),
        coverage["token_definitions"].as_u64().unwrap() as usize
    );
    assert!(!tokens.is_empty());
    for card in tokens {
        assert!(!card.is_standard_legal());
        assert!(!Card::ALL.contains(&card));
        assert!(card.info().mana_cost.is_empty());
        assert!(card.name().starts_with("@token/"));
        for rules in [
            DeckRules::Freeform,
            DeckRules::Constructed,
            DeckRules::Standard,
        ] {
            let mut cfg = config(vec![card; 60]);
            cfg.deck_rules = rules;
            assert!(
                Game::new(cfg)
                    .unwrap_err()
                    .to_string()
                    .contains("tokens cannot")
            );
        }
    }
}

#[test]
fn intrinsic_colors_match_every_pinned_face_and_token_factory() {
    let snapshot: serde_json::Value =
        serde_json::from_str(include_str!("../data/standard/cards.json")).unwrap();
    for row in snapshot["cards"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let card = rustmage::Card::from_name(name).unwrap();
        let colors = row.get("colors").unwrap_or(&row["card_faces"][0]["colors"]);
        let expected = colors.as_array().unwrap().iter().fold(0, |mask, c| {
            mask | (1 << "WUBRG".find(c.as_str().unwrap()).unwrap())
        });
        assert_eq!(card.colors(), expected, "{name}");
    }
    for card in rustmage::Card::catalogue() {
        if let Some(token) = card.token_info() {
            assert_eq!(card.colors(), token.colors);
        }
    }
}

#[test]
fn printed_starting_loyalty_matches_pinned_front_faces_without_admitting_their_bodies() {
    let snapshot: serde_json::Value =
        serde_json::from_str(include_str!("../data/standard/cards.json")).unwrap();
    let mut count = 0;
    for row in snapshot["cards"].as_array().unwrap() {
        let c = Card::from_name(row["name"].as_str().unwrap()).unwrap();
        if !c.is_planeswalker() {
            assert_eq!(c.starting_loyalty(), 0);
            continue;
        }
        let face = if row.get("loyalty").is_some() {
            row
        } else {
            &row["card_faces"][0]
        };
        assert_eq!(
            c.starting_loyalty(),
            face["loyalty"].as_str().unwrap().parse::<u64>().unwrap(),
            "{}",
            c.name()
        );
        count += 1;
    }
    assert!(count >= 28);
    for name in [
        "Vivien Reid",
        "Chandra, Torch of Defiance",
        "The Aetherspark",
    ] {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_planeswalker());
        assert!(!c.is_supported());
        let mut cfg = config(vec![c; 60]);
        cfg.deck_rules = DeckRules::Freeform;
        assert!(Game::new(cfg).is_err());
    }
}

#[test]
fn emblem_factories_are_noncard_objects_and_rejected_in_every_deck_mode() {
    let coverage: serde_json::Value =
        serde_json::from_str(include_str!("../data/standard/coverage.json")).unwrap();
    let emblems: Vec<_> = Card::catalogue().filter(|c| c.is_emblem()).collect();
    assert_eq!(
        emblems.len(),
        coverage["emblem_definitions"].as_u64().unwrap() as usize
    );
    assert!(!emblems.is_empty());
    for card in emblems {
        assert!(!card.is_token());
        assert!(!card.is_standard_legal());
        assert!(!Card::ALL.contains(&card));
        assert!(card.info().type_line.is_empty());
        assert_eq!(card.colors(), 0);
        for rules in [
            DeckRules::Freeform,
            DeckRules::Constructed,
            DeckRules::Standard,
        ] {
            let mut cfg = config(vec![card; 60]);
            cfg.deck_rules = rules;
            assert!(
                Game::new(cfg)
                    .unwrap_err()
                    .to_string()
                    .contains("emblems cannot")
            );
        }
    }
}
