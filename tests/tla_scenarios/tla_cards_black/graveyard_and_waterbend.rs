use super::*;

#[test]
fn foggy_swamp_visions_exiles_exactly_x_creatures_and_copies_them_until_its_own_end_step() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Foggy Swamp Visions", Zone::Hand);
    let bear = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let giant = put(&mut game, p.other(), "Hill Giant", Zone::Graveyard);
    let land = put(&mut game, p.other(), "Swamp", Zone::Graveyard);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    game.state.players[p.index()].mana = [0, 0, 2, 0, 0, 2];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, Action::ChooseX { value: 3 });
    let mut zero = game.clone();
    act(&mut zero, Action::ChooseX { value: 0 });
    pay(&mut zero);
    settle(&mut zero);
    assert_eq!(zero.object_view(bear).zone, Zone::Graveyard);
    assert!(zero.observe(p).delayed_sacrifices.is_empty());

    act(&mut game, Action::ChooseX { value: 2 });
    act(&mut game, target(bear));
    reject(&mut game, target(bear));
    reject(&mut game, target(land));
    reject(&mut game, Action::FinishTargets);
    act(&mut game, target(giant));
    act(&mut game, Action::BeginWaterbend);
    let object = game.object_ref(artifact);
    act(&mut game, Action::ChooseWaterbender { object });
    pay(&mut game);
    settle(&mut game);
    let bear_card = game.object_view(bear).card;
    let giant_card = game.object_view(giant).card;
    for (id, card) in [(bear, bear_card), (giant, giant_card)] {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
        assert_eq!(token_count(&game, card), 1);
    }
    start_end_step(&mut game, p.other());
    settle(&mut game);
    assert_eq!(token_count(&game, bear_card), 1);
    start_end_step(&mut game, p);
    settle(&mut game);
    assert_eq!(
        token_count(&game, bear_card) + token_count(&game, giant_card),
        0
    );
}

#[test]
fn ruinous_waterbending_shrinks_all_creatures_and_counts_deaths_only_if_paid() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Ruinous Waterbending", Zone::Hand);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Goblin Piker", Zone::Battlefield);
    let survivor = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let life = game.state.players[p.index()].life;
    act(&mut game, Action::AnnounceCast { card: spell });
    for (paid, gain) in [(false, 0), (true, 2)] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseWaterbendCost { pay: paid });
        pay(&mut branch);
        finish_ordered_triggers(&mut branch);
        assert_eq!(branch.object_view(own).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(stats(&branch, survivor), (1, 1));
        assert_eq!(branch.state.players[p.index()].life, life + gain);
    }
}

#[test]
fn sold_out_creates_a_clue_for_damage_to_the_exiled_incarnation() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Sold Out", Zone::Hand);
    let damage = put(&mut game, p, "Shock", Zone::Hand);
    let giant = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, damage, Some(Target::Card(giant)));
    settle(&mut game);
    game.state.objects[giant as usize].damage = 0;
    let mut blinked = game.clone();
    blinked.move_object(giant, Zone::Exile);
    blinked.move_object(giant, Zone::Battlefield);
    cast(&mut blinked, spell, Some(Target::Card(giant)));
    settle(&mut blinked);
    assert_eq!(blinked.object_view(giant).zone, Zone::Exile);
    assert_eq!(token_count(&blinked, Card(5296)), 0);

    cast(&mut game, spell, Some(Target::Card(giant)));
    settle(&mut game);
    assert_eq!(game.object_view(giant).zone, Zone::Exile);
    assert_eq!(token_count(&game, Card(5296)), 1);
}

#[test]
fn wolfbat_returns_for_black_on_the_second_draw_and_has_finality() {
    let p = Player::P1;
    let mut game = scenario(p.other());
    act(&mut game, Action::Pass);
    let wolfbat = put(&mut game, p, "Wolfbat", Zone::Graveyard);
    let other = put(&mut game, p.other(), "Wolfbat", Zone::Graveyard);
    let draw = put(&mut game, p, "Quick Study", Zone::Hand);
    cast(&mut game, draw, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.object_view(wolfbat).zone, Zone::Graveyard);
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [1, 0, 0, 0, 0, 0],
        },
    );
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 1, 0, 0, 0],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
    assert_eq!(game.object_view(wolfbat).zone, Zone::Battlefield);
    assert_eq!(stats(&game, wolfbat), (2, 2));
    assert!(game.keywords(wolfbat).contains(Keywords::FLYING));
    assert_eq!(game.counter_count(wolfbat, CounterKind::Finality), 1);
    game.move_object(wolfbat, Zone::Graveyard);
    assert_eq!(game.object_view(wolfbat).zone, Zone::Exile);
}
