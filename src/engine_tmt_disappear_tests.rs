//! Paid TMT scenarios, including owned raw/index branches, restore and private choices.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
fn history(g: &Game, player: Player, expected: bool) {
    assert_eq!(
        g.state.players[player.index()].permanent_left_battlefield_this_turn,
        expected
    );
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            g.observe(viewer).players[player.index()].permanent_left_battlefield_this_turn,
            expected
        );
    }
}
fn count(g: &Game, p: Player, factory: &str) -> usize {
    let card = Card::from_name(factory).unwrap();
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card == card)
        .count()
}
fn bounce(g: &mut Game, p: Player, id: CardId) {
    let spell = named(g, p, "Unsummon", Zone::Hand);
    cast(g, spell, None, &[Target::Card(id)]);
    settle(g);
}
fn end(g: &mut Game) {
    g.begin_step(Step::End);
    settle(g);
}
fn activate(g: &mut Game, source: CardId, target: Option<CardId>) {
    let a = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate {card, ability: 0, target: t, ..} if *card == source && *t == target.map(Target::Card))).expect("paid activation");
    checked(g, a);
    settle(g);
}

#[test]
fn tmt_disappear_history_tracks_all_destinations_and_ignores_nonbattlefield_moves() {
    for p in [Player::P0, Player::P1] {
        for destination in [Zone::Hand, Zone::Graveyard, Zone::Exile, Zone::Library] {
            let mut g = setup(p);
            for from in [
                Zone::Hand,
                Zone::Library,
                Zone::Stack,
                Zone::Graveyard,
                Zone::Exile,
            ] {
                if from == destination {
                    continue;
                }
                let id = named(&mut g, p, "Bear Cub", from);
                g.move_object(id, destination);
                history(&g, p, false);
            }
            let id = named(&mut g, p, "Forest", Zone::Battlefield);
            let snapshot = g.snapshot();
            let mut fork = {
                let parent = g.clone();
                parent.clone()
            };
            g.move_object(id, destination);
            fork.move_object(id, destination);
            assert_eq!(g.snapshot(), fork.snapshot());
            history(&g, p, true);
            history(&g, p.other(), false);
            g.restore(&snapshot).unwrap();
            history(&g, p, false);
            g.move_object(id, destination);
            g.begin_turn();
            history(&g, p, false);
        }
    }
}

#[test]
fn tmt_disappear_departure_belongs_to_pre_move_controller_including_foreign_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let stolen = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let mut transfer = g.activation_item(p, own, 0);
        transfer.targets.push(g.bind_target(Target::Card(stolen)));
        g.gain_control(&transfer, Subject::Targets, PlayerSet::Controller);
        history(&g, p, false);
        history(&g, p.other(), false);
        bounce(&mut g, p, stolen);
        assert_eq!(g.state.objects[stolen as usize].owner, p.other());
        assert_eq!(g.state.objects[stolen as usize].zone, Zone::Hand);
        history(&g, p, true);
        history(&g, p.other(), false);
        let mut h = setup(p);
        h.create_tokens(
            p,
            Card::from_name("@token/7506aa37538b239479a880e5").unwrap(),
            1,
            false,
        );
        history(&h, p, false);
        let id = h
            .state
            .objects
            .iter()
            .position(|o| o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        let token = h.object_ref(id);
        h.move_object(id, Zone::Exile);
        h.state_based_actions();
        history(&h, p, true);
        assert!(!h.on_battlefield(id));
        assert_ne!(h.object_ref(id), token);
    }
}

