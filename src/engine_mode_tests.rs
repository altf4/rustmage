use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}
fn modes(game: &Game) -> Vec<u16> {
    game.legal_actions()
        .into_iter()
        .filter_map(|a| {
            if let Action::ChooseMode { mode } = a {
                Some(mode)
            } else {
                None
            }
        })
        .collect()
}
fn announce(game: &mut Game, name: &str, mana: Mana) -> CardId {
    let id = named(game, Player::P0, name, Zone::Hand);
    game.state.players[0].mana = mana;
    act(game, Action::AnnounceCast { card: id });
    assert_eq!(game.state.pending, Pending::Modes { player: Player::P0 });
    id
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

#[test]
fn casting_requires_a_completable_mode_and_funding_without_shortcut_bypasses() {
    let mut game = scenario();
    let spell = named(&mut game, Player::P0, "Abrade", Zone::Hand);
    game.state.players[0].mana = [1, 0, 0, 1, 0, 0];
    assert!(!game.legal_actions().iter().any(|a| matches!(a,
        Action::AnnounceCast { card } if *card == spell)));
    let hexproof = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[hexproof as usize].gained_keywords = Keywords::HEXPROOF;
    assert!(!game.legal_actions().iter().any(|a| matches!(a,
        Action::AnnounceCast { card } if *card == spell)));
    let artifact = named(&mut game, Player::P0, "Gilded Lotus", Zone::Battlefield);
    rejected(
        &mut game,
        Action::Cast {
            card: spell,
            target: Some(Target::Card(artifact)),
            payment: [1, 0, 0, 1, 0, 0],
        },
    );
    rejected(
        &mut game,
        Action::BeginCast {
            card: spell,
            payment: [1, 0, 0, 1, 0, 0],
        },
    );
    act(&mut game, Action::AnnounceCast { card: spell });
    assert_eq!(modes(&game), vec![1]);
    rejected(&mut game, Action::ChooseMode { mode: 0 });
    rejected(&mut game, Action::ChooseMode { mode: u16::MAX });
    rejected(
        &mut game,
        Action::PayCosts {
            payment: [1, 0, 0, 1, 0, 0],
        },
    );
    act(&mut game, Action::ChooseMode { mode: 1 });
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(hexproof),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    rejected(&mut game, Action::ChooseMode { mode: 0 });
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
}

#[test]
fn modes_targets_and_payment_have_complete_independent_snapshot_boundaries() {
    let mut game = scenario();
    let target = named(&mut game, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = announce(&mut game, "Abrade", [1, 0, 0, 1, 0, 0]);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.object_view(spell).zone, Zone::Stack);
    assert_eq!(game.state.players[0].mana, [1, 0, 0, 1, 0, 0]);
    assert_eq!(game.observe(Player::P0).mode_choices.len(), 1);
    assert!(game.observe(Player::P1).mode_choices.is_empty());
    assert!(game.observe(Player::P1).actions.is_empty());
    assert_eq!(game.observe(Player::P1).announcing.unwrap().mode, None);
    let actions = [
        Action::ChooseMode { mode: 0 },
        Action::ChooseTarget {
            target: Target::Card(target),
        },
        Action::PayCosts {
            payment: [1, 0, 0, 1, 0, 0],
        },
    ];
    for action in actions {
        let before = game.snapshot();
        let mut branch = game.clone();
        act(&mut branch, action.clone());
        act(&mut game, action.clone());
        assert_eq!(game.snapshot(), branch.snapshot());
        let after = game.snapshot();
        game.restore(&before);
        act(&mut game, action);
        assert_eq!(game.snapshot(), after);
    }
    assert_eq!(game.state.stack[0].mode, Some(0));
    assert_eq!(game.state.players[0].mana, [0; 6]);
    let before = game.snapshot();
    pass_pair(&mut game);
    assert_eq!(game.object_view(target).damage, 3);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    let after = game.snapshot();
    game.restore(&before);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn mode_selection_precedes_targets_and_mana_activation_during_payment() {
    let mut game = scenario();
    let target = named(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let forest = named(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let mountain = named(&mut game, Player::P0, "Mountain", Zone::Battlefield);
    let spell = announce(&mut game, "Abrade", [0; 6]);
    rejected(&mut game, Action::TapMana { card: forest });
    act(&mut game, Action::ChooseMode { mode: 0 });
    rejected(&mut game, Action::TapMana { card: forest });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    act(&mut game, Action::TapMana { card: forest });
    let partial = game.snapshot();
    act(&mut game, Action::TapMana { card: mountain });
    pay(&mut game);
    assert_eq!(game.state.stack[0].mode, Some(0));
    assert_eq!(game.state.stack[0].card, spell);
    let after = game.snapshot();
    game.restore(&partial);
    act(&mut game, Action::TapMana { card: mountain });
    pay(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn unpayable_modal_spell_is_not_offered_even_with_legal_targets() {
    let mut game = scenario();
    named(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let spell = named(&mut game, Player::P0, "Abrade", Zone::Hand);
    game.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    rejected(&mut game, Action::AnnounceCast { card: spell });
    named(&mut game, Player::P0, "Mountain", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    assert_eq!(modes(&game), vec![0]);
}

#[test]
fn targetless_mode_can_cast_without_other_spells_and_cannot_counter_itself() {
    let mut game = scenario();
    let spell = announce(&mut game, "School Daze", [3, 2, 0, 0, 0, 0]);
    assert_eq!(modes(&game), vec![0]);
    let view = game.observe(Player::P0);
    assert!(view.mode_choices[0].text.starts_with("Do Homework —"));
    rejected(&mut game, Action::ChooseMode { mode: 1 });
    act(&mut game, Action::ChooseMode { mode: 0 });
    assert_eq!(game.state.pending, Pending::PayCosts { player: Player::P0 });
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(spell),
        },
    );
    pay(&mut game);
    let hand = game.hand_count(Player::P0);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), hand + 3);
}

#[test]
fn selected_counter_mode_uses_its_own_target_and_only_its_draw_effect() {
    let mut game = scenario();
    let shock = named(&mut game, Player::P0, "Shock", Zone::Hand);
    game.state.players[0].mana[3] = 1;
    cast(&mut game, shock, Some(Target::Player(Player::P1)));
    let spell = announce(&mut game, "School Daze", [3, 2, 0, 0, 0, 0]);
    assert_eq!(modes(&game), vec![0, 1]);
    act(&mut game, Action::ChooseMode { mode: 1 });
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(spell),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(shock),
        },
    );
    pay(&mut game);
    let hand = game.hand_count(Player::P0);
    pass_pair(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(Player::P0), hand + 1);
    assert_eq!(game.state.players[1].life, 20);
    assert!(game.state.stack.is_empty());
}

#[test]
fn illegal_target_fizzles_selected_mode_without_switching_or_running_tail() {
    let mut game = scenario();
    let artifact = named(&mut game, Player::P1, "Gilded Lotus", Zone::Battlefield);
    let spell = announce(&mut game, "Thorin's Last Stand", [2, 2, 0, 0, 0, 0]);
    act(&mut game, Action::ChooseMode { mode: 1 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    pay(&mut game);
    game.move_object(artifact, Zone::Hand);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn countered_modal_spell_executes_no_mode_and_new_cast_starts_unselected() {
    let mut game = scenario();
    let target = named(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let spell = announce(&mut game, "Abrade", [1, 0, 0, 1, 0, 0]);
    act(&mut game, Action::ChooseMode { mode: 0 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut game);
    act(&mut game, Action::Pass);
    let counter = named(&mut game, Player::P1, "Counterspell", Zone::Hand);
    game.state.players[1].mana[1] = 2;
    cast(&mut game, counter, Some(Target::Spell(spell)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(target).damage, 0);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    game.move_object(spell, Zone::Hand);
    game.state.players[0].mana = [1, 0, 0, 1, 0, 0];
    act(&mut game, Action::AnnounceCast { card: spell });
    assert_eq!(game.state.announcing.as_ref().unwrap().item.mode, None);
    assert_eq!(game.state.pending, Pending::Modes { player: Player::P0 });
}

#[test]
fn selected_surveil_mode_owns_continuation_and_replays_partial_library_choice() {
    let mut game = scenario();
    announce(&mut game, "Spellgyre", [2, 2, 0, 0, 0, 0]);
    act(&mut game, Action::ChooseMode { mode: 1 });
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.resolving.as_ref().unwrap().item.mode, Some(1));
    assert!(game.observe(Player::P1).library_choice.is_none());
    let cards = game
        .state
        .resolving
        .as_ref()
        .unwrap()
        .library
        .as_ref()
        .unwrap()
        .remaining
        .clone();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: cards[0],
            destination: LibraryDestination::Graveyard,
        },
    );
    let partial = game.snapshot();
    let hand = game.hand_count(Player::P0);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: cards[1],
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.hand_count(Player::P0), hand + 2);
    let after = game.snapshot();
    game.restore(&partial);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: cards[1],
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.snapshot(), after);
}

#[test]
fn mode_specific_graveyard_and_battlefield_predicates_stay_distinct() {
    let mut game = scenario();
    let yours = named(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let theirs = named(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let live = named(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    announce(&mut game, "Live or Die", [3, 0, 2, 0, 0, 0]);
    act(&mut game, Action::ChooseMode { mode: 0 });
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(theirs),
        },
    );
    rejected(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(live),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(yours),
        },
    );
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(yours).zone, Zone::Battlefield);
    assert_eq!(game.object_view(live).zone, Zone::Battlefield);
    assert_eq!(game.object_view(theirs).zone, Zone::Graveyard);
}
