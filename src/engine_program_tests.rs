use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn enter(game: &mut Game, player: Player, name: &str) -> CardId {
    let id = named(game, player, name, Zone::Hand);
    game.move_object(id, Zone::Battlefield);
    game.state_based_actions();
    game.prepare_triggers();
    id
}

fn order_all(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        let action = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
            .unwrap();
        act(game, action);
    }
}

#[test]
fn etb_is_a_separate_stack_object_and_survives_source_bounce() {
    let mut game = scenario();
    let hunter = named(&mut game, Player::P0, "Helpful Hunter", Zone::Hand);
    let bounce = named(&mut game, Player::P1, "Unsummon", Zone::Hand);
    let counter = named(&mut game, Player::P1, "Counterspell", Zone::Hand);
    game.state.players[0].mana[0] = 2;
    game.state.players[1].mana[1] = 3;
    let count = game.hand_count(Player::P0);
    cast(&mut game, hunter, None);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), count - 1); // ETB has not resolved.
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(
        game.state.stack[0].kind,
        StackKind::TriggeredAbility { ability: 0 }
    );
    let source = game.state.stack[0].object;
    act(&mut game, Action::Pass);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Cast {card, ..} if *card == counter))
    );
    cast(&mut game, bounce, Some(Target::Card(hunter)));
    pass_pair(&mut game);
    assert_eq!(game.state.objects[hunter as usize].zone, Zone::Hand);
    assert_ne!(game.object_ref(hunter), source);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), count + 1); // Returned hunter + the drawn card.
}

#[test]
fn countered_creature_never_enters_or_triggers() {
    let mut game = scenario();
    let hunter = named(&mut game, Player::P0, "Helpful Hunter", Zone::Hand);
    let counter = named(&mut game, Player::P1, "Counterspell", Zone::Hand);
    game.state.players[0].mana[0] = 2;
    game.state.players[1].mana[1] = 2;
    cast(&mut game, hunter, None);
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(hunter)));
    pass_pair(&mut game);
    assert_eq!(game.state.objects[hunter as usize].zone, Zone::Graveyard);
    assert!(game.state.stack.is_empty() && game.state.pending_triggers.is_empty());
}

#[test]
fn multiple_death_triggers_are_captured_simultaneously_and_ordered_apnap() {
    let mut game = scenario();
    game.state.active = Player::P1;
    let mut sources = Vec::new();
    for player in [Player::P0, Player::P1] {
        for _ in 0..2 {
            let card = named(&mut game, player, "Buzz Bots", Zone::Battlefield);
            sources.push(game.object_ref(card));
            game.state.objects[card as usize].damage = 100;
        }
    }
    game.state_based_actions();
    game.prepare_triggers();
    assert_eq!(game.actor(), Some(Player::P1));
    assert_eq!(game.state.ordering_triggers.len(), 4);
    assert!(
        game.state
            .ordering_triggers
            .iter()
            .all(|item| sources.contains(&item.object))
    );
    assert!(
        sources
            .iter()
            .all(|reference| game.object_ref(reference.card) != *reference)
    );
    let ordering = game.snapshot();
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::PutTrigger { .. }))
        .collect();
    assert_eq!(choices.len(), 2);
    act(&mut game, choices[1].clone());
    assert_eq!(game.actor(), Some(Player::P0));
    let choice = game_action_for_first_trigger(&game);
    act(&mut game, choice);
    assert_eq!(
        game.state
            .stack
            .iter()
            .map(|item| item.controller)
            .collect::<Vec<_>>(),
        vec![Player::P1, Player::P1, Player::P0, Player::P0]
    );
    let ordered = game.snapshot();
    game.restore(&ordering);
    act(&mut game, choices[1].clone());
    let choice = game_action_for_first_trigger(&game);
    act(&mut game, choice);
    assert_eq!(game.snapshot(), ordered);
    for _ in 0..4 {
        pass_pair(&mut game);
    }
    assert_eq!(game.hand_count(Player::P0), 2);
    assert_eq!(game.hand_count(Player::P1), 2);
}

fn game_action_for_first_trigger(game: &Game) -> Action {
    game.legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
        .unwrap()
}

