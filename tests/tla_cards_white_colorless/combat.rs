use super::*;

fn declare(game: &mut Game, attacker: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn cast_mode(game: &mut Game, spell: CardId, mode: u16, card: CardId) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    target(game, Target::Card(card));
    pay(game);
}

fn pay_if_open(game: &mut Game) {
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::PayCosts { .. }))
    {
        pay(game);
    }
}

#[test]
fn aang_and_airbending_lesson_exile_other_nonlands_and_lesson_grants_lifelink() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let aang = put(&mut game, player, "Aang, the Last Airbender", Zone::Hand);
    let bow = put(&mut game, other, "Short Bow", Zone::Battlefield);
    let bear = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, other, "Forest", Zone::Battlefield);
    cast(&mut game, aang, None);
    pass_pair(&mut game);
    for card in [aang, land] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    target(&mut game, Target::Card(bow));
    finish_targets(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(bow).zone, Zone::Exile);
    assert!(game.keywords(aang).contains(Keywords::FLYING));

    let lesson = put(&mut game, player, "Airbending Lesson", Zone::Hand);
    cast(&mut game, lesson, Some(Target::Card(bear)));
    drain(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Exile);
    assert_eq!(draw_count(&game, player), 1);
    assert!(game.keywords(aang).contains(Keywords::LIFELINK));
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.keywords(aang).contains(Keywords::LIFELINK));
}

#[test]
fn airbenders_reversal_destroys_an_attacker_or_airbends_a_friendly_creature() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Airbender's Reversal", Zone::Hand);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    declare(&mut game, attacker);
    let mut destroy = game.clone();
    act(&mut destroy, Action::AnnounceCast { card: spell });
    act(&mut destroy, Action::ChooseMode { mode: 0 });
    reject(
        &mut destroy,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    target(&mut destroy, Target::Card(attacker));
    pay(&mut destroy);
    drain(&mut destroy);
    assert_eq!(destroy.object_view(attacker).zone, Zone::Graveyard);

    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    target(&mut game, Target::Card(own));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Exile);
    assert_eq!(game.object_view(attacker).zone, Zone::Battlefield);
}

#[test]
fn loyal_appa_offers_both_modes_on_entry_and_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let appa = put(&mut game, player, "Appa, Loyal Sky Bison", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, appa, None);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseMode { mode: 0 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    target(&mut game, Target::Card(bear));
    drain(&mut game);
    assert!(game.keywords(bear).contains(Keywords::FLYING));
    assert!(game.keywords(appa).contains(Keywords::FLYING));

    game.state.objects[appa as usize].summoning_sick = false;
    declare(&mut game, appa);
    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(appa),
        },
    );
    target(&mut game, Target::Card(bow));
    drain(&mut game);
    assert_eq!(game.object_view(bow).zone, Zone::Exile);
}

#[test]
fn earth_kingdom_jailer_checks_control_and_mana_value_and_returns_its_prisoner() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let jailer = put(&mut game, player, "Earth Kingdom Jailer", Zone::Hand);
    let creature = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    let small = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enchantment = put(&mut game, other, "Doubling Season", Zone::Battlefield);
    let planeswalker = put(
        &mut game,
        other,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(planeswalker, CounterKind::Loyalty, 4);
    cast(&mut game, jailer, None);
    pass_pair(&mut game);
    for card in [small, own, planeswalker] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    let mut enchantment_branch = game.clone();
    target(&mut enchantment_branch, Target::Card(enchantment));
    finish_targets(&mut enchantment_branch);
    drain(&mut enchantment_branch);
    assert_eq!(
        enchantment_branch.object_view(enchantment).zone,
        Zone::Exile
    );

    let old = game.object_ref(creature);
    target(&mut game, Target::Card(creature));
    finish_targets(&mut game);
    let mut departed = game.clone();
    departed.move_object(jailer, Zone::Hand);
    drain(&mut departed);
    assert_eq!(departed.object_ref(creature), old);
    drain(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    game.move_object(jailer, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).controller, other);
    assert_ne!(game.object_ref(creature), old);
}

#[test]
fn earth_kingdom_protectors_sacrifice_for_another_ally() {
    let player = Player::P1;
    let mut game = scenario(player);
    let guard = put(
        &mut game,
        player,
        "Earth Kingdom Protectors",
        Zone::Battlefield,
    );
    let ally = put(&mut game, player, "Kyoshi Warriors", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert!(game.keywords(guard).contains(Keywords::VIGILANCE));
    activate(&mut game, guard, 0);
    for card in [guard, other] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    target(&mut game, Target::Card(ally));
    pay_if_open(&mut game);
    assert_eq!(game.object_view(guard).zone, Zone::Graveyard);
    drain(&mut game);
    assert!(game.keywords(ally).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn energybending_and_avatar_state_add_types_only_to_the_existing_cohort() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Abandoned Air Temple", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let energy = put(&mut game, player, "Energybending", Zone::Hand);
    cast(&mut game, energy, None);
    drain(&mut game);
    assert_eq!(draw_count(&game, player), 1);
    for subtype in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        assert!(game.has_subtype(land, subtype));
    }
    let later = put(&mut game, player, "Plains", Zone::Hand);
    act(&mut game, Action::PlayLand { card: later });
    assert!(!game.has_subtype(later, "Island"));

    let state = put(&mut game, player, "Enter the Avatar State", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: state });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(bear));
    pay(&mut game);
    drain(&mut game);
    assert!(game.has_subtype(bear, "Avatar"));
    assert!(game.has_subtype(bear, "Bear"));
    for keyword in [
        Keywords::FLYING,
        Keywords::FIRST_STRIKE,
        Keywords::LIFELINK,
        Keywords::HEXPROOF,
    ] {
        assert!(game.keywords(bear).contains(keyword));
    }
    let grantor = game.object_ref(land);
    act(
        &mut game,
        Action::TapGrantedMana {
            card: land,
            grantor,
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.has_subtype(land, "Island"));
    assert!(!game.has_subtype(bear, "Avatar"));
    assert!(!game.keywords(bear).contains(Keywords::HEXPROOF));
}

