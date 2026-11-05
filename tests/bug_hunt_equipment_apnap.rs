mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

fn reach(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("attachment fixture did not reach its decision");
}

fn dain_attachments(two_per_player: bool) -> (Game, ObjectRef, [Vec<ObjectRef>; 2]) {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Bard, King of Dale", Zone::Battlefield);
    put(&mut game, player, "High Fae Trickster", Zone::Battlefield);
    if two_per_player {
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
    }
    let dain = put(&mut game, player, "Dáin Ironfoot", Zone::Hand);
    cast(&mut game, dain, None);
    let choose_dain = Action::ChooseTarget {
        target: Target::Card(dain),
    };
    reach(&mut game, |game| {
        game.legal_actions().contains(&choose_dain)
    });
    act(&mut game, choose_dain);
    let recipient = game.object_ref(dain);
    let axes: Vec<_> = game
        .observe(player)
        .objects
        .iter()
        .filter(|object| object.name == "Axe" && object.zone == Zone::Battlefield)
        .map(|object| object.reference)
        .collect();
    assert_eq!(axes.len(), if two_per_player { 4 } else { 2 });
    let (donated, retained) = axes.split_at(axes.len() / 2);
    for &axe in donated {
        assert_eq!(game.actor(), Some(player));
        let offering = put(&mut game, player, "Harmless Offering", Zone::Hand);
        announce(
            &mut game,
            offering,
            &[Target::Player(player.other()), Target::Card(axe.card)],
        );
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.object_view(axe.card).reference, axe);
        assert_eq!(game.object_view(axe.card).controller, player.other());
    }
    assert!(
        axes.iter()
            .all(|axe| game.object_view(axe.card).attached_to.is_none())
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    (game, recipient, [retained.to_vec(), donated.to_vec()])
}

#[test]
fn donated_axe_attaches_with_the_rest_of_the_batch() {
    let (game, dain, equipment) = dain_attachments(false);
    assert_eq!(game.state.pending, Pending::Priority);
    for axe in equipment.into_iter().flatten() {
        assert_eq!(game.object_view(axe.card).attached_to, Some(dain));
    }
    assert_eq!(game.object_view(dain.card).power, 3);
}

#[test]
fn each_controller_orders_their_own_equipment_before_any_attachment_commits() {
    let (mut game, dain, [active, other]) = dain_attachments(true);
    let player = Player::P0;
    let order_action = |equipment| Action::ChooseEquipmentAttachment {
        equipment: Some(equipment),
    };
    let first = order_action(active[1]);
    assert_eq!(game.actor(), Some(player));
    let actions = game.legal_actions();
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action, Action::ChooseEquipmentAttachment { .. }))
            .cloned()
            .collect::<Vec<_>>(),
        active.iter().copied().map(order_action).collect::<Vec<_>>()
    );

    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = actions.iter().position(|action| *action == first).unwrap();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, first);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.actor(), Some(player));

    assert_eq!(game.actor(), Some(player.other()));
    let view = game.observe(player.other()).equipment_attachment.unwrap();
    assert_eq!(view.candidates, other);
    assert_eq!(view.prior_public_order, vec![active[1], active[0]]);
    assert!(
        active
            .iter()
            .chain(&other)
            .all(|axe| game.object_view(axe.card).attached_to.is_none())
    );
    reject(&mut game, order_action(active[0]));
    act(&mut game, order_action(other[1]));
    assert_eq!(game.state.pending, Pending::Priority);
    for axe in active.into_iter().chain(other) {
        assert_eq!(game.object_view(axe.card).attached_to, Some(dain));
    }
    assert_eq!(game.object_view(dain.card).power, 5);
}