#[test]
fn trigger_order_rejection_is_atomic_and_order_is_a_real_choice() {
    let mut game = scenario();
    named(
        &mut game,
        Player::P0,
        "Hinterland Sanctifier",
        Zone::Battlefield,
    );
    enter(&mut game, Player::P0, "Inspiring Overseer");
    assert!(matches!(game.state.pending, Pending::OrderTriggers { .. }));
    let before = game.snapshot();
    let revision = game.revision();
    for action in [
        Action::Pass,
        Action::PutTrigger { id: u64::MAX },
        Action::ChooseOptional { accept: true },
    ] {
        assert_eq!(game.apply(Player::P0, action), Err(Error::IllegalAction));
        assert_eq!(game.snapshot(), before);
        assert_eq!(game.revision(), revision);
    }
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::PutTrigger { .. }))
        .collect();
    let mut other = game.clone();
    act(&mut game, choices[0].clone());
    act(&mut other, choices[1].clone());
    assert_ne!(
        game.state
            .stack
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        other
            .state
            .stack
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn draw_discard_trigger_suspends_atomically_and_keeps_hands_private() {
    let mut game = scenario();
    let crier = enter(&mut game, Player::P0, "Bellowing Crier");
    pass_pair(&mut game);
    assert!(matches!(
        game.state.pending,
        Pending::EffectDiscard {
            player: Player::P0,
            remaining: 1
        }
    ));
    assert_eq!(game.state.objects[crier as usize].zone, Zone::Battlefield);
    assert!(game.state.resolving.is_some());
    let private = game.observe(Player::P1);
    assert!(private.actions.is_empty());
    assert!(
        !private
            .objects
            .iter()
            .any(|obj| obj.zone == Zone::Hand && obj.owner == Player::P0)
    );
    let before = game.snapshot();
    assert_eq!(
        game.apply(Player::P0, Action::Pass),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    act(&mut game, discard.clone());
    let after = game.snapshot();
    assert!(game.state.resolving.is_none());
    assert_eq!(game.hand_count(Player::P0), 0);
    game.restore(&before);
    act(&mut game, discard);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn spell_stays_on_stack_zone_until_draw_discard_program_completes() {
    let mut game = scenario();
    let spell = named(&mut game, Player::P0, "Ancestral Reminiscence", Zone::Hand);
    game.state.players[0].mana[1] = 4;
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.state.objects[spell as usize].zone, Zone::Stack);
    assert!(game.state.stack.is_empty()); // Resolving, cannot be responded to.
    assert_eq!(game.state.resolving.as_ref().unwrap().item.card, spell);
    assert_eq!(game.hand_count(Player::P0), 3);
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    act(&mut game, discard);
    assert_eq!(game.state.objects[spell as usize].zone, Zone::Graveyard);
    assert_eq!(game.hand_count(Player::P0), 2);
}

#[test]
fn optional_loot_can_be_declined_or_accepted_and_replayed() {
    let mut game = scenario();
    enter(&mut game, Player::P0, "Stadium Tidalmage");
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
    let optional = game.snapshot();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.hand_count(Player::P0), 0);
    assert_eq!(game.state.pending, Pending::Priority);
    game.restore(&optional);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.hand_count(Player::P0), 1);
    assert!(matches!(
        game.state.pending,
        Pending::EffectDiscard { remaining: 1, .. }
    ));
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    let saved = game.snapshot();
    act(&mut game, discard.clone());
    let after = game.snapshot();
    game.restore(&saved);
    act(&mut game, discard);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn optional_draw_choice_still_requires_discard_when_library_is_empty() {
    let mut game = scenario();
    enter(&mut game, Player::P0, "Stadium Tidalmage");
    pass_pair(&mut game);
    named(&mut game, Player::P0, "Forest", Zone::Hand);
    let library = game.state.players[0].library.clone();
    for id in library {
        game.move_object(id, Zone::Graveyard);
    }
    act(&mut game, Action::ChooseOptional { accept: true });
    // CR118.12 checks the choice to pay, not whether a card was actually drawn.
    // CR121.3 permits choosing the optional draw with an empty library.
    assert!(matches!(
        game.state.pending,
        Pending::EffectDiscard { remaining: 1, .. }
    ));
    assert_eq!(game.outcome(), None);
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    act(&mut game, discard);
    assert_eq!(game.hand_count(Player::P0), 0);
    assert_eq!(game.outcome(), Some(Outcome::Winner(Player::P1)));
    assert!(game.state.resolving.is_none());
}

#[test]
fn all_program_instructions_finish_before_loss_sba_even_across_discard() {
    static PROGRAM: &[Instruction] = &[
        Instruction::LoseLife(PlayerSet::Controller, 1),
        Instruction::Discard(PlayerSet::Controller, 1),
        Instruction::GainLife(PlayerSet::Controller, 2),
    ];
    let mut game = scenario();
    let hunter = enter(&mut game, Player::P0, "Helpful Hunter");
    let item = game.state.stack.pop().unwrap();
    named(&mut game, Player::P0, "Forest", Zone::Hand);
    game.state.players[0].life = 1;
    game.state.resolving = Some(programs::Resolution {
        item,
        program: PROGRAM,
        pc: 0,
        target_group: 0,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        search: None,
        choice: None,
    });
    game.run_program();
    assert_eq!(game.state.players[0].life, 0);
    assert_eq!(game.outcome(), None);
    let snapshot = game.snapshot();
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    act(&mut game, discard.clone());
    assert_eq!(game.state.players[0].life, 2);
    assert_eq!(game.outcome(), None);
    assert_eq!(game.state.objects[hunter as usize].zone, Zone::Battlefield);
    let after = game.snapshot();
    game.restore(&snapshot);
    act(&mut game, discard);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn friendly_teddy_draws_for_both_players_before_simultaneous_deck_loss() {
    let mut game = scenario();
    let teddy = named(&mut game, Player::P0, "Friendly Teddy", Zone::Battlefield);
    for player in [Player::P0, Player::P1] {
        let library = game.state.players[player.index()].library.clone();
        for id in library {
            game.move_object(id, Zone::Graveyard);
        }
    }
    game.move_object(teddy, Zone::Graveyard);
    game.prepare_triggers();
    pass_pair(&mut game);
    assert_eq!(game.outcome(), Some(Outcome::Draw));
    assert!(game.state.players.iter().all(|p| p.attempted_empty_draw));
}

#[test]
fn pelakka_wurm_entry_and_death_programs_use_distinct_ability_indices() {
    let mut game = scenario();
    let wurm = enter(&mut game, Player::P0, "Pelakka Wurm");
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 27);
    game.move_object(wurm, Zone::Graveyard);
    game.prepare_triggers();
    assert_eq!(
        game.state.stack[0].kind,
        StackKind::TriggeredAbility { ability: 1 }
    );
    let dead = game.snapshot();
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 1);
    assert_eq!(game.state.players[0].life, 27);
    let after = game.snapshot();
    game.restore(&dead);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn entry_watchers_distinguish_self_other_owner_and_land() {
    let mut game = scenario();
    named(
        &mut game,
        Player::P0,
        "Hinterland Sanctifier",
        Zone::Battlefield,
    );
    enter(&mut game, Player::P0, "Bogwater Lumaret");
    assert_eq!(game.state.ordering_triggers.len(), 2); // Sanctifier + Lumaret itself.
    order_all(&mut game);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 22);
    enter(&mut game, Player::P1, "Savannah Lions");
    assert!(game.state.stack.is_empty());
    let druid = enter(&mut game, Player::P0, "Eumidian Terrabotanist");
    order_all(&mut game);
    pass_pair(&mut game);
    pass_pair(&mut game); // Two creature-entry watchers.
    assert_eq!(game.state.players[0].life, 24);
    let land = named(&mut game, Player::P0, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].card, druid);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 25);
}

