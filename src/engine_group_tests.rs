use super::tests::{act, pass_pair, scenario, spawn};
use super::*;
fn named(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}
fn announce(game: &mut Game, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    let id = spawn(game, Player::P0, card, Zone::Hand);
    let (mut mana, generic) = card.cost();
    mana[4] += generic;
    game.state.players[0].mana = mana;
    act(game, Action::AnnounceCast { card: id });
    id
}
fn choose(game: &mut Game, id: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn pay(game: &mut Game) {
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(game, action);
}
fn rejected(game: &mut Game, action: Action) {
    let before = game.snapshot();
    let revision = game.revision;
    assert_eq!(
        game.apply(game.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    assert_eq!(game.revision, revision);
}
fn fight(game: &mut Game, a: CardId, b: CardId) {
    announce(game, "Bushwhack");
    act(game, Action::ChooseMode { mode: 1 });
    choose(game, a);
    choose(game, b);
    pay(game);
}

#[test]
fn groups_require_complete_own_opponent_pairs_and_reject_wrong_declarations() {
    let mut game = scenario();
    let spell = named(&mut game, Player::P0, "Quarrel", Zone::Hand);
    game.state.players[0].mana[4] = 4;
    let own = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    rejected(&mut game, Action::AnnounceCast { card: spell });
    let opp = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
    game.state.objects[opp as usize].gained_keywords = Keywords::HEXPROOF;
    rejected(&mut game, Action::AnnounceCast { card: spell });
    game.state.objects[opp as usize].gained_keywords = Keywords::NONE;
    act(&mut game, Action::AnnounceCast { card: spell });
    assert_eq!(game.observe(Player::P0).targeting_group, Some(0));
    assert_eq!(game.observe(Player::P1).targeting_group, None);
    rejected(&mut game, Action::FinishTargets);
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opp),
        },
    );
    choose(&mut game, own);
    assert_eq!(game.observe(Player::P0).targeting_group, Some(1));
    assert!(
        game.observe(Player::P1)
            .announcing
            .unwrap()
            .targets
            .is_empty()
    );
    rejected(&mut game, Action::FinishTargets);
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    rejected(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 2, 0],
        },
    );
    choose(&mut game, opp);
    pay(&mut game);
    assert_eq!(
        game.state.stack[0]
            .targets
            .iter()
            .map(|b| (b.group, b.target))
            .collect::<Vec<_>>(),
        vec![(0, Target::Card(own)), (1, Target::Card(opp))]
    );
    pass_pair(&mut game);
    assert_eq!(game.object_view(own).damage, 0);
    assert_eq!(game.object_view(opp).damage, 2);
}

#[test]
fn each_group_and_payment_boundary_clones_and_restores_independently() {
    let mut game = scenario();
    let own = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let opp = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
    announce(&mut game, "Kapow!");
    let mut actions = vec![
        Action::ChooseTarget {
            target: Target::Card(own),
        },
        Action::ChooseTarget {
            target: Target::Card(opp),
        },
    ];
    for action in actions.drain(..) {
        let before = game.snapshot();
        let mut branch = game.clone();
        act(&mut branch, action.clone());
        act(&mut game, action.clone());
        assert_eq!(game.snapshot(), branch.snapshot());
        let after = game.snapshot();
        game.restore(&before).unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), after);
    }
    let before = game.snapshot();
    pay(&mut game);
    pass_pair(&mut game);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opp).damage, 3);
    assert_eq!(game.object_view(opp).counters.plus_one, 0);
}

#[test]
fn fight_is_simultaneous_and_uses_lifelink_deathtouch_without_strike_multipliers() {
    let mut game = scenario();
    let own = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let opp = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
    game.state.objects[own as usize].gained_keywords = Keywords::LIFELINK
        .union(Keywords::DEATHTOUCH)
        .union(Keywords::DOUBLE_STRIKE);
    game.state.objects[opp as usize].gained_keywords =
        Keywords::LIFELINK.union(Keywords::FIRST_STRIKE);
    fight(&mut game, own, opp);
    pass_pair(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opp).zone, Zone::Graveyard);
    assert_eq!(game.state.players[0].life, 22);
    assert_eq!(game.state.players[1].life, 25);
    assert!(game.state.hits.is_empty());
    assert_eq!(game.state.step, Step::Main1);
}

