//! Supported Standard self-play with step-event and suspended-choice replay.
use rustmage::{Action, Card, Config, DeckRules, Game, Player, StackKind};
use std::collections::BTreeSet;
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
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let selesnya = deck(
        &[("Plains", 12), ("Forest", 12)],
        &[
            "Cautious Survivor",
            "Shrewd Storyteller",
            "Veteran Beastrider",
            "Flight-Deck Coordinator",
            "Loxodon Battle Priest",
            "Might of the Ancestors",
            "Anthem of Champions",
            "Llanowar Elves",
            "Surrak, the Hunt Caller",
        ],
    );
    let rakdos = deck(
        &[("Swamp", 12), ("Mountain", 12)],
        &[
            "Phyrexian Arena",
            "Bitterbloom Bearer",
            "Searslicer Goblin",
            "Frontline War-Rager",
            "Battle-Rattle Shaman",
            "Sinister Monolith",
            "Moonglove Extractor",
            "Shock",
            "Dragonmaster Outcast",
        ],
    );
    let step_cards = [
        "Cautious Survivor",
        "Shrewd Storyteller",
        "Veteran Beastrider",
        "Flight-Deck Coordinator",
        "Loxodon Battle Priest",
        "Might of the Ancestors",
        "Surrak, the Hunt Caller",
        "Phyrexian Arena",
        "Bitterbloom Bearer",
        "Searslicer Goblin",
        "Frontline War-Rager",
        "Battle-Rattle Shaman",
        "Sinister Monolith",
        "Dragonmaster Outcast",
    ];
    let mut decisions = 0;
    let mut triggers = 0;
    let mut replays = 0;
    let mut raid_turns = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [selesnya.clone(), rakdos.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut seen = BTreeSet::new();
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            let before = game.observe(actor);
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            let saved = game.snapshot();
            game.apply(actor, action.clone())?;
            let after = game.observe(actor);
            let mut new_triggers = 0;
            for item in after
                .stack
                .iter()
                .chain(&after.waiting_triggers)
                .chain(after.announcing.iter())
                .chain(after.resolving.iter())
            {
                if matches!(item.kind, StackKind::TriggeredAbility { .. })
                    && step_cards.contains(&item.source_card.name())
                    && seen.insert(item.id)
                {
                    new_triggers += 1;
                }
            }
            triggers += new_triggers;
            if before.step != after.step
                || new_triggers > 0
                || matches!(
                    action,
                    Action::PutTrigger { .. }
                        | Action::ChooseOptional { .. }
                        | Action::ChooseTarget { .. }
                )
            {
                let expected = game.snapshot();
                game.restore(&saved).unwrap();
                let mut fork = game.clone();
                fork.apply(actor, action.clone())?;
                game.apply(actor, action.clone())?;
                assert_eq!(game.snapshot(), fork.snapshot());
                assert_eq!(game.snapshot(), expected);
                game.restore(&saved).unwrap();
                game.apply(actor, action)?;
                assert_eq!(game.snapshot(), expected);
                replays += 1;
            }
            raid_turns += usize::from(
                !before.players[actor.index()].attacked_this_turn
                    && after.players[actor.index()].attacked_this_turn,
            );
            decisions += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without a terminal result".into());
        }
    }
    assert!(triggers > 0 && replays > 0 && raid_turns > 0);
    println!(
        "Completed {games} step-trigger games, {decisions} decisions, {triggers} observed step triggers, {replays} cloned/replayed boundaries/choices, {raid_turns} committed attack turns; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
