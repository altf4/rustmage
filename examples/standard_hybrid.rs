//! Native supported Standard games using two-color hybrid spells in mono-color decks.
use rustmage::{Action, Card, Config, DeckRules, Game, Player};
fn deck(land: &str, spells: &[&str]) -> Vec<Card> {
    let mut result = vec![Card::from_name(land).unwrap(); 24];
    for &name in spells {
        let c = Card::from_name(name).unwrap();
        assert!(c.is_supported(), "{name}");
        result.extend([c; 4]);
    }
    assert_eq!(result.len(), 60);
    result
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let green = deck(
        "Forest",
        &[
            "Gallant Citizen",
            "Pond Prophet",
            "Head of the Homestead",
            "Large Bear",
            "Duskwatch Hunter",
            "Noggle Robber",
            "Slithering Cryptid",
            "Mary Jane Watson",
            "Attercop",
        ],
    );
    let blue = deck(
        "Island",
        &[
            "Pond Prophet",
            "Mob Lookout",
            "Punk Frogs",
            "Tempest Angler",
            "Abandon Attachments",
            "Cat-Owl",
            "Flying Octobot",
            "Mechan Assembler",
            "Friendly Teddy",
        ],
    );
    let (mut decisions, mut choices, mut shortcuts, mut payments, mut replays, mut max_bytes) =
        (0, 0, 0, 0, 0, 0);
    for seed in 0..games {
        let mut g = Game::new(Config {
            decks: [green.clone(), blue.clone()],
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
            choices += usize::from(matches!(action, Action::ChooseSpellCost { .. }));
            if let Action::Cast { card, .. } | Action::BeginCast { card, .. } = action {
                shortcuts += usize::from(
                    before
                        .objects
                        .iter()
                        .any(|o| o.id == card && o.card.info().mana_cost.contains('/')),
                );
            }
            payments += usize::from(
                before.announcing_cost.is_some() && matches!(action, Action::PayCosts { .. }),
            );
            let saved = g.snapshot();
            g.apply(actor, action.clone())?;
            let after = g.observe(g.actor().unwrap_or(actor));
            if before.decision == "spell_cost"
                || after.decision == "spell_cost"
                || before.announcing_cost.is_some()
                || after.announcing_cost.is_some()
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
        assert!(choices > 0 && payments > 0 && replays > 0);
    }
    println!(
        "Completed {games} supported Standard hybrid games, {decisions} decisions, {choices} committed hybrid choices, {payments} staged hybrid payments, {shortcuts} prefunded hybrid shortcuts, {replays} cloned/replayed boundaries, {max_bytes} maximum initialized state bytes; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
