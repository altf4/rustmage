use super::tests::{act, cast as cast_target, scenario, spawn};
use super::*;
const NAMES: [&str; 13] = [
    "Attentive Sunscribe",
    "Mechan Navigator",
    "Pestered Wellguard",
    "Rescue Leopard",
    "Seedship Agrarian",
    "Silvergill Peddler",
    "Spider-Gwen, Free Spirit",
    "Starfighter Pilot",
    "Swarm Culler",
    "Volatile Wanderglyph",
    "Wanderbrine Preacher",
    "Wanderwine Distracter",
    "Wylie Duke, Atiin Hero",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let p = g.actor().unwrap();
    let r = g.revision();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(expected, fork.snapshot());
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn choose(g: &mut Game, accept: bool) -> usize {
    let p = g.actor().unwrap();
    let o = g.observe(p);
    let discarding = o
        .effect_choice
        .as_ref()
        .is_some_and(|c| c.kind == EffectChoiceKind::Discard);
    let private = usize::from(o.library_choice.is_some() || discarding);
    if discarding {
        assert!(g.observe(p.other()).effect_choice.is_none());
    }
    if private > 0 {
        assert!(g.observe(p.other()).library_choice.is_none());
    }
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,Action::ChooseOptional{accept:v} if *v==accept)
                || !accept && matches!(a, Action::DeclineEffectPayment)
        })
        .or_else(|| {
            g.legal_actions().into_iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::Discard { .. }
                        | Action::Sacrifice { .. }
                        | Action::ArrangeLibrary { .. }
                        | Action::Pass
                )
            })
        })
        .expect("continuation");
    replay(g, a);
    private
}
fn settle(g: &mut Game, accept: bool) -> usize {
    g.prepare_triggers();
    let mut private = 0;
    for _ in 0..200 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return private;
        }
        private += choose(g, accept);
    }
    panic!("unsettled");
}
fn cast(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        choose(g, true);
    }
    settle(g, true);
}
#[test]
fn thirteen_complete_bodies_cast_and_resolve_tap_effects_in_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, source);
            let expected = [
                (2, 2, Keywords::NONE),
                (2, 1, Keywords::NONE),
                (3, 2, Keywords::NONE),
                (4, 2, Keywords::NONE),
                (3, 3, Keywords::NONE),
                (2, 3, Keywords::NONE),
                (2, 3, Keywords::REACH),
                (2, 2, Keywords::NONE),
                (2, 4, Keywords::FLYING),
                (2, 2, Keywords::NONE),
                (2, 2, Keywords::NONE),
                (4, 3, Keywords::NONE),
                (4, 2, Keywords::VIGILANCE),
            ][i];
            let v = g.object_view(source);
            assert_eq!((v.power, v.toughness, v.keywords), expected);
            assert!(!v.tapped);
            assert!(g.state.pending_triggers.is_empty());
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            named(&mut g, p, "Forest", Zone::Hand);
            let food = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let hand = g.hand_count(p);
            let library = g.state.players[p.index()].library.len();
            g.tap_objects(&[source]);
            assert_eq!(g.state.pending_triggers.len(), 1);
            let packet = &g.state.pending_triggers[0];
            assert_eq!(packet.object, g.object_ref(source));
            assert!(packet.source_tapped);
            assert_eq!(packet.entering_object, None);
            let private = settle(&mut g, true);
            assert!(g.object_view(source).tapped);
            match i {
                0 | 7 => {
                    assert!(private > 0);
                    assert_eq!(g.hand_count(p), hand);
                }
                1 | 3 | 5 | 6 | 9 => {
                    assert_eq!(g.hand_count(p), hand);
                    assert_eq!(g.state.players[p.index()].library.len(), library - 1);
                }
                2 => {
                    let token = g
                        .state
                        .objects
                        .iter()
                        .enumerate()
                        .find(|(_, v)| {
                            v.zone == Zone::Battlefield
                                && v.card.token_info().is_some_and(|t| t.subtypes == "Faerie")
                        })
                        .unwrap()
                        .0;
                    let v = g.object_view(token as CardId);
                    assert_eq!((v.power, v.toughness, v.keywords), (1, 1, Keywords::FLYING));
                }
                4 => assert!(g.state.objects.iter().any(|v| v.zone == Zone::Battlefield
                    && v.card.token_info().is_some_and(|t| t.subtypes == "Lander"))),
                8 => {
                    assert_eq!(g.object_view(food).zone, Zone::Graveyard);
                    assert_eq!(g.hand_count(p), hand + 1);
                }
                10 => assert_eq!(g.state.players[p.index()].life, 22),
                11 => {
                    assert_eq!(g.object_view(enemy).power, -1);
                    g.finish_cleanup();
                    assert_eq!(g.object_view(enemy).power, 2);
                }
                12 => {
                    assert_eq!(g.state.players[p.index()].life, 21);
                    assert_eq!(g.hand_count(p), hand + 1);
                }
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn entering_tapped_and_retapping_do_not_create_events_but_untap_then_tap_does() {
    for name in NAMES {
        let mut g = setup(Player::P0);
        named(
            &mut g,
            Player::P1,
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let id = named(&mut g, Player::P0, name, Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        assert!(g.object_view(id).tapped);
        assert!(g.state.pending_triggers.iter().all(|v| v.card != id));
        g.state.pending_triggers.clear();
        g.tap_objects(&[id, id]);
        assert!(g.state.pending_triggers.is_empty());
        g.state.objects[id as usize].untap();
        g.tap_objects(&[id, id]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        g.tap_objects(&[id]);
        assert_eq!(g.state.pending_triggers.len(), 1);
    }
}
#[test]
fn batch_captures_all_sources_after_tapping_and_orders_apnap() {
    let mut g = setup(Player::P1);
    let a = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let b = named(&mut g, Player::P1, NAMES[10], Zone::Battlefield);
    let c = named(&mut g, Player::P1, NAMES[10], Zone::Battlefield);
    g.tap_objects(&[a, b, c, a]);
    assert_eq!(g.state.pending_triggers.len(), 3);
    assert!(
        g.state
            .pending_triggers
            .iter()
            .all(|v| v.source_tapped && v.entering_object.is_none())
    );
    g.prepare_triggers();
    assert_eq!(g.actor(), Some(Player::P1));
    while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut g, true);
    }
    assert_eq!(
        g.state
            .stack
            .iter()
            .map(|v| v.controller)
            .collect::<Vec<_>>(),
        vec![Player::P1, Player::P1, Player::P0]
    );
    settle(&mut g, true);
    assert_eq!((g.state.players[0].life, g.state.players[1].life), (22, 24));
}
#[test]
fn attack_selection_is_not_a_tap_and_vigilance_never_triggers() {
    let mut g = setup(Player::P0);
    let normal = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let wylie = named(&mut g, Player::P0, NAMES[12], Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: normal });
    replay(&mut g, Action::Attack { card: wylie });
    assert!(g.state.pending_triggers.is_empty());
    assert!(!g.object_view(normal).tapped);
    replay(&mut g, Action::FinishAttackers);
    assert!(g.object_view(normal).tapped);
    assert!(!g.object_view(wylie).tapped);
    assert_eq!(
        g.state
            .stack
            .iter()
            .filter(|v| v.kind == StackKind::TriggeredAbility { ability: 0 })
            .count(),
        1
    );
    assert_eq!(g.state.stack[0].object, g.object_ref(normal));
    settle(&mut g, true);
    assert_eq!(g.state.players[0].life, 22);
}
#[test]
fn actual_targeted_tap_draw_spell_queues_after_its_own_draw() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Mental Modulation", Zone::Hand);
    let hand = g.hand_count(Player::P0);
    cast_target(&mut g, spell, Some(Target::Card(source)));
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert!(g.object_view(source).tapped);
    assert_eq!(g.hand_count(Player::P0), hand);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.stack.len(), 1);
    settle(&mut g, true);
    assert_eq!(g.state.players[0].life, 22);
}
#[test]
fn optional_discard_can_be_declined_or_has_no_draw_with_empty_hand() {
    for name in [NAMES[3], NAMES[6], NAMES[9]] {
        for hand in [false, true] {
            let mut g = setup(Player::P0);
            let source = named(&mut g, Player::P0, name, Zone::Battlefield);
            if hand {
                named(&mut g, Player::P0, "Forest", Zone::Hand);
            }
            let lib = g.state.players[0].library.len();
            g.tap_objects(&[source]);
            settle(&mut g, false);
            assert_eq!(g.hand_count(Player::P0), usize::from(hand));
            assert_eq!(g.state.players[0].library.len(), lib);
        }
    }
}
#[test]
fn mandatory_loot_draws_before_discard_even_when_initial_hand_is_empty() {
    for name in [NAMES[1], NAMES[5]] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let lib = g.state.players[0].library.len();
        g.tap_objects(&[source]);
        settle(&mut g, true);
        assert_eq!(g.hand_count(Player::P0), 0);
        assert_eq!(g.state.players[0].library.len(), lib - 1);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == Player::P0 && o.zone == Zone::Graveyard)
                .count(),
            1
        );
    }
}
#[test]
fn culler_cannot_sacrifice_itself_an_enemy_or_a_land_and_can_decline() {
    for decline in [false, true] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, NAMES[8], Zone::Battlefield);
        named(&mut g, Player::P0, "Forest", Zone::Battlefield);
        named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let pick = named(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
        g.tap_objects(&[source]);
        g.prepare_triggers();
        while !g
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Sacrifice { .. }))
        {
            choose(&mut g, true);
        }
        let legal = g.legal_actions();
        assert_eq!(
            legal
                .iter()
                .filter(|a| matches!(a, Action::Sacrifice { .. }))
                .count(),
            1
        );
        assert!(legal.contains(&Action::Sacrifice {
            object: g.object_ref(pick)
        }));
        if decline {
            replay(&mut g, Action::DeclineEffectPayment);
        } else {
            let object = g.object_ref(pick);
            replay(&mut g, Action::Sacrifice { object });
        }
        settle(&mut g, true);
        assert_eq!(g.hand_count(Player::P0), usize::from(!decline));
        assert_eq!(
            g.object_view(pick).zone,
            if decline {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}
#[test]
fn source_departure_and_return_do_not_cancel_or_rebind_captured_trigger() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[12], Zone::Battlefield);
    g.tap_objects(&[source]);
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), old);
    assert_eq!(g.state.pending_triggers[0].object, old);
    settle(&mut g, true);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.hand_count(Player::P0), 1);
    assert!(!g.object_view(source).tapped);
}
#[test]
fn activation_tap_before_sacrifice_captures_old_source_incarnation() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let mut item = g.activation_item(Player::P0, source, 0); // Synthetic hook probe: none of the admitted tap observers has this activated ability.
    item.source_card = Card::from_name("Fanatical Firebrand").unwrap();
    let old = g.object_ref(source);
    g.pay_activation(&mut item, [0; 6], Counters::default());
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.pending_triggers.len(), 1);
    assert_eq!(g.state.pending_triggers[0].object, old);
    assert_eq!(
        g.state.pending_triggers[0].source_card,
        Card::from_name(NAMES[10]).unwrap()
    );
    settle(&mut g, true);
    assert_eq!(g.state.players[0].life, 22);
}
#[test]
fn seedship_landfall_and_lander_search_preserve_the_whole_body() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[4], Zone::Battlefield);
    let land = named(&mut g, Player::P0, "Forest", Zone::Hand);
    replay(&mut g, Action::PlayLand { card: land });
    settle(&mut g, true);
    assert_eq!(g.object_view(source).counters.plus_one, 1);
    g.tap_objects(&[source]);
    settle(&mut g, true);
    let lander = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, v)| {
            v.zone == Zone::Battlefield
                && v.card.token_info().is_some_and(|t| t.subtypes == "Lander")
        })
        .unwrap()
        .0 as CardId;
    let activation = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==lander))
        .expect("lander activation");
    replay(&mut g, activation);
    for _ in 0..60 {
        if g.observe(Player::P0).search_choice.is_some() {
            break;
        }
        choose(&mut g, true);
    }
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSearch { .. }))
        .expect("basic land choice");
    replay(&mut g, action);
    replay(&mut g, Action::FinishSearch);
    settle(&mut g, true);
    assert_eq!(g.object_view(source).counters.plus_one, 2);
    assert!(g.state.objects.iter().any(|v| v.owner == Player::P0
        && v.zone == Zone::Battlefield
        && v.card.is_basic_land()
        && v.tapped));
    assert_eq!(g.object_view(lander).zone, Zone::Outside);
}
#[test]
fn torpor_does_not_suppress_tap_events_and_funding_enumeration_is_pure() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    named(&mut g, Player::P1, "Torpor Orb", Zone::Battlefield);
    for _ in 0..3 {
        named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    }
    g.state.players[0].mana = [0; 6];
    let saved = g.snapshot();
    let goal = FundingGoal {
        actor: Player::P0,
        required: [0; 6],
        generic: 3,
        reserve: None,
        source: None,
    };
    assert!(g.can_fund(goal.clone()));
    for _ in 0..3 {
        let _ = g.legal_actions();
        let _ = g.observe(Player::P0);
    }
    assert_eq!(g.snapshot(), saved);
    g.tap_objects(&[source]);
    settle(&mut g, true);
    assert_eq!(g.state.players[0].life, 22);
}
#[test]
fn tap_trigger_id_overflow_rolls_back_the_entire_attack_declaration() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: source });
    g.state.next_stack_id = u64::MAX;
    let saved = g.snapshot();
    let revision = g.revision();
    assert!(matches!(
        g.apply(Player::P0, Action::FinishAttackers),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    assert!(!g.object_view(source).tapped);
}

#[test]
fn private_scry_and_surveil_allow_both_destinations_without_revealing_to_opponent() {
    for (name, surveil) in [(NAMES[0], false), (NAMES[7], true)] {
        for keep in [false, true] {
            let mut g = setup(Player::P1);
            let source = named(&mut g, Player::P1, name, Zone::Battlefield);
            let top = *g.state.players[1].library.last().unwrap();
            let count = g.state.players[1].library.len();
            g.tap_objects(&[source]);
            g.prepare_triggers();
            while g.observe(Player::P1).library_choice.is_none() {
                choose(&mut g, true);
            }
            assert!(g.observe(Player::P0).library_choice.is_none());
            assert!(!g.observe(Player::P0).objects.iter().any(|v| v.id == top));
            let destination = if keep {
                LibraryDestination::Top
            } else if surveil {
                LibraryDestination::Graveyard
            } else {
                LibraryDestination::Bottom
            };
            replay(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination,
                },
            );
            settle(&mut g, true);
            assert_eq!(
                g.state.players[1].library.len(),
                count - usize::from(surveil && !keep)
            );
            if keep {
                assert_eq!(g.state.players[1].library.last(), Some(&top));
            } else if surveil {
                assert_eq!(g.object_view(top).zone, Zone::Graveyard);
            } else {
                assert_eq!(g.state.players[1].library.first(), Some(&top));
            }
        }
    }
}