#[test]
fn tmt_foot_mystic_paid_entry_requires_own_history_and_creates_exact_black_ninja() {
    for p in [Player::P0, Player::P1] {
        for departure in [None, Some(p), Some(p.other())] {
            let mut g = setup(p);
            if let Some(who) = departure {
                let bear = named(&mut g, who, "Bear Cub", Zone::Battlefield);
                bounce(&mut g, p, bear);
            }
            let id = paid(&mut g, p, "Foot Mystic");
            assert_eq!(g.stats(id).power, 2);
            assert_eq!(g.stats(id).toughness, 4);
            assert!(g.keywords(id).contains(Keywords::LIFELINK));
            assert_eq!(
                count(&g, p, "@token/ninja_1_1_black"),
                usize::from(departure == Some(p))
            );
            if departure == Some(p) {
                let ninja = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find(|(_, o)| {
                        o.card == Card::from_name("@token/ninja_1_1_black").unwrap()
                            && o.zone == Zone::Battlefield
                    })
                    .unwrap()
                    .0 as CardId;
                let view = g.object_view(ninja);
                assert_eq!(view.colors, 4);
                assert!(g.has_subtype(ninja, "Ninja"));
                assert_eq!((view.power, view.toughness), (1, 1));
            }
        }
    }
}

#[test]
fn tmt_foot_mystic_trigger_survives_source_departure_and_qualifies_blink_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        bounce(&mut g, p, bear);
        let id = named(&mut g, p, "Foot Mystic", Zone::Hand);
        cast(&mut g, id, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        // The ETB packet is now queued with the historical controller.
        g.move_object(id, Zone::Exile);
        settle(&mut g);
        assert_eq!(count(&g, p, "@token/ninja_1_1_black"), 1);
        g.move_object(id, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(count(&g, p, "@token/ninja_1_1_black"), 2);
    }
}

#[test]
fn tmt_putrid_pals_paid_hybrid_entry_counters_are_replacement_not_etb_trigger() {
    for p in [Player::P0, Player::P1] {
        for (departed, expected) in [(false, 0), (true, 2)] {
            let mut g = setup(p);
            if departed {
                let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                bounce(&mut g, p, bear);
            }
            let id = paid(&mut g, p, "Putrid Pals");
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), expected);
            assert_eq!(g.stats(id).power, 3 + i128::from(expected));
            assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
            assert!(g.state.pending_triggers.is_empty());
        }
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        bounce(&mut g, p, bear);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let pals = paid(&mut g, p, "Putrid Pals");
        assert_eq!(g.counter_count(pals, CounterKind::PlusOne), 4);
    }
}

#[test]
fn tmt_michelangelo_end_step_requires_beginning_history_and_own_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Michelangelo, Game Master");
        end(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        // A later departure does not retroactively create a missed trigger.
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(bear, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        g.state.active = p.other();
        end(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        g.state.active = p;
        end(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        g.begin_turn();
        history(&g, p, false);
    }
}

#[test]
fn tmt_michelangelo_queued_counter_does_not_follow_source_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Michelangelo, Game Master");
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        bounce(&mut g, p, bear);
        g.begin_step(Step::End);
        g.prepare_triggers();
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
    }
}

#[test]
fn tmt_insectoid_scry_is_private_branchable_and_can_bottom_the_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Insectoid Exterminator");
        assert!(g.keywords(id).contains(Keywords::FLYING));
        end(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        bounce(&mut g, p, bear);
        let top = named(&mut g, p, "Shock", Zone::Library);
        g.begin_step(Step::End);
        for _ in 0..10 {
            if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                break;
            }
            g.prepare_triggers();
            let action = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PutTrigger { .. }))
                    .unwrap()
            };
            checked(&mut g, action);
        }
        assert_eq!(g.actor(), Some(p));
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        reject(
            &mut g,
            Action::ArrangeLibrary {
                card: top + 100,
                destination: LibraryDestination::Bottom,
            },
        );
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library[0], top);
    }
}

