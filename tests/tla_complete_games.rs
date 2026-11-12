//! Standard games and ownership checks using only Avatar release identities.
use rustmage::{
    Action, Card, CastMethod, Config, DeckRules, Error, Game, Observation, Outcome, Player,
    StackKind, policy,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "support/tmt_evidence.rs"]
mod evidence;

const PLAYERS: [Player; 2] = [Player::P0, Player::P1];
const ALL_BENDING_ELEMENTS: u8 = 0b1111;

#[derive(Deserialize)]
struct Identity {
    name: String,
    oracle_id: String,
}

impl Identity {
    fn card(&self) -> Card {
        let card = Card::from_name(&self.name).unwrap_or_else(|| panic!("unknown {}", self.name));
        assert_eq!(card.info().oracle_id, Some(self.oracle_id.as_str()));
        card
    }
}

#[derive(Deserialize)]
struct Set {
    card_count: usize,
    cards: Vec<Identity>,
    unresolved: Vec<String>,
}

fn pool() -> BTreeSet<Card> {
    let set: Set = serde_json::from_str(include_str!("../data/sets/tla.json")).unwrap();
    assert_eq!(set.card_count, 286);
    assert!(set.unresolved.is_empty());
    let cards: BTreeSet<_> = set.cards.iter().map(Identity::card).collect();
    assert_eq!(cards.len(), set.card_count);
    cards
}

fn standard(decks: [Vec<Card>; 2], seed: u64, starting_player: Player) -> Config {
    Config {
        decks,
        seed,
        starting_player,
        deck_rules: DeckRules::Standard,
    }
}

#[test]
fn avatar_supports_every_physical_identity_and_preserves_banned_legality() {
    let banned = ["Badgermole Cub", "Gran-Gran"].map(|name| Card::from_name(name).unwrap());
    let mut backs = BTreeSet::new();
    let cards = pool();
    assert!(banned.iter().all(|card| cards.contains(card)));
    for card in cards {
        assert!(card.is_supported(), "{} is gated", card.name());
        let legal = !banned.contains(&card);
        assert_eq!(card.is_standard_legal(), legal, "{}", card.name());
        let mut config = standard(
            [vec![Card::Forest; 60], vec![Card::Forest; 60]],
            81,
            Player::P0,
        );
        config.decks[0][0] = card;
        if !legal {
            assert!(matches!(
                Game::new(config.clone()),
                Err(Error::InvalidDeck(_))
            ));
            config.deck_rules = DeckRules::Freeform;
        }
        assert!(Game::new(config.clone()).is_ok(), "{}", card.name());
        if let Some(back) = card.back_face() {
            assert!(back.is_supported());
            assert!(backs.insert(back));
            assert_eq!(back.front_face(), card);
        }
    }
    assert_eq!(backs.len(), 8);
    for card in backs
        .into_iter()
        .chain((5705..=5710).map(Card::from_catalogue_id))
    {
        assert!(card.is_supported() && !card.is_standard_legal());
        let mut config = standard(
            [vec![Card::Forest; 60], vec![Card::Forest; 60]],
            81,
            Player::P0,
        );
        config.decks[0][0] = card;
        for rules in [DeckRules::Standard, DeckRules::Freeform] {
            config.deck_rules = rules;
            assert!(matches!(
                Game::new(config.clone()),
                Err(Error::InvalidDeck(_))
            ));
        }
    }
}

#[derive(Deserialize)]
struct DeckCard {
    #[serde(flatten)]
    identity: Identity,
    count: usize,
}

#[derive(Deserialize)]
struct Deck {
    id: String,
    name: String,
    cards: Vec<DeckCard>,
}

#[derive(Deserialize)]
struct Run {
    seed: u64,
    starting_player: usize,
}

#[derive(Deserialize)]
struct Pairing {
    p0: String,
    p1: String,
    runs: Vec<Run>,
}

#[derive(Deserialize)]
struct Workload {
    decks: Vec<Deck>,
    pairings: Vec<Pairing>,
}

