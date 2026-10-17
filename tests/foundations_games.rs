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
fn foundations_five_color_full_games_and_owned_search_replay() {
    let decks = [
        deck(
            "Plains",
            &[
                "Savannah Lions",
                "Bishop's Soldier",
                "Healer's Hawk",
                "Helpful Hunter",
                "Inspiring Overseer",
                "Pacifism",
                "Adamant Will",
                "Prideful Parent",
                "Serra Angel",
            ],
        ),
        deck(
            "Island",
            &[
                "Opt",
                "Quick Study",
                "Spectral Sailor",
                "Elementalist Adept",
                "Mocking Sprite",
                "Exclusion Mage",
                "Bigfin Bouncer",
                "Unsummon",
                "Tempest Djinn",
            ],
        ),
        deck(
            "Swamp",
            &[
                "Diregraf Ghoul",
                "Infestation Sage",
                "Vampire Interloper",
                "Vampire Nighthawk",
                "Skeleton Archer",
                "Maalfeld Twins",
                "Stab",
                "Bake into a Pie",
                "Moment of Craving",
            ],
        ),
        deck(
            "Mountain",
            &[
                "Swab Goblin",
                "Brazen Scourge",
                "Viashino Pyromancer",
                "Dragon Fodder",
                "Guttersnipe",
                "Burst Lightning",
                "Goblin Surprise",
                "Boltwave",
                "Shivan Dragon",
            ],
        ),
        deck(
            "Forest",
            &[
                "Llanowar Elves",
                "Bear Cub",
                "Druid of the Cowl",
                "Magnigoth Sentry",
                "Thornweald Archer",
                "Giant Growth",
                "Gnarlid Colony",
                "Aggressive Mammoth",
                "Overrun",
            ],
        ),
    ];
    let mut total_decisions = 0;
    let mut triggered = false;
    let mut token = false;
    let mut combat = false;
    let mut branches = 0;
    for pair in 0..decks.len() {
        for seed in [7, 41] {
            for starter in [Player::P0, Player::P1] {
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
                    triggered |= obs.stack.iter().any(|item| item.kind != StackKind::Spell);
                    token |= obs.objects.iter().any(|o| o.token.is_some());
                    combat |= !obs.attackers.is_empty();
                    if decisions % 47 == 0 {
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
                    decisions += 1;
                }
                assert!(decisions > 50, "game ended before meaningful play");
                total_decisions += decisions;
            }
        }
    }
    assert!(triggered && token && combat);
    assert!(branches >= 20);
    println!(
        "FDN initial matrix: 20 natural full games, {total_decisions} decisions, {branches} owned replay checks; 45 nonland card identities plus basics, not full-set acceptance"
    );
}