#[test]
fn zero_and_negative_power_deal_no_damage_but_opponent_still_fights() {
    for power in [0, -2] {
        let mut game = scenario();
        let own = named(&mut game, Player::P0, "Shivan Dragon", Zone::Battlefield);
        let opp = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[own as usize].growth.power = power - 5;
        game.state.objects[own as usize].gained_keywords =
            Keywords::LIFELINK.union(Keywords::DEATHTOUCH);
        fight(&mut game, own, opp);
        pass_pair(&mut game);
        assert_eq!(game.object_view(own).damage, 2);
        assert_eq!(game.object_view(opp).damage, 0);
        assert_eq!(game.state.players[0].life, 20);
        assert_eq!(game.object_view(opp).zone, Zone::Battlefield);
    }
}

#[test]
fn power_damage_uses_current_derived_power_including_prior_instructions() {
    let mut game = scenario();
    named(
        &mut game,
        Player::P0,
        "Anthem of Champions",
        Zone::Battlefield,
    );
    let own = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let opp = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
    game.state.objects[own as usize].counters.plus_one = 1;
    announce(&mut game, "Huatli's Final Strike");
    choose(&mut game, own);
    choose(&mut game, opp);
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(own).power, 5);
    assert_eq!(game.object_view(own).toughness, 4);
    assert_eq!(game.object_view(own).damage, 0);
    assert_eq!(game.object_view(opp).zone, Zone::Graveyard);
}

#[test]
fn one_illegal_fight_participant_prevents_damage_but_keeps_other_group_effects() {
    for remove_own in [false, true] {
        let mut game = scenario();
        let own = named(&mut game, Player::P0, "Shivan Dragon", Zone::Battlefield);
        let opp = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
        announce(&mut game, "Kapow!");
        choose(&mut game, own);
        choose(&mut game, opp);
        pay(&mut game);
        let missing = if remove_own { own } else { opp };
        game.move_object(missing, Zone::Hand);
        game.move_object(missing, Zone::Battlefield);
        pass_pair(&mut game);
        assert_eq!(game.object_view(own).damage, 0);
        assert_eq!(game.object_view(opp).damage, 0);
        assert_eq!(
            game.object_view(own).counters.plus_one,
            u64::from(!remove_own)
        );
    }
}

#[test]
fn repeated_object_across_occurrences_receives_each_groups_distinct_instruction() {
    let mut game = scenario();
    let creature = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
    announce(&mut game, "Vibrant Outburst");
    choose(&mut game, creature);
    choose(&mut game, creature);
    pay(&mut game);
    assert_eq!(game.state.stack[0].targets.len(), 2);
    pass_pair(&mut game);
    assert_eq!(game.object_view(creature).damage, 3);
    assert!(game.object_view(creature).tapped);
}

#[test]
fn group_failure_is_independent_and_all_targets_illegal_prevents_every_effect() {
    for remove_both in [false, true] {
        let mut game = scenario();
        let a = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
        let b = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
        let spell = announce(&mut game, "Vibrant Outburst");
        choose(&mut game, a);
        choose(&mut game, b);
        pay(&mut game);
        game.move_object(a, Zone::Hand);
        if remove_both {
            game.move_object(b, Zone::Hand);
            game.move_object(b, Zone::Battlefield);
        }
        pass_pair(&mut game);
        assert_eq!(game.object_view(b).damage, 0);
        assert_eq!(game.object_view(b).tapped, !remove_both);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn optional_later_group_can_be_skipped_and_player_effect_does_not_use_creature_group() {
    for creature in [false, true] {
        let mut game = scenario();
        let own = if creature {
            Some(named(
                &mut game,
                Player::P0,
                "Grizzly Bears",
                Zone::Battlefield,
            ))
        } else {
            None
        };
        announce(&mut game, "Combat Tutorial");
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(Player::P1),
            },
        );
        if let Some(own) = own {
            choose(&mut game, own);
        }
        pay(&mut game);
        pass_pair(&mut game);
        assert_eq!(game.hand_count(Player::P1), 2);
        assert_eq!(game.hand_count(Player::P0), 0);
        if let Some(own) = own {
            assert_eq!(game.object_view(own).counters.plus_one, 1);
        }
    }
}

#[test]
fn target_focus_survives_private_library_continuation_and_replays() {
    let mut game = scenario();
    let a = named(&mut game, Player::P0, "Shivan Dragon", Zone::Battlefield);
    let b = named(&mut game, Player::P0, "Shivan Dragon", Zone::Battlefield);
    announce(&mut game, "Vibrant Outburst");
    choose(&mut game, a);
    choose(&mut game, b);
    pay(&mut game);
    static PROGRAM: &[Instruction] = &[
        Instruction::SelectGroup(1),
        Instruction::Scry(1),
        Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        },
    ];
    let item = game.state.stack.pop().unwrap();
    game.state.resolving = Some(Resolution {
        connive: None,
        item,
        program: PROGRAM,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        selection: None,
        search: None,
        choice: None,
    });
    game.run_program();
    assert_eq!(game.state.resolving.as_ref().unwrap().target_group, 1);
    assert!(game.observe(Player::P1).library_choice.is_none());
    let card = game
        .state
        .resolving
        .as_ref()
        .unwrap()
        .library
        .as_ref()
        .unwrap()
        .remaining[0];
    let before = game.snapshot();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(a).counters.plus_one, 0);
    assert_eq!(game.object_view(b).counters.plus_one, 1);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.snapshot(), after);
}

