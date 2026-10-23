//! Natural TMT games with explicit mechanic execution and owned replay evidence.
//! Card-level obligations are verified separately by the complete set rules suite.
use rustmage::{Action, Card, Config, DeckRules, Error, Game, Observation, Outcome, Player, Zone};
use std::collections::BTreeSet;
use std::time::Instant;
#[path = "support/tmt_evidence.rs"]
mod tmt_evidence;
use tmt_evidence::Evidence;
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
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/tmt.json")).unwrap();
    let members = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            Card::from_name(row["name"].as_str().unwrap())
                .expect("frozen TMT identity must be known")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(members.len(), 195, "frozen physical membership changed");
    for card in members {
        assert!(
            card.is_supported(),
            "full-set membership still gates {}",
            card.identity()
        );
    }
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
    assert_eq!(expected_casts.len(), 45);
    let mut evidence = Evidence::default();
    let mut controls = BTreeSet::new();
    for pair in 0..decks.len() {
        for seed in [0, 7, 41, 73] {
            let config = || Config {
                decks: [decks[pair].clone(), decks[(pair + 1) % decks.len()].clone()],
                seed,
                starting_player: if seed == 0 { Player::P0 } else { Player::P1 },
                deck_rules: DeckRules::Standard,
            };
            let mut game = Game::new(config()).unwrap();
            let mut journal = Vec::new();
            let mut decisions = 0usize;
            while game.outcome().is_none() {
                assert!(
                    decisions < 25000,
                    "TMT workload stalled pair={pair} seed={seed}"
                );
                let actor = game.actor().unwrap();
                let actions = game.legal_actions();
                assert!(!actions.is_empty());
                let obs = game.observe(actor);
                assert_eq!(obs.actions, actions);
                let index = rustmage::policy::smoke_index_from_observation(&obs, &actions);
                let action = actions[index].clone();
                assert!(!matches!(action, Action::Concede));
                for viewer in [Player::P0, Player::P1] {
                    let o = game.observe(viewer);
                    assert!(o.objects.iter().all(|object| object.zone != Zone::Hand
                        || object.owner == viewer
                        || o.known_hand.contains(&object.reference)));
                    assert!(
                        o.outside_game_cards
                            .iter()
                            .all(|object| object.owner == viewer)
                    );
                    assert!(
                        o.objects
                            .iter()
                            .all(|object| object.zone != Zone::Outside || object.owner == viewer)
                    );
                    if viewer != actor {
                        assert!(o.actions.is_empty());
                        assert!(o.library_choice.is_none());
                        assert!(o.search_choice.is_none());
                        assert!(o.connive_choice.is_none());
                        assert!(
                            o.entry_value
                                .as_ref()
                                .is_none_or(|value| value.opponent_hand.is_none())
                        );
                        assert!(
                            o.effect_choice
                                .as_ref()
                                .and_then(|choice| choice.hand_reveal.as_ref())
                                .is_none_or(|reveal| reveal.revealed)
                        );
                        if let Some(search) = &o.legendary_split_search {
                            assert!(search.revealed);
                            assert!(search.library.is_empty() && search.outside.is_empty());
                        }
                    }
                }
                let hidden = private_choice(&obs);
                evidence.private += usize::from(hidden);
                if decisions.is_multiple_of(64) || hidden {
                    let before = game.snapshot();
                    let revision = game.revision();
                    assert_eq!(
                        game.apply(actor.other(), action.clone()),
                        Err(Error::WrongActor)
                    );
                    assert_eq!(game.snapshot(), before);
                    let invalid = Action::ChooseMode { mode: u16::MAX };
                    assert!(!actions.contains(&invalid));
                    assert_eq!(game.apply(actor, invalid), Err(Error::IllegalAction));
                    assert_eq!(game.snapshot(), before);
                    assert_eq!(game.revision(), revision);
                    let (mut raw_child, mut indexed_child) = {
                        let parent = game.clone();
                        (parent.clone(), parent.clone())
                    };
                    raw_child.apply(actor, action.clone()).unwrap();
                    indexed_child.apply_index(actor, revision, index).unwrap();
                    game.apply(actor, action.clone()).unwrap();
                    assert_eq!(raw_child.snapshot(), game.snapshot());
                    assert_eq!(indexed_child.snapshot(), game.snapshot());
                    for viewer in [Player::P0, Player::P1] {
                        assert_eq!(
                            serde_json::to_value(raw_child.observe(viewer)).unwrap(),
                            serde_json::to_value(game.observe(viewer)).unwrap()
                        );
                        assert_eq!(
                            serde_json::to_value(indexed_child.observe(viewer)).unwrap(),
                            serde_json::to_value(game.observe(viewer)).unwrap()
                        );
                    }
                    let after = game.snapshot();
                    game.restore(&before).unwrap();
                    assert_eq!(
                        game.apply_index(actor, revision, index),
                        Err(Error::StaleRevision)
                    );
                    game.apply(actor, action.clone()).unwrap();
                    assert_eq!(game.snapshot(), after);
                    evidence.branches += 1;
                } else {
                    game.apply(actor, action.clone()).unwrap();
                }
                let after = game.observe(actor);
                check_negative_controls(&obs, &after, &action, &mut controls);
                evidence.transition(&obs, &after, &action);
                journal.push((actor, action));
                decisions += 1;
            }
            assert!(decisions > 50);
            let mut replay = Game::new(config()).unwrap();
            for (actor, action) in journal {
                replay.apply(actor, action).unwrap();
                evidence.replayed += 1;
            }
            assert_eq!(replay.snapshot(), game.snapshot());
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    semantic_observation(&replay, viewer),
                    semantic_observation(&game, viewer)
                );
            }
            match game.outcome().unwrap() {
                Outcome::Winner(Player::P0) => evidence.wins[0] += 1,
                Outcome::Winner(Player::P1) => evidence.wins[1] += 1,
                Outcome::Draw => evidence.draws += 1,
            }
            evidence.drawn_to_empty += usize::from(
                game.observe(Player::P0)
                    .players
                    .iter()
                    .any(|p| p.library_count == 0),
            );
            evidence.decisions += decisions;
            evidence.games += 1;
        }
    }
    assert_eq!(evidence.games, 20);
    assert_eq!(evidence.wins.iter().sum::<usize>() + evidence.draws, 20);
    assert_eq!(evidence.replayed, evidence.decisions);
    assert!(evidence.branches > 0 && evidence.private > 0 && evidence.combat > 0);
    for mechanic in [
        "sneak casts committed",
        "class levels gained",
        "disappear counter abilities resolved",
        "alliance damage abilities resolved",
        "tokens entered",
        "plus counters placed",
        "triggered ability packets consumed",
        "activated ability packets consumed",
    ] {
        assert!(
            evidence.executed.get(mechanic).is_some_and(|&n| n > 0),
            "mechanic not executed: {mechanic}"
        );
    }
    assert_eq!(
        controls.len(),
        2,
        "both actual positive transitions must exercise their negative controls"
    );
    assert!(
        expected_casts.is_subset(&evidence.casts),
        "selected identities never committed: {:?}",
        expected_casts
            .difference(&evidence.casts)
            .collect::<Vec<_>>()
    );
    println!(
        "TMT_GAMES games={} decisions={} owned_branches={} private_decisions={} replayed_actions={} wins={:?} draws={} end_states_with_empty_library={} seconds={:.6} decisions_per_second={:.3} casts={:?} triggered={:?} executed={:?} negative_control_families={}",
        evidence.games,
        evidence.decisions,
        evidence.branches,
        evidence.private,
        evidence.replayed,
        evidence.wins,
        evidence.draws,
        evidence.drawn_to_empty,
        start.elapsed().as_secs_f64(),
        evidence.decisions as f64 / start.elapsed().as_secs_f64(),
        evidence.casts,
        evidence.triggers,
        evidence.executed,
        controls.len()
    );
}

