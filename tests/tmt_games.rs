//! Natural TMT games with explicit mechanic execution and owned replay evidence.
//! Card-level obligations are verified separately by the complete set rules suite.
use rustmage::{Action, Card, Config, DeckRules, Error, Game, Player, StackKind, Zone};
use std::collections::BTreeSet;
use std::time::Instant;
fn deck(land: &str, names: &[&str]) -> Vec<Card> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/tmt.json")).unwrap();
    let members: BTreeSet<_> = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    let mut result = vec![Card::from_name(land).unwrap(); 24];
    assert!(members.contains(land));
    for &name in names {
        assert!(members.contains(name), "not TMT: {name}");
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported(), "{name}");
        result.extend([card; 4]);
    }
    assert_eq!(result.len(), 60);
    result
}
#[test]
fn tmt_five_color_set_only_natural_games_and_owned_replay() {
    let decks = [
        deck(
            "Plains",
            &[
                "Action News Crew",
                "Featherbrained Filcher",
                "Leonardo, Leader in Blue",
                "Leonardo, Cutting Edge",
                "Leonardo, Big Brother",
                "Leader's Talent",
                "Foot Elite",
                "Prehistoric Pet",
                "Hamato Guardian Stance",
            ],
        ),
        deck(
            "Island",
            &[
                "April, Reporter of the Weird",
                "Buzz Bots",
                "Crustacean Commando",
                "Does Machines",
                "Donatello's Technique",
                "Sewer-veillance Cam",
                "Ray Fillet, Man Ray",
                "Mind Transfer Protocol",
                "Metalhead",
            ],
        ),
        deck(
            "Swamp",
            &[
                "Squirrelanoids",
                "Dream Beavers",
                "Tunnel Rats",
                "Foot Ninjas",
                "Ninja Teen",
                "Death in the Family",
                "Super Shredder",
                "Splinter, Hamato Yoshi",
                "Pain 101",
            ],
        ),
        deck(
            "Mountain",
            &[
                "Casey Jones, Jury-Rig Justiciar",
                "Mouser Foundry",
                "Mouser Attack!",
                "Mouser Mark III",
                "Manhole Missile",
                "Raphael, Tough Turtle",
                "Ravenous Robots",
                "Purple Dragon Punks",
                "Spicy Oatmeal Pizza",
            ],
        ),
        deck(
            "Forest",
            &[
                "Frog Butler",
                "Michelangelo, Game Master",
                "Party Dude",
                "Zoo Escapees",
                "Tenderize",
                "Michelangelo, Weirdness to 11",
                "Michelangelo, Improviser",
                "Mona Lisa, Science Geek",
                "Mutagen Man, Living Ooze",
            ],
        ),
    ];
    let expected_casts = decks
        .iter()
        .flatten()
        .filter(|c| !c.is_land())
        .map(|c| c.name())
        .collect::<BTreeSet<_>>();
    let start = Instant::now();
    let mut total = 0usize;
    let mut branches = 0usize;
    let mut sneak_casts = 0usize;
    let mut classes = 0usize;
    let mut counters = 0usize;
    let mut departure_triggers = 0usize;
    let mut private = 0usize;
    let mut terminal = 0usize;
    let mut casts = BTreeSet::new();
    let mut triggers = BTreeSet::new();
    let mut combat = false;
    for pair in 0..decks.len() {
        for seed in [0, 7, 41, 73] {
            let mut game = Game::new(Config {
                decks: [decks[pair].clone(), decks[(pair + 1) % decks.len()].clone()],
                seed,
                starting_player: if seed == 0 { Player::P0 } else { Player::P1 },
                deck_rules: DeckRules::Standard,
            })
            .unwrap();
            let mut decisions = 0usize;
            while game.outcome().is_none() {
                assert!(
                    decisions < 25000,
                    "TMT workload stalled pair={pair} seed={seed}"
                );
                let actor = game.actor().unwrap();
                let actions = game.legal_actions();
                assert!(!actions.is_empty());
                let index = rustmage::policy::smoke_index(&game, &actions);
                let action = actions[index].clone();
                assert!(!matches!(action, Action::Concede));
                let obs = game.observe(actor);
                for viewer in [Player::P0, Player::P1] {
                    let o = game.observe(viewer);
                    assert!(o.objects.iter().all(|object| object.zone != Zone::Hand
                        || object.owner == viewer
                        || o.known_hand.contains(&object.reference)));
                }
                for item in &obs.stack {
                    if item.kind == StackKind::Spell {
                        casts.insert(item.source_card.name());
                    } else {
                        triggers.insert(item.source_card.name());
                    }
                }
                sneak_casts += usize::from(matches!(action, Action::AnnounceSneak { .. }));
                classes += usize::from(obs.class_levels.iter().any(|row| row.level >= 2));
                counters += usize::from(obs.objects.iter().any(|o| o.counters.plus_one > 0));
                departure_triggers += usize::from(
                    obs.players
                        .iter()
                        .any(|p| p.permanent_left_battlefield_this_turn)
                        && obs.stack.iter().any(|s| s.kind != StackKind::Spell),
                );
                combat |= !obs.attackers.is_empty();
                let hidden = obs.library_choice.is_some()
                    || obs
                        .effect_choice
                        .as_ref()
                        .is_some_and(|c| c.kind == rustmage::EffectChoiceKind::Discard);
                if hidden {
                    private += 1;
                    assert!(game.observe(actor.other()).library_choice.is_none());
                }
                if decisions.is_multiple_of(64) || hidden {
                    let before = game.snapshot();
                    let revision = game.revision();
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
            assert!(decisions > 50);
            total += decisions;
            terminal += 1;
        }
    }
    assert!(
        combat
            && sneak_casts > 0
            && classes > 0
            && counters > 0
            && departure_triggers > 0
            && private > 0
            && !triggers.is_empty()
    );
    assert!(
        expected_casts.is_subset(&casts),
        "selected identities never cast: {:?}",
        expected_casts.difference(&casts).collect::<Vec<_>>()
    );
    println!(
        "TMT_GAMES games={terminal} decisions={total} owned_branches={branches} sneak_casts={sneak_casts} class_observations={classes} counter_observations={counters} departure_trigger_observations={departure_triggers} private_decisions={private} seconds={:.6} decisions_per_second={:.3} casts={casts:?} triggered={triggers:?}",
        start.elapsed().as_secs_f64(),
        total as f64 / start.elapsed().as_secs_f64()
    );
}
