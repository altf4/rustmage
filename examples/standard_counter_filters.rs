//! Public legal Standard workload for restricted counterspells and a flash trigger.
use rustmage::{
    Action, Card, Config, DeckRules, Game, Observation, Player, StackKind, Target, Zone,
};
use std::collections::BTreeSet;
const COUNTERS: [&str; 7] = [
    "Annul",
    "Flashfreeze",
    "Precise Redaction",
    "Spell Snare",
    "Disdainful Stroke",
    "Spectral Interference",
    "Voracious Greatshark",
];
fn deck(counter: bool) -> Vec<Card> {
    let mut result = Vec::new();
    let lands: &[(&str, usize)] = if counter {
        &[("Island", 32)]
    } else {
        &[("Forest", 8), ("Mountain", 8), ("Plains", 8)]
    };
    for &(name, n) in lands {
        result.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    let spells: &[&str] = if counter {
        &COUNTERS
    } else {
        &[
            "Bear Cub",
            "Shivan Dragon",
            "Hinterland Sanctifier",
            "Three Tree Mascot",
            "Goldvein Pick",
            "Anthem of Champions",
            "Ordinary Bear",
            "Goblin Oriflamme",
            "Dwynen's Elite",
        ]
    };
    for &name in spells {
        result.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(result.len(), 60);
    assert!(
        result
            .iter()
            .all(|c| c.is_supported() && c.is_standard_legal())
    );
    result
}
fn printed_mana_value(card: Card) -> u64 {
    card.info()
        .mana_cost
        .split('}')
        .filter_map(|part| part.strip_prefix('{'))
        .map(|symbol| {
            symbol.parse::<u64>().unwrap_or_else(|_| {
                assert!(
                    symbol
                        .split('/')
                        .all(|s| matches!(s, "W" | "U" | "B" | "R" | "G" | "C"))
                );
                1
            })
        })
        .sum()
}
fn matches(name: &str, target: Card) -> bool {
    let kind = target.info().type_line.split(" — ").next().unwrap();
    let has = |word| kind.split_whitespace().any(|w| w == word);
    match name {
        "Annul" => has("Artifact") || has("Enchantment"),
        "Spectral Interference" | "Voracious Greatshark" => has("Artifact") || has("Creature"),
        "Flashfreeze" => target.colors() & (8 | 16) != 0,
        "Precise Redaction" => target.colors() & (1 | 4) != 0,
        "Spell Snare" => printed_mana_value(target) == 2,
        "Disdainful Stroke" => printed_mana_value(target) >= 4,
        _ => false,
    }
}
fn score(obs: &Observation, action: &Action, paid: bool) -> i32 {
    if obs.actor == Some(Player::P0)
        && matches!(action, Action::Cast { .. } | Action::AnnounceCast { .. })
        && obs.stack.iter().any(|s| s.controller == Player::P0)
    {
        return -100;
    }
    match action {
        Action::Concede | Action::Mulligan => -1000,
        Action::Cast {
            card,
            target: Some(Target::Spell(id)),
            ..
        } => {
            let object = obs.objects.iter().find(|o| o.id == *card).unwrap();
            let victim = obs.stack.iter().find(|o| o.card == *id).unwrap();
            if victim.controller != obs.actor.unwrap() && COUNTERS.contains(&object.card.name()) {
                150
            } else {
                -100
            }
        }
        Action::Activate { .. } => -20,
        Action::PayResolutionMana { .. } => {
            if paid {
                200
            } else {
                -100
            }
        }
        Action::DeclineResolutionMana => {
            if paid {
                -100
            } else {
                200
            }
        }
        Action::ChooseTarget {
            target: Target::Spell(id),
        } => {
            if obs
                .stack
                .iter()
                .any(|s| s.card == *id && s.controller != obs.actor.unwrap())
            {
                160
            } else {
                -50
            }
        }
        Action::PlayLand { .. } => 120,
        Action::TapMana { .. } => {
            if obs.decision == "resolution_mana"
                || obs.decision == "pay_costs"
                || (obs.actor == Some(Player::P1)
                    && obs.active == Player::P1
                    && matches!(obs.step, rustmage::Step::Main1 | rustmage::Step::Main2))
                || (obs.actor == Some(Player::P0)
                    && obs.stack.iter().any(|s| s.controller == Player::P1))
            {
                120
            } else {
                -100
            }
        }
        Action::Cast { card, .. } | Action::AnnounceCast { card } => {
            let source = obs.objects.iter().find(|o| o.id == *card).unwrap().card;
            if COUNTERS.contains(&source.name()) && source.name() != "Voracious Greatshark" {
                return if obs.stack.iter().any(|s| {
                    s.controller != obs.actor.unwrap() && matches(source.name(), s.source_card)
                }) {
                    80
                } else {
                    -100
                };
            }
            if source.name() == "Voracious Greatshark"
                && !obs.stack.iter().any(|s| {
                    s.controller != obs.actor.unwrap() && matches(source.name(), s.source_card)
                })
                && obs.turn < 20
            {
                -20
            } else {
                80
            }
        }
        Action::PutTrigger { .. } | Action::FinishTargets => 140,
        Action::Attack { .. } | Action::AttackPlaneswalker { .. } => {
            if obs.turn >= 12 {
                60
            } else {
                -50
            }
        }
        Action::Block { .. } => 60,
        Action::Keep | Action::Bottom { .. } | Action::Discard { .. } => 90,
        Action::Pass | Action::FinishAttackers | Action::FinishBlockers => 0,
        _ => 40,
    }
}
fn main() {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()
        .unwrap();
    assert!(games > 0);
    let (mut decisions, mut checks, mut branches, mut max_bytes) = (0, 0, 0, 0);
    let mut seen = BTreeSet::new();
    let mut payments = BTreeSet::new();
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [deck(true), deck(false)],
            seed: seed as u64,
            starting_player: Player::P0,
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
            for action in &actions {
                if let Action::Cast {
                    card,
                    target: Some(Target::Spell(id)),
                    ..
                } = action
                {
                    let source = before.objects.iter().find(|o| o.id == *card).unwrap().card;
                    if COUNTERS.contains(&source.name()) {
                        let victim = before.stack.iter().find(|o| o.card == *id).unwrap();
                        assert!(matches(source.name(), victim.source_card));
                        assert_eq!(victim.kind, StackKind::Spell);
                        assert_ne!(*card, *id);
                        checks += 1;
                    }
                }
            }
            let action = actions
                .iter()
                .enumerate()
                .max_by_key(|(i, a)| (score(&before, a, seed % 2 == 0), std::cmp::Reverse(*i)))
                .unwrap()
                .1
                .clone();
            let saved = game.snapshot();
            let fork = if step % 4 == 0 {
                let mut fork = game.clone();
                fork.apply(actor, action.clone()).unwrap();
                branches += 1;
                Some(fork.snapshot())
            } else {
                None
            };
            game.apply(actor, action.clone()).unwrap();
            if let Some(expected) = fork {
                assert_eq!(expected, game.snapshot());
                game.restore(&saved).unwrap();
                assert_eq!(
                    game.apply_index(actor, before.revision, 0),
                    Err(rustmage::Error::StaleRevision)
                );
                game.apply(actor, action.clone()).unwrap();
                assert_eq!(expected, game.snapshot());
            }
            let after = game.observe(actor);
            if let Some(item) = before.stack.last()
                && after.stack.iter().all(|s| s.id != item.id)
                && item.source().is_some_and(|source| COUNTERS.contains(&source.card.name()))
            {
                if let Some(Target::Spell(id)) = item.targets.first().map(|b| b.target) {
                    let Some(victim) = before.stack.iter().find(|s| s.card == id) else {
                        // Multiple responses may share a victim. Later responses
                        // fizzle after the first one has already countered it.
                        assert_eq!(
                            after.objects.iter().find(|o| o.id == id).unwrap().zone,
                            Zone::Graveyard
                        );
                        assert!(after.resolving.as_ref().is_none_or(|s| s.id != item.id));
                        checks += 1;
                        decisions += 1;
                        max_bytes = max_bytes.max(game.state_bytes());
                        continue;
                    };
                    assert!(matches(item.source_card.name(), victim.source_card));
                    if after.resolving.as_ref().is_some_and(|s| s.id == item.id) {
                        assert_eq!(item.source_card.name(), "Spectral Interference");
                        assert_eq!(after.resolution_payment.as_ref().unwrap().cost.generic, 4);
                    } else {
                        assert_eq!(
                            after.objects.iter().find(|o| o.id == id).unwrap().zone,
                            Zone::Graveyard
                        );
                        seen.insert(item.source_card.name());
                    }
                    checks += 1;
                } else if item.source().is_some_and(|source| source.card.name() == "Voracious Greatshark")
                    && item.kind == StackKind::Spell
                {
                    let source = after.objects.iter().find(|o| o.id == item.card).unwrap();
                    assert_eq!(source.zone, Zone::Battlefield);
                    assert_eq!((source.power, source.toughness), (5, 4));
                    checks += 1;
                }
            }
            if let Some(item) = &before.resolving
                && item.source().is_some_and(|source| source.card.name() == "Spectral Interference")
                && matches!(
                    action,
                    Action::PayResolutionMana { .. } | Action::DeclineResolutionMana
                )
            {
                let Target::Spell(id) = item.targets[0].target else {
                    panic!()
                };
                let paid = matches!(action, Action::PayResolutionMana { .. });
                assert_eq!(
                    after.objects.iter().find(|o| o.id == id).unwrap().zone,
                    if paid { Zone::Stack } else { Zone::Graveyard }
                );
                seen.insert("Spectral Interference");
                payments.insert(paid);
                checks += 1;
            }
            decisions += 1;
            max_bytes = max_bytes.max(game.state_bytes());
        }
        assert!(game.outcome().is_some());
    }
    assert!(!seen.is_empty());
    println!(
        "Completed {games} legal Standard counter-filter games, {decisions} decisions, {checks} independent public checks, {branches} cloned/replayed boundaries, {max_bytes} max initialized state bytes; resolved {seen:?}; spectral paid outcomes {payments:?}; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
}