fn semantic_observation(game: &Game, viewer: Player) -> Vec<u8> {
    let mut view = game.observe(viewer);
    view.revision = 0;
    serde_json::to_vec(&view).unwrap()
}
fn private_choice(observation: &Observation) -> bool {
    observation
        .legendary_split_search
        .as_ref()
        .is_some_and(|c| !c.revealed)
        || observation.library_choice.is_some()
        || observation.search_choice.is_some()
        || observation.connive_choice.is_some()
        || observation
            .entry_value
            .as_ref()
            .is_some_and(|v| v.opponent_hand.is_some())
        || observation.effect_choice.as_ref().is_some_and(|choice| {
            matches!(
                choice.kind,
                rustmage::EffectChoiceKind::Discard | rustmage::EffectChoiceKind::OutsideGameCard
            ) || choice
                .hand_reveal
                .as_ref()
                .is_some_and(|view| !view.revealed)
        })
}

// Counterfactual controls start from genuinely played positive transitions. They
// change observations only; they neither invent another bot nor alter gameplay.
fn check_negative_controls(
    before: &Observation,
    after: &Observation,
    action: &Action,
    checked: &mut BTreeSet<&'static str>,
) {
    if checked.len() == 2 {
        return;
    }
    let finishing = before.resolving.as_ref().or_else(|| {
        matches!(action, Action::Pass)
            .then(|| before.stack.last())
            .flatten()
    });
    if !finishing.is_some_and(|packet| {
        matches!(
            packet.source_card.name(),
            "Michelangelo, Game Master" | "Raphael, Tough Turtle"
        )
    }) {
        return;
    }
    let mut actual = Evidence::default();
    actual.transition(before, after, action);
    for key in [
        "disappear counter abilities resolved",
        "alliance damage abilities resolved",
    ] {
        if checked.contains(key) || !actual.executed.contains_key(key) {
            continue;
        }
        assert_eq!(actual.executed.get(key), Some(&1));
        let mut ineffective = after.clone();
        if key == "disappear counter abilities resolved" {
            let packet = before
                .resolving
                .as_ref()
                .unwrap_or_else(|| before.stack.last().unwrap());
            let source = ineffective
                .objects
                .iter_mut()
                .find(|o| o.reference == packet.object)
                .unwrap();
            source.counters = before
                .objects
                .iter()
                .find(|o| o.reference == packet.object)
                .unwrap()
                .counters
                .clone();
            // A real consumed Michelangelo packet plus a counter on an unrelated
            // object must not count as its self-counter effect.
            let unrelated = ineffective
                .objects
                .iter_mut()
                .find(|o| o.zone == Zone::Battlefield && o.reference != packet.object)
                .unwrap();
            unrelated.counters.plus_one += 1;
        } else {
            let packet = before
                .resolving
                .as_ref()
                .unwrap_or_else(|| before.stack.last().unwrap());
            ineffective.players[packet.controller.other().index()].life =
                before.players[packet.controller.other().index()].life;
            // Damage to the controller is not Raphael's opponent-damage witness.
            ineffective.players[packet.controller.index()].life -= 1;
        }
        let mut rejected = Evidence::default();
        rejected.transition(before, &ineffective, action);
        assert!(
            !rejected.executed.contains_key(key),
            "ineffective consumed packet satisfied {key}"
        );
        assert_eq!(
            rejected.executed.get("triggered ability packets consumed"),
            Some(&1)
        );
        // The genuine effect delta without the corresponding consumed source
        // ability also cannot satisfy the witness.
        let mut wrong_source = before.clone();
        let packet = if let Some(packet) = wrong_source.resolving.as_mut() {
            packet
        } else {
            wrong_source.stack.last_mut().unwrap()
        };
        packet.source_card = Card::from_name("Forest").unwrap();
        let mut rejected = Evidence::default();
        rejected.transition(&wrong_source, after, action);
        assert!(
            !rejected.executed.contains_key(key),
            "unrelated consumed packet satisfied {key}"
        );
        checked.insert(key);
    }
}
