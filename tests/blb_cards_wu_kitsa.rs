mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn activate(game: &mut Game, source: CardId, ability: u16, target: Option<Target>) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        act(game, Action::ChooseTarget { target });
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn kitsas_prowess_enables_its_copy_guard_and_its_other_ability_loots() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kitsa = put(
        &mut game,
        player,
        "Kitsa, Otterball Elite",
        Zone::Battlefield,
    );
    let draw = put(&mut game, player, "Divination", Zone::Hand);
    assert!(
        game.object_view(kitsa)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    let copy = Action::AnnounceActivate {
        card: kitsa,
        ability: 1,
        counter_payment: Counters::default(),
    };
    reject(&mut game, copy.clone());

    let mut loot = game.clone();
    let next = put(&mut loot, player, "Opt", Zone::Library);
    activate(&mut loot, kitsa, 0, None);
    act(&mut loot, Action::Pass);
    act(&mut loot, Action::Pass);
    act(&mut loot, Action::Discard { card: next });
    settle(&mut loot);
    assert!(loot.object_view(kitsa).tapped);
    assert_eq!(loot.object_view(next).zone, Zone::Graveyard);
    assert_eq!(loot.state.players[player.index()].cards_drawn_this_turn, 1);

    let first = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    cast(&mut game, first, None);
    settle(&mut game);
    assert_eq!(stats(&game, kitsa), (2, 4));
    cast(&mut game, draw, None);
    reject(&mut game, copy);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(stats(&game, kitsa), (3, 5));
    activate(&mut game, kitsa, 1, Some(Target::Spell(draw)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
    assert_eq!(stats(&game, kitsa), (3, 5));
    assert!(game.object_view(kitsa).tapped);
}
