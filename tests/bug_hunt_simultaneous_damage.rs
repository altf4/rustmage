mod support;
use rustmage as _;
use rustmage_core::{cards::CounterKind, engine::*};
use support::*;

fn resolve_howl(gift: bool, target_creature: bool, shield: bool) -> (Game, CardId) {
    let p = Player::P0;
    let mut game = scenario(p);
    let invigorator = put(
        &mut game,
        p.other(),
        "Hexhaven Invigorator",
        Zone::Battlefield,
    );
    let howl = put(&mut game, p, "Wildfire Howl", Zone::Hand);
    if shield {
        game.add_counter(invigorator, CounterKind::Shield, 1);
    }
    act(&mut game, Action::AnnounceCast { card: howl });
    act(
        &mut game,
        Action::ChooseGift {
            beneficiary: gift.then_some(p.other()),
        },
    );
    if gift {
        act(
            &mut game,
            Action::ChooseTarget {
                target: if target_creature {
                    Target::Card(invigorator)
                } else {
                    Target::Player(p.other())
                },
            },
        );
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(howl).zone, Zone::Graveyard);
    (game, invigorator)
}

#[test]
fn wildfire_howl_deals_one_damage_event_after_either_gift_choice() {
    for (gift, target_creature, shield, expected) in [
        (true, true, false, 3),
        (true, false, false, 2),
        (false, false, false, 2),
        (true, true, true, 0),
    ] {
        let (game, invigorator) = resolve_howl(gift, target_creature, shield);
        assert_eq!(game.object_view(invigorator).damage, expected);
        assert_eq!(game.counter_count(invigorator, CounterKind::Shield), 0);
        assert_eq!(
            game.state.players[Player::P1.index()].life,
            if gift && !target_creature { 19 } else { 20 }
        );
        let amounts: Vec<_> = game
            .state
            .stack
            .iter()
            .chain(&game.state.ordering_triggers)
            .chain(&game.state.pending_triggers)
            .filter(|packet| packet.source.is_some_and(|source| source.id == invigorator))
            .map(|packet| packet.damage_received.as_ref().unwrap().amount)
            .collect();
        assert_eq!(
            amounts,
            if expected == 0 {
                vec![]
            } else {
                vec![expected as u128]
            }
        );
    }
}
