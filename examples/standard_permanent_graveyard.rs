//! Full Standard games exercising frozen noncreature departures and owned replay.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player, StackKind, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 10] = [
    "Ashiok's Reaper",
    "Hopeful Vigil",
    "Knight of Doves",
    "Long Feng, Grand Secretariat",
    "Mephitic Draught",
    "Nutrient Block",
    "Reach for the Sky",
    "Savior of the Sleeping",
    "Warehouse Tabby",
    "Wicked Visitor",
];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn deck(family: usize) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, amount) in if family == 0 {
        vec![("Plains", 12), ("Swamp", 12)]
    } else {
        vec![
            ("Plains", 4),
            ("Forest", 8),
            ("Swamp", 8),
            ("Evolving Wilds", 4),
        ]
    } {
        d.extend(vec![Card::from_name(n).unwrap(); amount]);
    }
    let ids = if family == 0 {
        vec![0, 1, 2, 4, 5, 7, 8, 9]
    } else {
        vec![3, 6, 0, 1, 5, 4, 8]
    };
    let extra = if family == 0 {
        vec!["Disenchant"]
    } else {
        vec!["Bear Cub", "Vampiric Rites"]
    };
    for n in ids.into_iter().map(|i| NAMES[i]).chain(extra) {
        let c = Card::from_name(n).unwrap();
        assert!(c.is_supported() && c.is_standard_legal());
        d.extend([c; 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn choose(g: &Game, o: &Observation, actions: &[Action]) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    let destroyable = o.objects.iter().any(|b| {
        b.zone == Zone::Battlefield
            && !b.keywords.contains(Keywords::INDESTRUCTIBLE)
            && matches!(
                b.card.name(),
                "Mephitic Draught" | "Hopeful Vigil" | "Reach for the Sky"
            )
    });
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 250,
            Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if b.card.name() == "Disenchant" {
                    if destroyable && o.stack.is_empty() {
                        240
                    } else {
                        -100
                    }
                } else {
                    230
                }
            }
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if !o.stack.is_empty() {
                    -100
                } else {
                    match b.card.name() {
                        "Hopeful Vigil" | "Nutrient Block" | "Vampiric Rites" => 220,
                        "Evolving Wilds" => 225,
                        "Warehouse Tabby"
                            if *ability == 0 && !b.keywords.contains(Keywords::DEATHTOUCH) =>
                        {
                            50
                        }
                        _ => -100,
                    }
                }
            }
            Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. } => -100,
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if o.decision == "pay_costs" {
                    180
                } else {
                    -100
                }
            }
            Action::ChooseAdditionalCost { object } => {
                let b = o.objects.iter().find(|b| b.reference == *object).unwrap();
                if o.objects.iter().any(|a| {
                    a.zone == Zone::Battlefield
                        && a.card.name() == "Reach for the Sky"
                        && a.attached_to == Some(b.reference)
                }) {
                    240
                } else {
                    200 - b.power.clamp(0, 100) as i32
                }
            }
            Action::ChooseTarget {
                target: rustmage::Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                let root = o.announcing.as_ref().unwrap();
                if root.source().is_some_and(|source| source.card.name() == "Disenchant") {
                    if b.owner == o.actor.unwrap()
                        && matches!(
                            b.card.name(),
                            "Mephitic Draught" | "Hopeful Vigil" | "Reach for the Sky"
                        )
                    {
                        250
                    } else {
                        100
                    }
                } else if b.owner == o.actor.unwrap() {
                    220
                } else {
                    50
                }
            }
            Action::Sacrifice { .. } => 210,
            Action::Attack { .. } if o.turn < 12 => -100,
            _ => {
                if *i == fallback {
                    150
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn live(o: &Observation, id: u64) -> bool {
    o.stack.iter().any(|s| s.id == id)
        || o.resolving.as_ref().is_some_and(|s| s.id == id)
        || o.announcing.as_ref().is_some_and(|s| s.id == id)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let decks = [deck(0), deck(1)];
    let (mut decisions, mut replays, mut private, mut history_checks, mut max_bytes) =
        (0, 0, 0, 0, 0);
    let (mut casts, mut triggers, mut resolutions) = ([0; 10], [0; 10], [0; 10]);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [decks[seed % 2].clone(), decks[seed % 2].clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut tracked = BTreeMap::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let p = g.actor().unwrap();
            let before = g.observe(p);
            let actions = g.legal_actions();
            let action = actions[choose(&g, &before, &actions)].clone();
            if before.library_choice.is_some() || before.search_choice.is_some() {
                let other = g.observe(p.other());
                assert!(
                    other.library_choice.is_none()
                        && other.search_choice.is_none()
                        && other.actions.is_empty()
                );
                private += 1;
            }
            let boundary = before.announcing.is_some()
                || before.resolving.is_some()
                || before.stack.iter().any(|s| s.source().and_then(|source| index(source.card)).is_some());
            let saved = g.snapshot();
            let mut fork = boundary.then(|| g.clone());
            g.apply(p, action.clone())?;
            if let Some(f) = &mut fork {
                f.apply(p, action.clone())?;
            }
            let after = g.observe(p);
            for s in &after.stack {
                if before.stack.iter().any(|old| old.id == s.id)
                    || before.resolving.as_ref().is_some_and(|old| old.id == s.id)
                {
                    continue;
                }
                let Some(i) = s.source().and_then(|source| index(source.card)) else {
                    continue;
                };
                if s.kind == StackKind::Spell {
                    casts[i] += 1;
                } else if matches!(s.kind, StackKind::TriggeredAbility { .. })
                    && (!matches!(i, 1 | 4 | 5 | 6)
                        || after
                            .objects
                            .iter()
                            .any(|o| o.id == s.card && o.zone == Zone::Graveyard))
                {
                    assert!(!tracked.contains_key(&s.id));
                    tracked.insert(s.id, i);
                    triggers[i] += 1;
                }
            }
            let completed: Vec<_> = tracked
                .iter()
                .filter(|(id, _)| !live(&after, **id))
                .map(|(&id, &i)| (id, i))
                .collect();
            for (id, i) in completed {
                tracked.remove(&id);
                resolutions[i] += 1;
            }
            if before.turn == after.turn {
                // This deck has no exile/bounce or zero-toughness incoming
                // creatures. Departed creature tokens cease before observation.
                let died = before
                    .objects
                    .iter()
                    .filter(|b| b.zone == Zone::Battlefield && b.card.is_creature())
                    .filter(|b| match after.objects.iter().find(|o| o.id == b.id) {
                        Some(o) => o.zone == Zone::Graveyard && o.reference != b.reference,
                        None => b.token.is_some(),
                    })
                    .count() as u64;
                assert_eq!(
                    after.creatures_died_this_turn,
                    before.creatures_died_this_turn + died
                );
                history_checks += 1;
            }
            if let Some(f) = fork {
                let expected = g.snapshot();
                assert_eq!(f.snapshot(), expected);
                g.restore(&saved)?;
                assert_eq!(
                    g.apply_index(p, before.revision, 0),
                    Err(Error::StaleRevision)
                );
                g.apply(p, action)?;
                assert_eq!(g.snapshot(), expected);
                replays += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} unfinished");
    }
    if games >= 100 {
        assert!(casts.iter().all(|n| *n > 0), "casts {casts:?}");
        assert!(triggers.iter().all(|n| *n > 0), "triggers {triggers:?}");
        assert!(
            resolutions.iter().all(|n| *n > 0),
            "resolutions {resolutions:?}"
        );
        assert!(private > 0);
    }
    println!(
        "Completed {games} Standard permanent-graveyard games, {decisions} decisions, casts {casts:?}, graveyard triggers {triggers:?}, finished triggers {resolutions:?}, {history_checks} creature-history checks, {private} private choices, {replays} replay boundaries, {max_bytes} maximum initialized logical bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
