//! Standard games checking permanent-incarnation exhaust limits independently.
use rustmage::{
    Action, Card, Config, DeckRules, Game, ObjectRef, Observation, Player, SpellView, StackKind,
    Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NEW: [&str; 10] = [
    "Camera Launcher",
    "Draconautics Engineer",
    "Greenbelt Guardian",
    "Hazard of the Dunes",
    "Keen Buccaneer",
    "Loot, the Pathfinder",
    "Pacesetter Paragon",
    "Prowcatcher Specialist",
    "Skystreak Engineer",
    "Stampeding Scurryfoot",
];
fn mask(name: &str) -> u64 {
    match name {
        "Greenbelt Guardian" => 2,
        "Draconautics Engineer" => 3,
        "Loot, the Pathfinder" => 7,
        n if NEW.contains(&n) => 1,
        _ => 0,
    }
}
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for name in ["Island", "Mountain", "Forest"] {
        d.extend([Card::from_name(name).unwrap(); 8]);
    }
    for name in NEW {
        d.extend([Card::from_name(name).unwrap(); 3]);
    }
    for name in ["Unsummon", "Shore Up", "Llanowar Elves"] {
        d.extend([Card::from_name(name).unwrap(); 2]);
    }
    assert_eq!(d.len(), 60);
    assert!(d.iter().all(|c| c.is_supported() && c.is_standard_legal()));
    d
}
fn activated(a: &Action) -> Option<(u32, u16)> {
    match a {
        Action::Activate { card, ability, .. }
        | Action::ActivateMana { card, ability, .. }
        | Action::BeginActivate { card, ability, .. }
        | Action::AnnounceActivate { card, ability, .. }
        | Action::AnnounceMana { card, ability, .. } => Some((*card, *ability)),
        _ => None,
    }
}
fn completed(o: &Observation, a: &Action) -> Option<(ObjectRef, Card, u16)> {
    match a {
        Action::PayCosts { .. } => o.announcing.as_ref().and_then(|s| {
            if let StackKind::ActivatedAbility { ability } = s.kind {
                Some((s.object, s.source_card, ability))
            } else {
                None
            }
        }),
        Action::Activate { card, ability, .. } | Action::ActivateMana { card, ability, .. } => o
            .objects
            .iter()
            .find(|b| b.id == *card)
            .map(|b| (b.reference, b.card, *ability)),
        _ => None,
    }
}
fn choose(
    g: &Game,
    o: &Observation,
    actions: &[Action],
    used: &BTreeSet<(ObjectRef, u16)>,
) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 300,
            Action::Cast { card, .. } | Action::AnnounceCast { card } => {
                let name = o
                    .objects
                    .iter()
                    .find(|b| b.id == *card)
                    .unwrap()
                    .card
                    .name();
                if name == "Unsummon" {
                    if o.objects.iter().any(|b| {
                        b.owner == o.actor.unwrap()
                            && b.zone == Zone::Battlefield
                            && mask(b.card.name()) != 0
                            && b.exhausted_abilities == mask(b.card.name())
                    }) {
                        270
                    } else {
                        -100
                    }
                } else if name == "Shore Up" {
                    if o.objects.iter().any(|b| {
                        b.owner == o.actor.unwrap()
                            && b.zone == Zone::Battlefield
                            && b.card.name() == "Loot, the Pathfinder"
                            && b.tapped
                            && b.exhausted_abilities != 7
                    }) {
                        250
                    } else {
                        -100
                    }
                } else {
                    220
                }
            }
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if used.contains(&(b.reference, *ability)) {
                    -100
                } else if NEW.contains(&b.card.name()) {
                    240
                } else {
                    -100
                }
            }
            Action::AnnounceMana {
                card,
                ability,
                production,
                ..
            } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if b.card.name() == "Loot, the Pathfinder"
                    && *ability == 0
                    && *production == [0, 0, 0, 0, 3, 0]
                {
                    260
                } else {
                    -100
                }
            }
            Action::ActivateMana { .. } | Action::TapMana { .. } => {
                if o.decision == "pay_costs" {
                    170
                } else {
                    -100
                }
            }
            Action::Activate { .. } | Action::BeginActivate { .. } => -100,
            Action::ChooseTarget {
                target: Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                let root = o.announcing.as_ref().unwrap();
                match root.source_card.name() {
                    "Unsummon" => {
                        if b.owner == root.controller
                            && mask(b.card.name()) != 0
                            && b.exhausted_abilities == mask(b.card.name())
                        {
                            290
                        } else {
                            -100
                        }
                    }
                    "Shore Up" => {
                        if b.owner == root.controller
                            && b.card.name() == "Loot, the Pathfinder"
                            && b.tapped
                        {
                            290
                        } else {
                            -100
                        }
                    }
                    "Greenbelt Guardian" => {
                        if b.owner == root.controller {
                            240
                        } else {
                            -100
                        }
                    }
                    _ => {
                        if b.owner != root.controller {
                            230
                        } else {
                            -100
                        }
                    }
                }
            }
            Action::ChooseTarget {
                target: Target::Player(p),
            } => {
                if *p != o.actor.unwrap() {
                    250
                } else {
                    -100
                }
            }
            Action::Attack { .. } => {
                if o.turn >= 18 {
                    80
                } else {
                    -50
                }
            }
            Action::Block { .. } => 80,
            _ => {
                if *i == fallback {
                    120
                } else {
                    0
                }
            }
        })
        .unwrap()
        .0
}
fn live_items(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(o.announcing.as_ref())
        .chain(o.resolving.as_ref())
}
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let (
        mut decisions,
        mut checks,
        mut replays,
        mut activations,
        mut turns_checked,
        mut private_choices,
        mut returns,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0, 0);
    let mut entries = BTreeSet::new();
    let mut bodies = BTreeSet::new();
    let mut token_types = BTreeSet::new();
    let mut resolved = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut spent = BTreeMap::<ObjectRef, u64>::new();
        let mut used = BTreeSet::new();
        for step in 0usize..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999);
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            for a in &actions {
                if let Some((id, index)) = activated(a)
                    && let Some(b) = before.objects.iter().find(|b| b.id == id)
                    && mask(b.card.name()) & (1u64 << index) != 0
                {
                    assert_eq!(
                        spent.get(&b.reference).copied().unwrap_or(0) & (1u64 << index),
                        0
                    );
                    checks += 1;
                }
            }
            let a = actions[choose(&g, &before, &actions, &used)].clone();
            if let Action::AnnounceActivate { card, ability, .. } = a {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                used.insert((b.reference, ability));
            }
            if let Some((reference, card, index)) = completed(&before, &a) {
                let bit = 1u64 << index;
                if mask(card.name()) & bit != 0 {
                    let old = spent.entry(reference).or_default();
                    assert_eq!(*old & bit, 0);
                    *old |= bit;
                    activations += 1;
                    bodies.insert((card.name(), index));
                } else if card.name() == "Greenbelt Guardian" {
                    bodies.insert((card.name(), index));
                }
            }
            if matches!(a, Action::Discard { .. })
                && before
                    .resolving
                    .as_ref()
                    .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Keen Buccaneer"))
            {
                assert!(g.observe(actor.other()).actions.is_empty());
                assert!(g.observe(actor.other()).effect_discards.is_empty());
                private_choices += 1;
            }
            let root = before
                .stack
                .last()
                .filter(|_| {
                    matches!(a, Action::Pass)
                        && before.resolving.is_none()
                        && before.decision == "priority"
                })
                .or(before.resolving.as_ref())
                .cloned();
            let saved = g.snapshot();
            let replay = step.is_multiple_of(4)
                || before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty();
            let expected = if replay {
                let mut f = g.clone();
                f.apply(actor, a.clone()).unwrap();
                Some(f.snapshot())
            } else {
                None
            };
            g.apply(actor, a.clone()).unwrap();
            if let Some(e) = expected {
                assert_eq!(g.snapshot(), e);
                g.restore(&saved).unwrap();
                assert_eq!(
                    g.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                g.apply(actor, a).unwrap();
                assert_eq!(g.snapshot(), e);
                replays += 1;
            }
            let after = g.observe(actor);
            if after.turn != before.turn {
                used.clear();
            }
            for b in after
                .objects
                .iter()
                .filter(|b| b.zone == Zone::Battlefield && mask(b.card.name()) != 0)
            {
                let predicted = spent.get(&b.reference).copied().unwrap_or(0);
                assert_eq!(b.exhausted_abilities, predicted);
                assert_eq!(b.used_abilities & mask(b.card.name()), predicted);
                assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
                entries.insert(b.card.name());
                checks += 1;
                if after.turn != before.turn && predicted != 0 {
                    turns_checked += 1;
                }
            }
            for b in after
                .objects
                .iter()
                .filter(|b| b.zone == Zone::Battlefield && b.card.is_token())
            {
                if b.card.has_subtype("Dinosaur") {
                    token_types.insert("Dinosaur Dragon");
                }
                if b.card.has_subtype("Thopter") {
                    token_types.insert("Thopter");
                }
                if b.card.has_subtype("Elephant") {
                    token_types.insert("Elephant");
                }
            }
            if let Some(root) = root
                && live_items(&after).all(|s| s.id != root.id)
            {
                if let StackKind::ActivatedAbility { ability } = root.kind {
                    if NEW.contains(&root.source_card.name()) {
                        resolved.insert((root.source_card.name(), ability));
                        let amount = match (root.source_card.name(), ability) {
                            ("Camera Launcher", 0)
                            | ("Draconautics Engineer", 0)
                            | ("Keen Buccaneer", 0)
                            | ("Pacesetter Paragon", 0)
                            | ("Stampeding Scurryfoot", 0) => 1,
                            ("Greenbelt Guardian", 1) | ("Hazard of the Dunes", 0) => 3,
                            ("Prowcatcher Specialist", 0) | ("Skystreak Engineer", 0) => 2,
                            _ => 0,
                        };
                        if let Some(old) = before
                            .objects
                            .iter()
                            .find(|b| b.reference == root.object && b.zone == Zone::Battlefield)
                            && let Some(b) = after
                                .objects
                                .iter()
                                .find(|b| b.reference == root.object && b.zone == Zone::Battlefield)
                        {
                            assert_eq!(b.counters.plus_one, old.counters.plus_one + amount);
                            checks += 1;
                        }
                    }
                } else if root.source().is_some_and(|source| source.card.name() == "Unsummon")
                    && let Some(reference) = root.targets.first().and_then(|t| t.object)
                    && before.objects.iter().any(|b| {
                        b.reference == reference
                            && b.zone == Zone::Battlefield
                            && spent.get(&reference).copied().unwrap_or(0) != 0
                    })
                {
                    let owner = g.observe(root.controller);
                    if let Some(b) = owner
                        .objects
                        .iter()
                        .find(|b| b.id == reference.card && b.zone == Zone::Hand)
                    {
                        assert_ne!(b.reference, reference);
                        assert_eq!(b.exhausted_abilities, 0);
                        returns += 1;
                    }
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(entries.contains(name), "entry {name}");
            for index in 0..3 {
                if mask(name) & (1u64 << index) != 0 {
                    assert!(bodies.contains(&(name, index)), "activation {name}/{index}");
                    if !(name == "Loot, the Pathfinder" && index == 0) {
                        assert!(
                            resolved.contains(&(name, index)),
                            "resolution {name}/{index}"
                        );
                    }
                }
            }
        }
        assert!(bodies.contains(&("Greenbelt Guardian", 0)));
        assert_eq!(token_types.len(), 3);
        assert!(private_choices > 0 && returns > 0 && turns_checked > 0);
    }
    println!(
        "{games} Standard games; {decisions} decisions; {checks} mask/body checks; {activations} exhaust activations; {turns_checked} persisted turn boundaries; {private_choices} private discards; {returns} reset returns; {replays} clone/restore replays; {max_bytes} maximum initialized state bytes"
    );
}