#[test]
fn artifact_entry_loot_does_not_trigger_on_nonartifact_creature() {
    let mut game = scenario();
    named(&mut game, Player::P0, "Rook Turret", Zone::Battlefield);
    enter(&mut game, Player::P0, "Savannah Lions");
    assert!(game.state.stack.is_empty());
    let giant = enter(&mut game, Player::P0, "Iron Giant");
    assert!(game.state.objects[giant as usize].card.is_artifact());
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
}

#[test]
fn attack_triggers_wait_for_committed_declaration_and_can_be_ordered() {
    let mut game = scenario();
    let mut attackers = Vec::new();
    for _ in 0..2 {
        attackers.push(named(
            &mut game,
            Player::P0,
            "Herald of Faith",
            Zone::Battlefield,
        ));
    }
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in attackers {
        act(&mut game, Action::Attack { card });
    }
    assert!(game.state.pending_triggers.is_empty());
    act(&mut game, Action::FinishAttackers);
    assert!(matches!(game.state.pending, Pending::OrderTriggers { .. }));
    order_all(&mut game);
    assert_eq!(game.state.step, Step::Attackers);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 24);
    assert_eq!(game.state.step, Step::Attackers);
}

#[test]
fn simultaneous_discards_keep_all_selections_private_until_committed() {
    static PROGRAM: &[Instruction] = &[Instruction::Discard(PlayerSet::Each, 2)];
    let mut game = scenario();
    enter(&mut game, Player::P0, "Helpful Hunter");
    let item = game.state.stack.pop().unwrap();
    game.state.active = Player::P1;
    let mut hands = [Vec::new(), Vec::new()];
    for player in [Player::P0, Player::P1] {
        for _ in 0..2 {
            hands[player.index()].push(named(&mut game, player, "Forest", Zone::Hand));
        }
    }
    game.state.resolving = Some(programs::Resolution {
        item,
        program: PROGRAM,
        pc: 0,
        target_group: 0,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        search: None,
        choice: None,
    });
    game.run_program();
    assert_eq!(game.actor(), Some(Player::P1));
    let before_bytes = game.state_bytes();
    act(&mut game, Action::Discard { card: hands[1][0] });
    assert_eq!(
        game.state_bytes(),
        before_bytes + std::mem::size_of::<CardId>()
    );
    assert!(
        !game
            .legal_actions()
            .contains(&Action::Discard { card: hands[1][0] })
    );
    act(&mut game, Action::Discard { card: hands[1][1] });
    assert_eq!(game.actor(), Some(Player::P0));
    assert_eq!(game.observe(Player::P1).effect_discards, hands[1]);
    assert!(game.observe(Player::P0).effect_discards.is_empty());
    assert!(
        !game
            .observe(Player::P0)
            .objects
            .iter()
            .any(|o| hands[1].contains(&o.id))
    );
    assert!(
        hands
            .iter()
            .flatten()
            .all(|&id| game.state.objects[id as usize].zone == Zone::Hand)
    );
    let pending = game.snapshot();
    for &card in &hands[0] {
        act(&mut game, Action::Discard { card });
    }
    assert!(
        hands
            .iter()
            .flatten()
            .all(|&id| game.state.objects[id as usize].zone == Zone::Graveyard)
    );
    let after = game.snapshot();
    game.restore(&pending);
    for &card in &hands[0] {
        act(&mut game, Action::Discard { card });
    }
    assert_eq!(game.snapshot(), after);
}

