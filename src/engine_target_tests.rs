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
fn fund(game: &mut Game, player: Player, card: CardId) -> Mana {
    let (mut payment, generic) = game.state.objects[card as usize].card.cost();
    payment[0] += generic;
    game.state.players[player.index()].mana = payment;
    payment
}
fn begin_fancy(game: &mut Game) -> CardId {
    let card = named(game, Player::P0, "Fancy Footwork", Zone::Hand);
    let payment = fund(game, Player::P0, card);
    act(game, Action::BeginCast { card, payment });
    card
}

#[test]
fn targeted_entry_waits_for_announcement_before_stack_and_resolves_after_source_leaves() {
    let mut game = scenario();
    let archer = enter(&mut game, Player::P0, "Skeleton Archer");
    assert_eq!(game.state.pending, Pending::Targets { player: Player::P0 });
    assert!(game.state.stack.is_empty());
    assert!(!game.legal_actions().contains(&Action::Pass));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    let item = &game.state.stack[0];
    assert_eq!(item.targets[0].target, Target::Player(Player::P1));
    let old_source = item.object;
    game.move_object(archer, Zone::Hand);
    let pending = game.snapshot();
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 19);
    assert_ne!(game.object_ref(archer), old_source);
    let after = game.snapshot();
    game.restore(&pending).unwrap();
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn required_target_is_not_optional_even_when_effect_is_may() {
    let mut game = scenario();
    let sage = enter(&mut game, Player::P0, "Reclamation Sage");
    assert!(game.state.stack.is_empty() && game.state.announcing.is_none()); // No artifact/enchantment target.
    let giant = named(&mut game, Player::P1, "Iron Giant", Zone::Battlefield);
    game.move_object(sage, Zone::Hand);
    game.move_object(sage, Zone::Battlefield);
    game.prepare_triggers();
    assert!(!game.legal_actions().contains(&Action::FinishTargets));
    assert!(
        !game
            .legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(giant),
        },
    );
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
    let optional = game.snapshot();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.state.objects[giant as usize].zone, Zone::Battlefield);
    game.restore(&optional).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.state.objects[giant as usize].zone, Zone::Graveyard);
}

#[test]
fn illegal_announcement_choices_are_atomic_and_private_until_committed() {
    let mut game = scenario();
    let a = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let b = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = begin_fancy(&mut game);
    let before = game.snapshot();
    let revision = game.revision();
    for action in [
        Action::Pass,
        Action::FinishTargets,
        Action::ChooseTarget {
            target: Target::Player(Player::P0),
        },
        Action::ChooseTarget {
            target: Target::Card(u32::MAX),
        },
    ] {
        assert_eq!(game.apply(Player::P0, action), Err(Error::IllegalAction));
        assert_eq!(game.snapshot(), before);
        assert_eq!(game.revision(), revision);
    }
    assert_eq!(
        game.apply(
            Player::P1,
            Action::ChooseTarget {
                target: Target::Card(a)
            }
        ),
        Err(Error::WrongActor)
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    assert!(!game.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(a)
    }));
    assert!(game.legal_actions().contains(&Action::FinishTargets));
    assert_eq!(
        game.observe(Player::P0).announcing.unwrap().targets.len(),
        1
    );
    assert!(
        game.observe(Player::P1)
            .announcing
            .unwrap()
            .targets
            .is_empty()
    );
    let pending = game.snapshot();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    assert!(game.state.announcing.is_none());
    assert_eq!(game.state.stack[0].card, spell);
    assert_eq!(game.observe(Player::P1).stack[0].targets.len(), 2);
    let after = game.snapshot();
    game.restore(&pending).unwrap();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    assert_eq!(game.snapshot(), after);
}

#[test]
fn partially_invalid_multitarget_spell_affects_only_surviving_incarnations() {
    let mut game = scenario();
    let a = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let b = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[a as usize].tapped = true;
    game.state.objects[b as usize].tapped = true;
    begin_fancy(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    let chosen = game.snapshot();
    game.move_object(a, Zone::Hand);
    game.move_object(a, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.object_view(a).power, 2);
    assert_eq!(game.object_view(b).power, 4);
    assert!(!game.object_view(b).tapped);
    game.restore(&chosen).unwrap();
    game.move_object(a, Zone::Hand);
    game.move_object(b, Zone::Hand);
    pass_pair(&mut game);
    assert!(game.state.resolving.is_none());
    assert_eq!(game.object_view(a).power, 2);
    assert_eq!(game.object_view(b).power, 2);
}

#[test]
fn every_complete_small_target_set_is_reachable_without_duplicates_or_dead_ends() {
    use std::collections::BTreeSet;
    fn enumerate(game: Game, out: &mut BTreeSet<Vec<CardId>>) {
        if game.state.announcing.is_none() {
            let selected = game
                .state
                .stack
                .last()
                .unwrap()
                .targets
                .iter()
                .map(|bound| match bound.target {
                    Target::Card(id) => id,
                    _ => panic!("creature target"),
                })
                .collect();
            out.insert(selected);
            return;
        }
        let choices: Vec<_> = game
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets))
            .collect();
        assert!(!choices.is_empty());
        for choice in choices {
            let before = game.snapshot();
            let mut branch = game.clone();
            act(&mut branch, choice);
            assert_eq!(game.snapshot(), before);
            enumerate(branch, out);
        }
    }
    for n in 1..=4 {
        let mut game = scenario();
        let ids: Vec<_> = (0..n)
            .map(|_| named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield))
            .collect();
        begin_fancy(&mut game);
        let mut actual = BTreeSet::new();
        enumerate(game, &mut actual);
        let mut expected = BTreeSet::new();
        for (i, &a) in ids.iter().enumerate() {
            expected.insert(vec![a]);
            for &b in ids.iter().skip(i + 1) {
                expected.insert(vec![a, b]);
            }
        }
        assert_eq!(actual, expected);
    }
}

