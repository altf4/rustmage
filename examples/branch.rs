//! A minimal search substrate example: compare root-action rollouts on independent
//! clones. This is privileged perfect-information planning, not fair Magic MCTS.
use rustmage::{Action, Card, Config, DeckRules, Game, Outcome, Player};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let deck = [vec![Card::Mountain; 24], vec![Card::LightningBolt; 36]].concat();
    let mut game = Game::new(Config {
        decks: [deck.clone(), deck],
        seed: 42,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    })?;
    game.apply(Player::P0, Action::Keep)?;
    game.apply(Player::P1, Action::Keep)?;
    let root = game.snapshot();
    let actor = game.actor().unwrap();
    for action in game
        .legal_actions()
        .into_iter()
        .filter(|a| *a != Action::Concede)
    {
        let mut branch = game.clone();
        branch.apply(actor, action.clone())?;
        let mut actions = Vec::new();
        for _ in 0..10_000 {
            if branch.outcome().is_some() {
                break;
            }
            branch.legal_actions_into(&mut actions);
            let i = rustmage::policy::smoke_index(&branch, &actions);
            branch.apply(branch.actor().unwrap(), actions[i].clone())?;
        }
        let value = match branch.outcome() {
            Some(Outcome::Winner(p)) if p == actor => Some(1),
            Some(Outcome::Winner(_)) => Some(-1),
            Some(Outcome::Draw) => Some(0),
            None => None, // Budget cutoff, not a fabricated terminal loss.
        };
        println!("{action:?}: {value:?}");
        assert_eq!(game.snapshot(), root);
    }
    game.restore(&root);
    Ok(())
}