#[test]
fn glider_staff_airbends_on_entry_then_equips_for_two() {
    let player = Player::P0;
    let mut game = scenario(player);
    let staff = put(&mut game, player, "Glider Staff", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, staff, None);
    pass_pair(&mut game);
    target(&mut game, Target::Card(opposing));
    finish_targets(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(opposing).zone, Zone::Exile);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, staff, 0);
    target(&mut game, Target::Card(bear));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(
        game.object_view(staff).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!(stats(&game, bear), (3, 3));
    assert!(game.keywords(bear).contains(Keywords::FLYING));
}

#[test]
fn path_to_redemption_restricts_combat_and_exiles_after_sacrificing_the_aura() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let path = put(&mut game, player, "Path to Redemption", Zone::Hand);
    let victim = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    cast(&mut game, path, Some(Target::Card(victim)));
    drain(&mut game);
    assert_eq!(
        game.object_view(path).attached_to,
        Some(game.object_ref(victim))
    );

    let mut off_turn = game.clone();
    off_turn.state.active = other;
    off_turn.state.priority = player;
    reject(
        &mut off_turn,
        Action::AnnounceActivate {
            card: path,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    off_turn.state.step = Step::Attackers;
    off_turn.state.pending = Pending::Attackers;
    reject(&mut off_turn, Action::Attack { card: victim });

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
    activate(&mut game, path, 0);
    pay(&mut game);
    assert_eq!(game.object_view(path).zone, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(tokens_of(&game, player, "Ally").len(), 1);
}

#[test]
fn razor_rings_only_hits_combat_creatures_and_gains_actual_excess() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Razor Rings", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let rest = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    declare(&mut game, bear);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(rest),
        },
    );
    target(&mut game, Target::Card(bear));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(life(&game, player), 22);
}

#[test]
fn sandbenders_storm_destroys_large_creatures_or_earthbends_a_land() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Sandbenders' Storm", Zone::Hand);
    let large = put(
        &mut game,
        player.other(),
        "Gigantosaurus",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let mut destroy = game.clone();
    cast_mode(&mut destroy, spell, 0, large);
    drain(&mut destroy);
    assert_eq!(destroy.object_view(large).zone, Zone::Graveyard);

    cast_mode(&mut game, spell, 1, land);
    drain(&mut game);
    assert_eq!(stats(&game, land), (3, 3));
    assert!(game.is_creature(land));
    assert!(game.has_subtype(land, "Forest"));
    assert!(game.keywords(land).contains(Keywords::HASTE));
    let old = game.object_ref(land);
    game.move_object(land, Zone::Graveyard);
    drain(&mut game);
    assert_ne!(game.object_ref(land), old);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    assert!(!game.is_creature(land));
}

#[test]
fn vengeful_villagers_tap_then_offer_a_sacrifice_for_a_stun_counter() {
    let player = Player::P0;
    let mut game = scenario(player);
    let villagers = put(&mut game, player, "Vengeful Villagers", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    declare(&mut game, villagers);
    target(&mut game, Target::Card(foe));
    pass_pair(&mut game);
    assert!(game.object_view(foe).tapped);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.counter_count(foe, CounterKind::Stun), 0);
    let object = game.object_ref(bow);
    act(&mut game, Action::Sacrifice { object });
    drain(&mut game);
    assert_eq!(game.object_view(bow).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(foe, CounterKind::Stun), 1);
}

#[test]
fn zukos_exile_gives_the_clue_to_the_targets_controller() {
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Zuko's Exile", Zone::Hand);
    let permanent = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let enchantment = put(&mut game, other, "Impact Tremors", Zone::Battlefield);
    let planeswalker = put(
        &mut game,
        other,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(planeswalker, CounterKind::Loyalty, 4);
    game.record_control(game.object_ref(permanent), other, None);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(planeswalker),
        },
    );
    let mut enchantment_branch = game.clone();
    target(&mut enchantment_branch, Target::Card(enchantment));
    pay(&mut enchantment_branch);
    drain(&mut enchantment_branch);
    assert_eq!(
        enchantment_branch.object_view(enchantment).zone,
        Zone::Exile
    );

    target(&mut game, Target::Card(permanent));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Exile);
    assert_eq!(game.object_view(permanent).owner, player);
    assert!(tokens_of(&game, player, "Clue").is_empty());
    assert_eq!(tokens_of(&game, other, "Clue").len(), 1);
}
