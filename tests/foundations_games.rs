//! Initial full-game regression matrix for the Foundations milestone.
//! This exercises admitted cards; it does not certify all 517 set obligations.
use rustmage::{Action, Card, Config, DeckRules, Error, Game, Player, StackKind, Zone};
use std::collections::BTreeSet;

fn deck(land: &str, names: &[&str]) -> Vec<Card> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/fdn.json")).unwrap();
    let members: BTreeSet<_> = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    let mut cards = vec![Card::from_name(land).unwrap(); 24];
    assert!(members.contains(land));
    for name in names {
        assert!(members.contains(name), "not in full FDN inventory: {name}");
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported(), "not implemented: {name}");
        cards.extend([card; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}

#[test]
fn foundations_chosen_values_control_and_death_cards_full_games() {
    let mut decks = [
        deck(
            "Plains",
            &[
                "Diamond Mare",
                "Heraldic Banner",
                "Adaptive Automaton",
                "Banner of Kinship",
                "Healer's Hawk",
                "Leonin Vanguard",
                "Resolute Reinforcements",
                "Angel of Vitality",
                "Inspiring Overseer",
            ],
        ),
        deck(
            "Swamp",
            &[
                "Wishclaw Talisman",
                "Harmless Offering",
                "Garna, Bloodfist of Keld",
                "Crossway Troublemakers",
                "Desecration Demon",
                "Vampire Interloper",
                "Vengeful Bloodwitch",
                "Feed the Swarm",
                "Diregraf Ghoul",
            ],
        ),
    ];
    for land in &mut decks[0][..4] {
        *land = Card::from_name("Uncharted Haven").unwrap();
    }
    for land in &mut decks[1][..8] {
        *land = Card::from_name("Mountain").unwrap();
    }
    run_matrix_actions(
        &decks,
        &[
            "Diamond Mare",
            "Heraldic Banner",
            "Adaptive Automaton",
            "Banner of Kinship",
            "Wishclaw Talisman",
            "Harmless Offering",
            "Garna, Bloodfist of Keld",
            "Crossway Troublemakers",
            "Desecration Demon",
        ],
        &["Uncharted Haven"],
        &["Uncharted Haven", "Heraldic Banner"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
    );
}

#[test]
fn foundations_qualified_graveyard_and_resolution_cards_full_games() {
    let mut decks = [
        deck(
            "Swamp",
            &[
                "Knight of Grace",
                "Knight of Malice",
                "Elenda, Saint of Dusk",
                "Myojin of Night's Reach",
                "Resolute Reinforcements",
                "Healer's Hawk",
                "Angel of Vitality",
                "Gilded Lotus",
                "Feed the Swarm",
            ],
        ),
        deck(
            "Mountain",
            &[
                "Flamewake Phoenix",
                "Gate Colossus",
                "Fiery Annihilation",
                "Venom Connoisseur",
                "Llanowar Elves",
                "Garruk's Uprising",
                "Rampaging Baloths",
                "Goldvein Pick",
                "Solemn Simulacrum",
            ],
        ),
    ];
    for land in &mut decks[0][..8] {
        *land = Card::from_name("Plains").unwrap();
    }
    for land in &mut decks[1][..4] {
        *land = Card::from_name("Gruul Guildgate").unwrap();
    }
    for land in &mut decks[1][4..8] {
        *land = Card::from_name("Golgari Guildgate").unwrap();
    }
    for land in &mut decks[1][8..16] {
        *land = Card::from_name("Forest").unwrap();
    }
    run_matrix_actions(
        &decks,
        &[
            "Knight of Grace",
            "Knight of Malice",
            "Elenda, Saint of Dusk",
            "Myojin of Night's Reach",
            "Flamewake Phoenix",
            "Gate Colossus",
            "Fiery Annihilation",
            "Venom Connoisseur",
        ],
        &["Gruul Guildgate"],
        &["Gruul Guildgate"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
    );
}

#[test]
fn foundations_five_bounded_cards_natural_openings_combat_and_full_games() {
    let mut decks = [
        deck(
            "Mountain",
            &[
                "Dropkick Bomber",
                "Leyline Axe",
                "Mazemind Tome",
                "Frenzied Goblin",
                "Goblin Smuggler",
                "Krenko, Mob Boss",
                "Goblin Surprise",
                "Goblin Boarders",
                "Heartfire Immolator",
            ],
        ),
        deck(
            "Forest",
            &[
                "Predator Ooze",
                "Leyline Axe",
                "Mazemind Tome",
                "Llanowar Elves",
                "Dwynen's Elite",
                "Bear Cub",
                "Garruk's Uprising",
                "Affectionate Indrik",
                "Primal Might",
            ],
        ),
    ];
    for deck in &mut decks {
        for land in &mut deck[..4] {
            *land = Card::from_name("Demolition Field").unwrap();
        }
    }
    for land in &mut decks[0][4..8] {
        *land = Card::from_name("Rogue's Passage").unwrap();
    }
    for land in &mut decks[1][4..8] {
        *land = Card::from_name("Rugged Highlands").unwrap();
    }
    run_matrix_actions(
        &decks,
        &[
            "Dropkick Bomber",
            "Leyline Axe",
            "Mazemind Tome",
            "Predator Ooze",
        ],
        &["Demolition Field"],
        &["Demolition Field"],
        &["Leyline Axe"],
        &[0, 1, 2, 3, 7, 13, 41, 73],
    );
}

#[test]
fn foundations_five_color_full_games_and_owned_search_replay() {
    let mut decks = [
        deck(
            "Plains",
            &[
                "Angel of Vitality",
                "Exemplar of Light",
                "Fiendish Panda",
                "Skyknight Squire",
                "Leonin Vanguard",
                "Hare Apparent",
                "Healer's Hawk",
                "Inspiring Overseer",
                "Stroke of Midnight",
            ],
        ),
        deck(
            "Island",
            &[
                "Run Away Together",
                "Inspiration from Beyond",
                "Trygon Predator",
                "Fog Bank",
                "Corsair Captain",
                "Drake Hatcher",
                "Exclusion Mage",
                "Unsummon",
                "Harbinger of the Tides",
            ],
        ),
        deck(
            "Swamp",
            &[
                "Painful Quandary",
                "Vengeful Bloodwitch",
                "Ravenous Amulet",
                "Duress",
                "Pilfer",
                "Feed the Swarm",
                "Dread Summons",
                "Perforating Artist",
                "Tragic Banshee",
            ],
        ),
        deck(
            "Mountain",
            &[
                "Involuntary Employment",
                "Giant Cindermaw",
                "Heartfire Immolator",
                "Goblin Smuggler",
                "Bulk Up",
                "Hidetsugu's Second Rite",
                "Burst Lightning",
                "Ruby, Daring Tracker",
                "Juggernaut",
            ],
        ),
        deck(
            "Forest",
            &[
                "Primal Might",
                "Mild-Mannered Librarian",
                "Rise of the Dark Realms",
                "Llanowar Elves",
                "Elvish Archdruid",
                "Dwynen, Gilt-Leaf Daen",
                "Inspiring Call",
                "Affectionate Indrik",
                "Heroes' Bane",
            ],
        ),
    ];
    // Supply printed off-color costs within the Foundations inventory.
    for land in &mut decks[0][..8] {
        *land = Card::from_name("Swamp").unwrap();
    }
    for land in &mut decks[1][..8] {
        *land = Card::from_name("Forest").unwrap();
    }
    for land in &mut decks[2][..8] {
        *land = Card::from_name("Mountain").unwrap();
    }
    for land in &mut decks[3][..8] {
        *land = Card::from_name("Forest").unwrap();
    }
    for land in &mut decks[4][..10] {
        *land = Card::from_name("Swamp").unwrap();
    }
    run_matrix(
        &decks,
        &[
            "Run Away Together",
            "Involuntary Employment",
            "Pilfer",
            "Skyknight Squire",
            "Mild-Mannered Librarian",
            "Drake Hatcher",
            "Ravenous Amulet",
            "Juggernaut",
            "Painful Quandary",
            "Perforating Artist",
        ],
    );
}

#[test]
fn foundations_cast_counter_and_program_cards_full_games() {
    let mut decks = [
        deck(
            "Plains",
            &[
                "Jazal Goldmane",
                "Mentor of the Meek",
                "Linden, the Steadfast Queen",
                "Ayli, Eternal Pilgrim",
                "Healer's Hawk",
                "Angel of Vitality",
                "Exemplar of Light",
                "Fleeting Flight",
                "Raise the Past",
            ],
        ),
        deck(
            "Island",
            &[
                "Kiora, the Rising Tide",
                "Koma, World-Eater",
                "Prime Speaker Zegana",
                "Lunar Insight",
                "Fog Bank",
                "Corsair Captain",
                "High Fae Trickster",
                "Exclusion Mage",
                "Unsummon",
            ],
        ),
        deck(
            "Swamp",
            &[
                "Soul-Shackled Zombie",
                "Tribute to Hunger",
                "Midnight Snack",
                "Vengeful Bloodwitch",
                "Duress",
                "Feed the Swarm",
                "Diregraf Ghoul",
                "Vampire Nighthawk",
                "Tragic Banshee",
            ],
        ),
        deck(
            "Mountain",
            &[
                "Ashroot Animist",
                "Frenzied Goblin",
                "Mindsparker",
                "Taurean Mauler",
                "Stromkirk Noble",
                "Slumbering Cerberus",
                "Quick-Draw Katana",
                "Heartfire Immolator",
                "Burst Lightning",
            ],
        ),
        deck(
            "Forest",
            &[
                "Sylvan Scavenging",
                "Mold Adder",
                "Mossborn Hydra",
                "Wildwood Scourge",
                "Scavenging Ooze",
                "Ramos, Dragon Engine",
                "Llanowar Elves",
                "Inspiring Call",
                "Dwynen's Elite",
            ],
        ),
    ];
    for land in &mut decks[0][..8] {
        *land = Card::from_name("Swamp").unwrap();
    }
    for land in &mut decks[1][..12] {
        *land = Card::from_name("Forest").unwrap();
    }
    for land in &mut decks[3][..8] {
        *land = Card::from_name("Forest").unwrap();
    }
    run_matrix(
        &decks,
        &["Mentor of the Meek", "Frenzied Goblin", "Mold Adder"],
    );
}

/// Broad admitted-pool interaction regression, not proof of every printed rule.
#[test]
fn foundations_library_damage_events_and_cast_choices_full_games() {
    let mut decks = [
        deck(
            "Plains",
            &[
                "Felidar Retreat",
                "Divine Resilience",
                "Crystal Barricade",
                "Healer's Hawk",
                "Hare Apparent",
                "Angel of Vitality",
                "Jazal Goldmane",
                "Mentor of the Meek",
                "Linden, the Steadfast Queen",
            ],
        ),
        deck(
            "Island",
            &[
                "Faebloom Trick",
                "Finale of Revelation",
                "Sphinx of the Final Word",
                "Niv-Mizzet, Visionary",
                "Consuming Aberration",
                "Exclusion Mage",
                "Kiora, the Rising Tide",
                "High Fae Trickster",
                "Fog Bank",
            ],
        ),
        deck(
            "Swamp",
            &[
                "Kalastria Highborn",
                "Vampire Gourmand",
                "Gutless Plunderer",
                "Eaten Alive",
                "Blasphemous Edict",
                "Alesha, Who Laughs at Fate",
                "Immersturm Predator",
                "Vengeful Bloodwitch",
                "Tribute to Hunger",
            ],
        ),
        deck(
            "Mountain",
            &[
                "Strongbox Raider",
                "Redcap Gutter-Dweller",
                "Gratuitous Violence",
                "Twinflame Tyrant",
                "Frenzied Goblin",
                "Mindsparker",
                "Shivan Dragon",
                "Burst Lightning",
                "Krenko, Mob Boss",
            ],
        ),
        deck(
            "Forest",
            &[
                "Vivien Reid",
                "Loot, Exuberant Explorer",
                "Spinner of Souls",
                "Genesis Wave",
                "Ordeal of Nylea",
                "Llanowar Elves",
                "Bear Cub",
                "Mossborn Hydra",
                "Wildwood Scourge",
            ],
        ),
    ];
    for land in &mut decks[1][..6] {
        *land = Card::from_name("Swamp").unwrap();
    }
    for land in &mut decks[1][6..12] {
        *land = Card::from_name("Mountain").unwrap();
    }
    for land in &mut decks[2][..8] {
        *land = Card::from_name("Mountain").unwrap();
    }
    run_matrix(
        &decks,
        &["Gutless Plunderer", "Divine Resilience", "Vampire Gourmand"],
    );
}

#[test]
fn foundations_every_admitted_card_in_set_only_full_games() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/fdn.json")).unwrap();
    let mut cards: Vec<_> = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            Card::from_name(row["name"].as_str().unwrap())
                .expect("full Foundations inventory must have a catalogue identity")
        })
        .filter(|card| card.is_supported())
        .collect();
    cards.sort_by_key(|card| (card.colors(), card.fixed_mana_value(), card.name()));
    let (set_lands, spells): (Vec<_>, Vec<_>) = cards.iter().copied().partition(|c| c.is_land());
    let groups: Vec<_> = spells.chunks(9).collect();
    assert!(!groups.is_empty());
    // Include every land alongside spells instead of creating all-land decks
    // whose only usual route to an outcome is drawing an empty library.
    let deck_count = groups.len().max(set_lands.len().div_ceil(6));
    let basics = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
    let mut decks = Vec::new();
    for index in 0..deck_count {
        let group = groups[index % groups.len()];
        let colors = group.iter().fold(0u8, |mask, card| mask | card.colors());
        let mut lands: Vec<_> = basics
            .iter()
            .enumerate()
            .filter(|(index, _)| colors & (1 << index) != 0)
            .map(|(_, name)| Card::from_name(name).unwrap())
            .collect();
        if lands.is_empty() {
            lands.push(Card::from_name("Forest").unwrap());
        }
        let mut deck: Vec<_> = (0..24).map(|i| lands[i % lands.len()]).collect();
        for &card in group {
            deck.extend([card; 4]);
        }
        while deck.len() < 60 {
            deck.push(lands[deck.len() % lands.len()]);
        }
        decks.push(deck);
    }
    for (index, land) in set_lands.into_iter().enumerate() {
        let first = 4 * (index / deck_count);
        decks[index % deck_count][first..first + 4].fill(land);
    }
    let covered: BTreeSet<_> = decks.iter().flatten().copied().collect();
    assert_eq!(covered, cards.into_iter().collect());
    assert!(
        decks
            .iter()
            .all(|deck| deck.len() == 60 && deck.iter().any(|c| !c.is_land()))
    );
    run_matrix(&decks, &[]);
}

