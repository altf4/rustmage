use super::*;

#[test]
fn gran_gran_loots_on_tap_and_reduces_noncreature_spells_at_three_lessons() {
    let p = Player::P1;
    let mut game = scenario(p);
    let gran = put(&mut game, p, "Gran-Gran", Zone::Battlefield);
    assert!(!game.object_view(gran).card.is_standard_legal());
    let third = put(&mut game, p, "Accumulate Wisdom", Zone::Graveyard);
    for _ in 0..2 {
        put(&mut game, p, "It'll Quench Ya!", Zone::Graveyard);
    }
    put(&mut game, p.other(), "Firebending Lesson", Zone::Graveyard);
    let scroll = put(&mut game, p, "Waterbending Scroll", Zone::Hand);
    let creature = put(&mut game, p, "First-Time Flyer", Zone::Hand);
    game.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    let announce = Action::AnnounceCast { card: scroll };
    let mut short = game.clone();
    short.move_object(third, Zone::Exile);
    reject(&mut short, announce.clone());
    reject(&mut game, Action::AnnounceCast { card: creature });
    act(&mut game, announce);
    pay(&mut game);
    settle(&mut game);

    game.state.pending = Pending::Attackers;
    game.state.step = Step::Attackers;
    act(&mut game, Action::Attack { card: gran });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    let card = game
        .legal_actions()
        .into_iter()
        .find_map(|a| match a {
            Action::Discard { card } => Some(card),
            _ => None,
        })
        .unwrap();
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    act(&mut game, Action::Discard { card });
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
}

#[test]
fn honest_work_changes_the_opponents_creature_and_grants_colorless_mana() {
    let p = Player::P0;
    let mut game = scenario(p);
    let aura = put(&mut game, p, "Honest Work", Zone::Hand);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(
        &mut game,
        p.other(),
        "Malcolm, Alluring Scoundrel",
        Zone::Battlefield,
    );
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    game.add_counter(enemy, CounterKind::Stun, 1);
    act(&mut game, Action::AnnounceCast { card: aura });
    reject(&mut game, target(own));
    act(&mut game, target(enemy));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(enemy).tapped);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(enemy, CounterKind::Stun), 0);
    assert_eq!(stats(&game, enemy), (1, 1));
    assert_eq!(game.object_view(enemy).name(), "Humble Merchant");
    assert!(game.has_subtype(enemy, "Citizen"));
    assert!(!game.has_subtype(enemy, "Pirate"));
    assert!(!game.keywords(enemy).contains(Keywords::FLYING));
    assert!(game.object_view(enemy).is_legendary());
    game.state.objects[enemy as usize].tapped = false;
    act(&mut game, Action::Pass);
    let grantor = game.object_ref(aura);
    let mana = game.state.players[p.other().index()].mana[5];
    act(
        &mut game,
        Action::TapGrantedMana {
            card: enemy,
            grantor,
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(game.state.players[p.other().index()].mana[5], mana + 1);
}

#[test]
fn invasion_submersible_bounces_another_nonland_then_permanently_exhausts() {
    let p = Player::P0;
    let mut game = scenario(p);
    let vehicle = put(&mut game, p, "Invasion Submersible", Zone::Hand);
    let enemy = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let land = put(&mut game, p, "Island", Zone::Battlefield);
    cast(&mut game, vehicle, None);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(vehicle));
    reject(&mut game, target(land));
    let mut decline = game.clone();
    act(&mut decline, Action::FinishTargets);
    settle(&mut decline);
    assert_eq!(decline.object_view(enemy).zone, Zone::Battlefield);
    act(&mut game, target(enemy));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Hand);
    assert_eq!(game.current_kinds(vehicle), 16);
    act(&mut game, activate(vehicle, 0));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.current_kinds(vehicle), 17);
    assert_eq!(stats(&game, vehicle), (3, 3));
    assert!(game.has_subtype(vehicle, "Vehicle"));
    reject(&mut game, activate(vehicle, 0));
    game.finish_cleanup();
    assert_eq!(game.current_kinds(vehicle), 17);
    game.state.priority = p;
    game.state.players[p.index()].mana = [8; 6];
    reject(&mut game, activate(vehicle, 0));
    game.move_object(vehicle, Zone::Exile);
    game.move_object(vehicle, Zone::Battlefield);
    game.prepare_triggers();
    finish_ordered_triggers(&mut game);
    game.state.priority = p;
    assert!(game.legal_actions().contains(&activate(vehicle, 0)));
}

