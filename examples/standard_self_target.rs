//! Standard self-play with an independent target-event model and branch replay.
use rustmage::{
    Action, Card, Config, DeckRules, Game, ObjectRef, Observation, Player, SpellView, StackKind,
    Target, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
const NEW: [&str; 8] = [
    "Cactarantula",
    "Mouse Trapper",
    "Nettle Guard",
    "Old Fat Spider",
    "Ruric Thar, Biomagus",
    "Seedglaive Mentor",
    "Veteran Guardmouse",
    "Whiskerquill Scribe",
];
fn valiant(name: &str) -> bool {
    NEW.contains(&name)
        && !matches!(
            name,
            "Cactarantula" | "Old Fat Spider" | "Ruric Thar, Biomagus"
        )
}
fn deck() -> Vec<Card> {
    let mut d = Vec::new();
    for land in ["Plains", "Island", "Mountain", "Forest"] {
        d.extend([Card::from_name(land).unwrap(); 5]);
    }
    d.extend([Card::from_name("Bristling Backwoods").unwrap(); 4]);
    for name in NEW {
        d.extend([Card::from_name(name).unwrap(); 3]);
    }
    for name in ["Giant Growth", "Shock", "Gold Pan"] {
        d.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    assert!(d.iter().all(|c| c.is_supported() && c.is_standard_legal()));
    d
}
fn items(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.announcing.as_ref())
        .chain(o.resolving.as_ref())
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
            Action::Cast { .. } | Action::AnnounceCast { .. } => 220,
            Action::AnnounceActivate { card, ability, .. } => {
                let b = o.objects.iter().find(|b| b.id == *card).unwrap();
                if used.contains(&(b.reference, *ability)) {
                    -100
                } else if b.card.name() == "Gold Pan" {
                    240
                } else if b.card.name() == "Nettle Guard" {
                    210
                } else {
                    -100
                }
            }
            Action::Activate { .. }
            | Action::BeginActivate { .. }
            | Action::AnnounceMana { .. }
            | Action::ActivateMana { .. } => -100,
            Action::TapMana { .. } => {
                if o.decision == "pay_costs" {
                    180
                } else {
                    -100
                }
            }
            Action::ChooseTarget {
                target: Target::Card(id),
            } => {
                let b = o.objects.iter().find(|b| b.id == *id).unwrap();
                let root = o.announcing.as_ref().unwrap();
                if root
                    .source()
                    .is_some_and(|source| source.card.name() == "Shock")
                {
                    if b.owner != root.controller && NEW.contains(&b.card.name()) {
                        300
                    } else {
                        100
                    }
                } else if b.owner == root.controller
                    && valiant(b.card.name())
                    && !o
                        .triggered_this_turn
                        .iter()
                        .any(|t| t.source == b.reference)
                {
                    330
                } else if b.owner != root.controller
                    && matches!(
                        b.card.name(),
                        "Cactarantula" | "Old Fat Spider" | "Ruric Thar, Biomagus"
                    )
                {
                    250
                } else {
                    140
                }
            }
            Action::ChooseTarget {
                target: Target::Player(_),
            } => 100,
            Action::Attack { .. } => {
                if o.turn >= 14 {
                    80
                } else {
                    -50
                }
            }
            Action::Block { .. } => 80,
            Action::ChooseOptional { accept: true } => 150,
            Action::Discard { .. } => 150,
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
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let (mut decisions, mut checks, mut replays, mut max_bytes, mut private_choices) =
        (0, 0, 0, 0, 0);
    let mut entries = BTreeSet::new();
    let mut bodies = BTreeSet::new();
    let mut routes = BTreeSet::new();
    let mut activations = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut used = BTreeSet::new();
        let mut first = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut expected = BTreeMap::<&str, usize>::new();
        let mut actual = BTreeMap::<&str, usize>::new();
        for step in 0usize..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999);
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let actions = g.legal_actions();
            let a = actions[choose(&g, &before, &actions, &used)].clone();
            if let Action::AnnounceActivate { card, ability, .. } = a {
                let b = before.objects.iter().find(|b| b.id == card).unwrap();
                used.insert((b.reference, ability));
                activations.insert(b.card.name());
            }
            let binding = match &a {
                Action::ChooseTarget {
                    target: Target::Card(id),
                } => before.announcing.as_ref().map(|s| {
                    (
                        *id,
                        s.controller,
                        s.kind,
                        s.targets
                            .iter()
                            .any(|t| t.object.is_some_and(|r| r.card == *id)),
                    )
                }),
                Action::Cast {
                    target: Some(Target::Card(id)),
                    ..
                } => Some((*id, actor, StackKind::Spell, false)),
                Action::Activate {
                    target: Some(Target::Card(id)),
                    ..
                } => Some((
                    *id,
                    actor,
                    StackKind::ActivatedAbility { ability: 0 },
                    false,
                )),
                _ => None,
            };
            if let Some((id, p, kind, duplicate)) = binding
                && let Some(b) = before.objects.iter().find(|b| {
                    b.id == id && b.zone == Zone::Battlefield && NEW.contains(&b.card.name())
                })
            {
                let name = b.card.name();
                let yours = p == b.owner;
                let should = !duplicate
                    && if valiant(name) {
                        yours && first.insert(b.reference)
                    } else {
                        !yours
                    };
                // A mandatory targeted trigger with no legal target is removed
                // before any stack/ordering observation. Its first-event marker
                // remains public and is checked separately below.
                let announceable = name != "Mouse Trapper"
                    || before.objects.iter().any(|other| {
                        other.owner != b.owner
                            && other.zone == Zone::Battlefield
                            && other.card.is_creature()
                    });
                if should && announceable {
                    *expected.entry(name).or_default() += 1;
                    routes.insert(if kind == StackKind::Spell {
                        "spell"
                    } else {
                        "ability"
                    });
                }
                checks += 1;
            }
            if matches!(a, Action::ArrangeLibrary { .. } | Action::Discard { .. })
                && before.resolving.as_ref().is_some_and(|s| {
                    matches!(
                        s.source()
                            .expect("example expects an object-backed packet")
                            .card
                            .name(),
                        "Veteran Guardmouse" | "Whiskerquill Scribe"
                    )
                })
            {
                let hidden = g.observe(actor.other());
                assert!(hidden.actions.is_empty());
                assert!(hidden.library_choice.is_none());
                assert!(hidden.effect_discards.is_empty());
                private_choices += 1;
            }
            let saved = g.snapshot();
            let replay = step.is_multiple_of(4)
                || before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty();
            let expected_state = if replay {
                let mut f = g.clone();
                f.apply(actor, a.clone()).unwrap();
                Some(f.snapshot())
            } else {
                None
            };
            g.apply(actor, a.clone()).unwrap();
            if let Some(e) = expected_state {
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
            if before.turn != after.turn {
                used.clear();
                first.clear();
            }
            for b in &after.objects {
                if b.zone == Zone::Battlefield && NEW.contains(&b.card.name()) {
                    entries.insert(b.card.name());
                }
            }
            for item in items(&after) {
                let name = item
                    .source()
                    .expect("example expects an object-backed packet")
                    .card
                    .name();
                let index = if name == "Ruric Thar, Biomagus" { 2 } else { 0 };
                if NEW.contains(&name)
                    && item.kind == (StackKind::TriggeredAbility { ability: index })
                    && seen.insert(item.id)
                {
                    *actual.entry(name).or_default() += 1;
                    bodies.insert(name);
                    assert!(item.related_stack.is_none());
                }
            }
            // Every first own target consumes its marker, including an
            // unannounceable Trapper trigger and a departed source incarnation.
            for reference in &first {
                assert!(
                    after
                        .triggered_this_turn
                        .iter()
                        .any(|t| t.source == *reference && t.ability == 0)
                );
            }
            // The public first-trigger markers must agree with the event model.
            for t in &after.triggered_this_turn {
                if after
                    .objects
                    .iter()
                    .any(|b| b.reference == t.source && valiant(b.card.name()))
                {
                    assert!(first.contains(&t.source));
                }
            }
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some());
        assert_eq!(actual, expected, "seed {seed}");
    }
    if games >= 100 {
        for name in NEW {
            assert!(entries.contains(name), "entry {name}");
            assert!(bodies.contains(name), "trigger {name}");
        }
        assert_eq!(routes.len(), 2);
        assert!(private_choices > 0);
        assert!(activations.contains("Nettle Guard") && activations.contains("Gold Pan"));
    }
    println!(
        "{games} Standard games; {decisions} decisions; {checks} target-event checks; {private_choices} private choices; {replays} clone/restore replays; {max_bytes} maximum initialized state bytes"
    );
}