#[test]
fn optional_zero_targets_differs_from_all_chosen_targets_becoming_illegal() {
    // A synthetic optional-target variant of this supported program checks the
    // interpreter distinction without admitting a card with unsupported text.
    let mut game = scenario();
    let target = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = named(&mut game, Player::P0, "Moment of Craving", Zone::Hand);
    let payment = fund(&mut game, Player::P0, spell);
    let item = game.cast_item(Player::P0, spell, payment);
    assert!(game.start_announcement(item, false));
    game.state.announcing.as_mut().unwrap().spec = TargetSpec {
        rule: TargetRule::Creature,
        min: 0,
        max: 2,
    };
    let pending = game.snapshot();
    act(&mut game, Action::FinishTargets);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 22); // Zero selected targets: gain still happens.
    game.restore(&pending).unwrap();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    act(&mut game, Action::FinishTargets);
    game.move_object(target, Zone::Hand);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 20); // All selected targets invalid: whole spell fails.
}

#[test]
fn targeted_player_effect_uses_chosen_player_and_preserves_discard_ownership() {
    let mut game = scenario();
    enter(&mut game, Player::P0, "Vault Plunderer");
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P1), 1);
    assert_eq!(game.state.players[1].life, 19);
    assert_eq!(game.hand_count(Player::P0), 0);
    enter(&mut game, Player::P0, "Corrupt Court Official");
    assert!(!game.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Player(Player::P0)
    }));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(Player::P1));
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.hand_count(Player::P1), 0);
}

#[test]
fn reanimation_binds_graveyard_incarnation_and_dispatches_new_entry_triggers() {
    let mut game = scenario();
    let hunter = named(&mut game, Player::P0, "Helpful Hunter", Zone::Graveyard);
    let enemy = named(&mut game, Player::P1, "Helpful Hunter", Zone::Graveyard);
    let spell = named(&mut game, Player::P0, "Emerge from the Cocoon", Zone::Hand);
    fund(&mut game, Player::P0, spell);
    assert!(!game.legal_actions().iter().any(|a| matches!(a, Action::Cast {card, target: Some(Target::Card(id)), ..} if *card == spell && *id == enemy)));
    cast(&mut game, spell, Some(Target::Card(hunter)));
    let old = game.object_ref(hunter);
    let pending = game.snapshot();
    pass_pair(&mut game);
    assert_ne!(game.object_ref(hunter), old);
    assert_eq!(game.state.objects[hunter as usize].zone, Zone::Battlefield);
    assert!(game.state.objects[hunter as usize].summoning_sick);
    assert_eq!(game.state.players[0].life, 23);
    assert_eq!(game.state.stack.len(), 1); // Helpful Hunter entry trigger.
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 1);
    let after = game.snapshot();
    game.restore(&pending).unwrap();
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn all_invalid_targets_suppress_nontargeted_remainder_of_spell() {
    let mut game = scenario();
    let bears = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = named(&mut game, Player::P0, "Moment of Craving", Zone::Hand);
    fund(&mut game, Player::P0, spell);
    cast(&mut game, spell, Some(Target::Card(bears)));
    game.move_object(bears, Zone::Hand);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.state.objects[spell as usize].zone, Zone::Graveyard);
}

#[test]
fn object_filters_respect_owner_type_zone_and_original_source_reference() {
    let mut game = scenario();
    let source = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let other = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let land = named(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let old = game.object_ref(source);
    let rule = TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1,
        relation: 0,
        other: true,
        nonland: true,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    };
    assert!(!game.target_matches(rule, Player::P0, old, Target::Card(source)));
    assert!(game.target_matches(rule, Player::P0, old, Target::Card(other)));
    for target in [enemy, land] {
        assert!(!game.target_matches(rule, Player::P0, old, Target::Card(target)));
    }
    game.move_object(other, Zone::Graveyard);
    assert!(!game.target_matches(rule, Player::P0, old, Target::Card(other)));
    game.move_object(source, Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    assert!(game.target_matches(rule, Player::P0, old, Target::Card(source))); // A different object is "other".
}

#[test]
fn target_announcements_interleave_with_apnap_order_without_granting_priority() {
    let mut game = scenario();
    let archer = named(&mut game, Player::P0, "Skeleton Archer", Zone::Hand);
    let plunderer = named(&mut game, Player::P1, "Vault Plunderer", Zone::Hand);
    game.move_object(archer, Zone::Battlefield);
    game.move_object(plunderer, Zone::Battlefield);
    game.prepare_triggers();
    assert_eq!(game.actor(), Some(Player::P0));
    assert!(matches!(game.state.pending, Pending::Targets { .. }));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(game.actor(), Some(Player::P1));
    assert!(matches!(game.state.pending, Pending::Targets { .. }));
    assert_eq!(game.state.stack.len(), 1);
    assert!(!game.legal_actions().contains(&Action::Pass));
    let pending = game.snapshot();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(
        game.state
            .stack
            .iter()
            .map(|item| item.controller)
            .collect::<Vec<_>>(),
        vec![Player::P0, Player::P1]
    );
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.actor(), Some(Player::P0));
    let after = game.snapshot();
    game.restore(&pending).unwrap();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    assert_eq!(game.snapshot(), after);
}