#[test]
fn lost_days_lets_the_owner_choose_its_library_position_and_investigates() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Lost Days", Zone::Hand);
    let subject = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    game.change_controller(game.object_ref(subject), p);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(artifact));
    act(&mut game, target(subject));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(p.other()));
    let object = game.object_ref(subject);
    for destination in [
        LibraryDestination::SecondFromTop,
        LibraryDestination::Bottom,
    ] {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ChooseLibraryDestination {
                object,
                destination,
            },
        );
        let library = &branch.state.players[p.other().index()].library;
        let index = if destination == LibraryDestination::Bottom {
            0
        } else {
            library.len() - 2
        };
        assert_eq!(library[index], subject);
        assert_eq!(branch.object_view(token(&branch, Card(5296))).controller, p);
    }
}

#[test]
fn pakku_has_prowess_and_mills_a_target_player_from_the_current_lesson_count() {
    let p = Player::P1;
    let mut game = scenario(p);
    let pakku = put(&mut game, p, "Master Pakku", Zone::Battlefield);
    let spell = put(&mut game, p, "Quick Study", Zone::Hand);
    let first = put(&mut game, p, "Accumulate Wisdom", Zone::Graveyard);
    put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    put(&mut game, p.other(), "It'll Quench Ya!", Zone::Graveyard);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(stats(&game, pakku), (2, 4));
    game.state.pending = Pending::Attackers;
    game.state.step = Step::Attackers;
    act(&mut game, Action::Attack { card: pakku });
    act(&mut game, Action::FinishAttackers);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    game.move_object(first, Zone::Exile);
    let before = game.state.players[p.other().index()].library.len();
    settle(&mut game);
    assert_eq!(
        game.state.players[p.other().index()].library.len(),
        before - 1
    );
}

#[test]
fn haiku_has_independent_spell_and_land_targets_and_resolves_surviving_parts() {
    let p = Player::P1;
    let mut game = scenario(p.other());
    let enemy = put(&mut game, p.other(), "Divination", Zone::Hand);
    let haiku = put(&mut game, p, "Sokka's Haiku", Zone::Hand);
    let land = put(&mut game, p.other(), "Island", Zone::Battlefield);
    game.state.objects[land as usize].tapped = true;
    cast(&mut game, enemy, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: haiku });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(enemy),
        },
    );
    act(&mut game, target(land));
    pay(&mut game);
    let mut without_land = game.clone();
    without_land.move_object(land, Zone::Exile);
    settle(&mut without_land);
    assert_eq!(without_land.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(
        without_land.state.players[p.other().index()].cards_drawn_this_turn,
        0
    );
    assert_eq!(
        without_land.state.players[p.index()].cards_drawn_this_turn,
        1
    );
    settle(&mut game);
    assert!(!game.object_view(land).tapped);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[p.index()].library.len(), 4);
}

#[test]
fn teo_loots_once_for_multiple_flying_attackers_and_counters_after_a_nonland_discard() {
    let p = Player::P0;
    let mut game = scenario(p);
    let teo = put(&mut game, p, "Teo, Spirited Glider", Zone::Battlefield);
    let flyer = put(&mut game, p, "First-Time Flyer", Zone::Battlefield);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let discard = put(&mut game, p, "Divination", Zone::Hand);
    let land = game.top_library_card(p).unwrap();
    game.state.pending = Pending::Attackers;
    game.state.step = Step::Attackers;
    for card in [teo, flyer, own] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    reject(&mut game, Action::ChooseOptional { accept: false });
    let mut chose_land = game.clone();
    act(&mut chose_land, Action::Discard { card: land });
    assert!(!matches!(chose_land.state.pending, Pending::Targets { .. }));
    act(&mut game, Action::Discard { card: discard });
    reject(&mut game, target(enemy));
    act(&mut game, target(own));
    settle(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
}

#[test]
fn the_mechanist_investigates_and_animates_only_controlled_artifact_tokens() {
    let p = Player::P1;
    let mut game = scenario(p);
    let mechanic = put(
        &mut game,
        p,
        "The Mechanist, Aerial Artisan",
        Zone::Battlefield,
    );
    let draw = put(&mut game, p, "Quick Study", Zone::Hand);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    cast(&mut game, draw, None);
    settle(&mut game);
    let clue = token(&game, Card(5296));
    act(&mut game, activate(mechanic, 0));
    reject(&mut game, target(artifact));
    act(&mut game, target(clue));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, clue), (3, 1));
    assert_eq!(game.current_kinds(clue), 17);
    assert!(game.has_subtype(clue, "Construct") && game.has_subtype(clue, "Clue"));
    assert!(game.keywords(clue).contains(Keywords::FLYING));
    game.finish_cleanup();
    assert_eq!(game.current_kinds(clue), 16);
}

