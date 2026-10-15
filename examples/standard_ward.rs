//! Supported Standard ward and resolution-payment smoke games.
use rustmage::{Action, Card, Config, DeckRules, Game, Player};
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
    let simic = deck(
        &[("Forest", 12), ("Island", 12)],
        &[
            "Hulking Raptor",
            "Toadstool Admirer",
            "Unflinching Hortimancer",
            "Wrecking Gecko",
            "Archive Dragon",
            "Llanowar Elves",
            "It'll Quench Ya!",
            "Don't Make a Sound",
            "Divert Disaster",
        ],
    );
    let jeskai = deck(
        &[("Island", 10), ("Mountain", 8), ("Plains", 6)],
        &[
            "Shock",
            "Fanatical Firebrand",
            "It'll Quench Ya!",
            "Spell Pierce",
            "No More Lies",
            "Diversion Unit",
            "Archive Dragon",
            "Healer's Hawk",
            "Fear of Failed Tests",
        ],
    );
    let (
        mut decisions,
        mut requests,
        mut wards,
        mut pays,
        mut declines,
        mut mana,
        mut replays,
        mut max_bytes,
    ) = (0, 0, 0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [simic.clone(), jeskai.clone()],
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
            if before.decision == "resolution_mana" && seed % 3 == 0 {
                action = Action::DeclineResolutionMana;
            }
            pays += usize::from(matches!(action, Action::PayResolutionMana { .. }));
            declines += usize::from(matches!(action, Action::DeclineResolutionMana));
            mana += usize::from(
                before.decision == "resolution_mana"
                    && matches!(action, Action::TapMana { .. } | Action::ActivateMana { .. }),
            );
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(actor);
            if before.resolution_payment.is_none() && after.resolution_payment.is_some() {
                requests += 1;
                wards += usize::from(
                    after
                        .resolving
                        .as_ref()
                        .is_some_and(|s| s.related_stack.is_some()),
                );
            }
            if before.resolution_payment.is_some()
                || after.resolution_payment.is_some()
                || before.announcing.is_some()
                || before
                    .waiting_triggers
                    .iter()
                    .any(|s| s.related_stack.is_some())
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
        assert!(requests > 0 && wards > 0 && pays > 0 && declines > 0 && mana > 0 && replays > 0);
    }
    println!(
        "Completed {games} Standard ward/payment games, {decisions} decisions, {requests} requests, {wards} ward requests, {pays} payments, {declines} declines, {mana} resolution mana activations, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
