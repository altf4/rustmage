mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn outrageous_robbery_offers_a_cast_fundable_by_off_color_lands() {
    let p = Player::P0;
    let mut game = scenario(p);
    let robbery = put(&mut game, p, "Outrageous Robbery", Zone::Hand);
    let angel = put(&mut game, p.other(), "Serra Angel", Zone::Library);
    let lands: Vec<_> = (0..5)
        .map(|_| put(&mut game, p, "Island", Zone::Battlefield))
        .collect();
    game.state.players[p.index()].mana = [0, 0, 3, 0, 0, 0];

    act(&mut game, Action::AnnounceCast { card: robbery });
    act(&mut game, Action::ChooseX { value: 1 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(angel).zone, Zone::Exile);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);

    cast_with_untapped_islands(game, p, angel, lands);
}

#[test]
fn threshold_linked_exile_is_also_fundable_by_off_color_lands() {
    let p = Player::P0;
    let mut game = scenario(p);
    let summoner = put(&mut game, p, "Null Summoner", Zone::Hand);
    let angel = put(&mut game, p.other(), "Serra Angel", Zone::Hand);
    let lands: Vec<_> = (0..5)
        .map(|_| put(&mut game, p, "Island", Zone::Battlefield))
        .collect();
    for _ in 0..7 {
        put(&mut game, p, "Forest", Zone::Graveyard);
    }
    game.state.players[p.index()].mana = [0, 1, 1, 0, 0, 2];
    cast(&mut game, summoner, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    let object = Some(game.object_ref(angel));
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishTargets)
        });
    }
    act(&mut game, Action::ChooseHandCard { object });
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
    cast_with_untapped_islands(game, p, angel, lands);
}

fn cast_with_untapped_islands(mut game: Game, p: Player, angel: CardId, lands: Vec<CardId>) {
    let announce = Action::AnnounceExile { card: angel };
    assert!(lands.iter().all(|id| !game.object_view(*id).tapped));
    let mut four_islands = game.clone();
    four_islands.state.objects[lands[0] as usize].tapped = true;
    reject(&mut four_islands, announce.clone());

    act(&mut game, announce);
    for land in lands {
        act(&mut game, Action::TapMana { card: land });
    }
    assert_eq!(game.state.players[p.index()].mana, [0, 5, 0, 0, 0, 0]);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(angel).zone, Zone::Battlefield);
    assert_eq!(game.object_view(angel).controller, p);
}