#[test]
fn opponent_discards_control_their_own_choice_and_empty_hands_do_not_stall() {
    for hand_size in 0..3 {
        let mut game = scenario();
        let hand: Vec<_> = (0..hand_size)
            .map(|_| named(&mut game, Player::P1, "Forest", Zone::Hand))
            .collect();
        let rat = enter(&mut game, Player::P0, "Burglar Rat");
        pass_pair(&mut game);
        if hand_size == 0 {
            assert!(game.state.resolving.is_none());
        } else {
            assert_eq!(game.actor(), Some(Player::P1));
            let before = game.snapshot();
            assert_eq!(
                game.apply(Player::P0, Action::Discard { card: hand[0] }),
                Err(Error::WrongActor)
            );
            assert_eq!(game.snapshot(), before);
            act(&mut game, Action::Discard { card: hand[0] });
            assert_eq!(game.hand_count(Player::P1), hand_size - 1);
        }
        assert_eq!(game.state.pending, Pending::Priority);
        assert_eq!(game.actor(), Some(Player::P0));
        assert_eq!(game.state.objects[rat as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn death_triggers_require_battlefield_to_graveyard_and_do_not_repeat_on_priority() {
    let mut game = scenario();
    for zone in [Zone::Hand, Zone::Library, Zone::Stack] {
        let bot = named(&mut game, Player::P0, "Buzz Bots", zone);
        game.move_object(bot, Zone::Graveyard);
    }
    let bot = named(&mut game, Player::P0, "Buzz Bots", Zone::Battlefield);
    game.move_object(bot, Zone::Hand);
    assert!(game.state.pending_triggers.is_empty());
    game.move_object(bot, Zone::Battlefield);
    game.move_object(bot, Zone::Graveyard);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 1);
    for _ in 0..2 {
        pass_pair(&mut game);
        assert!(game.state.pending_triggers.is_empty());
        assert!(game.state.stack.is_empty());
    }
}