#[test]
fn queued_tap_trigger_waits_through_another_sources_private_library_frame() {
    let mut g = setup(Player::P0);
    let life = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let scry = named(&mut g, Player::P0, NAMES[0], Zone::Battlefield);
    g.tap_objects(&[life, scry]);
    g.prepare_triggers();
    let life_id = g
        .state
        .ordering_triggers
        .iter()
        .find(|v| v.card == life)
        .unwrap()
        .id;
    replay(&mut g, Action::PutTrigger { id: life_id });
    while g.observe(Player::P0).library_choice.is_none() {
        choose(&mut g, true);
    }
    assert!(
        g.state
            .stack
            .iter()
            .any(|v| v.object == g.object_ref(life) && v.entering_object.is_none())
    );
    assert_eq!(g.state.players[0].life, 20);
    settle(&mut g, true);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn tap_life_resolution_overflow_restores_the_existing_tapped_state_and_pending_stack() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[12], Zone::Battlefield);
    g.tap_objects(&[source]);
    g.prepare_triggers();
    replay(&mut g, Action::Pass);
    g.state.players[0].life = i128::MAX;
    let saved = g.snapshot();
    let revision = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    assert!(g.object_view(source).tapped);
}

#[test]
fn distracter_requires_an_opponent_and_does_not_debuff_a_returned_target() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[11], Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.tap_objects(&[source]);
    g.prepare_triggers();
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(source)
    }));
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    g.move_object(enemy, Zone::Hand);
    g.move_object(enemy, Zone::Battlefield);
    settle(&mut g, true);
    assert_eq!(g.object_view(enemy).power, 2);
}

