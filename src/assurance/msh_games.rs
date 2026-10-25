//! Legal full games for a bounded admitted MSH pool, not entire-set acceptance.
use super::*;
use std::collections::BTreeSet;
use std::time::Instant;

const DECKS: [(&str, [&str; 9]); 5] = [
    (
        "Plains",
        [
            "Agent Maria Hill",
            "Agent Phil Coulson",
            "Agent of Atlas",
            "Captain America, Wings of Freedom",
            "Crowd of True Believers",
            "Patriot, Shield Wielder",
            "S.H.I.E.L.D. Spy Kit",
            "Take Up the Shield",
            "Helicarrier Strike",
        ],
    ),
    (
        "Island",
        [
            "Aerial Doombot",
            "Hydraulic Helper",
            "Namor the Sub-Mariner",
            "Attuma, Atlantean Warlord",
            "Thirst for Knowledge",
            "I Am Iron Man",
            "The Vision",
            "S.H.I.E.L.D. Flying Car",
            "Super Suit",
        ],
    ),
    (
        "Swamp",
        [
            "Agents of HYDRA",
            "Baron Strucker, HYDRA Overlord",
            "HYDRA Troopers",
            "Ninja of the Hand",
            "Whiplash, Vengeful Engineer",
            "Stolen Stark Tech",
            "Too Evil to Stay Dead",
            "Dark Deed",
            "Arnim Zola, Bio-Fanatic",
        ],
    ),
    (
        "Mountain",
        [
            "Quicksilver, Brash Blur",
            "K'un-Lun Warrior",
            "Human Torch, Johnny Storm",
            "Jessica Jones, Private Eye",
            "Hawkeye's Bow",
            "Lightning Strike",
            "HULK SMASH!",
            "HYDRA Assault Robot",
            "Stark Industries Executive",
        ],
    ),
    (
        "Forest",
        [
            "Reptil, Dinomorpher",
            "Hulkling, Burgeoning Bruiser",
            "Serpent Specialist",
            "Guerrilla Gorilla",
            "Ant-Man's Army",
            "She-Hulk, Jade Defender",
            "Go Nuts!",
            "Giant Growth",
            "Undercover Skrull",
        ],
    ),
];

fn deck(index: usize) -> Vec<Card> {
    let (basic, spells) = DECKS[index];
    let mut cards = vec![Card::from_name(basic).unwrap(); 24];
    for name in spells {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported() && card.is_standard_legal(), "{name}");
        cards.extend([card; 4]);
    }
    cards
}

// Neither the policy nor these checks receives an opponent's hidden identities.
// The counterfactual changes only unobserved cards and below-top library order.
fn hidden_information_contract(game: &Game, viewer: Player) {
    let observation = serde_json::to_value(game.observe(viewer)).unwrap();
    let mut alternative = game.clone();
    let hidden: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let id = i as CardId;
            (o.owner == viewer.other()
                && o.zone == Zone::Hand
                && !game.state.known_hand.contains(&game.object_ref(id)))
            .then_some(i)
        })
        .take(2)
        .collect();
    if hidden.len() == 2 {
        let first = alternative.state.objects[hidden[0]].card;
        alternative.state.objects[hidden[0]].card = alternative.state.objects[hidden[1]].card;
        alternative.state.objects[hidden[1]].card = first;
    }
    let library = &mut alternative.state.players[viewer.other().index()].library;
    let below_top = library.len().saturating_sub(1);
    library[..below_top].reverse();
    assert_eq!(
        observation,
        serde_json::to_value(alternative.observe(viewer)).unwrap()
    );
    assert_eq!(game.legal_actions(), alternative.legal_actions());
}

fn owned_checked_step(mut game: Game, actor: Player, index: usize, action: Action) -> Game {
    let revision = game.revision();
    let before = game.snapshot();
    assert_eq!(
        game.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(game.snapshot(), before);
    assert_eq!(
        game.apply_index(actor, revision, usize::MAX),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    let (mut raw, mut indexed) = (game.clone(), game.clone());
    drop(game);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(raw.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

#[test]
fn msh_legal_five_color_decks_natural_games_replay_privacy_and_owned_search() {
    let membership: serde_json::Value =
        serde_json::from_str(include_str!("../../data/sets/msh.json")).unwrap();
    let members: BTreeSet<_> = membership["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    for (basic, spells) in DECKS {
        for name in std::iter::once(basic).chain(spells) {
            assert!(members.contains(name));
        }
    }
    let start = Instant::now();
    let mut games = Vec::new();
    let mut cast_names = BTreeSet::new();
    for first in 0..5 {
        for starting_player in [Player::P0, Player::P1] {
            for seed in [20261024, 20261025] {
                let config = Config {
                    decks: [deck(first), deck((first + 1) % 5)],
                    seed,
                    starting_player,
                    deck_rules: DeckRules::Standard,
                };
                let mut game = Game::new(config.clone()).unwrap();
                let mut ledger = Vec::new();
                let mut contracts = 0;
                while game.outcome().is_none() {
                    let actor = game.actor().expect("unfinished game has an actor");
                    let actions = game.legal_actions();
                    let index = crate::policy::smoke_index(&game, &actions);
                    let action = actions[index].clone();
                    assert!(!matches!(action, Action::Concede));
                    if matches!(action, Action::PayCosts { .. })
                        && let Some(announcement) = &game.state.announcing
                        && announcement.item.kind == StackKind::Spell
                    {
                        cast_names.insert(announcement.item.source_card.name());
                    }
                    ledger.push((actor, action.clone()));
                    if ledger.len() % 97 == 1 {
                        hidden_information_contract(&game, actor);
                        game = owned_checked_step(game, actor, index, action);
                        contracts += 1;
                    } else {
                        game.apply(actor, action).unwrap();
                    }
                    assert!(
                        ledger.len() < 20_000,
                        "bounded policy did not reach a natural outcome, deck{first}, seed{seed}, starter{starting_player:?}, pending{:?}",
                        game.state.pending
                    );
                }
                let mut replay = Game::new(config).unwrap();
                for (actor, action) in &ledger {
                    replay.apply(*actor, action.clone()).unwrap();
                }
                assert_eq!(game.snapshot(), replay.snapshot());
                assert_eq!(game.outcome(), replay.outcome());
                assert!(contracts > 1);
                games.push(serde_json::json!({"decks":[DECKS[first].0,DECKS[(first+1)%5].0],"starting_player":starting_player,"seed":seed,"decisions":ledger.len(),"outcome":game.outcome(),"turn":game.state.turn,"owned_privacy_checkpoints":contracts}));
            }
        }
    }
    assert_eq!(games.len(), 20);
    // Explicit export is local cache only, never an execution input.
    if let Ok(output) = std::env::var("RUSTMAGE_RECORD_MSH_GAMES") {
        let output = std::path::PathBuf::from(output);
        assert!(
            output.starts_with(
                std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache")
            )
        );
        std::fs::write(output,serde_json::to_string_pretty(&serde_json::json!({"scope":"20 legal Standard-deck MSH-only games across50admitted identities; deterministic observation-only smoke policy. Bounded game/replay/privacy/owned-search evidence, not full-set acceptance or a simulation benchmark: wall time includes checks and replay.","decks":DECKS,"games":games,"cast_card_names_observed":cast_names,"checked_execution_seconds":start.elapsed().as_secs_f64()})).unwrap()+"\n").unwrap();
    }
}
