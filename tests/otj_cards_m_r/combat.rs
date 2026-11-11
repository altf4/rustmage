use super::*;

fn attackers(game: &mut Game) {
    reach(game, |g| g.state.pending == Pending::Attackers);
}

fn blockers(game: &mut Game) {
    reach(game, |g| g.state.pending == Pending::Blockers);
}

fn saddle(game: &mut Game, mount: CardId, rider: CardId) {
    announce_activation(game, mount, 0);
    let object = game.object_ref(rider);
    act(game, Action::ChooseSaddler { object });
    act(game, Action::FinishSaddle);
    quiet(game);
}

#[test]
fn mobile_homestead_and_miriam_cover_the_mount_and_vehicle_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    board(&mut game, player, "Miriam, Herd Whisperer");
    let crew = board(&mut game, player, "Grizzly Bears");
    let vehicle = hand(&mut game, player, "Mobile Homestead");
    cast(&mut game, vehicle, None);
    quiet(&mut game);
    assert!(!keywords(&game, vehicle).contains(Keywords::HASTE));
    assert!(keywords(&game, vehicle).contains(Keywords::HEXPROOF));
    assert_eq!(game.object_view(vehicle).kinds & 1, 0);

    let mount = board(&mut game, player, "Quilled Charger");
    assert!(keywords(&game, mount).contains(Keywords::HEXPROOF));
    assert!(keywords(&game, vehicle).contains(Keywords::HASTE));
    announce_activation(&mut game, vehicle, 0);
    let rider = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object: rider });
    act(&mut game, Action::FinishCrew);
    quiet(&mut game);
    assert_eq!(stats(&game, vehicle), (3, 3));
    let land = top(&mut game, player, "Plains");
    attackers(&mut game);
    act(&mut game, Action::Attack { card: vehicle });
    act(&mut game, Action::Attack { card: mount });
    act(&mut game, Action::FinishAttackers);

    let select = Action::SelectLibrary { card: land };
    until_action(&mut game, &select);
    assert!(game.observe(player.other()).library_selection.is_none());
    let mut decline = game.clone();
    act(&mut decline, Action::FinishLibrarySelection);
    quiet(&mut decline);
    assert_eq!(zone(&decline, land), Zone::Library);
    choose_library(&mut game, land);
    quiet(&mut game);
    assert_eq!(zone(&game, land), Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    assert_eq!(counters(&game, vehicle), 1);
    assert_eq!(counters(&game, mount), 1);
    blockers(&mut game);
    act(&mut game, Action::FinishBlockers);
    reach(&mut game, |g| {
        g.state.active == player.other() && g.state.step == Step::Main1
    });
    assert!(!keywords(&game, vehicle).contains(Keywords::HEXPROOF));
    assert!(!keywords(&game, mount).contains(Keywords::HEXPROOF));
}

#[test]
fn obeka_adds_only_the_actual_player_damage_as_upkeeps_and_brigand_draws() {
    let player = Player::P1;
    let mut game = scenario(player);
    let obeka = board(&mut game, player, "Obeka, Splitter of Seconds");
    let brigand = board(&mut game, player, "Nimble Brigand");
    let defender = board(&mut game, player.other(), "Grizzly Bears");
    let shock = hand(&mut game, player, "Shock");
    cast(&mut game, shock, Some(Target::Player(player.other())));
    quiet(&mut game);
    assert!(keywords(&game, obeka).contains(Keywords::MENACE));
    attackers(&mut game);
    act(&mut game, Action::Attack { card: obeka });
    act(&mut game, Action::Attack { card: brigand });
    act(&mut game, Action::FinishAttackers);
    blockers(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: defender,
            attacker: brigand,
        },
    );
    act(&mut game, Action::FinishBlockers);
    reach(&mut game, |g| {
        g.state.step == Step::EndCombat
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
    });
    assert_eq!(life(&game, player.other()), 15);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    let inserted = game.observe(player).after_combat_phases.unwrap();
    assert_eq!(inserted.len(), 1);
    assert_eq!(inserted[0].first_step, Step::Upkeep);
    assert_eq!(inserted[0].remaining, 2);
    reach(&mut game, |g| g.state.step == Step::Main2);
    assert!(game.object_view(obeka).tapped);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn ornery_tumblewagg_doubles_counters_after_its_beginning_of_combat_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mount = board(&mut game, player, "Ornery Tumblewagg");
    let creature = board(&mut game, player, "Grizzly Bears");
    game.add_counter(creature, CounterKind::PlusOne, 1);
    saddle(&mut game, mount, creature);
    let target = Action::ChooseTarget {
        target: Target::Card(creature),
    };
    until_action(&mut game, &target);
    act(&mut game, target.clone());
    attackers(&mut game);
    assert_eq!(counters(&game, creature), 2);
    act(&mut game, Action::Attack { card: mount });
    act(&mut game, Action::FinishAttackers);
    until_action(&mut game, &target);
    act(&mut game, target);
    quiet(&mut game);
    assert_eq!(counters(&game, creature), 4);
}

