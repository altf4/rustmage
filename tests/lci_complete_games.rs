//! Exact LCI Standard decks with real rules transitions and owned replay.
use rustmage::{
    Action, CATALOGUE_VERSION, Card, CardId, CastMethod, Config, DeckRules, Error, Game, Keywords,
    ObjectRef, ObjectView, Observation, Outcome, Player, SpellView, StackKind, Step, Zone, policy,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "support/tmt_evidence.rs"]
mod tmt_evidence;
use tmt_evidence::{check_privacy, private_choice};

const MAX_DECISIONS: usize = 4096;
const PERMANENT_KINDS: u16 = 1 | 2 | 16 | 32 | 64 | 256;

#[derive(Deserialize)]
struct Identity {
    name: String,
    oracle_id: String,
}

#[derive(Deserialize)]
struct FrozenSet {
    card_count: usize,
    cards: Vec<Identity>,
    unresolved: Vec<String>,
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

fn identified_card(row: &Identity) -> Card {
    let card = Card::from_name(&row.name).unwrap_or_else(|| panic!("unknown LCI {}", row.name));
    assert_eq!(
        card.info().oracle_id,
        Some(row.oracle_id.as_str()),
        "{}",
        row.name
    );
    card
}

fn frozen_pool() -> BTreeMap<Card, Identity> {
    let set: FrozenSet = serde_json::from_str(include_str!("../data/sets/lci.json")).unwrap();
    assert_eq!(set.card_count, 291);
    assert!(set.unresolved.is_empty());
    let pool = set
        .cards
        .into_iter()
        .map(|row| (identified_card(&row), row))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(pool.len(), set.card_count);
    pool
}

#[test]
fn lci_frozen_pool_supports_every_physical_identity_and_its_other_face() {
    let pool = frozen_pool();
    let gated: Vec<_> = pool
        .iter()
        .filter(|(card, _)| !card.is_supported())
        .map(|(_, row)| &row.name)
        .collect();
    assert!(gated.is_empty(), "gated LCI identities {gated:?}");
    let banned = Card::from_name("Abuelo's Awakening").unwrap();
    let mut face_count = 0;
    for (&card, row) in &pool {
        assert_eq!(card.is_standard_legal(), card != banned, "{}", row.name);
        if let Some((_, name)) = row.name.split_once(" // ") {
            let face = card
                .back_face()
                .or_else(|| card.adventure_spell())
                .unwrap_or_else(|| panic!("missing other face of {}", row.name));
            assert_eq!(face.name(), name);
            assert!(face.is_supported(), "gated face {name}");
            face_count += 1;
        }
    }
    assert_eq!(face_count, 36);

    let forest = Card::from_name("Forest").unwrap();
    let mut config = Config {
        decks: [vec![forest; 60], vec![forest; 60]],
        seed: 0,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    };
    config.decks[0][0] = banned;
    assert!(matches!(
        Game::new(config.clone()),
        Err(Error::InvalidDeck(_))
    ));
    config.deck_rules = DeckRules::Freeform;
    Game::new(config).unwrap();
}

fn deck_cards(deck: &Deck, pool: &BTreeMap<Card, Identity>) -> Vec<Card> {
    let mut seen = BTreeSet::new();
    let mut cards = Vec::new();
    for row in &deck.cards {
        let card = identified_card(&row.identity);
        assert_eq!(
            pool.get(&card).map(|identity| &identity.name),
            Some(&row.identity.name),
            "{} outside LCI",
            row.identity.name,
        );
        assert!(card.is_standard_legal() && card.is_supported());
        assert!(seen.insert(card) && row.count > 0);
        assert!(row.count <= 4 || card.is_basic_land());
        cards.extend(std::iter::repeat_n(card, row.count));
    }
    assert_eq!(cards.len(), 60, "{}", deck.id);
    assert_eq!(cards.iter().filter(|card| card.is_land()).count(), 24);
    cards
}

fn observations(game: &Game) -> Value {
    let mut views = [game.observe(Player::P0), game.observe(Player::P1)];
    for view in &mut views {
        view.revision = 0;
    }
    serde_json::to_value(views).unwrap()
}

fn prove_branch(game: &Game, view: &Observation, index: usize) -> bool {
    let actor = view.actor.unwrap();
    let chosen = &view.actions[index];
    let Some(alternative) = view
        .actions
        .iter()
        .find(|action| *action != chosen && !matches!(action, Action::Concede))
    else {
        return false;
    };
    let parent = game.clone();
    let saved = parent.snapshot();
    let revision = parent.revision();
    let (mut raw, mut indexed, mut sibling) = (parent.clone(), parent.clone(), parent.clone());
    drop(parent);

    assert_eq!(
        raw.apply(actor.other(), chosen.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(
        raw.apply(actor, Action::ChooseMode { mode: u16::MAX }),
        Err(Error::IllegalAction)
    );
    assert_eq!(raw.snapshot(), saved);
    assert_eq!(raw.revision(), revision);
    raw.apply(actor, chosen.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(observations(&raw), observations(&indexed));
    assert_eq!(sibling.snapshot(), saved);
    sibling.apply(actor, alternative.clone()).unwrap();
    assert_eq!(game.snapshot(), saved);
    if sibling.snapshot() == raw.snapshot() {
        return false;
    }
    sibling.restore(&saved).unwrap();
    assert_eq!(sibling.snapshot(), saved);
    assert_eq!(observations(&sibling), observations(game));
    assert_eq!(
        sibling.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    sibling
        .apply_index(actor, sibling.revision(), index)
        .unwrap();
    assert_eq!(sibling.snapshot(), raw.snapshot());
    assert_eq!(observations(&sibling), observations(&raw));
    true
}

fn object(view: &Observation, card: CardId) -> Option<&ObjectView> {
    view.objects.iter().find(|object| object.id == card)
}

fn exact(view: &Observation, reference: ObjectRef) -> Option<&ObjectView> {
    object(view, reference.card).filter(|object| object.reference == reference)
}

fn permanent_cards(view: &Observation, player: Player) -> usize {
    view.objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Graveyard
                && object.owner == player
                && !object.is_token()
                && object.kinds & PERMANENT_KINDS != 0
        })
        .count()
}

fn resolving(before: &Observation) -> Option<&SpellView> {
    before.resolving.as_ref().or_else(|| before.stack.last())
}

#[derive(Default)]
struct Coverage {
    mechanics: BTreeMap<&'static str, usize>,
    cast_names: BTreeSet<&'static str>,
    casts: usize,
    colors: u8,
    branches: usize,
    public_branches: usize,
    private_branches: usize,
    private_choices: usize,
    discover_hit: Option<ObjectRef>,
}

impl Coverage {
    fn record(&mut self, event: &'static str) {
        *self.mechanics.entry(event).or_default() += 1;
    }

    fn transition(&mut self, before: &Observation, after: &Observation, action: &Action) {
        // After the hit moves to the stack, the cascade frame still owns its
        // original exile reference until the free-cast continuation completes.
        let previous_hit = before
            .cascade
            .as_ref()
            .and_then(|cascade| cascade.discovered);
        if let Some(cascade) = &after.cascade
            && let Some(hit) = cascade.discovered
            && previous_hit != Some(hit)
        {
            let revealed = exact(after, hit).expect("new public Discover result");
            assert_eq!(revealed.zone, Zone::Exile);
            assert!(!revealed.card.is_land());
            assert!(revealed.card.fixed_mana_value() < cascade.maximum_mana_value);
            self.record("Discover hit");
            self.discover_hit = Some(hit);
        }
        for packet in &after.stack {
            if before.stack.iter().any(|old| old.id == packet.id) {
                continue;
            }
            let Some((reference, card)) = packet.source_identity() else {
                continue;
            };
            if packet.kind != StackKind::Spell || packet.is_copy {
                continue;
            }
            let Some(cast) = packet.cast else {
                continue;
            };
            self.casts += 1;
            self.cast_names.insert(card.name());
            self.colors |= card.colors();
            if cast.treasure_mana_spent {
                self.record("Treasure spent on a spell");
            }
            if cast.origin == Zone::Exile
                && cast.method == CastMethod::Free
                && self
                    .discover_hit
                    .is_some_and(|hit| hit.card == reference.card && hit != reference)
            {
                self.record("Discover cast from exile");
                self.discover_hit = None;
            }
        }
        if let Some(hit) = self.discover_hit
            && matches!(action, Action::FinishResolutionCasts)
            && object(after, hit.card)
                .is_some_and(|object| object.zone == Zone::Hand && object.reference != hit)
        {
            self.record("Discover to hand");
            self.discover_hit = None;
        }

        if let Some(event) = &after.last_explore
            && before
                .last_explore
                .as_ref()
                .is_none_or(|old| old.sequence != event.sequence)
        {
            if let Some(revealed) = event.revealed {
                if revealed.card.is_land() && event.destination == Some(Zone::Hand) {
                    self.record("Explore land to hand");
                }
                if !revealed.card.is_land()
                    && matches!(event.destination, Some(Zone::Library | Zone::Graveyard))
                    && event.counter_added
                {
                    self.record("Explore nonland and counter");
                }
            }
            if resolving(before).is_some_and(|packet| {
                matches!(packet.kind, StackKind::ActivatedAbility { .. })
                    && packet.source_identity().is_some_and(|(source, card)| {
                        card.has_subtype("Map") && exact(after, source).is_none()
                    })
                    && packet
                        .targets
                        .iter()
                        .any(|target| target.object == Some(event.explorer.object))
            }) {
                self.record("Map sacrificed to explore");
            }
        }

        for new in after
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Battlefield)
        {
            if new.is_token() && exact(before, new.reference).is_none() {
                if new.has_subtype("Map") {
                    self.record("Map created");
                }
                if new.has_subtype("Treasure") {
                    self.record("Treasure created");
                }
            }
            if new.face.as_ref().is_some_and(|face| face.back)
                && object(before, new.id)
                    .is_some_and(|old| old.zone == Zone::Exile && old.reference != new.reference)
                && new.controller == new.owner
                && resolving(before).is_some_and(|packet| {
                    packet
                        .source_identity()
                        .is_some_and(|(source, _)| source.card == new.id)
                        && packet.craft_materials.as_ref().is_some_and(|materials| {
                            materials.iter().any(|material| {
                                after.linked_exile.iter().any(|link| {
                                    link.source == new.reference
                                        && link.exiled == *material
                                        && exact(after, *material)
                                            .is_some_and(|card| card.zone == Zone::Exile)
                                })
                            })
                        })
                })
            {
                self.record("Craft returned transformed with its exact material");
            }
            if new.name == "Echo of Dusk"
                && permanent_cards(after, new.controller) >= 4
                && new.toughness >= 3
                && new.keywords.contains(Keywords::LIFELINK)
            {
                self.record("Live Descend 4");
            }
            if new.name == "Echo of Dusk"
                && permanent_cards(after, new.controller) < 4
                && !new.keywords.contains(Keywords::LIFELINK)
                && exact(before, new.reference).is_some_and(|old| {
                    old.keywords.contains(Keywords::LIFELINK) && old.toughness == new.toughness + 1
                })
            {
                self.record("Live Descend bonus lost");
            }
            if new.name == "Stalactite Stalker"
                && before.step == Step::End
                && after.players[new.controller.index()].descended_this_turn > 0
                && exact(before, new.reference)
                    .is_some_and(|old| old.counters.plus_one < new.counters.plus_one)
            {
                self.record("Historical Descend payoff");
            }
        }

        if before.turn == after.turn {
            for player in [Player::P0, Player::P1] {
                if after.players[player.index()].descended_this_turn
                    > before.players[player.index()].descended_this_turn
                    && after.graveyard_entries_this_turn.iter().any(|reference| {
                        !before.graveyard_entries_this_turn.contains(reference)
                            && exact(after, *reference).is_some_and(|object| {
                                object.owner == player
                                    && !object.is_token()
                                    && object.kinds & PERMANENT_KINDS != 0
                            })
                    })
                {
                    self.record("Permanent card descended");
                }
            }
        } else if before
            .players
            .iter()
            .any(|player| player.descended_this_turn > 0)
            && after
                .players
                .iter()
                .all(|player| player.descended_this_turn == 0)
        {
            self.record("Descend history reset");
        }
        if matches!(
            before.step,
            Step::Blockers | Step::FirstStrikeDamage | Step::CombatDamage
        ) && before.stack.is_empty()
            && before
                .players
                .iter()
                .zip(&after.players)
                .any(|(old, new)| new.life < old.life)
        {
            self.record("Combat life loss");
        }
    }

    fn merge(&mut self, other: Self) {
        for (event, count) in other.mechanics {
            *self.mechanics.entry(event).or_default() += count;
        }
        self.cast_names.extend(other.cast_names);
        self.casts += other.casts;
        self.colors |= other.colors;
        self.branches += other.branches;
        self.public_branches += other.public_branches;
        self.private_branches += other.private_branches;
        self.private_choices += other.private_choices;
    }
}

fn is_craft_choice(view: &Observation) -> bool {
    view.announcing.as_ref().is_some_and(|packet| {
        let StackKind::ActivatedAbility { ability } = packet.kind else {
            return false;
        };
        packet.source_identity().is_some_and(|(_, card)| {
            card.back_face().is_some()
                && card
                    .activations()
                    .get(usize::from(ability))
                    .is_some_and(|a| a.cost.exile_self)
        })
    })
}

fn preferred_action(view: &Observation, cover: &Coverage, prefer_hand: bool) -> Option<usize> {
    if prefer_hand
        && !cover.mechanics.contains_key("Discover to hand")
        && view
            .cascade
            .as_ref()
            .is_some_and(|cascade| cascade.discovered.is_some())
        && let Some(index) = view
            .actions
            .iter()
            .position(|action| *action == Action::FinishResolutionCasts)
    {
        return Some(index);
    }
    if !cover.mechanics.contains_key("Treasure spent on a spell")
        && let Some(spell) = view
            .announcing
            .as_ref()
            .filter(|packet| packet.kind == StackKind::Spell)
        && let Some((_, definition)) = spell.source_identity()
        && let Some(index) = view.actions.iter().position(|action| {
            matches!(action, Action::ActivateMana { card, production, .. }
            if object(view, *card).is_some_and(|object| object.has_subtype("Treasure"))
                && (definition.colors() == 0
                    || production[..5].iter().enumerate().any(|(color, &amount)| {
                        amount > 0 && definition.colors() & (1 << color) != 0
                    })))
        })
    {
        return Some(index);
    }
    if is_craft_choice(view)
        && let Some(index) = view.actions.iter().position(|action| {
            matches!(action, Action::ChooseAdditionalCost { object } if
                exact(view, *object).is_some_and(|card| card.zone == Zone::Graveyard))
        })
    {
        return Some(index);
    }
    view.actions.iter().position(|action| {
        let card_id = match *action {
            Action::AnnounceActivate { card, ability, .. }
            | Action::Activate { card, ability, .. }
            | Action::BeginActivate { card, ability, .. } => (card, ability),
            _ => return false,
        };
        object(view, card_id.0).is_some_and(|object| {
            (!cover.mechanics.contains_key("Map sacrificed to explore")
                && object.has_subtype("Map"))
                || (!cover
                    .mechanics
                    .contains_key("Craft returned transformed with its exact material")
                    && object.card.back_face().is_some()
                    && object
                        .card
                        .activations()
                        .get(usize::from(card_id.1))
                        .is_some_and(|a| a.cost.exile_self))
        })
    })
}

fn evidence(directory: Option<&Path>, name: &str, data: Value) {
    if let Some(directory) = directory {
        std::fs::write(
            directory.join(name),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn lci_standard_games_exercise_the_mechanics_and_replay_to_natural_outcomes() {
    let pool = frozen_pool();
    let workload: Workload =
        serde_json::from_str(include_str!("../data/decks/lci-set-only.json")).unwrap();
    let decks = workload
        .decks
        .iter()
        .map(|deck| {
            (
                deck.id.as_str(),
                (deck.name.as_str(), deck_cards(deck, &pool)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(decks.len(), 5);
    let directory =
        std::env::var_os("RUSTMAGE_TDM_EVIDENCE_DIR").map(|path| PathBuf::from(path).join("lci"));
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).unwrap();
    }
    let mut total = Coverage::default();
    let mut decisions = 0;
    let mut completed = 0;
    let mut winners = [0; 2];
    let mut draws = 0;
    let mut gameplay = Duration::ZERO;
    let mut replay_time = Duration::ZERO;

    for (pair_index, pair) in workload.pairings.iter().enumerate() {
        let (left_name, left) = &decks[pair.p0.as_str()];
        let (right_name, right) = &decks[pair.p1.as_str()];
        for run in &pair.runs {
            let label = format!("{pair_index}-{}-{}", run.seed, run.starting_player);
            let config = Config {
                decks: [left.clone(), right.clone()],
                seed: run.seed,
                starting_player: [Player::P0, Player::P1][run.starting_player],
                deck_rules: DeckRules::Standard,
            };
            let mut game = Game::new(config.clone()).unwrap();
            let mut journal = Vec::new();
            let mut cover = Coverage::default();
            let started = Instant::now();
            while game.outcome().is_none() && journal.len() < MAX_DECISIONS {
                let actor = game.actor().unwrap();
                let before = game.observe(actor);
                check_privacy(&before, actor);
                let index = preferred_action(&before, &cover, run.seed == 7).unwrap_or_else(|| {
                    policy::smoke_index_from_observation(&before, &before.actions)
                });
                let action = before.actions[index].clone();
                assert_ne!(action, Action::Concede);
                let private = private_choice(&before);
                if private {
                    check_privacy(&game.observe(actor.other()), actor);
                    cover.private_choices += 1;
                }
                let scry = before.library_choice.is_some();
                let public = before
                    .cascade
                    .as_ref()
                    .is_some_and(|cascade| cascade.discovered.is_some())
                    || is_craft_choice(&before);
                if (cover.branches == 0
                    || scry && cover.private_branches == 0
                    || public && cover.public_branches == 0)
                    && !matches!(action, Action::Keep | Action::Pass)
                    && prove_branch(&game, &before, index)
                {
                    cover.branches += 1;
                    cover.private_branches += usize::from(scry);
                    cover.public_branches += usize::from(public);
                }
                if let Err(error) = game.apply(actor, action.clone()) {
                    evidence(
                        directory.as_deref(),
                        &format!("{label}-failure.json"),
                        json!({
                            "config": config, "actions": journal, "actor": actor, "failing_action": action,
                            "error": format!("{error:?}"),
                        }),
                    );
                    panic!("{label} decision {} {action:?}: {error:?}", journal.len());
                }
                cover.transition(&before, &game.observe(actor), &action);
                journal.push((actor, action));
            }
            gameplay += started.elapsed();
            evidence(
                directory.as_deref(),
                &format!("{label}.json"),
                json!({
                    "catalogue": CATALOGUE_VERSION, "config": config, "decks": [left_name, right_name],
                    "actions": journal, "outcome": game.outcome(), "spell_colors": cover.colors,
                    "actual_casts": cover.casts, "cast_names": cover.cast_names, "mechanics": cover.mechanics,
                    "owned_branches": cover.branches, "owned_public_branches": cover.public_branches,
                    "owned_private_branches": cover.private_branches,
                }),
            );
            assert!(
                game.outcome().is_some() && cover.casts > 0 && cover.branches > 0,
                "{label} incomplete after {} decisions",
                journal.len()
            );

            let started = Instant::now();
            let mut replay = Game::new(config).unwrap();
            for (actor, action) in &journal {
                replay.apply(*actor, action.clone()).unwrap();
            }
            assert_eq!(game.snapshot(), replay.snapshot(), "{label}");
            assert_eq!(game.outcome(), replay.outcome(), "{label}");
            assert_eq!(observations(&game), observations(&replay), "{label}");
            replay_time += started.elapsed();

            match game.outcome().unwrap() {
                Outcome::Winner(player) => winners[player.index()] += 1,
                Outcome::Draw => draws += 1,
            }
            completed += 1;
            decisions += journal.len();
            total.merge(cover);
        }
    }
    let summary = json!({
        "catalogue": CATALOGUE_VERSION,
        "completed_games": completed, "decisions": decisions, "winners": winners, "draws": draws,
        "spell_colors": total.colors, "actual_casts": total.casts, "cast_names": total.cast_names,
        "mechanics": total.mechanics, "owned_branches": total.branches,
        "owned_public_branches": total.public_branches,
        "owned_private_branches": total.private_branches, "private_choices": total.private_choices,
        "gameplay_checked_decisions_per_second": decisions as f64 / gameplay.as_secs_f64(),
        "replay_checked_decisions_per_second": decisions as f64 / replay_time.as_secs_f64(),
    });
    evidence(directory.as_deref(), "summary.json", summary.clone());
    println!("{summary}");
    assert_eq!(completed, 15);
    assert!(winners.iter().sum::<usize>() > 0);
    assert_eq!(total.colors & 31, 31);
    assert!(total.private_branches > 0 && total.public_branches > 0);
    for event in [
        "Explore land to hand",
        "Explore nonland and counter",
        "Discover hit",
        "Discover cast from exile",
        "Discover to hand",
        "Craft returned transformed with its exact material",
        "Permanent card descended",
        "Historical Descend payoff",
        "Descend history reset",
        "Live Descend 4",
        "Map created",
        "Map sacrificed to explore",
        "Treasure created",
        "Treasure spent on a spell",
        "Combat life loss",
    ] {
        assert!(total.mechanics.contains_key(event), "games missed {event}");
    }
}
