//! Ghost Vacuum's linked exile, paid return, and persistent returned characteristics.
use super::*;

fn vacuum_exile(g: &mut Game, source: CardId, target: CardId) {
    big_helpers::checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    big_helpers::checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
    big_helpers::settle(g);
    g.state.objects[source as usize].tapped = false;
}

#[test]
fn ghost_vacuum_returns_linked_creatures_under_controller_and_leaves_others() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Hand);
        big_helpers::cast(&mut g, source, None, &[]);
        big_helpers::settle(&mut g);
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        for card in [own, enemy, land] {
            vacuum_exile(&mut g, source, card);
        }
        let before = g.snapshot();
        let mut branch = g.clone();
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        for id in [own, enemy] {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.object_view(id).controller, p);
            assert_eq!((g.stats(id).power, g.stats(id).toughness), (1, 1));
            assert_eq!(g.counter_count(id, CounterKind::Flying), 1);
            assert!(g.keywords(id).contains(Keywords::FLYING));
            assert!(g.has_subtype(id, "Spirit"));
        }
        assert!(g.has_subtype(own, "Bear"));
        assert!(g.has_subtype(enemy, "Giant"));
        g.finish_cleanup();
        assert_eq!((g.stats(enemy).power, g.stats(enemy).toughness), (1, 1));
        assert!(g.has_subtype(enemy, "Spirit"));
        branch.restore(&before).unwrap();
        big_helpers::checked(
            &mut branch,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut branch);
        big_helpers::settle(&mut branch);
        branch.finish_cleanup();
        assert_eq!(branch.snapshot(), g.snapshot());
    }
}

#[test]
fn ghost_vacuum_later_incarnations_do_not_reuse_old_links_and_timing_is_sorcery() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        vacuum_exile(&mut g, source, target);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        g.state.active = p.other();
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        g.state.active = p;
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
    }
}

#[test]
fn ghost_vacuum_entry_counters_use_replacements_and_queued_entry_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
        for id in [own, enemy] {
            vacuum_exile(&mut g, source, id);
        }
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Haliya, Guided by Light", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::Flying), 2);
        assert_eq!(g.counter_count(enemy, CounterKind::Flying), 2);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}

#[test]
fn ghost_vacuum_spirit_type_is_visible_to_entry_replacement_projection() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let entrant = put(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
        vacuum_exile(&mut g, source, entrant);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::EntryValue { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        let batch = g.state.entry_batch.as_ref().unwrap();
        let movement = batch.moves.iter().find(|m| m.before.id == entrant).unwrap();
        let types = Game::entry_choice_subtypes(movement);
        assert!(types.has(movement.before.card, "Spirit"));
        assert!(types.has(movement.before.card, "Construct"));
        big_helpers::settle(&mut g);
        assert!(g.has_subtype(entrant, "Spirit"));
    }
}

#[test]
fn ghost_vacuum_entry_observer_sees_the_one_power_characteristic() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let creature = put(&mut g, p, "Quakestrider Ceratops", Zone::Graveyard);
        vacuum_exile(&mut g, source, creature);
        put(&mut g, p, "Garruk's Uprising", Zone::Battlefield);
        let hand = g.hand_count(p);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.stats(creature).power, 1);
        assert_eq!(g.hand_count(p), hand);
    }
}

#[test]
fn ghost_vacuum_preserves_angel_type_for_giada_entry_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let angel = put(&mut g, p, "Serra Angel", Zone::Graveyard);
        vacuum_exile(&mut g, source, angel);
        put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert!(g.has_subtype(angel, "Angel"));
        assert!(g.has_subtype(angel, "Spirit"));
        assert_eq!(g.counter_count(angel, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(angel, CounterKind::Flying), 1);
        assert_eq!((g.stats(angel).power, g.stats(angel).toughness), (2, 2));
    }
}

#[test]
fn ghost_vacuum_effect_is_not_copied_and_remains_after_recipient_changes_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let returned = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        vacuum_exile(&mut g, source, returned);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        let copy = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let copied = g.copiable_recipe(g.object_ref(returned));
        g.apply_temporary_copy(g.object_ref(copy), copied, CopyDeadline::EndOfTurn);
        assert_eq!((g.stats(copy).power, g.stats(copy).toughness), (3, 3));
        assert!(!g.has_subtype(copy, "Spirit"));
        assert_eq!(g.counter_count(copy, CounterKind::Flying), 0);
        let recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.apply_temporary_copy(g.object_ref(returned), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(
            (g.stats(returned).power, g.stats(returned).toughness),
            (1, 1)
        );
        assert!(g.has_subtype(returned, "Spirit"));
        assert!(g.has_subtype(returned, "Bear"));
        assert_eq!(g.counter_count(returned, CounterKind::Flying), 1);
    }
}

#[test]
fn ghost_vacuum_sequential_counter_replacements_preserve_entering_characteristics() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let angel = put(&mut g, p, "Serra Angel", Zone::Graveyard);
        vacuum_exile(&mut g, source, angel);
        put(&mut g, p, "Giada, Font of Hope", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(
            &mut g,
            p,
            "Doc Samson, Super Psychiatrist",
            Zone::Battlefield,
        );
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        for _ in 0..60 {
            if g.object_view(angel).zone == Zone::Battlefield
                && matches!(g.state.pending, Pending::Priority)
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
            {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseReplacement { .. } | Action::Pass | Action::PutTrigger { .. }
                    )
                })
                .unwrap();
            big_helpers::checked(&mut g, action);
        }
        assert!(g.has_subtype(angel, "Angel"));
        assert!(g.has_subtype(angel, "Spirit"));
        let plus = g.counter_count(angel, CounterKind::PlusOne);
        let flying = g.counter_count(angel, CounterKind::Flying);
        assert!(matches!(plus, 3 | 4));
        assert!(matches!(flying, 3 | 4));
        assert_eq!(g.stats(angel).power, 1 + i128::from(plus));
    }
}

#[test]
fn ghost_vacuum_simultaneous_giada_entries_use_controller_and_prior_angels() {
    for player in [Player::P0, Player::P1] {
        for owner in [player, player.other()] {
            let mut game = scenario(player);
            let vacuum = put(&mut game, player, "Ghost Vacuum", Zone::Battlefield);
            put(&mut game, player, "Giada, Font of Hope", Zone::Battlefield);
            let skrull = put(&mut game, owner, "Undercover Skrull", Zone::Graveyard);
            let angel = put(&mut game, player, "Serra Angel", Zone::Graveyard);
            vacuum_exile(&mut game, vacuum, skrull);
            vacuum_exile(&mut game, vacuum, angel);
            // Skrull's ability checks the new controller's graveyard.
            put(&mut game, player, "Bear Cub", Zone::Graveyard);
            put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
            big_helpers::checked(
                &mut game,
                Action::AnnounceActivate {
                    card: vacuum,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut game);
            big_helpers::settle(&mut game);
            for id in [skrull, angel] {
                assert_eq!(game.object_view(id).zone, Zone::Battlefield);
                assert_eq!(game.object_view(id).controller, player);
                assert!(game.has_subtype(id, "Angel"));
                assert_eq!(
                    game.counter_count(id, CounterKind::PlusOne),
                    1,
                    "only Giada was an Angel before both entrants"
                );
                assert_eq!(game.counter_count(id, CounterKind::Flying), 1);
            }
            assert_eq!(game.object_view(skrull).owner, owner);
        }
    }
}