#[test]
fn tmt_lord_dregg_disappear_token_can_pay_draw_and_non_tokens_cannot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Lord Dregg, Insect Invader");
        assert!(g.keywords(id).contains(Keywords::FLYING));
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.legal_actions().iter().any(|a| matches!(a,Action::AnnounceActivate{card,..}|Action::BeginActivate{card,..}|Action::Activate{card,..} if *card==id)));
        bounce(&mut g, p, bear);
        end(&mut g);
        assert_eq!(count(&g, p, "@token/insect_warrior_1_1_black_flying"), 1);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.card == Card::from_name("@token/insect_warrior_1_1_black_flying").unwrap()
            })
            .unwrap()
            .0 as CardId;
        assert!(g.keywords(token).contains(Keywords::FLYING));
        assert!(g.has_subtype(token, "Insect"));
        assert!(g.has_subtype(token, "Warrior"));
        let hand = g.observe(p).players[p.index()].hand_count;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let not_token = g.object_ref(id);
        reject(&mut g, Action::ChooseAdditionalCost { object: not_token });
        let witness = g.object_ref(token);
        checked(&mut g, Action::ChooseAdditionalCost { object: witness });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(id, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.observe(p).players[p.index()].hand_count, hand + 1);
        assert_eq!(count(&g, p, "@token/insect_warrior_1_1_black_flying"), 0);
    }
}

#[test]
fn tmt_filcher_food_and_escapees_mutagen_leave_triggers_work_on_bounce_and_death() {
    for p in [Player::P0, Player::P1] {
        for (name, factory) in [
            ("Featherbrained Filcher", "@token/283fd83a9aec83a51b736926"),
            ("Zoo Escapees", "@token/7506aa37538b239479a880e5"),
        ] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            assert_eq!(count(&g, p, factory), 0);
            bounce(&mut g, p, id);
            assert_eq!(count(&g, p, factory), 1);
            cast(&mut g, id, None, &[]);
            settle(&mut g);
            let shock = named(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, None, &[Target::Card(id)]);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(count(&g, p, factory), 2);
            let token = g
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| {
                    o.zone == Zone::Battlefield && o.card == Card::from_name(factory).unwrap()
                })
                .unwrap()
                .0 as CardId;
            if name == "Featherbrained Filcher" {
                let life = g.state.players[p.index()].life;
                activate(&mut g, token, None);
                assert_eq!(g.state.players[p.index()].life, life + 3);
            } else {
                let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.step = Step::End;
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a,Action::Activate{card,..}if *card==token))
                );
                g.state.step = Step::Main2;
                activate(&mut g, token, Some(bear));
                assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
            }
            assert_eq!(count(&g, p, factory), 1);
        }
    }
}

#[test]
fn tmt_action_news_crew_channel_is_paid_from_hand_at_instant_speed_and_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let crew = named(&mut g, p, "Action News Crew", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.active = p.other();
        g.state.step = Step::End;
        g.state.players[p.index()].mana = [0; 6];
        let channel = Action::Activate {
            card: crew,
            ability: 0,
            target: None,
            payment: [0, 0, 0, 0, 0, 6],
            counter_payment: Counters::default(),
        };
        reject(&mut g, channel.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        let before = g.observe(p).players[p.index()].hand_count;
        checked(&mut g, channel);
        assert_eq!(g.state.objects[crew as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 0);
        assert_eq!(g.observe(p).players[p.index()].hand_count, before);
        history(&g, p, false); // Discarding the channel source is not a permanent departure.
        let mut h = setup(p);
        let crew = paid(&mut h, p, "Action News Crew");
        assert_eq!((h.stats(crew).power, h.stats(crew).toughness), (2, 2));
        assert!(h.keywords(crew).contains(Keywords::VIGILANCE));
        assert!(
            !h.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Activate{card,..}if *card==crew))
        );
    }
}

