//! Supported Standard damage-trigger games with owned continuation replay.
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
    let esper = deck(
        &[("Island", 10), ("Swamp", 8), ("Plains", 6)],
        &[
            "Thieving Otter",
            "Fear of Failed Tests",
            "April, Reporter of the Weird",
            "Locke Cole",
            "Festerleech",
            "Scream Puff",
            "Goldvein Pick",
            "Yathan Tombguard",
            "SP//dr, Piloted by Peni",
        ],
    );
    let naya = deck(
        &[("Forest", 8), ("Plains", 8), ("Mountain", 8)],
        &[
            "Llanowar Elves",
            "Haliya, Ascendant Cadet",
            "Invasion Tactics",
            "Kyoshi Warriors",
            "The Thing, Ben Grimm",
            "Goldvein Pick",
            "Ball Lightning",
            "Dragon Mage",
            "Eager Trufflesnout",
        ],
    );
    let (
        mut decisions,
        mut triggers,
        mut aggregate,
        mut attached,
        mut discards,
        mut replays,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [esper.clone(), naya.clone()],
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
            discards += usize::from(
                matches!(action, Action::Discard { .. })
                    && before
                        .resolving
                        .as_ref()
                        .is_some_and(|s| s.damage_event.is_some()),
            );
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            let new: Vec<_> = after
                .stack
                .iter()
                .chain(&after.waiting_triggers)
                .chain(after.announcing.iter())
                .filter(|s| {
                    matches!(s.kind, StackKind::TriggeredAbility { .. })
                        && s.damage_event.is_some()
                        && !before
                            .stack
                            .iter()
                            .chain(&before.waiting_triggers)
                            .chain(before.announcing.iter())
                            .any(|old| old.id == s.id)
                })
                .collect();
            triggers += new.len();
            aggregate += new
                .iter()
                .filter(|s| s.damage_event.unwrap().dealer.is_none())
                .count();
            attached += new
                .iter()
                .filter(|s| s.source_card.name() == "Goldvein Pick")
                .count();
            if !new.is_empty()
                || before
                    .resolving
                    .as_ref()
                    .is_some_and(|s| s.damage_event.is_some())
                || before
                    .waiting_triggers
                    .iter()
                    .any(|s| s.damage_event.is_some())
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
        assert!(triggers > 0 && aggregate > 0 && attached > 0 && discards > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard damage-trigger games, {decisions} decisions, {triggers} observed trigger announcements, {aggregate} aggregate announcements, {attached} Equipment announcements, {discards} event-bound discard choices, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
