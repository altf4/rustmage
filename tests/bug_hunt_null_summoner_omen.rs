mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn null_summoner_can_cast_the_exiled_card_as_an_omen() {
    let p = Player::P0;
    let mut game = scenario(p);
    let summoner = put(&mut game, p, "Null Summoner", Zone::Hand);
    let omen = put(&mut game, p.other(), "Riling Dawnbreaker", Zone::Hand);
    for _ in 0..7 {
        put(&mut game, p, "Island", Zone::Graveyard);
    }

    cast(&mut game, summoner, None);
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            ) || matches!(a, Action::ChooseTarget { target: Target::Player(target) } if *target == p.other())
        });
    }
    let object = Some(game.object_ref(omen));
    act(&mut game, Action::ChooseHandCard { object });
    settle(&mut game);

    assert_eq!(game.object_view(summoner).zone, Zone::Battlefield);
    assert_eq!(game.object_view(omen).zone, Zone::Exile);
    let mut insufficient_threshold = game.clone();
    let departed = insufficient_threshold
        .observe(p)
        .objects
        .into_iter()
        .find(|o| o.owner == p && o.zone == Zone::Graveyard)
        .unwrap()
        .id;
    insufficient_threshold.move_object(departed, Zone::Exile);
    reject(
        &mut insufficient_threshold,
        Action::AnnounceOmen { card: omen },
    );
    reject(
        &mut insufficient_threshold,
        Action::AnnounceExile { card: omen },
    );
    let legal = game.legal_actions();
    assert!(
        legal.contains(&Action::AnnounceExile { card: omen }),
        "{legal:?}"
    );
    assert!(
        legal.contains(&Action::AnnounceOmen { card: omen }),
        "{legal:?}"
    );
    act(&mut game, Action::AnnounceOmen { card: omen });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(omen).zone, Zone::Library);
    assert!(
        game.state.players[p.other().index()]
            .library
            .contains(&omen)
    );
}
