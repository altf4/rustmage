//! Supported Standard discard/cycling observers and replayed continuations.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, SpellView};
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
fn has_event(s: &SpellView) -> bool {
    s.discard_event.is_some()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let temur = deck(
        &[("Forest", 8), ("Island", 8), ("Mountain", 8)],
        &[
            "Marauding Mako",
            "Scrounging Skyray",
            "Magmakin Artillerist",
            "Agonasaur Rex",
            "Howler's Heavy",
            "Rapturous Moment",
            "Stockman, Mad Fly-entist",
            "Llanowar Elves",
            "Lightshield Parry",
        ],
    );
    let jund = deck(
        &[("Forest", 10), ("Swamp", 8), ("Mountain", 6)],
        &[
            "Titanbones, Towering Heart",
            "Tinybones, Pocket Nuisance",
            "Hobgoblin, Mantled Marauder",
            "Marauding Mako",
            "Magmakin Artillerist",
            "Apex Witchstalker",
            "Discerning Peddler",
            "Seeker's Folly",
            "Llanowar Elves",
        ],
    );
    let (
        mut decisions,
        mut events,
        mut cycling,
        mut aggregate,
        mut cleanup,
        mut replays,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [temur.clone(), jund.clone()],
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
            cleanup += usize::from(
                before.decision == "discard" && matches!(action, Action::Discard { .. }),
            );
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            for item in after
                .stack
                .iter()
                .chain(&after.waiting_triggers)
                .chain(&after.announcing)
                .chain(&after.resolving)
            {
                if let Some(event) = item.discard_event
                    && seen.insert(item.id)
                {
                    events += 1;
                    cycling += usize::from(event.cycling);
                    aggregate += usize::from(event.amount > 1);
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
                    .any(has_event)
                || matches!(
                    before.decision,
                    "discard" | "effect_discard" | "effect_choice"
                )
            {
                let expected = g.snapshot();
                g.restore(&saved);
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
        assert!(events > 0 && cycling > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard discard games, {decisions} decisions, {events} event-bound trigger announcements, {cycling} cycling-bound announcements, {aggregate} multi-card announcements, {cleanup} cleanup discard choices, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