fn deck_cards(deck: &Deck, pool: &BTreeSet<Card>) -> Vec<Card> {
    let mut seen = BTreeSet::new();
    let mut cards = Vec::new();
    for row in &deck.cards {
        let card = row.identity.card();
        assert!(
            pool.contains(&card) && card.is_standard_legal(),
            "{} in {}",
            card.name(),
            deck.name
        );
        assert!(seen.insert(card) && row.count > 0);
        assert!(row.count <= 4 || card.is_basic_land());
        cards.extend(std::iter::repeat_n(card, row.count));
    }
    assert_eq!(cards.len(), 60, "{}", deck.name);
    assert_eq!(cards.iter().filter(|card| card.is_land()).count(), 24);
    cards
}

fn prove_owned_branch(game: &Game, view: &Observation, chosen: usize) -> bool {
    let Some(alternate) = view.actions.iter().enumerate().find_map(|(index, action)| {
        (index != chosen && *action != Action::Concede).then_some(index)
    }) else {
        return false;
    };
    let checkpoint = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    raw.apply(view.viewer, view.actions[chosen].clone())
        .unwrap();
    indexed
        .apply_index(view.viewer, view.revision, chosen)
        .unwrap();
    let successor = indexed.snapshot();
    assert_eq!(raw.snapshot(), successor);
    assert_eq!(
        indexed.apply_index(view.viewer, view.revision, chosen),
        Err(Error::StaleRevision)
    );
    assert_eq!(indexed.snapshot(), successor);
    indexed.restore(&checkpoint).unwrap();
    indexed
        .apply_index(view.viewer, indexed.revision(), alternate)
        .unwrap();
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), checkpoint);
    assert_eq!(game.snapshot(), checkpoint);
    true
}

fn workload_index(view: &Observation) -> usize {
    let smoke = policy::smoke_index_from_observation(view, &view.actions);
    if matches!(view.actions[smoke], Action::PlayLand { .. }) {
        return smoke;
    }
    view.actions
        .iter()
        .position(|action| {
            matches!(
                action,
                Action::AnnounceForetell { .. }
                    | Action::AnnounceForetold { .. }
                    | Action::AnnounceAirbend { .. }
                    | Action::AnnounceLinkedWaterbend { .. }
            )
        })
        .unwrap_or(smoke)
}