fn run_matrix(decks: &[Vec<Card>], expected_casts: &[&str]) {
    run_matrix_actions(decks, expected_casts, &[], &[], &[], &[7, 41]);
}

fn run_matrix_actions(
    decks: &[Vec<Card>],
    expected_casts: &[&str],
    expected_lands: &[&str],
    expected_mana_sources: &[&str],
    expected_openings: &[&str],
    seeds: &[u64],
) {
    run_matrix_actions_observed(
        decks,
        expected_casts,
        expected_lands,
        expected_mana_sources,
        expected_openings,
        seeds,
        None,
    );
}

type MatrixObserver<'a> = dyn FnMut(&rustmage::Observation, &rustmage::Observation, bool) + 'a;

fn run_matrix_actions_observed(
    decks: &[Vec<Card>],
    expected_casts: &[&str],
    expected_lands: &[&str],
    expected_mana_sources: &[&str],
    expected_openings: &[&str],
    seeds: &[u64],
    mut observer: Option<&mut MatrixObserver<'_>>,
) {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/fdn.json")).unwrap();
    let members: BTreeSet<Card> = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| Card::from_name(row["name"].as_str().unwrap()).unwrap())
        .collect();
    for card in decks.iter().flatten() {
        assert!(
            members.contains(card),
            "non-Foundations card after deck edits: {}",
            card.identity()
        );
    }
    let mut opening_permanents = BTreeSet::new();
    let mut played_lands = BTreeSet::new();
    let mut used_mana_sources = BTreeSet::new();
    let mut total_decisions = 0;
    let mut triggered = false;
    let mut token = false;
    let mut combat = false;
    let mut branches = 0;
    let mut new_spells = BTreeSet::new();
    let trace_games = std::env::var_os("RUSTMAGE_TRACE_GAMES").is_some();
    for pair in 0..decks.len() {
        for &seed in seeds {
            for starter in [Player::P0, Player::P1] {
                if trace_games {
                    eprintln!("FDN game start: pair={pair}, seed={seed}, starter={starter:?}");
                }
                let mut game = Game::new(Config {
                    decks: [decks[pair].clone(), decks[(pair + 1) % decks.len()].clone()],
                    seed,
                    starting_player: starter,
                    deck_rules: DeckRules::Standard,
                })
                .unwrap();
                for viewer in [Player::P0, Player::P1] {
                    let obs = game.observe(viewer);
                    assert!(
                        obs.objects
                            .iter()
                            .all(|o| o.zone != Zone::Hand || o.owner == viewer)
                    );
                }
                let mut decisions = 0;
                while game.outcome().is_none() {
                    assert!(
                        decisions < 25_000,
                        "FDN game stalled: pair={pair}, seed={seed}, starter={starter:?}"
                    );
                    let actor = game.actor().unwrap();
                    let actions = game.legal_actions();
                    assert!(!actions.is_empty());
                    let index = rustmage::policy::smoke_index(&game, &actions);
                    let action = actions[index].clone();
                    assert!(!matches!(action, Action::Concede));
                    let obs = game.observe(actor);
                    match &action {
                        Action::BeginOnBattlefield { object } => {
                            let chosen =
                                obs.objects.iter().find(|o| o.reference == *object).unwrap();
                            opening_permanents.insert(chosen.card.name());
                        }
                        Action::PlayLand { card } => {
                            let object = obs.objects.iter().find(|o| o.id == *card).unwrap();
                            played_lands.insert(object.card.name());
                        }
                        Action::ActivateMana { card, .. } | Action::TapMana { card } => {
                            let object = obs.objects.iter().find(|o| o.id == *card).unwrap();
                            used_mana_sources.insert(object.card.name());
                        }
                        _ => {}
                    }
                    for item in &obs.stack {
                        if item.kind == StackKind::Spell {
                            new_spells.insert(item.source_card.name());
                        }
                    }
                    triggered |= obs.stack.iter().any(|item| item.kind != StackKind::Spell);
                    token |= obs.objects.iter().any(|o| o.token.is_some());
                    combat |= !obs.attackers.is_empty();
                    let owned_replay = decisions % 47 == 0;
                    if owned_replay {
                        let revision = game.revision();
                        let before = game.snapshot();
                        assert_eq!(
                            game.apply(actor.other(), action.clone()),
                            Err(Error::WrongActor)
                        );
                        assert_eq!(game.snapshot(), before);
                        let mut child = {
                            let parent = game.clone();
                            parent.clone()
                        };
                        child.apply_index(actor, revision, index).unwrap();
                        game.apply(actor, action.clone()).unwrap();
                        assert_eq!(child.snapshot(), game.snapshot());
                        for viewer in [Player::P0, Player::P1] {
                            assert_eq!(
                                serde_json::to_value(child.observe(viewer)).unwrap(),
                                serde_json::to_value(game.observe(viewer)).unwrap()
                            );
                        }
                        let after = game.snapshot();
                        game.restore(&before).unwrap();
                        assert_eq!(
                            game.apply_index(actor, revision, index),
                            Err(Error::StaleRevision)
                        );
                        game.apply(actor, action).unwrap();
                        assert_eq!(game.snapshot(), after);
                        branches += 1;
                    } else {
                        game.apply(actor, action).unwrap();
                    }
                    if let Some(observer) = &mut observer {
                        observer(&obs, &game.observe(actor), owned_replay);
                    }
                    decisions += 1;
                }
                assert!(decisions > 50, "game ended before meaningful play");
                total_decisions += decisions;
                if trace_games {
                    eprintln!(
                        "FDN game complete: pair={pair}, seed={seed}, starter={starter:?}, decisions={decisions}"
                    );
                }
            }
        }
    }
    assert!(
        triggered && token && combat,
        "matrix events: triggered={triggered}, token={token}, combat={combat}; spells={new_spells:?}"
    );
    for name in expected_casts {
        assert!(
            new_spells.contains(name),
            "new card was not cast in full games: {name}; observed spells={new_spells:?}"
        );
    }
    for name in expected_openings {
        assert!(
            opening_permanents.contains(name),
            "opening-hand permanent was not naturally placed: {name}"
        );
    }
    for name in expected_lands {
        assert!(played_lands.contains(name), "land was not played: {name}");
    }
    for name in expected_mana_sources {
        assert!(
            used_mana_sources.contains(name),
            "mana source was not used: {name}"
        );
    }
    assert!(branches >= 20);
    let games = decks.len() * seeds.len() * 2;
    println!(
        "FDN matrix: {games} natural full games, {total_decisions} decisions, {branches} owned replay checks; {} cast identities, not full-set acceptance",
        new_spells.len()
    );
}

#[path = "common/foundations_copy_games.rs"]
mod copy_games;

#[path = "common/foundations_mana_games.rs"]
mod mana_games;

#[path = "common/foundations_characteristic_modal_games.rs"]
mod characteristic_modal_games;

#[path = "common/foundations_kaito_games.rs"]
mod kaito_games;