#[test]
fn attached_aura_tap_observes_its_established_continuous_bonus() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let aura = named(&mut g, Player::P0, "Colossification", Zone::Hand);
    cast_target(&mut g, aura, Some(Target::Card(source)));
    settle(&mut g, true);
    let view = g.object_view(source);
    assert!(view.tapped);
    assert_eq!((view.power, view.toughness), (22, 22));
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn actual_mass_tap_queues_only_opposing_changed_creatures() {
    let mut g = setup(Player::P0);
    let own = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let a = named(&mut g, Player::P1, NAMES[10], Zone::Battlefield);
    let b = named(&mut g, Player::P1, NAMES[10], Zone::Battlefield);
    let old = named(&mut g, Player::P1, NAMES[10], Zone::Battlefield);
    g.state.objects[old as usize].tapped = true;
    let source = named(&mut g, Player::P0, "Ultimecia, Temporal Threat", Zone::Hand);
    cast(&mut g, source);
    assert!(!g.object_view(own).tapped);
    assert!(g.object_view(a).tapped && g.object_view(b).tapped);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.players[1].life, 24);
}

#[test]
fn actual_two_target_tap_stun_spell_triggers_each_source_once() {
    let mut g = setup(Player::P0);
    let a = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let b = named(&mut g, Player::P0, NAMES[10], Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Out Cold", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: spell });
    while !matches!(g.state.pending, Pending::Targets { .. }) {
        choose(&mut g, true);
    }
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    settle(&mut g, true);
    assert_eq!(g.state.players[0].life, 24);
    for id in [a, b] {
        let v = g.object_view(id);
        assert!(v.tapped);
        assert_eq!(v.counters.stun, 1);
    }
    assert!(g.state.objects.iter().any(|v| v.zone == Zone::Battlefield
        && v.card.token_info().is_some_and(|t| t.subtypes == "Clue")));
}
