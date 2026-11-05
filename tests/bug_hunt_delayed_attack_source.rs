mod support;

use rustmage as _;
use rustmage_core::{cards::Counters, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

#[test]
fn delayed_attack_ability_uses_its_exact_sources_last_battlefield_kinds() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Dalkovan Encampment", Zone::Battlefield);
    let echo = put(
        &mut game,
        player,
        "Echo, Perceptive Prodigy",
        Zone::Battlefield,
    );
    let ascension = put(&mut game, player, "Earthbender Ascension", Zone::Hand);
    let murder = put(&mut game, player.other(), "Murder", Zone::Hand);
    let original_land = game.object_ref(land);
    act(&mut game, activate(land, 1));
    pay(&mut game);
    pass_pair(&mut game);
    let never_animated = game.clone();

    cast(&mut game, ascension, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    pass_pair(&mut game);
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(land).reference, original_land);
    assert!(game.object_view(land).is_creature());

    let mut departed = game.clone();
    act(&mut departed, Action::Pass);
    cast(&mut departed, murder, Some(Target::Card(land)));
    settle(&mut departed);
    assert_eq!(departed.object_view(land).zone, Zone::Battlefield);
    assert_ne!(departed.object_view(land).reference, original_land);
    assert!(!departed.object_view(land).is_creature());

    for (mut branch, can_copy) in [(never_animated, false), (game, true), (departed, true)] {
        branch.state.step = Step::Attackers;
        branch.state.pending = Pending::Attackers;
        branch.state.priority = player;
        act(&mut branch, Action::Attack { card: echo });
        act(&mut branch, Action::FinishAttackers);
        assert_eq!(branch.state.stack.len(), 1);
        assert_eq!(branch.state.stack[0].source.unwrap().object, original_land);
        let delayed = branch.state.stack[0].id;
        let action = activate(echo, 0);
        assert_eq!(branch.legal_actions().contains(&action), can_copy);
        if can_copy {
            act(&mut branch, action);
            act(
                &mut branch,
                Action::ChooseTarget {
                    target: Target::Stack(delayed),
                },
            );
            pay(&mut branch);
            settle(&mut branch);
            let warriors = branch
                .observe(player)
                .objects
                .iter()
                .filter(|object| {
                    object.zone == Zone::Battlefield
                        && object.is_token()
                        && object.has_subtype("Warrior")
                })
                .count();
            assert_eq!(warriors, 4);
        }
    }
}
