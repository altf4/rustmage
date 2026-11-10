//! Standard-legal EOE decks, terminal games, and public and private decision replay.
use rustmage::{
    Action, Card, CastMethod, Config, DeckRules, Error, Game, ObjectRef, Observation, Player,
    StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

const PAIRINGS: &[(usize, usize)] = &[
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 0),
    (4, 5),
    (4, 4),
    (5, 2),
];

type DeckList = (&'static [(&'static str, usize)], &'static [&'static str]);

const DECKS: &[DeckList] = &[
    (
        &[("Plains", 24)],
        &[
            "Dockworker Drone",
            "Honored Knight-Captain",
            "Honor",
            "Lumen-Class Frigate",
            "Starfighter Pilot",
            "Rayblade Trooper",
            "Starfield Shepherd",
            "Sunstar Chaplain",
            "Seam Rip",
        ],
    ),
    (
        &[("Island", 24)],
        &[
            "Illvoi Galeblade",
            "Mechan Navigator",
            "Synthesizer Labship",
            "Illvoi Operative",
            "Desculpting Blast",
            "Steelswarm Operator",
            "Quantum Riddler",
            "Starfield Vocalist",
            "Emissary Escort",
        ],
    ),
    (
        &[("Swamp", 24)],
        &[
            "Hullcarver",
            "Timeline Culler",
            "Tragic Trajectory",
            "Beamsaw Prospector",
            "Fell Gravship",
            "Lightless Evangel",
            "Faller's Faithful",
            "Entropic Battlecruiser",
            "Umbral Collar Zealot",
        ],
    ),
    (
        &[("Mountain", 20), ("Island", 4)],
        &[
            "Kavaron Harrier",
            "Weapons Manufacturing",
            "Rust Harvester",
            "Zookeeper Mechan",
            "Plasma Bolt",
            "Possibility Technician",
            "Roving Actuator",
            "Warmaker Gunship",
            "Pinnacle Emissary",
        ],
    ),
    (
        &[("Forest", 26)],
        &[
            "Gene Pollinator",
            "Galactic Wayfarer",
            "Sami's Curiosity",
            "Bioengineered Future",
            "Famished Worldsire",
            "Sledge-Class Seedship",
            "Mightform Harmonizer",
            "Eusocial Engineering",
            "Terrasymbiosis",
        ],
    ),
    (
        &[("Plains", 10), ("Mountain", 10), ("Sacred Foundry", 4)],
        &[
            "Sami, Ship's Engineer",
            "Ragost, Deft Gastronaut",
            "Dockworker Drone",
            "Rust Harvester",
            "Weapons Manufacturing",
            "Sunstar Chaplain",
            "Nutrient Block",
            "Lumen-Class Frigate",
            "Kavaron Harrier",
        ],
    ),
];

fn deck(members: &BTreeSet<Card>, lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let checked = |name| {
        let card = Card::from_name(name).unwrap_or_else(|| panic!("unknown EOE card {name}"));
        assert!(members.contains(&card), "outside EOE: {name}");
        card
    };
    let mut deck = Vec::new();
    for &(name, count) in lands {
        deck.extend(std::iter::repeat_n(checked(name), count));
    }
    for &name in spells {
        // The ramp deck trades two top-end slots for lands.
        let count = if name == "Famished Worldsire" { 2 } else { 4 };
        deck.extend(std::iter::repeat_n(checked(name), count));
    }
    assert_eq!(deck.len(), 60);
    deck
}

fn decision(observation: &Observation) -> usize {
    // Explore the optional routes unique to this set when the engine offers them.
    if observation
        .entry_devour
        .as_ref()
        .is_some_and(|choice| choice.selected.len() < 2)
        && let Some(index) = observation
            .actions
            .iter()
            .position(|action| matches!(action, Action::ChooseEntryDevour { .. }))
    {
        return index;
    }
    if let Some(index) = observation.actions.iter().position(|action| {
        matches!(
            action,
            Action::AnnounceWarp { .. } | Action::AnnounceGrantedWarp { .. }
        )
    }) {
        return index;
    }
    rustmage::policy::smoke_index_from_observation(observation, &observation.actions)
}

