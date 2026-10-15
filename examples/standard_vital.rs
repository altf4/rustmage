//! Supported Standard self-play with life-gain and creature-death trigger chains.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, StackKind};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
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
    let orzhov = deck(
        &[("Plains", 12), ("Swamp", 12)],
        &[
            "Ajani's Pridemate",
            "Marauding Blight-Priest",
            "Cat Collector",
            "Vanguard Seraph",
            "Voracious Vermin",
            "Midnight Reaper",
            "High-Society Hunter",
            "Inspiring Overseer",
            "Vampire Nighthawk",
        ],
    );
    let rakdos = deck(
        &[("Swamp", 12), ("Mountain", 12)],
        &[
            "Boggart Cursecrafter",
            "Engine Rat",
            "Dragon Fodder",
            "High-Society Hunter",
            "Agent Venom",
            "Midnight Reaper",
            "Voracious Vermin",
            "Pulse Tracker",
            "Shock",
        ],
    );
    let (mut decisions, mut gains, mut triggers, mut sacrifices, mut replays) = (0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [orzhov.clone(), rakdos.clone()],
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
            let action = actions[rustmage::policy::smoke_index(&g, &actions)].clone();
            sacrifices += usize::from(matches!(action, Action::Sacrifice { .. }));
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            let gained = (0..2)
                .map(|p| {
                    after.players[p]
                        .life_gain_events_this_turn
                        .saturating_sub(before.players[p].life_gain_events_this_turn)
                })
                .sum::<u64>();
            gains += gained;
            let new_triggers = after
                .stack
                .iter()
                .chain(&after.waiting_triggers)
                .filter(|t| matches!(t.kind, StackKind::TriggeredAbility { .. }))
                .filter(|t| {
                    !before
                        .stack
                        .iter()
                        .chain(&before.waiting_triggers)
                        .any(|old| old.id == t.id)
                })
                .count();
            triggers += new_triggers;
            if gained > 0
                || new_triggers > 0
                || matches!(
                    action,
                    Action::PutTrigger { .. }
                        | Action::Sacrifice { .. }
                        | Action::ArrangeLibrary { .. }
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
        }
        if g.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    assert!(gains > 0 && triggers > 0 && sacrifices > 0 && replays > 0);
    println!(
        "Completed {games} vital-trigger games, {decisions} decisions, {gains} life-gain events, {triggers} observed triggers, {sacrifices} sacrifices, {replays} cloned/replayed boundaries; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
