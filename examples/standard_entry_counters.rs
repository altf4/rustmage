//! Counter quantities checked from public pre-entry observations during legal games.
use rustmage::{Action, Card, Config, DeckRules, Game, Observation, Player, Zone};
use std::collections::BTreeSet;
const NEW: [&str; 3] = ["Goblin Boarders", "Effortless Master", "Luxknight Breacher"];
fn deck(blue: bool) -> Vec<Card> {
    let (lands, spells) = if blue {
        (
            ["Island", "Mountain"],
            [
                "Effortless Master",
                "Goblin Boarders",
                "Three Tree Mascot",
                "Hullcarver",
                "Opt",
                "Chart a Course",
                "Storm Fleet Spy",
                "Shock",
                "Quick Study",
            ],
        )
    } else {
        (
            ["Plains", "Forest"],
            [
                "Luxknight Breacher",
                "Three Tree Mascot",
                "Hullcarver",
                "Ordinary Bear",
                "Wary Thespian",
                "Gold Pan",
                "Llanowar Elves",
                "Savannah Lions",
                "Giant Growth",
            ],
        )
    };
    let mut deck = Vec::new();
    for name in lands {
        deck.extend([Card::from_name(name).unwrap(); 12]);
    }
    for name in spells {
        deck.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(deck.len(), 60);
    assert!(
        deck.iter()
            .all(|c| c.is_supported() && c.is_standard_legal())
    );
    deck
}
fn choose(g: &Game, before: &Observation, actions: &[Action], seed: usize) -> usize {
    let fallback = rustmage::policy::smoke_index(g, actions);
    actions
        .iter()
        .enumerate()
        .max_by_key(|(index, action)| match action {
            Action::AnnounceCast { card, .. } | Action::Cast { card, .. } => {
                let object = before.objects.iter().find(|o| o.id == *card).unwrap();
                let name = object.card.name();
                if seed % 2 == 1
                    && before.step == rustmage::Step::Main1
                    && before.objects.iter().any(|o| {
                        o.owner == object.owner
                            && o.zone == Zone::Battlefield
                            && o.card.is_creature()
                            && !o.summoning_sick
                            && !o.tapped
                    })
                {
                    return -50;
                }
                if name == "Effortless Master"
                    && seed % 2 == 1
                    && before.players[object.owner.index()].spells_cast_this_turn == 0
                {
                    return 20;
                }
                (if NEW.contains(&name) { 150 } else { 100 })
                    + i32::from(matches!(action, Action::AnnounceCast { .. }))
            }
            Action::PlayLand { .. } => 200,
            Action::TapMana { .. } | Action::ActivateMana { .. } => {
                if before.decision == "pay_costs" {
                    140
                } else {
                    -100
                }
            }
            Action::Concede => -1000,
            Action::Attack { .. } => 80,
            _ => {
                if *index == fallback {
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
    let (mut decisions, mut entries, mut branches, mut max_bytes) = (0, 0, 0, 0);
    let mut covered = BTreeSet::new();
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [deck(true), deck(false)],
            seed: seed as u64,
            starting_player: if seed % 2 == 0 {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        for step in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            assert!(step < 99_999, "game must terminate");
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            let actions = game.legal_actions();
            let action = actions[choose(&game, &before, &actions, seed)].clone();
            let saved = game.snapshot();
            let expected = if step % 4 == 0
                || before.announcing.is_some()
                || before.resolving.is_some()
                || !before.stack.is_empty()
            {
                let mut fork = game.clone();
                fork.apply(actor, action.clone()).unwrap();
                branches += 1;
                Some(fork.snapshot())
            } else {
                None
            };
            game.apply(actor, action.clone()).unwrap();
            if let Some(expected) = expected {
                assert_eq!(game.snapshot(), expected);
                game.restore(&saved).unwrap();
                assert_eq!(
                    game.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                game.apply(actor, action).unwrap();
                assert_eq!(game.snapshot(), expected);
            }
            let after = game.observe(actor);
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
                let (count, power, toughness) = match object.card.name() {
                    "Goblin Boarders" => (
                        u64::from(before.players[object.owner.index()].attacked_this_turn),
                        3,
                        2,
                    ),
                    "Effortless Master" => (
                        if before.players[object.owner.index()].spells_cast_this_turn >= 2 {
                            2
                        } else {
                            0
                        },
                        4,
                        3,
                    ),
                    "Luxknight Breacher" => (
                        before
                            .objects
                            .iter()
                            .filter(|o| {
                                o.owner == object.owner
                                    && o.zone == Zone::Battlefield
                                    && (o.card.is_creature() || o.card.is_artifact())
                            })
                            .count() as u64,
                        2,
                        2,
                    ),
                    _ => unreachable!(),
                };
                assert_eq!(object.counters.plus_one, count);
                assert_eq!(object.power, power + i128::from(count));
                assert_eq!(object.toughness, toughness + i128::from(count));
                covered.insert((object.card.name(), count > 0));
                entries += 1;
            }
            max_bytes = max_bytes.max(game.state_bytes());
            decisions += 1;
        }
        assert!(game.outcome().is_some());
    }
    if games >= 100 {
        for name in NEW {
            for positive in [false, true] {
                assert!(
                    covered.contains(&(name, positive)),
                    "missing {name} positive={positive}"
                );
            }
        }
    }
    println!(
        "{}",
        serde_json::json!({"games":games,"decisions":decisions,"checked_entries":entries,"clone_restore_replays":branches,"max_initialized_state_bytes":max_bytes,"entry_branches":covered})
    );
}