fn private(view: &Observation) -> bool {
    view.library_choice.is_some() || view.search_choice.is_some() || view.mana_payment.is_some()
}

#[derive(Default)]
struct Evidence {
    games: usize,
    decisions: usize,
    verified_branches: usize,
    private_choices: usize,
    casts: BTreeSet<&'static str>,
    warped_refs: BTreeSet<ObjectRef>,
    events: BTreeMap<&'static str, usize>,
}

impl Evidence {
    fn record(&mut self, before: &Observation, after: &Observation, action: &Action) {
        if matches!(
            action,
            Action::FinishEntryDevour | Action::ChooseEntryDevour { .. }
        ) && let Some(choice) = &before.entry_devour
            && !choice.selected.is_empty()
            && after.objects.iter().any(|o| {
                o.id == choice.entrant.card
                    && o.zone == Zone::Battlefield
                    && o.counters.get(rustmage::CounterKind::PlusOne) > 0
            })
        {
            *self.events.entry("Devour payment").or_default() += 1;
        }
        for item in &after.stack {
            if before.stack.iter().any(|previous| previous.id == item.id) {
                continue;
            }
            let Some((_, definition)) = item.source_identity() else {
                continue;
            };
            let Some(cast) = item.cast.filter(|_| item.kind == StackKind::Spell) else {
                continue;
            };
            self.casts.insert(definition.name());
            if cast.method == CastMethod::Warp {
                *self.events.entry("Warp cast").or_default() += 1;
            }
            if cast.origin == Zone::Exile
                && let Some((stack, _)) = item.source_identity()
                && self.warped_refs.iter().any(|exiled| {
                    exiled.card == stack.card
                        && exiled.generation.checked_add(1) == Some(stack.generation)
                })
            {
                *self.events.entry("Warp exile recast").or_default() += 1;
            }
            if before.void_active_this_turn
                && matches!(definition.name(), "Tragic Trajectory" | "Roving Actuator")
            {
                *self.events.entry("Void eligible cast").or_default() += 1;
            }
        }
        self.warped_refs
            .extend(after.warped_cards.iter().map(|warp| warp.object));
        for entrant in after.objects.iter().filter(|o| o.zone == Zone::Battlefield) {
            let previous = before
                .objects
                .iter()
                .find(|old| old.reference == entrant.reference && old.zone == Zone::Battlefield);
            if entrant.has_subtype("Spacecraft")
                && previous.is_some_and(|old| {
                    entrant.counters.get(rustmage::CounterKind::Charge)
                        > old.counters.get(rustmage::CounterKind::Charge)
                })
            {
                *self.events.entry("Station charge placed").or_default() += 1;
            }
            if previous.is_none()
                && let Some(token) = entrant.token
            {
                match token.name {
                    "Lander Token" => *self.events.entry("Lander entered").or_default() += 1,
                    "Robot Token" => *self.events.entry("Robot entered").or_default() += 1,
                    "Drone Token" => *self.events.entry("Drone entered").or_default() += 1,
                    "Munitions" => *self.events.entry("Munitions entered").or_default() += 1,
                    _ => (),
                }
            }
        }
    }
}

