//! Independent public loss totals in legal decks containing no life-gain effects.
use rustmage::{Action, Card, Config, DeckRules, Game, Observation, Player, StackKind, Step, Zone};
use std::collections::BTreeSet;
const NEW: [&str; 3] = [
    "Bloodtithe Collector",
    "Frilled Sparkshooter",
    "Stromkirk Bloodthief",
];
fn deck() -> Vec<Card> {
    let mut d = vec![Card::from_name("Swamp").unwrap(); 12];
    d.extend([Card::Mountain; 12]);
    for n in NEW.into_iter().chain([
        "Shock",
        "Murder",
        "Highborn Vampire",
        "Fire Elemental",
        "Shivan Dragon",
        "Desolation Prowler",
    ]) {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    assert!(d.iter().all(|c| c.is_supported() && c.is_standard_legal()));
    d
}
fn choose(g: &Game, o: &Observation, actions: &[Action], seed: usize) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 200,
            Action::Cast { card, .. } | Action::AnnounceCast { card, .. } => {
                let name = o
                    .objects
                    .iter()
                    .find(|b| b.id == *card)
                    .unwrap()
                    .card
                    .name();
                if seed % 2 == 1
                    && o.step == Step::Main1
                    && o.objects.iter().any(|b| {
                        b.owner == o.active
                            && b.zone == Zone::Battlefield
                            && b.card.is_creature()
                            && !b.tapped
                            && !b.summoning_sick
                    })
                {
                    return -50;
                }
                if NEW.contains(&name) { 160 } else { 100 }
            }
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if o.decision == "pay_costs" {
                    140
                } else {
                    -100
                }
            }
            Action::Activate { card, .. }
            | Action::AnnounceActivate { card, .. }
            | Action::BeginActivate { card, .. } => {
                let name = o
                    .objects
                    .iter()
                    .find(|b| b.id == *card)
                    .unwrap()
                    .card
                    .name();
                if name == "Desolation Prowler" && o.players[o.actor.unwrap().index()].life > 4 {
                    90
                } else {
                    -100
                }
            }
            Action::Attack { .. } => {
                if o.turn >= 8 {
                    80
                } else {
                    -50
                }
            }
            Action::Block { .. } => 80,
            _ => {
                if *i == fallback {
                    60
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
    let (mut decisions, mut checks, mut replays, mut max_bytes) = (0, 0, 0, 0);
    let mut entries = BTreeSet::new();
    let mut frilled = BTreeSet::new();
    let mut triggers = BTreeSet::new();
    let mut entry_flags = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut lost = [0u128; 2];
        for step in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            assert_eq!(
                [
                    before.players[0].life_lost_this_turn,
                    before.players[1].life_lost_this_turn
                ],
                lost
            );
            let actions = g.legal_actions();
            let a = actions[choose(&g, &before, &actions, seed)].clone();
            let saved = g.snapshot();
            let expected = if step % 4 == 0
                || before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty()
            {
                let mut fork = g.clone();
                fork.apply(actor, a.clone()).unwrap();
                replays += 1;
                Some(fork.snapshot())
            } else {
                None
            };
            g.apply(actor, a.clone()).unwrap();
            if let Some(expected) = expected {
                assert_eq!(g.snapshot(), expected);
                g.restore(&saved).unwrap();
                assert_eq!(
                    g.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                g.apply(actor, a).unwrap();
                assert_eq!(g.snapshot(), expected);
            }
            let after = g.observe(actor);
            // These decks have no life gain or setters. Their positive life
            // decreases independently account for damage and paid life alike.
            if before.turn != after.turn {
                lost = [0, 0];
            } else {
                for p in [Player::P0, Player::P1] {
                    lost[p.index()] += (before.players[p.index()].life
                        - after.players[p.index()].life)
                        .max(0) as u128;
                }
            }
            for viewer in [actor, actor.other()] {
                let o = g.observe(viewer);
                assert_eq!(
                    [
                        o.players[0].life_lost_this_turn,
                        o.players[1].life_lost_this_turn
                    ],
                    lost
                );
            }
            checks += 3;
            for object in after
                .objects
                .iter()
                .filter(|b| b.zone == Zone::Battlefield && NEW.contains(&b.card.name()))
            {
                if before
                    .objects
                    .iter()
                    .any(|b| b.reference == object.reference)
                {
                    continue;
                }
                entries.insert(object.card.name());
                let positive = lost[object.owner.other().index()] > 0;
                if object.card.name() == "Frilled Sparkshooter" {
                    assert_eq!(object.counters.plus_one, u64::from(positive));
                    assert_eq!(
                        (object.power, object.toughness),
                        (3 + i128::from(positive), 3 + i128::from(positive))
                    );
                    frilled.insert(positive);
                }
                if object.card.name() == "Bloodtithe Collector" {
                    entry_flags.insert(positive);
                }
            }
            for item in before.stack.iter().filter(|s| {
                matches!(s.kind, StackKind::TriggeredAbility { .. })
                    && s.source().is_some_and(|source| NEW.contains(&source.card.name()))
            }) {
                assert!(before.players[item.controller.other().index()].life_lost_this_turn > 0);
                if after.stack.iter().all(|s| s.id != item.id) {
                    triggers.insert(item.source_card.name());
                }
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(entries.contains(name));
        }
        assert_eq!(frilled.len(), 2);
        assert_eq!(entry_flags.len(), 2);
        for name in ["Bloodtithe Collector", "Stromkirk Bloodthief"] {
            assert!(triggers.contains(name));
        }
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"public_history_checks":checks,"clone_restore_replays":replays,"max_initialized_state_bytes":max_bytes,"entries":entries,"frilled_entry_branches":frilled,"collector_entry_branches":entry_flags,"trigger_kinds_started":triggers})
    );
}
