//! Supported Standard self-mill/recovery versus targeted-mill games.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, Zone};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, count) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); count]);
    }
    for &name in spells {
        let card = Card::from_name(name).unwrap();
        assert!(card.is_supported(), "{name}");
        cards.extend([card; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let golgari = deck(
        &[("Forest", 12), ("Swamp", 12)],
        &[
            "Llanowar Elves",
            "Deathcap Marionette",
            "Mineshaft Spider",
            "Midnight Tilling",
            "Rapid Rescue",
            "Something Worth Saving",
            "Scarblade Scout",
            "Crow of Dark Tidings",
            "Venomized Cat",
        ],
    );
    let esper = deck(
        &[("Island", 8), ("Swamp", 8), ("Plains", 8)],
        &[
            "Mysterio's Phantasm",
            "Watcher of the Wayside",
            "Space-Time Anomaly",
            "Screaming Phantom",
            "Crow of Dark Tidings",
            "Scarblade Scout",
            "Daggerfang Duo",
            "Deathcap Marionette",
            "Shinra Reinforcements",
        ],
    );
    let (
        mut decisions,
        mut milled,
        mut recoveries,
        mut declines,
        mut replays,
        mut max_bytes,
        mut dynamic_casts,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [golgari.clone(), esper.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            g.legal_actions_into(&mut actions);
            let mut action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            // Exercise both optional recovery outcomes using only public choices.
            if before.recovery_choice.is_some() && seed % 3 == 0 {
                action = actions
                    .iter()
                    .find(|a| matches!(a, Action::RecoverMilled { object: None }))
                    .unwrap()
                    .clone();
            }
            recoveries += usize::from(matches!(action, Action::RecoverMilled { object: Some(_) }));
            declines += usize::from(matches!(action, Action::RecoverMilled { object: None }));
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            dynamic_casts += after
                .stack
                .iter()
                .filter(|item| {
                    item.kind == rustmage::StackKind::Spell
                        && item
                            .source()
                            .is_some_and(|source| source.card.name() == "Space-Time Anomaly")
                        && !before.stack.iter().any(|old| old.id == item.id)
                })
                .count();
            let changes = after
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Graveyard && !before.objects.iter().any(|old| old.id == o.id)
                })
                .count();
            milled += changes;
            if changes > 0
                || before.recovery_choice.is_some()
                || after.recovery_choice.is_some()
                || before.decision == "optional_effect"
            {
                let expected = g.snapshot();
                g.restore(&saved).unwrap();
                let mut fork = g.clone();
                g.apply(actor, action.clone())?;
                fork.apply(actor, action)?;
                assert_eq!(g.snapshot(), expected);
                assert_eq!(fork.snapshot(), expected);
                replays += 1;
            }
            max_bytes = max_bytes.max(g.state_bytes());
            decisions += 1;
        }
        assert!(g.outcome().is_some(), "game {seed} did not finish");
    }
    assert!(milled > 0 && replays > 0);
    if games >= 10 {
        assert!(recoveries > 0 && declines > 0 && dynamic_casts > 0);
    }
    println!(
        "Completed {games} Standard milling games, {decisions} decisions, {milled} newly public milled graveyard objects, {recoveries} recoveries, {declines} declines, {dynamic_casts} life-based mill casts, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
