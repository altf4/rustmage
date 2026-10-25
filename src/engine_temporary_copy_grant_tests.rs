//! Supported paid grant interactions plus an explicitly bounded restricted-grant fixture.
use super::modal_origin_tests::{announce, choose, modes, shock};
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
fn priority(game: &mut Game, player: Player) {
    if game.actor() != Some(player) {
        checked(game, Action::Pass);
    }
}
fn finish_payment(game: &mut Game) {
    for _ in 0..12 {
        if game.state.announcing.is_none() {
            return;
        }
        let action = game
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseAdditionalCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(game, action);
    }
    panic!("grant interaction remained unpaid");
}
fn activation(
    game: &mut Game,
    player: Player,
    source: CardId,
    ability: u16,
    target: Option<CardId>,
) {
    priority(game, player);
    game.state.players[player.index()].mana = [20; 6];
    checked(
        game,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(
            game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    finish_payment(game);
    settle(game);
}
#[test]
fn msh_temporary_copy_bounded_paid_grantor_transform_preserves_grant_but_reattach_renews() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Monica Rambeau", Zone::Battlefield);
        let first = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        let reference = game.object_ref(id);
        checked(
            &mut game,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        finish_payment(&mut game);
        // Bounded copy/attachment before the paid transform resolves. The later
        // equip actions and spell-trigger packets use the ordinary paid engine.
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Black Mage's Rod").unwrap()),
            CopyDeadline::NextTurn {
                player,
                created_turn: game.state.turn,
            },
        );
        assert!(game.attach_equipment(reference, game.object_ref(first)));
        let origin = game.copied_grant_origin(reference).unwrap();
        let intrinsic = game.copy_ability_origin(reference);
        let time = game.temporary_copy_view(reference).unwrap().layers[0].provider_timestamp;
        settle(&mut game);
        assert_eq!(game.copied_grant_origin(reference), Some(origin));
        assert_eq!(game.copy_ability_origin(reference), intrinsic);
        assert!(game.temporary_copy_view(reference).unwrap().layers[0].provider_timestamp > time);
        shock(&mut game, player);
        assert!(
            game.state
                .stack
                .iter()
                .chain(&game.state.pending_triggers)
                .any(|item| item.attachment_trigger.is_some()
                    && item.source_ability_origin == Some(origin))
        );
        settle(&mut game);
        activation(&mut game, player, id, 0, Some(first));
        assert_eq!(game.copied_grant_origin(reference), Some(origin));
        activation(&mut game, player, id, 0, Some(second));
        let second_origin = game.copied_grant_origin(reference).unwrap();
        assert_ne!(second_origin, origin);
        activation(&mut game, player, id, 0, Some(first));
        assert_ne!(game.copied_grant_origin(reference), Some(second_origin));
        assert_eq!(game.copy_ability_origin(reference), intrinsic);
    }
}
#[test]
fn msh_temporary_copy_bounded_restored_intrinsic_and_external_grant_origins_are_independent() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let rod = named(&mut game, player, "Black Mage's Rod", Zone::Battlefield);
        let recipient = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        activation(&mut game, player, rod, 0, Some(recipient));
        let reference = game.object_ref(rod);
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Black Mage's Rod").unwrap()),
            CopyDeadline::NextTurn {
                player,
                created_turn: game.state.turn,
            },
        );
        let intrinsic = game.copy_ability_origin(reference);
        let old_grant = game.copied_grant_origin(reference);
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Arc Reactor").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(game.copied_grant_origin(reference), None);
        assert!(game.expire_temporary_copies(true));
        assert_eq!(game.copy_ability_origin(reference), intrinsic);
        let restored = game.copied_grant_origin(reference);
        assert!(restored.is_some());
        assert_ne!(restored, old_grant);
        game.state.turn += 1;
        game.state.active = player;
        assert!(game.expire_temporary_copies(false));
        assert!(game.temporary_copy_view(reference).is_none());
        assert_eq!(game.copy_ability_origin(reference), None);
        let base = game.copied_grant_origin(reference);
        assert!(base.is_some());
        assert_ne!(base, restored);
        assert_eq!(game.object_view(rod).provided_ability_origin, base);
        let mut child = game.clone();
        drop(game);
        shock(&mut child, player);
        assert!(
            child
                .state
                .stack
                .iter()
                .chain(&child.state.pending_triggers)
                .any(|item| item.card == recipient
                    && item.attachment_trigger.is_some()
                    && item.source_ability_origin == base)
        );
        settle(&mut child);
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_pole_grant_uses_grantor_origin_and_old_queued_context() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let pole = named(&mut game, player, "Fishing Pole", Zone::Battlefield);
        let recipient = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        activation(&mut game, player, pole, 0, Some(recipient));
        let pole_ref = game.object_ref(pole);
        game.apply_temporary_copy(
            game.object_ref(recipient),
            CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        game.apply_temporary_copy(
            pole_ref,
            CopyRecipe::new(Card::from_name("Fishing Pole").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let grant_origin = game.copied_grant_origin(pole_ref);
        assert_ne!(
            grant_origin,
            game.copy_ability_origin(game.object_ref(recipient))
        );
        priority(&mut game, player);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut game,
            Action::AnnounceAttachedAbility {
                grantor: pole_ref,
                ability: 1,
            },
        );
        assert_eq!(
            game.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source_ability_origin,
            grant_origin
        );
        finish_payment(&mut game);
        let packet = game.state.stack.last().unwrap();
        assert_eq!(packet.source_ability_origin, grant_origin);
        assert_eq!(
            packet.attachment_trigger.as_ref().unwrap().origin,
            grant_origin
        );
        // Bounded layer-one event while this actual paid ability is queued.
        game.apply_temporary_copy(
            pole_ref,
            CopyRecipe::new(Card::from_name("Fishing Pole").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_ne!(game.copied_grant_origin(pole_ref), grant_origin);
        settle(&mut game);
        assert_eq!(game.counter_count(pole, CounterKind::Bait), 1);
        assert_eq!(game.counter_count(recipient, CounterKind::Bait), 0);
        assert!(game.state.objects[pole as usize].tapped);
        assert!(game.state.objects[recipient as usize].tapped);
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_rod_packet_keeps_old_program_after_grantor_copy_changes() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let rod = named(&mut game, player, "Black Mage's Rod", Zone::Battlefield);
        let recipient = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        activation(&mut game, player, rod, 0, Some(recipient));
        let reference = game.object_ref(rod);
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Black Mage's Rod").unwrap()),
            CopyDeadline::NextTurn {
                player,
                created_turn: game.state.turn,
            },
        );
        let old = game.copied_grant_origin(reference);
        shock(&mut game, player);
        let packet = game
            .state
            .stack
            .iter()
            .chain(&game.state.pending_triggers)
            .chain(game.state.announcing.iter().map(|a| &a.item))
            .find(|item| item.attachment_trigger.is_some())
            .unwrap()
            .clone();
        assert_eq!(packet.source_ability_origin, old);
        assert_eq!(packet.trigger_definition().name(), "Black Mage's Rod");
        game.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Red Mage's Rapier").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let new = game.copied_grant_origin(reference);
        assert_ne!(old, new);
        let mut child = game.clone();
        drop(game);
        settle(&mut child);
        assert_eq!(child.state.players[player.other().index()].life, 17);
        assert_eq!(child.state.objects[recipient as usize].growth.power, 0);
        shock(&mut child, player);
        assert!(
            child
                .state
                .stack
                .iter()
                .chain(&child.state.pending_triggers)
                .any(|item| item.card == recipient
                    && item.attachment_trigger.is_some()
                    && item.source_ability_origin == new)
        );
        settle(&mut child);
        assert_eq!(child.state.players[player.other().index()].life, 15);
        assert_eq!(child.state.objects[recipient as usize].growth.power, 2);
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_malice_grant_returns_physical_card_after_copied_recipient_dies()
{
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let id = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        let object = game.object_ref(id);
        let malice = named(&mut game, player, "Undying Malice", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
        cast(&mut game, malice, None, &[Target::Card(id)]);
        settle(&mut game);
        game.apply_temporary_copy(
            object,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        announce(&mut game, player, "Murder", &[Target::Card(id)]);
        settle(&mut game);
        assert!(game.on_battlefield(id));
        assert_eq!(game.object_view(id).name(), "Hill Giant");
        assert_ne!(game.object_ref(id), object);
        assert!(game.state.objects[id as usize].tapped);
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 1);
        assert!(game.state.temporary_copies.is_none());
    }
}
#[test]
fn msh_temporary_copy_bounded_paid_hall_resolution_grant_survives_recipient_copy_and_control_change()
 {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let hall = named(
            &mut game,
            player,
            "Great Hall of the Biblioplex",
            Zone::Battlefield,
        );
        activation(&mut game, player, hall, 2, None);
        let original = game.state.temporary_abilities.clone();
        game.apply_temporary_copy(
            game.object_ref(hall),
            CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        shock(&mut game, player);
        settle(&mut game);
        assert_eq!(game.state.objects[hall as usize].growth.power, 1);
        assert_eq!(game.state.temporary_abilities, original);
        assert!(
            original
                .as_ref()
                .unwrap()
                .iter()
                .all(|grant| grant.origin.is_none())
        );
        game.state.objects[hall as usize].controller = player.other();
        shock(&mut game, player.other());
        settle(&mut game);
        assert_eq!(game.state.objects[hall as usize].growth.power, 2);
        assert_eq!(game.state.temporary_abilities, original);
    }
}
#[test]
fn msh_temporary_copy_bounded_restricted_grant_history_is_not_recipient_copy_history() {
    fn bounded_event(
        game: &mut Game,
        player: Player,
        recipient: CardId,
        grantor: CardId,
        origin: u64,
    ) {
        shock(game, player);
        // The pool has no provider granting a choose-once program. Inject only
        // this bounded grant packet, cloned from the actual paid cast's owned
        // context; the real modal announcement, history and resolution follow.
        let mut item = game.state.stack.last().unwrap().clone();
        item.id = game.next_stack_id();
        item.card = recipient;
        item.object = game.object_ref(recipient);
        item.source_card = game.state.objects[recipient as usize].card;
        item.kind = StackKind::TriggeredAbility { ability: 0 };
        item.targets.clear();
        item.cast = None;
        item.mode = None;
        item.source_ability_origin = Some(origin);
        item.attachment_trigger = Some(Box::new(AttachmentTriggerView {
            grantor: game.object_ref(grantor),
            granting_card: Card::from_name("The Vision").unwrap(),
            recipient: game.object_ref(recipient),
            defender: None,
            granted: true,
            origin: Some(origin),
        }));
        game.state.pending_triggers.push(item);
    }
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let recipient = named(&mut game, player, "Hill Giant", Zone::Battlefield);
        let grantor = named(&mut game, player, "Bear Cub", Zone::Battlefield);
        let origin = game.next_role_timestamp();
        // Bounded restricted-grant fixture using existing Vision modal bytecode.
        // No playable card is claimed to grant this program or added to admission.
        game.state.temporary_abilities = Some(Box::new(vec![TemporaryAbilityView {
            recipient: game.object_ref(recipient),
            grantor: game.object_ref(grantor),
            granting_card: Card::from_name("The Vision").unwrap(),
            ability: 0,
            grantor_controller: player,
            expires_turn: game.state.turn,
            permanent: true,
            timestamp: origin,
            origin: Some(origin),
        }]));
        bounded_event(&mut game, player, recipient, grantor, origin);
        choose(&mut game, 0);
        game.apply_temporary_copy(
            game.object_ref(recipient),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        bounded_event(&mut game, player, recipient, grantor, origin);
        assert_eq!(modes(&mut game), vec![1, 2]);
        choose(&mut game, 1);
        assert!(
            game.state
                .modal_history
                .as_ref()
                .unwrap()
                .iter()
                .any(|row| row.object.card == recipient
                    && row.origin == Some(origin)
                    && row.modes == [0, 1])
        );
        game.state.objects[recipient as usize].controller = player.other();
        bounded_event(&mut game, player.other(), recipient, grantor, origin);
        assert_eq!(modes(&mut game), vec![2]);
        choose(&mut game, 2);
    }
}
