use rustmage::{Action, Card, Config, DeckRules, Game, Player, Step, Target};

#[test]
fn bouncing_only_declared_attacker_does_not_skip_blockers() {
    // Exercise the review reproduction using only public gameplay APIs: shuffled
    // decks, real land drops, mana, casting, attack declaration and a response.
    let mut game = Game::new(Config {
        decks: [
            [vec![Card::Forest; 24], vec![Card::GrizzlyBears; 36]].concat(),
            [
                vec![Card::Island; 24],
                vec![Card::from_name("Unsummon").unwrap(); 36],
            ]
            .concat(),
        ],
        seed: 42,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    let mut cast_bear = false;
    let mut bounced = false;
    for _ in 0..1000 {
        let player = game.actor().expect("no early terminal result");
        let obs = game.observe(player);
        if bounced && obs.step != Step::Attackers {
            assert_eq!(obs.step, Step::Blockers);
            assert!(obs.attackers.is_empty());
            assert!(obs.actions.contains(&Action::FinishBlockers));
            return;
        }
        let actions = &obs.actions;
        let action = actions.iter().find(|a| matches!(a, Action::Keep | Action::FinishOpeningHand))
            .or_else(|| actions.iter().find(|a| matches!(a, Action::PlayLand { .. })))
            .or_else(|| {
                let need_mana = (player == Player::P0 && !cast_bear && obs.step == Step::Main1)
                    || (player == Player::P1 && !bounced && obs.step == Step::Attackers && !obs.attackers.is_empty());
                need_mana.then(|| actions.iter().find(|a| matches!(a, Action::TapMana { .. }))).flatten()
            })
            .or_else(|| actions.iter().find(|a| player == Player::P0 && !cast_bear && matches!(a, Action::Cast { .. })))
            .or_else(|| actions.iter().find(|a| player == Player::P1 && !bounced && obs.step == Step::Attackers
                && matches!(a, Action::Cast { target: Some(Target::Card(id)), .. } if obs.attackers.contains(id))))
            .or_else(|| actions.iter().find(|a| matches!(a, Action::Attack { .. })))
            .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishAttackers | Action::FinishBlockers | Action::Discard { .. })))
            .or_else(|| actions.iter().find(|a| matches!(a, Action::Pass)))
            .expect("policy has a legal action").clone();
        if matches!(action, Action::Cast { .. }) {
            if player == Player::P0 {
                cast_bear = true;
            } else {
                assert_eq!(obs.attackers.len(), 1);
                bounced = true;
            }
        }
        game.apply(player, action).unwrap();
    }
    panic!("did not reach post-bounce blockers step");
}
