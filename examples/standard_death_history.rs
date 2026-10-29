//! Public transition history checks over complete legal Standard games.
use rustmage::{Action, Card, Config, DeckRules, Game, Observation, Player, StackKind, Step, Zone};
use std::collections::BTreeSet;
const NEW: [&str; 5] = [
    "Cackling Prowler",
    "Cackling Slasher",
    "Dreaded Bat-Cloud",
    "Needletooth Pack",
    "Scorpion, Seething Striker",
];
fn deck() -> Vec<Card> {
    let mut result = vec![Card::from_name("Swamp").unwrap(); 12];
    result.extend([Card::Forest; 12]);
    for name in NEW.into_iter().chain([
        "Ordinary Bear",
        "Llanowar Elves",
        "Murder",
        "Corrupted Conviction",
    ]) {
        result.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(result.len(), 60);
    assert!(
        result
            .iter()
            .all(|c| c.is_supported() && c.is_standard_legal())
    );
    // No token makers, bounce, exile or graveyard replacement effects: public
    // BF-to-graveyard transitions independently identify every actual death.
    result
}
fn choose(g: &Game, obs: &Observation, actions: &[Action], seed: usize) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(i, a)| match a {
            Action::Concede => -1000,
            Action::PlayLand { .. } => 200,
            Action::AnnounceCast { card, .. } | Action::Cast { card, .. } => {
                let name = obs
                    .objects
                    .iter()
                    .find(|o| o.id == *card)
                    .unwrap()
                    .card
                    .name();
                if seed % 2 == 1
                    && obs.step == Step::Main1
                    && obs.objects.iter().any(|o| {
                        o.owner == obs.active
                            && o.zone == Zone::Battlefield
                            && o.card.is_creature()
                            && !o.summoning_sick
                            && !o.tapped
                    })
                {
                    return -50;
                }
                if ["Dreaded Bat-Cloud", "Cackling Slasher"].contains(&name)
                    && seed % 2 == 1
                    && obs.creatures_died_this_turn == 0
                {
                    return -50;
                }
                if name == "Cackling Slasher" && obs.creatures_died_this_turn > 0 {
                    180
                } else if NEW.contains(&name) {
                    160
                } else {
                    100
                }
            }
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if obs.decision == "pay_costs" {
                    140
                } else {
                    -100
                }
            }
            Action::Attack { .. } => {
                if obs.turn >= 8 {
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
    let (mut decisions, mut public_checks, mut replays, mut max_bytes) = (0, 0, 0, 0);
    let mut entries = BTreeSet::new();
    let mut guarded_triggers = BTreeSet::new();
    let mut bat_payments = BTreeSet::new();
    let mut slashers = BTreeSet::new();
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [deck(), deck()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut deaths = 0u64;
        for step in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            assert_eq!(before.creatures_died_this_turn, deaths);
            let actions = g.legal_actions();
            let a = actions[choose(&g, &before, &actions, seed)].clone();
            if before.decision == "pay_costs"
                && before
                    .announcing
                    .as_ref()
                    .is_some_and(|s| s.source().is_some_and(|source| source.card.name() == "Dreaded Bat-Cloud"))
            {
                let request = before.payment_request.as_ref().unwrap();
                assert_eq!(request.generic, if deaths > 0 { 1 } else { 4 });
                assert_eq!(request.required, [0, 0, 1, 0, 0, 0]);
                if matches!(a, Action::PayCosts { .. }) {
                    bat_payments.insert(deaths > 0);
                }
            }
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
            if before.turn != after.turn {
                deaths = 0;
            }
            // Object identity slots are stable for these nontoken decks.
            deaths += before
                .objects
                .iter()
                .filter(|old| {
                    old.zone == Zone::Battlefield
                        && old.card.is_creature()
                        && after
                            .objects
                            .iter()
                            .any(|new| new.id == old.id && new.zone == Zone::Graveyard)
                })
                .count() as u64;
            assert_eq!(after.creatures_died_this_turn, deaths);
            assert_eq!(g.observe(actor.other()).creatures_died_this_turn, deaths);
            public_checks += 3;
            for object in after
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && NEW.contains(&o.card.name()))
            {
                if before
                    .objects
                    .iter()
                    .any(|o| o.reference == object.reference)
                {
                    continue;
                }
                entries.insert(object.card.name());
                if object.card.name() == "Cackling Slasher" {
                    assert_eq!(object.counters.plus_one, u64::from(deaths > 0));
                    slashers.insert(deaths > 0);
                }
            }
            for item in before.stack.iter().filter(|s| {
                matches!(s.kind, StackKind::TriggeredAbility { ability } if ability == u16::from(s.source().is_some_and(|source| source.card.name() == "Cackling Prowler")))
                    && NEW.contains(&s.source_card.name())
            }) {
                assert!(deaths > 0, "morbid trigger requires death history");
                if after.stack.iter().all(|s| s.id != item.id) {
                    assert_eq!(before.step, Step::End);
                    guarded_triggers.insert(item.source_card.name());
                }
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
        assert!(g.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            assert!(entries.contains(name), "missing entry {name}");
        }
        for name in [
            "Cackling Prowler",
            "Needletooth Pack",
            "Scorpion, Seething Striker",
        ] {
            assert!(guarded_triggers.contains(name), "missing trigger {name}");
        }
        assert_eq!(bat_payments.len(), 2);
        assert_eq!(slashers.len(), 2);
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"public_history_checks":public_checks,"clone_restore_replays":replays,"max_initialized_state_bytes":max_bytes,"entries":entries,"guarded_triggers":guarded_triggers,"bat_payment_branches":bat_payments,"slasher_entry_branches":slashers})
    );
}