#[test]
fn rambling_possum_can_return_only_the_creatures_that_paid_saddle() {
    let player = Player::P1;
    let mut game = scenario(player);
    let mount = board(&mut game, player, "Rambling Possum");
    let rider = board(&mut game, player, "Bear Cub");
    let stranger = board(&mut game, player, "Grizzly Bears");
    saddle(&mut game, mount, rider);
    attackers(&mut game);
    act(&mut game, Action::Attack { card: mount });
    act(&mut game, Action::FinishAttackers);
    let choice = Action::ChooseEffectObject {
        object: game.object_ref(rider),
    };
    until_action(&mut game, &choice);
    let wrong = game.object_ref(stranger);
    reject(&mut game, Action::ChooseEffectObject { object: wrong });
    let mut decline = game.clone();
    act(&mut decline, Action::FinishEffectChoice);
    quiet(&mut decline);
    assert_eq!(zone(&decline, rider), Zone::Battlefield);

    act(&mut game, choice);
    if has(&game, &Action::FinishEffectChoice) {
        act(&mut game, Action::FinishEffectChoice);
    }
    quiet(&mut game);
    assert_eq!(zone(&game, rider), Zone::Hand);
    assert_eq!(stats(&game, mount), (4, 5));
}

#[test]
fn roadrunner_excludes_coyotes_even_if_they_have_haste_and_filters_other_blockers() {
    let player = Player::P0;
    let mut base = scenario(player);
    let roadrunner = board(&mut base, player, "Resilient Roadrunner");
    base.state.objects[roadrunner as usize].summoning_sick = true;
    let coyote = board(&mut base, player.other(), "Cunning Coyote");
    let haste = board(&mut base, player.other(), "Reckless Lackey");
    let normal = board(&mut base, player.other(), "Grizzly Bears");
    for activated in [false, true] {
        let mut game = base.clone();
        if activated {
            activate(&mut game, roadrunner, 0, &[]);
            quiet(&mut game);
        }
        attackers(&mut game);
        act(&mut game, Action::Attack { card: roadrunner });
        act(&mut game, Action::FinishAttackers);
        blockers(&mut game);
        let block = |blocker| Action::Block {
            blocker,
            attacker: roadrunner,
        };
        reject(&mut game, block(coyote));
        assert!(has(&game, &block(haste)));
        assert_eq!(has(&game, &block(normal)), !activated);
    }
}

#[test]
fn prairie_dog_counter_replacements_stack_and_a_hand_cast_in_response_suppresses_the_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let dog = board(&mut game, player, "Prairie Dog");
    assert!(keywords(&game, dog).contains(Keywords::LIFELINK));
    for _ in 0..2 {
        activate(&mut game, dog, 0, &[]);
        quiet(&mut game);
    }
    reach(&mut game, |g| {
        g.state.step == Step::End
            && g.state.pending == Pending::Priority
            && !g.state.stack.is_empty()
    });
    let mut response = game.clone();
    restock_mana(&mut response, player);
    let shock = hand(&mut response, player, "Shock");
    cast(&mut response, shock, Some(Target::Player(player.other())));
    quiet(&mut response);
    assert_eq!(counters(&response, dog), 0);

    quiet(&mut game);
    assert_eq!(counters(&game, dog), 3);
    let turn = game.state.turn;
    reach(&mut game, |g| {
        g.state.turn > turn && g.state.active == player && g.state.step == Step::End
    });
    quiet(&mut game);
    assert_eq!(counters(&game, dog), 4);
}

#[test]
fn mirage_mesa_enters_tapped_and_only_produces_its_chosen_color() {
    let player = Player::P1;
    let mut game = scenario(player);
    let mesa = hand(&mut game, player, "Mirage Mesa");
    act(&mut game, Action::PlayLand { card: mesa });
    let color = EntryChoiceValue::Color(ChoiceColor::Black);
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| matches!(action, Action::ChooseEntryValue { .. }))
            .count(),
        5
    );
    act(&mut game, Action::ChooseEntryValue { value: color });
    assert_eq!(game.object_view(mesa).entry_choice, Some(color));
    assert!(game.object_view(mesa).tapped);
    let turn = game.state.turn;
    reach(&mut game, |g| {
        g.state.turn > turn && g.state.active == player && g.state.step == Step::Main1
    });
    assert!(!game.object_view(mesa).tapped);
    let mana = game.state.players[player.index()].mana;
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 1, 0, 0, 0], .. } if *card == mesa),
    );
    assert_eq!(game.state.players[player.index()].mana[2], mana[2] + 1);
}

#[test]
fn roxanne_creates_tapped_meteorites_and_doubles_only_the_mana_they_produce() {
    let player = Player::P1;
    let mut game = scenario(player);
    let roxanne = hand(&mut game, player, "Roxanne, Starfall Savant");
    cast(&mut game, roxanne, None);
    let damage = Action::ChooseTarget {
        target: Target::Player(player.other()),
    };
    until_action(&mut game, &damage);
    let first = tokens(&game, player, METEORITE)[0];
    let meteorite = game.object_view(first);
    assert!(meteorite.tapped);
    assert_eq!(meteorite.colors, 0);
    assert_eq!(meteorite.kinds & 16, 16);
    act(&mut game, damage.clone());
    quiet(&mut game);
    assert_eq!(life(&game, player.other()), 18);

    let turn = game.state.turn;
    reach(&mut game, |g| {
        g.state.turn > turn && g.state.active == player && g.state.step == Step::Main1
    });
    let before = game.state.players[player.index()].mana;
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 1, 0, 0, 0, 0], .. } if *card == first),
    );
    assert_eq!(game.state.players[player.index()].mana[1], before[1] + 2);
    attackers(&mut game);
    act(&mut game, Action::Attack { card: roxanne });
    act(&mut game, Action::FinishAttackers);
    until_action(&mut game, &damage);
    let second = tokens(&game, player, METEORITE)[1];
    assert!(game.object_view(second).tapped);
    act(&mut game, damage);
    quiet(&mut game);
    assert_eq!(life(&game, player.other()), 16);
}