#[test]
fn within_group_distinctness_does_not_forbid_reuse_in_the_next_group() {
    let mut game = scenario();
    let a = named(&mut game, Player::P0, "Shivan Dragon", Zone::Battlefield);
    announce(&mut game, "Vibrant Outburst");
    static GROUPS: &[TargetSpec] = &[
        TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 2,
        },
        TargetSpec::one(TargetRule::Creature),
    ];
    let announcement = game.state.announcing.as_mut().unwrap();
    announcement.plan = TargetPlan::Groups(GROUPS);
    announcement.spec = GROUPS[0];
    choose(&mut game, a);
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    act(&mut game, Action::FinishTargets);
    choose(&mut game, a);
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(a).damage, 3);
    assert!(game.object_view(a).tapped);
}

#[test]
fn fighting_itself_is_one_noncombat_event_for_twice_current_power() {
    let mut game = scenario();
    let source = named(&mut game, Player::P0, "Shivan Dragon", Zone::Battlefield);
    game.state.objects[source as usize].growth.toughness = 10;
    game.state.objects[source as usize].gained_keywords = Keywords::LIFELINK;
    announce(&mut game, "Vibrant Outburst");
    choose(&mut game, source);
    choose(&mut game, source);
    pay(&mut game);
    static PROGRAM: &[Instruction] = &[Instruction::Fight {
        source: 0,
        recipient: 1,
    }];
    let item = game.state.stack.pop().unwrap();
    game.state.resolving = Some(Resolution {
        connive: None,
        item,
        program: PROGRAM,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        selection: None,
        search: None,
        choice: None,
    });
    game.run_program();
    game.state_based_actions();
    assert_eq!(game.object_view(source).damage, 10);
    assert_eq!(game.state.players[0].life, 30);
    assert!(!game.object_view(source).tapped);
    assert!(game.state.hits.is_empty());
}

#[test]
fn independent_own_opponent_target_masks_match_manual_complete_pairs() {
    // Enumerate physical ownership and opponent hexproof on four creatures.
    // The oracle uses manual owner/flag facts, never targeting helpers.
    for owners in 0u8..16 {
        for hexproof in [0u8, 5, 10, 15] {
            let mut game = scenario();
            let mut yours = Vec::new();
            let mut theirs = Vec::new();
            for index in 0..4 {
                let owner = if owners & (1 << index) == 0 {
                    Player::P0
                } else {
                    Player::P1
                };
                let id = named(&mut game, owner, "Shivan Dragon", Zone::Battlefield);
                if hexproof & (1 << index) != 0 {
                    game.state.objects[id as usize].gained_keywords = Keywords::HEXPROOF;
                }
                if owner == Player::P0 {
                    yours.push(id);
                } else if hexproof & (1 << index) == 0 {
                    theirs.push(id);
                }
            }
            let spell = named(&mut game, Player::P0, "Quarrel", Zone::Hand);
            let (mut mana, generic) = game.state.objects[spell as usize].card.cost();
            mana[4] += generic;
            game.state.players[0].mana = mana;
            let available = game
                .legal_actions()
                .contains(&Action::AnnounceCast { card: spell });
            assert_eq!(available, !yours.is_empty() && !theirs.is_empty());
            if !available {
                continue;
            }
            act(&mut game, Action::AnnounceCast { card: spell });
            let offered = |game: &Game| {
                game.legal_actions()
                    .into_iter()
                    .filter_map(|a| {
                        if let Action::ChooseTarget {
                            target: Target::Card(id),
                        } = a
                        {
                            Some(id)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(offered(&game), yours);
            let start = game.snapshot();
            for &own in &yours {
                game.restore(&start).unwrap();
                choose(&mut game, own);
                assert_eq!(offered(&game), theirs);
                let second = game.snapshot();
                for &opp in &theirs {
                    game.restore(&second).unwrap();
                    choose(&mut game, opp);
                    assert!(
                        game.legal_actions()
                            .iter()
                            .any(|a| matches!(a, Action::PayCosts { .. }))
                    );
                }
            }
        }
    }
}