#[test]
fn tmt_channel_applies_to_resolution_world_and_draws_even_without_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let crew = named(&mut g, p, "Action News Crew", Zone::Hand);
        checked(
            &mut g,
            Action::Activate {
                card: crew,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 0, 0, 6],
                counter_payment: Counters::default(),
            },
        );
        // A creature entering after activation is included in the non-targeted effect.
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 2);
        let mut h = setup(p);
        let crew = named(&mut h, p, "Action News Crew", Zone::Hand);
        let top = *h.state.players[p.index()].library.last().unwrap();
        checked(
            &mut h,
            Action::Activate {
                card: crew,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 0, 0, 6],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut h);
        assert_eq!(h.state.objects[top as usize].zone, Zone::Hand);
    }
}

fn armaggon_trigger(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Armaggon, Future Shark", Zone::Hand);
    // Flash permits the real paid spell in the opponent's end step.
    g.state.active = p.other();
    g.state.step = Step::End;
    cast(g, id, None, &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    g.prepare_triggers();
    id
}
#[test]
fn tmt_armaggon_flash_allows_zero_through_three_targets_and_rejects_fourth_or_duplicate() {
    for p in [Player::P0, Player::P1] {
        for amount in 0..=3 {
            let mut g = setup(p);
            let ids = (0..4)
                .map(|_| named(&mut g, p.other(), "Hill Giant", Zone::Battlefield))
                .collect::<Vec<_>>();
            let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            let source = armaggon_trigger(&mut g, p);
            assert!(g.keywords(source).contains(Keywords::FLASH));
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (9, 6));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            for id in ids.iter().take(amount) {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(*id),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(*id),
                    },
                );
            }
            if amount == 3 {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(ids[3]),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            for (index, id) in ids.iter().enumerate() {
                assert_eq!(
                    g.state.objects[*id as usize].zone,
                    if index < amount {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}

#[test]
fn tmt_armaggon_partial_target_invalidation_and_source_departure_preserve_remaining_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let second = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = armaggon_trigger(&mut g, p);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        checked(&mut g, Action::FinishTargets);
        // The targeted ability has an independent lifetime from its source.
        g.move_object(source, Zone::Exile);
        g.move_object(first, Zone::Hand);
        g.move_object(first, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn tmt_high_flying_ace_paid_sorcery_grant_filters_both_announcement_and_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "High-Flying Ace");
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let grant = Action::Activate {
            card: source,
            ability: 0,
            target: Some(Target::Card(bear)),
            payment: [1, 0, 0, 0, 0, 3],
            counter_payment: Counters::default(),
        };
        reject(
            &mut g,
            Action::Activate {
                card: source,
                ability: 0,
                target: Some(Target::Card(source)),
                payment: [1, 0, 0, 0, 0, 3],
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::End;
        reject(&mut g, grant.clone());
        g.state.step = Step::Main2;
        checked(&mut g, grant);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::FLYING));

        let mut h = setup(p);
        let source = paid(&mut h, p, "High-Flying Ace");
        let bear = named(&mut h, p, "Bear Cub", Zone::Battlefield);
        checked(
            &mut h,
            Action::Activate {
                card: source,
                ability: 0,
                target: Some(Target::Card(bear)),
                payment: [1, 0, 0, 0, 0, 3],
                counter_payment: Counters::default(),
            },
        );
        // An instant resolves before the ability and gives the target flying.
        let leap = named(&mut h, p, "Acrobatic Leap", Zone::Hand);
        cast(&mut h, leap, None, &[Target::Card(bear)]);
        checked(&mut h, Action::Pass);
        checked(&mut h, Action::Pass);
        assert!(h.keywords(bear).contains(Keywords::FLYING));
        settle(&mut h);
        assert_eq!(
            h.observe(p)
                .keyword_effects
                .iter()
                .filter(|e| e.recipient.card == bear && e.keywords.contains(Keywords::FLYING))
                .count(),
            1
        );
    }
}

fn resolve_choices(g: &mut Game, target: CardId, accept: bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let desired = Action::ChooseTarget {
            target: Target::Card(target),
        };
        let action = if actions.contains(&desired) {
            desired
        } else if actions.contains(&Action::ChooseOptional { accept }) {
            Action::ChooseOptional { accept }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .expect("resolution choice")
        };
        checked(g, action);
    }
    panic!("unresolved {:?}", g.state.pending);
}
#[test]
fn tmt_mouser_foundry_paid_entry_and_sacrifice_create_robot_and_deal_damage_from_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Mouser Foundry");
        assert_eq!(count(&g, p, "@token/667dad32c9f20b70d5e9bda3"), 1);
        let robot = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.card == Card::from_name("@token/667dad32c9f20b70d5e9bda3").unwrap()
            })
            .unwrap()
            .0 as CardId;
        assert_eq!((g.stats(robot).power, g.stats(robot).toughness), (1, 1));
        assert_eq!(g.object_view(robot).colors, 0);
        assert_eq!(g.current_kinds(robot) & (1 | 16), 1 | 16);
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==source&&*t==victim)).unwrap();
        checked(&mut g, action);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        assert_eq!(count(&g, p, "@token/667dad32c9f20b70d5e9bda3"), 2);
        history(&g, p, true);
    }
}
#[test]
fn tmt_mouser_foundry_bounce_and_exile_each_trigger_without_a_graveyard_requirement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Mouser Foundry");
        g.move_object(source, Zone::Hand);
        settle(&mut g);
        assert_eq!(count(&g, p, "@token/667dad32c9f20b70d5e9bda3"), 2);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!(count(&g, p, "@token/667dad32c9f20b70d5e9bda3"), 3);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(count(&g, p, "@token/667dad32c9f20b70d5e9bda3"), 4);
    }
}
#[test]
fn tmt_sewer_cam_flash_entry_optional_tap_and_sacrifice_optional_untap_plus_two_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.step = Step::End;
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = named(&mut g, p, "Sewer-veillance Cam", Zone::Hand);
        cast(&mut g, source, None, &[]);
        resolve_choices(&mut g, target, true);
        assert!(g.object_view(target).tapped);
        let hand = g.observe(p).players[p.index()].hand_count;
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,..}if *card==source))
            .unwrap();
        checked(&mut g, action);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
        resolve_choices(&mut g, target, true);
        assert!(!g.object_view(target).tapped);
        assert_eq!(g.observe(p).players[p.index()].hand_count, hand + 2);
    }
}
#[test]
fn tmt_sewer_cam_choice_is_at_resolution_can_decline_and_obeys_stun_replacement() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let source = named(&mut g, p, "Sewer-veillance Cam", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            g.prepare_triggers();
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            // The target changes state after announcement: the actual choice uses its current state.
            g.state.objects[target as usize].tapped = true;
            g.state.objects[target as usize].counters.stun = 1;
            resolve_choices(&mut g, target, accept);
            assert!(g.object_view(target).tapped);
            assert_eq!(
                g.counter_count(target, CounterKind::Stun),
                if accept { 0 } else { 1 }
            );
        }
    }
}
#[test]
fn tmt_ragamuffin_raptor_recovers_own_creature_or_food_and_allows_no_target() {
    for p in [Player::P0, Player::P1] {
        for target_name in ["Bear Cub", "Omni-Cheese Pizza"] {
            let mut g = setup(p);
            let target = named(&mut g, p, target_name, Zone::Graveyard);
            let enemy = named(&mut g, p.other(), target_name, Zone::Graveyard);
            let other = named(&mut g, p, "Mouser Foundry", Zone::Graveyard);
            let source = named(&mut g, p, "Ragamuffin Raptor", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            g.prepare_triggers();
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(other),
                },
            );
            resolve_choices(&mut g, target, true);
            assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (4, 3));
        }
        let mut g = setup(p);
        let raptor = paid(&mut g, p, "Ragamuffin Raptor");
        assert_eq!(g.state.objects[raptor as usize].zone, Zone::Battlefield);
    }
}
