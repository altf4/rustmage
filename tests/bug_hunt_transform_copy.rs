mod support;
use rustmage as _;
use rustmage_core::{cards::Counters, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn copy_transform(game: &mut Game, echo: CardId, original: u64) {
    act(
        game,
        Action::AnnounceActivate {
            card: echo,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Stack(original),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
    pass_pair(game);
    assert_eq!(game.state.stack.len(), 2);
    assert!(game.state.stack[1].is_copy);
    assert_eq!(game.state.stack[0].id, original);
}

#[test]
fn later_ability_copy_uses_its_own_transform_timestamp() {
    let p = Player::P0;
    let mut g = scenario(p);
    let echo = put(&mut g, p, "Echo, Perceptive Prodigy", Zone::Battlefield);
    let tony = put(&mut g, p, "Tony Stark", Zone::Battlefield);
    let shore = put(&mut g, p, "Shore Up", Zone::Hand);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: tony,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    let original = g.state.stack[0].id;
    copy_transform(&mut g, echo, original);
    pass_pair(&mut g);
    assert_eq!(g.object_view(tony).name, "The Invincible Iron Man");
    assert!(g.object_view(echo).tapped);

    cast(&mut g, shore, Some(Target::Card(echo)));
    pass_pair(&mut g);
    assert_eq!(g.state.stack.len(), 1);
    assert!(!g.object_view(echo).tapped);
    copy_transform(&mut g, echo, original);
    pass_pair(&mut g);
    assert_eq!(g.object_view(tony).name, "Tony Stark");

    pass_pair(&mut g);
    assert_eq!(g.object_view(tony).name, "Tony Stark");
    assert!(g.state.stack.is_empty());
}