#[test]
fn spirit_oasis_draws_for_current_shrines_then_only_other_own_shrine_entries() {
    let p = Player::P0;
    let mut game = scenario(p);
    let oasis = put(&mut game, p, "The Spirit Oasis", Zone::Hand);
    let first = put(&mut game, p, "Northern Air Temple", Zone::Battlefield);
    put(
        &mut game,
        p.other(),
        "Northern Air Temple",
        Zone::Battlefield,
    );
    cast(&mut game, oasis, None);
    finish_ordered_triggers(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
    game.move_object(first, Zone::Exile);
    let shrine = put(&mut game, p, "Northern Air Temple", Zone::Hand);
    cast(&mut game, shrine, None);
    finish_ordered_triggers(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 3);
}

#[test]
fn ty_lee_can_flash_in_and_her_untap_restriction_ends_on_losing_control() {
    let p = Player::P1;
    let mut game = scenario(p.other());
    let ty = put(&mut game, p, "Ty Lee, Chi Blocker", Zone::Hand);
    let target_creature = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, p, "Quick Study", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, ty, None);
    choose_pending_targets(&mut game, Some(target_creature));
    settle(&mut game);
    assert!(game.object_view(target_creature).tapped);
    let mut lost = game.clone();
    lost.change_controller(lost.object_ref(ty), p.other());
    assert!(!lost.untap_active_player());
    assert!(!lost.object_view(target_creature).tapped);

    if game.actor() != Some(p) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(stats(&game, ty), (3, 2));
    assert!(!game.untap_active_player());
    assert!(game.object_view(target_creature).tapped);
}

#[test]
fn bloodbending_exiles_itself_and_schedules_the_printed_duration() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Secret of Bloodbending", Zone::Hand);
    game.state.players[p.index()].mana = [0, 4, 0, 0, 0, 10];
    for paid in [false, true] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        act(&mut branch, Action::ChooseWaterbendCost { pay: paid });
        let own = Action::ChooseTarget {
            target: Target::Player(p),
        };
        reject(&mut branch, own);
        act(
            &mut branch,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut branch);
        settle(&mut branch);
        let control = branch.observe(p).player_control.unwrap();
        assert_eq!(control.pending[p.other().index()], paid.then_some(p));
        assert_eq!(
            control.pending_combat[p.other().index()],
            (!paid).then_some(p)
        );
        assert_eq!(branch.object_view(spell).zone, Zone::Exile);
        reach(&mut branch, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        assert_eq!(branch.actor(), Some(if paid { p } else { p.other() }));
        reach(&mut branch, |g| g.state.step == Step::BeginCombat);
        assert_eq!(branch.actor(), Some(p));
    }
}

#[test]
fn kuruk_scries_and_draws_twice_then_transforms_creates_spirits_and_takes_an_extra_turn() {
    let p = Player::P0;
    let mut game = scenario(p);
    let saga = put(
        &mut game,
        p,
        "The Legend of Kuruk // Avatar Kuruk",
        Zone::Hand,
    );
    let spell = put(&mut game, p, "Quick Study", Zone::Hand);
    cast(&mut game, saga, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    finish_scry(&mut game, p, 2);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    game.begin_step(Step::Main1);
    game.give_priority();
    game.prepare_triggers();
    pass_pair(&mut game);
    finish_scry(&mut game, p, 2);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
    let before = game.object_ref(saga);
    game.begin_step(Step::Main1);
    game.give_priority();
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(saga).card, Card(5700));
    assert_ne!(game.object_ref(saga), before);
    assert_eq!(stats(&game, saga), (4, 3));

    cast(&mut game, spell, None);
    settle(&mut game);
    let spirit = token(&game, Card(5705));
    assert_eq!(stats(&game, spirit), (1, 1));
    assert!(game.has_subtype(spirit, "Spirit"));
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 19];
    let activation = activate(saga, 0);
    let mut short = game.clone();
    short.state.objects[spirit as usize].tapped = true;
    short.state.objects[saga as usize].tapped = true;
    reject(&mut short, activation.clone());
    act(&mut game, activation.clone());
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, spirit);
    pay(&mut game);
    settle(&mut game);
    game.state.players[p.index()].mana[5] = 20;
    reject(&mut game, activation);
    let turn = game.state.turn;
    reach(&mut game, |g| g.state.turn > turn);
    assert_eq!(game.state.active, p);
}
