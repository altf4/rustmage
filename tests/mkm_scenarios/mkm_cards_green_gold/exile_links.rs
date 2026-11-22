use super::*;

const CLUE: u16 = 5296;

#[test]
fn buried_in_the_garden_uses_exact_aura_lifetime_and_gives_the_land_controller_bonus_mana() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let aura = put(&mut game, player, "Buried in the Garden", Zone::Hand);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    cast(&mut game, aura, Some(Target::Card(land)));
    pass_pair(&mut game);
    reject(&mut game, target(land));
    reject(&mut game, target(ally));
    act(&mut game, target(enemy));
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(land))
    );
    let mut removed_before = game.clone();
    removed_before.move_object(aura, Zone::Graveyard);
    settle(&mut removed_before);
    assert_eq!(removed_before.object_view(enemy).zone, Zone::Battlefield);

    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Exile);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    game.state.players[opponent.index()].mana = [0; 6];
    let aura_controllers_mana = game.state.players[player.index()].mana;
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(
        game.state.players[opponent.index()].mana,
        [0, 0, 0, 0, 1, 0]
    );
    reject(
        &mut game,
        Action::ChooseMana {
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    act(
        &mut game,
        Action::ChooseMana {
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(
        game.state.players[opponent.index()].mana,
        [0, 1, 0, 0, 1, 0]
    );
    assert_eq!(
        game.state.players[player.index()].mana,
        aura_controllers_mana
    );
    assert!(game.state.stack.is_empty());

    game.move_object(aura, Zone::Graveyard);
    assert!(!game.advance_entries());
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enemy).controller, opponent);
}

#[test]
fn lazav_attacks_to_link_a_graveyard_card_then_may_copy_only_an_exact_linked_creature() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let lazav = put(&mut game, player, "Lazav, Wearer of Faces", Zone::Hand);
    let donor = put(&mut game, opponent, "Llanowar Elves", Zone::Graveyard);
    let noncreature = put(&mut game, player, "Shock", Zone::Graveyard);
    let unrelated = put(&mut game, player, "Hill Giant", Zone::Exile);
    cast(&mut game, lazav, None);
    settle(&mut game);
    assert_eq!(stats(&game, lazav), (2, 3));
    let original = game.object_ref(lazav);
    game.state.objects[lazav as usize].summoning_sick = false;
    attack(&mut game, &[lazav]);

    let mut spell = game.clone();
    act(&mut spell, target(noncreature));
    settle(&mut spell);
    assert_eq!(spell.object_view(noncreature).zone, Zone::Exile);
    let clue = tokens(&spell, player, CLUE)[0];
    act(&mut spell, activation(clue, 0));
    pay(&mut spell);
    settle(&mut spell);
    assert_eq!(stats(&spell, lazav), (2, 3));

    act(&mut game, target(donor));
    let mut invalid = game.clone();
    invalid.move_object(donor, Zone::Hand);
    settle(&mut invalid);
    assert!(tokens(&invalid, player, CLUE).is_empty());

    settle(&mut game);
    let clue = tokens(&game, player, CLUE)[0];
    assert_eq!(game.object_view(donor).zone, Zone::Exile);
    let exact_donor = game.object_ref(donor);
    act(&mut game, activation(clue, 0));
    pay(&mut game);
    let mut stale = game.clone();
    stale.move_object(donor, Zone::Hand);
    stale.move_object(donor, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stats(&stale, lazav), (2, 3));

    pass_pair(&mut game);
    let object = game.object_ref(unrelated);
    reject(&mut game, Action::ChooseLinkedExile { object });
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert_eq!(stats(&declined, lazav), (2, 3));
    assert_eq!(
        declined.state.players[player.index()].cards_drawn_this_turn,
        1
    );

    act(
        &mut game,
        Action::ChooseLinkedExile {
            object: exact_donor,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_ref(lazav), original);
    assert_eq!(game.object_view(lazav).card, game.object_view(donor).card);
    assert_eq!(stats(&game, lazav), (1, 1));
    assert_eq!(game.object_view(lazav).controller, player);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.finish_cleanup();
    assert_eq!(stats(&game, lazav), (2, 3));
    assert_eq!(game.object_ref(lazav), original);
}