#[test]
fn eoe_standard_games_reach_terminal_outcomes_and_replay_owned_choices() {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/eoe.json")).unwrap();
    let members = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| Card::from_name(row["name"].as_str().unwrap()).unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(members.len(), 266);
    for &member in &members {
        assert!(
            member.is_supported(),
            "{} is still gated",
            member.identity()
        );
    }
    let decks: Vec<_> = DECKS
        .iter()
        .map(|(lands, spells)| deck(&members, lands, spells))
        .collect();
    let start = Instant::now();
    let mut evidence = Evidence::default();
    let trace = std::env::var_os("RUSTMAGE_EOE_TRACE").is_some();
    for &(a, b) in PAIRINGS {
        for (i, seed) in [4, 9].into_iter().enumerate() {
            let config = || Config {
                decks: [decks[a].clone(), decks[b].clone()],
                seed,
                starting_player: if i.is_multiple_of(2) {
                    Player::P0
                } else {
                    Player::P1
                },
                deck_rules: DeckRules::Standard,
            };
            evidence.warped_refs.clear();
            let mut game = Game::new(config()).unwrap();
            let mut journal = Vec::new();
            let case_start = Instant::now();
            if trace {
                eprintln!("EOE game {a} vs {b} seed {seed} started");
            }
            while game.outcome().is_none() {
                assert!(
                    journal.len() < 4096,
                    "EOE game {a} vs {b}, seed {seed} did not terminate"
                );
                let actor = game.actor().unwrap();
                if trace && journal.len().is_multiple_of(64) {
                    eprintln!(
                        "EOE game {a} vs {b} seed {seed} action {} in {:.3}s",
                        journal.len(),
                        case_start.elapsed().as_secs_f64(),
                    );
                }
                let before = game.observe(actor);
                let index = decision(&before);
                let action = before.actions[index].clone();
                assert_ne!(action, Action::Concede);
                let sensitive = private(&before);
                // A Devour choice on a public spell has a public entrant and lands.
                if sensitive || before.entry_devour.is_some() || journal.len().is_multiple_of(64) {
                    let opposing = game.observe(actor.other());
                    assert!(opposing.actions.is_empty());
                    assert!(!private(&opposing));
                    let saved = game.snapshot();
                    let revision = game.revision();
                    assert_eq!(
                        game.apply(actor.other(), action.clone()),
                        Err(Error::WrongActor)
                    );
                    assert_eq!(
                        game.apply(actor, Action::ChooseMode { mode: u16::MAX }),
                        Err(Error::IllegalAction)
                    );
                    assert_eq!(game.snapshot(), saved);
                    assert_eq!(game.revision(), revision);
                    let mut owned = {
                        let parent = game.clone();
                        parent.clone()
                    };
                    owned.apply_index(actor, revision, index).unwrap();
                    game.apply(actor, action.clone()).unwrap();
                    let result = game.snapshot();
                    assert_eq!(owned.snapshot(), result);
                    owned.restore(&saved).unwrap();
                    assert_eq!(
                        owned.apply_index(actor, revision, index),
                        Err(Error::StaleRevision)
                    );
                    owned.apply(actor, action.clone()).unwrap();
                    assert_eq!(owned.snapshot(), result);
                    evidence.verified_branches += 1;
                    evidence.private_choices += usize::from(sensitive);
                } else {
                    game.apply(actor, action.clone()).unwrap();
                }
                let observer = game.actor().map_or(actor, Player::other);
                evidence.record(&before, &game.observe(observer), &action);
                journal.push((actor, action));
            }
            let mut replay = Game::new(config()).unwrap();
            evidence.decisions += journal.len();
            for (actor, action) in journal {
                replay.apply(actor, action).unwrap();
            }
            assert_eq!(replay.snapshot(), game.snapshot());
            assert_eq!(replay.outcome(), game.outcome());
            evidence.games += 1;
            if trace {
                eprintln!(
                    "EOE game {a} vs {b} seed {seed} verified in {:.3}s",
                    case_start.elapsed().as_secs_f64(),
                );
            }
        }
    }
    assert!(evidence.verified_branches > 0 && evidence.private_choices > 0);
    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "EOE_GAMES games={} decisions={} verified_branches={} private_choices={} cards_cast={} seconds={elapsed:.6} decisions_per_second={:.3} events={:?}",
        evidence.games,
        evidence.decisions,
        evidence.verified_branches,
        evidence.private_choices,
        evidence.casts.len(),
        evidence.decisions as f64 / elapsed,
        evidence.events,
    );
    for event in [
        "Devour payment",
        "Warp cast",
        "Warp exile recast",
        "Void eligible cast",
        "Station charge placed",
        "Lander entered",
        "Robot entered",
        "Drone entered",
        "Munitions entered",
    ] {
        assert!(
            evidence.events.contains_key(event),
            "no EOE game evidence for {event}"
        );
    }
}
