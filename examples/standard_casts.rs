//! Supported Standard self-play with cast-event clone/restore and trigger ordering.
use rustmage::{Action, Card, Config, DeckRules, Game, Player};
fn deck(lands: &[(&str, usize)], spells: &[&str]) -> Vec<Card> {
    let mut cards = Vec::new();
    for &(name, n) in lands {
        cards.extend(vec![Card::from_name(name).unwrap(); n]);
    }
    for &name in spells {
        cards.extend([Card::from_name(name).unwrap(); 4]);
    }
    cards
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let izzet = deck(
        &[("Island", 12), ("Mountain", 12)],
        &[
            "Elementalist Adept",
            "Guttersnipe",
            "Firebrand Archer",
            "Unruly Catapult",
            "Malcolm, the Eyes",
            "Crackling Cyclops",
            "Shock",
            "Boltwave",
            "Opt",
        ],
    );
    let boros = deck(
        &[("Plains", 12), ("Mountain", 12)],
        &[
            "Agent of Atlas",
            "Cori Mountain Stalwart",
            "Devoted Duelist",
            "Jeskai Devotee",
            "Poised Practitioner",
            "Thor Odinson",
            "Kree Commandos",
            "Boltwave",
            "Shock",
        ],
    );
    let mut total = 0;
    let mut casts = 0;
    let mut trigger_orders = 0;
    let mut second_spells = 0;
    for seed in 0..games {
        let mut game = Game::new(Config {
            decks: [izzet.clone(), boros.clone()],
            seed: seed as u64,
            starting_player: Player::P0,
            deck_rules: DeckRules::Standard,
        })?;
        let mut actions = Vec::new();
        for _ in 0..100_000 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            game.legal_actions_into(&mut actions);
            let action = actions[rustmage::policy::smoke_index(&game, &actions)].clone();
            trigger_orders += usize::from(matches!(action, Action::PutTrigger { .. }));
            let before = game.observe(actor).players[actor.index()].spells_cast_this_turn;
            let saved = game.snapshot();
            game.apply(actor, action.clone())?;
            let after = game.observe(actor).players[actor.index()].spells_cast_this_turn;
            if after > before {
                assert_eq!(after, before + 1);
                casts += 1;
                second_spells += usize::from(after == 2);
                let expected = game.snapshot();
                game.restore(&saved);
                let mut fork = game.clone();
                fork.apply(actor, action.clone())?;
                game.apply(actor, action.clone())?;
                assert_eq!(game.snapshot(), expected);
                assert_eq!(game.snapshot(), fork.snapshot());
                let expected = game.snapshot();
                game.restore(&saved);
                game.apply(actor, action)?;
                assert_eq!(game.snapshot(), expected);
                total += 1;
                continue;
            }
            total += 1;
        }
        if game.outcome().is_none() {
            return Err("self-play budget reached without terminal result".into());
        }
    }
    assert!(casts > 0 && trigger_orders > 0 && second_spells > 0);
    println!(
        "Completed {games} spell-trigger games, {total} decisions, {casts} cloned/replayed casts, {second_spells} second spells, {trigger_orders} trigger-order choices; catalogue {}",
        rustmage::CATALOGUE_VERSION
    );
    Ok(())
}