#[test]
fn avatar_only_standard_games_finish_and_replay_with_all_four_bends() {
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/tla-set-only.json")).unwrap();
    let pool = pool();
    let decks: BTreeMap<_, _> = workload
        .decks
        .iter()
        .map(|deck| (deck.id.as_str(), deck_cards(deck, &pool)))
        .collect();
    assert_eq!(decks.len(), 6);
    assert_eq!(workload.pairings.len(), decks.len());
    let directory = std::env::var_os("RUSTMAGE_TLA_EVIDENCE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").map(|dir| PathBuf::from(dir).join("tla"))
        });
    if let Some(dir) = &directory {
        std::fs::create_dir_all(dir).unwrap();
    }
    let mut games = 0;
    let mut decisions = 0;
    let mut private_checks = 0;
    let mut wins = [0; 2];
    let mut draws = 0;
    let mut branches = [0; 2];
    let mut bends = 0;
    let mut spell_colors = 0;
    let mut foretold_casts = 0;
    let mut casts = BTreeSet::new();
    let mut gameplay = Duration::ZERO;
    let mut replay = Duration::ZERO;
    for pairing in &workload.pairings {
        assert_ne!(pairing.p0, pairing.p1);
        assert_eq!(pairing.runs.len(), 2);
        assert_eq!(
            [
                pairing.runs[0].starting_player,
                pairing.runs[1].starting_player
            ],
            [0, 1]
        );
        for run in &pairing.runs {
            let label = format!("{}-{}-{}", pairing.p0, pairing.p1, run.seed);
            let config = standard(
                [
                    decks[pairing.p0.as_str()].clone(),
                    decks[pairing.p1.as_str()].clone(),
                ],
                run.seed,
                PLAYERS[run.starting_player],
            );
            let mut game = Game::new(config.clone()).unwrap();
            let mut journal = Vec::new();
            let mut branched = [false; 2];
            let started = Instant::now();
            while let Some(actor) = game.actor() {
                assert!(journal.len() < 4096, "{label} exceeded its decision limit");
                let view = game.observe(actor);
                evidence::check_privacy(&view, actor);
                let index = workload_index(&view);
                let action = view.actions[index].clone();
                assert_ne!(action, Action::Concede);
                if !branched[actor.index()] && prove_owned_branch(&game, &view, index) {
                    branches[actor.index()] += 1;
                    branched[actor.index()] = true;
                }
                if evidence::private_choice(&view)
                    || view.plot_payment.is_some()
                    || !view.foretold_cards.is_empty()
                {
                    let other = game.observe(actor.other());
                    evidence::check_privacy(&other, actor);
                    assert!(other.plot_payment.is_none());
                    assert!(
                        other
                            .foretold_cards
                            .iter()
                            .all(|row| row.owner != actor || row.object.is_none())
                    );
                    private_checks += 1;
                }
                game.apply_index(actor, view.revision, index).unwrap();
                let after = game.observe(actor);
                if let Some(facts) = after.turn_facts {
                    bends |= facts[0].bent_elements | facts[1].bent_elements;
                }
                for packet in &after.stack {
                    if packet.kind != StackKind::Spell
                        || packet.is_copy
                        || view.stack.iter().any(|previous| previous.id == packet.id)
                    {
                        continue;
                    }
                    if let Some((_, card)) = packet.source_identity() {
                        casts.insert(card.name());
                        spell_colors |= card.colors();
                    }
                    if packet
                        .cast
                        .is_some_and(|cast| cast.method == CastMethod::Foretell)
                    {
                        foretold_casts += 1;
                    }
                }
                journal.push((actor, action));
            }
            gameplay += started.elapsed();
            let outcome = game.outcome().expect("complete game");
            match outcome {
                Outcome::Winner(player) => wins[player.index()] += 1,
                Outcome::Draw => draws += 1,
            }
            let started = Instant::now();
            let mut replayed = Game::new(config).unwrap();
            for (actor, action) in &journal {
                replayed.apply(*actor, action.clone()).unwrap();
            }
            assert_eq!(replayed.outcome(), Some(outcome));
            assert_eq!(replayed.snapshot(), game.snapshot(), "{label}");
            replay += started.elapsed();
            if let Some(dir) = &directory {
                std::fs::write(
                    dir.join(format!("{label}.json")),
                    serde_json::to_vec(&json!({
                        "seed": run.seed,
                        "p0": pairing.p0,
                        "p1": pairing.p1,
                        "starting_player": run.starting_player,
                        "outcome": outcome,
                        "journal": journal,
                    }))
                    .unwrap(),
                )
                .unwrap();
            }
            games += 1;
            decisions += journal.len();
        }
    }
    assert_eq!(games, 12);
    assert_eq!(spell_colors, 0b11111);
    assert_eq!(bends, ALL_BENDING_ELEMENTS);
    assert!(foretold_casts > 0);
    assert!(private_checks > 0 && branches.into_iter().all(|n| n == games));
    let seconds = gameplay.as_secs_f64();
    let summary = json!({
        "workload": "TLA-only Standard, six decks, both starting seats",
        "games": games,
        "decisions": decisions,
        "wins": wins,
        "draws": draws,
        "owned_branches": branches,
        "private_observation_checks": private_checks,
        "bending_elements": bends,
        "spell_colors": spell_colors,
        "distinct_spells": casts.len(),
        "foretold_casts": foretold_casts,
        "gameplay_seconds": seconds,
        "games_per_second": games as f64 / seconds,
        "decisions_per_second": decisions as f64 / seconds,
        "replay_seconds": replay.as_secs_f64(),
    });
    eprintln!("{summary}");
    if let Some(dir) = &directory {
        std::fs::write(
            dir.join("summary.json"),
            serde_json::to_vec_pretty(&summary).unwrap(),
        )
        .unwrap();
    }
}
