//! Supported Standard draw observers, public ordinals and boundary replay.
use rustmage::{Card, Config, DeckRules, Game, Player, SpellView};
use std::collections::BTreeSet;
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported(), "{name}");
        cards.extend([c; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
fn has_event(item: &SpellView) -> bool {
    item.draw_event.is_some()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let azorius = deck(
        &[
            ("Island", 12),
            ("Plains", 8),
            ("Temple of Enlightenment", 4),
        ],
        &[
            "Mischievous Mystic",
            "Erudite Wizard",
            "Bard the Bowman",
            "Knowledge Seeker",
            "Clinquant Skymage",
            "Gleaming Splendor",
            "Tiger-Seal",
            "Scrawling Crawler",
            "Lightfoot Technique",
        ],
    );
    let simic = deck(
        &[("Island", 12), ("Forest", 12)],
        &[
            "Atlantean Cavalry",
            "Lakeshore Apothecary",
            "Loxodon Eavesdropper",
            "Otter-Penguin",
            "Ravenhill Flock",
            "Stratosoarer",
            "Lightshield Parry",
            "Knowledge Seeker",
            "Mischievous Mystic",
        ],
    );
    let (mut decisions, mut draws, mut events, mut seconds, mut replays, mut max_bytes) =
        (0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [azorius.clone(), simic.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut seen = BTreeSet::new();
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            g.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            draws += (0..2)
                .map(|i| {
                    after.players[i]
                        .cards_drawn_this_turn
                        .saturating_sub(before.players[i].cards_drawn_this_turn)
                })
                .sum::<u64>();
            for item in after
                .stack
                .iter()
                .chain(&after.waiting_triggers)
                .chain(&after.announcing)
                .chain(&after.resolving)
            {
                if let Some(event) = item.draw_event
                    && seen.insert(item.id)
                {
                    events += 1;
                    seconds += usize::from(event.ordinal == 2);
                }
            }
            if before
                .stack
                .iter()
                .chain(&before.waiting_triggers)
                .chain(&before.announcing)
                .chain(&before.resolving)
                .any(has_event)
                || after
                    .stack
                    .iter()
                    .chain(&after.waiting_triggers)
                    .chain(&after.announcing)
                    .chain(&after.resolving)
                    .any(has_event)
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
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "game {seed} did not finish");
    }
    if games >= 20 {
        assert!(draws > 0 && events > 0 && seconds > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard draw games, {decisions} decisions, {draws} successful in-game draws, {events} event-bound trigger announcements, {seconds} ordinal-two announcements, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
