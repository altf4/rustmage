//! Focused printed outcomes; shared engine invariants live in core tests.
//! Original admitted SOS W/U cards: printed outcomes, not only catalogue admission.
use super::*;
fn choice(g: &mut Game) {
    for _ in 0..40 {
        if g.state.pending != Pending::Priority {
            return;
        }
        big_helpers::checked(g, Action::Pass);
    }
    panic!("missing choice")
}
fn finish(g: &mut Game) {
    big_helpers::settle(g)
}
fn paid(g: &mut Game, id: CardId, targets: &[Target]) {
    big_helpers::cast(g, id, None, targets)
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    big_helpers::checked(g, a)
}
fn activate(g: &mut Game, id: CardId) {
    big_helpers::checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    )
}
fn arrange(g: &mut Game, id: CardId, destination: LibraryDestination) {
    big_helpers::checked(
        g,
        Action::ArrangeLibrary {
            card: id,
            destination,
        },
    )
}
fn pool(g: &mut Game, p: Player, m: Mana) {
    g.state.players[p.index()].mana = m
}
#[test]
fn sos_audit_wu_conditional_costs_lock_target_and_actual_payment() {
    {
        let p = Player::P0;
        for tapped in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[target as usize].tapped = tapped;
            let spell = put(&mut g, p, "Ajani's Response", Zone::Hand);
            pool(&mut g, p, [1, 0, 0, 0, 0, if tapped { 1 } else { 4 }]);
            paid(&mut g, spell, &[Target::Card(target)]);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().mana_spent(),
                if tapped { 2 } else { 5 }
            );
            finish(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
        for instant in [false, true] {
            let mut g = scenario(p);
            let spell = put(
                &mut g,
                p,
                if instant { "Quick Study" } else { "Bear Cub" },
                Zone::Hand,
            );
            paid(&mut g, spell, &[]);
            let counter = put(&mut g, p, "Brush Off", Zone::Hand);
            pool(
                &mut g,
                p,
                [
                    0,
                    if instant { 1 } else { 2 },
                    0,
                    0,
                    0,
                    if instant { 1 } else { 2 },
                ],
            );
            paid(&mut g, counter, &[Target::Spell(spell)]);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().mana_spent(),
                if instant { 2 } else { 4 }
            );
            finish(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
        for threshold in [false, true] {
            let mut g = scenario(p);
            for _ in 0..if threshold { 5 } else { 4 } {
                put(&mut g, p, "Bear Cub", Zone::Battlefield);
            }
            put(&mut g, p.other(), "Koma, World-Eater", Zone::Battlefield);
            let dragon = put(&mut g, p, "Orysa, Tide Choreographer", Zone::Hand);
            pool(&mut g, p, [0, 1, 0, 0, 0, if threshold { 1 } else { 4 }]);
            paid(&mut g, dragon, &[]);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().mana_spent(),
                if threshold { 2 } else { 5 }
            );
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            assert_eq!(stats(&g, dragon), (2, 2));
        }
    }
}
#[test]
fn sos_audit_wu_repartee_all_consumers_trigger_once_on_cast_not_resolution() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let graduation = put(&mut g, p, "Graduation Day", Zone::Battlefield);
        let inkwright = put(&mut g, p, "Informed Inkwright", Zone::Battlefield);
        let demonstrator = put(&mut g, p, "Inkshape Demonstrator", Zone::Battlefield);
        let debater = put(&mut g, p, "Rehearsed Debater", Zone::Battlefield);
        let hopesinger = put(&mut g, p, "Stirring Hopesinger", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);

        assert!(
            g.keywords(hopesinger)
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
        let spell = put(&mut g, p, "Interjection", Zone::Hand);
        paid(&mut g, spell, &[Target::Card(enemy)]);
        // A single creature-targeting cast produces all five repartee triggers.
        let sources: Vec<_> = g
            .state
            .stack
            .iter()
            .chain(g.state.pending_triggers.iter())
            .chain(g.state.ordering_triggers.iter())
            .filter(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
            .map(|s| s.source.unwrap().id)
            .collect();
        for id in [graduation, inkwright, demonstrator, debater, hopesinger] {
            assert_eq!(sources.iter().filter(|&&s| s == id).count(), 1);
        }
        finish(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        for id in [inkwright, demonstrator, debater, hopesinger] {
            assert!(g.counter_count(id, CounterKind::PlusOne) >= 1);
        }
        assert_eq!(
            stats(&g, demonstrator).0,
            4 + i128::from(g.counter_count(demonstrator, CounterKind::PlusOne))
        );
        assert_eq!(
            stats(&g, debater).0,
            4 + i128::from(g.counter_count(debater, CounterKind::PlusOne))
        );
        assert!(g.keywords(demonstrator).contains(Keywords::LIFELINK));
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .unwrap()
            .0 as CardId;
        assert_eq!(stats(&g, token), (1, 1));
        assert_eq!(g.object_view(token).colors, 1 | 4);
        assert!(g.keywords(token).contains(Keywords::FLYING));
        assert!(g.has_subtype(token, "Inkling"));
        let before = g.state.objects.len();
        let unqualified = put(&mut g, p, "Quick Study", Zone::Hand);
        paid(&mut g, unqualified, &[]);
        finish(&mut g);
        assert_eq!(g.state.objects.len(), before + 1);
        g.finish_cleanup();
        assert!(!g.keywords(demonstrator).contains(Keywords::LIFELINK));
        assert_eq!(
            stats(&g, debater).0,
            3 + i128::from(g.counter_count(debater, CounterKind::PlusOne))
        );
    }
}
#[test]
fn sos_audit_wu_opus_frozen_actual_spend_and_deluge_entry_stun() {
    {
        let p = Player::P0;
        for high in [false, true] {
            let mut g = big_helpers::setup(p);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let deluge = put(&mut g, p, "Deluge Virtuoso", Zone::Hand);
            paid(&mut g, deluge, &[]);
            choice(&mut g);
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            finish(&mut g);
            assert!(g.object_view(enemy).tapped);
            assert_eq!(g.counter_count(enemy, CounterKind::Stun), 1);
            let tide = put(&mut g, p, "Exhibition Tidecaller", Zone::Battlefield);
            let muse = put(&mut g, p, "Muse Seeker", Zone::Battlefield);
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = put(
                &mut g,
                p,
                if high {
                    "Ajani's Response"
                } else {
                    "Chase Inspiration"
                },
                Zone::Hand,
            );
            paid(
                &mut g,
                spell,
                &[Target::Card(if high { victim } else { muse })],
            );
            // Put the cast triggers on the stack, then counter the original spell.
            // Its removed stack object cannot change the frozen actual payment.
            for _ in 0..16 {
                if g.state.pending == Pending::Priority {
                    break;
                }
                let actions = g.legal_actions();
                let target = Action::ChooseTarget {
                    target: Target::Player(p.other()),
                };
                let action = if actions.contains(&target) {
                    target
                } else {
                    actions
                        .into_iter()
                        .find(|a| !matches!(a, Action::Concede))
                        .unwrap()
                };
                big_helpers::checked(&mut g, action);
            }
            big_helpers::checked(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(p.other()));
            let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
            paid(&mut g, negate, &[Target::Spell(spell)]);
            let expected = if high { 10 } else { 3 };
            let old = g.state.players[p.other().index()].library.len();
            // Tidecaller is the only trigger that needs an announced target.
            for _ in 0..60 {
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.pending_triggers.is_empty()
                {
                    break;
                }
                let acts = g.legal_actions();
                let a = if acts.contains(&Action::ChooseTarget {
                    target: Target::Player(p.other()),
                }) {
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    }
                } else if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    acts.into_iter()
                        .find(|a| !matches!(a, Action::Concede))
                        .unwrap()
                };
                big_helpers::checked(&mut g, a);
            }
            assert_eq!(
                g.state.players[p.other().index()].library.len(),
                old - expected
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(
                g.state.players[p.index()].cards_discarded_this_turn,
                if high { 0 } else { 1 }
            );
            assert_eq!(stats(&g, deluge), if high { (4, 4) } else { (3, 3) });
            assert_eq!(stats(&g, tide), (0, 2));
        }
    }
}
#[test]
fn sos_audit_wu_dustspeaker_other_controlled_entry_and_optional_any_graveyard() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let friend = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Ascendant Dustspeaker", Zone::Hand);
        paid(&mut g, source, &[]);
        choice(&mut g);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(friend),
            },
        );
        finish(&mut g);
        assert_eq!(g.counter_count(friend, CounterKind::PlusOne), 1);

        assert_eq!(stats(&g, source), (3, 4));
        let grave = put(&mut g, p.other(), "Quick Study", Zone::Graveyard);
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        choice(&mut g);
        let mut decline = g.clone();
        big_helpers::checked(&mut decline, Action::FinishTargets);
        finish(&mut decline);
        assert_eq!(decline.object_view(grave).zone, Zone::Graveyard);
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grave),
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(grave).zone, Zone::Exile);
        g.state.active = p.other();
        g.begin_step(Step::BeginCombat);
        g.prepare_triggers();
        assert!(g.state.stack.is_empty());
    }
}

#[test]
fn sos_audit_wu_tokens_surveillance_and_tabulator_live_increment() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let glyph = put(&mut g, p, "Eager Glyphmage", Zone::Hand);
        paid(&mut g, glyph, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, glyph), (3, 3));
        let ink = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .unwrap()
            .0 as CardId;
        assert_eq!(stats(&g, ink), (1, 1));
        assert_eq!(g.object_view(ink).colors, 1 | 4);
        assert!(g.keywords(ink).contains(Keywords::FLYING));
        assert!(g.has_subtype(ink, "Inkling"));
        let top = put(&mut g, p, "Bear Cub", Zone::Library);
        let next = put(&mut g, p, "Quick Study", Zone::Library);
        let muse = put(&mut g, p, "Muse's Encouragement", Zone::Hand);
        paid(&mut g, muse, &[]);
        choice(&mut g);
        assert!(g.observe(p.other()).library_choice.is_none());
        arrange(&mut g, next, LibraryDestination::Graveyard);
        arrange(&mut g, top, LibraryDestination::Top);
        finish(&mut g);
        assert_eq!(g.object_view(next).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&top));
        let elem = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card != g.state.objects[ink as usize].card
            })
            .unwrap()
            .0 as CardId;
        assert_eq!(stats(&g, elem), (3, 3));
        assert_eq!(g.object_view(elem).colors, 2 | 8);
        assert!(g.keywords(elem).contains(Keywords::FLYING));
        assert!(g.has_subtype(elem, "Elemental"));
        let tab = put(&mut g, p, "Textbook Tabulator", Zone::Hand);
        paid(&mut g, tab, &[]);
        choice(&mut g);
        let a = *g.state.players[p.index()].library.last().unwrap();
        arrange(&mut g, a, LibraryDestination::Graveyard);
        let b = g.observe(p).library_choice.unwrap().remaining[0]
            .reference
            .card;
        arrange(&mut g, b, LibraryDestination::Top);
        finish(&mut g);
        assert_eq!(stats(&g, tab), (0, 3));
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        paid(&mut g, spell, &[]);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        paid(&mut g, growth, &[Target::Card(tab)]);
        finish(&mut g);
        assert_eq!(g.counter_count(tab, CounterKind::PlusOne), 1);
        assert_eq!(stats(&g, tab), (4, 7));
    }
}
#[test]
fn sos_audit_wu_graveyard_activations_paid_own_sorcery_only() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let docent = put(&mut g, p, "Stone Docent", Zone::Graveyard);
        let drom = put(&mut g, p, "Summoned Dromedary", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Stone Docent", Zone::Graveyard);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: enemy,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        activate(&mut g, docent);
        pay(&mut g);
        assert_eq!(g.object_view(docent).zone, Zone::Exile);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: drom,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        choice(&mut g);
        let top = *g.state.players[p.index()].library.last().unwrap();
        arrange(&mut g, top, LibraryDestination::Graveyard);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        activate(&mut g, drom);
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(drom).zone, Zone::Hand);
        paid(&mut g, drom, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, drom), (4, 3));
    }
}
#[test]
fn sos_audit_wu_inventory_flashback_and_acolyte_sacrifice_cost() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let dig = put(&mut g, p, "Dig Site Inventory", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: dig });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert!(g.keywords(bear).contains(Keywords::VIGILANCE));
        g.finish_cleanup();
        assert!(!g.keywords(bear).contains(Keywords::VIGILANCE));
        // Cleanup advanced to the opponent's turn. Return through legal steps
        // to our next main phase before this sorcery-speed flashback.
        big_helpers::next_turn(&mut g);
        for _ in 0..4 {
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.step, Step::Main1);
        pool(&mut g, p, [8; 6]);
        big_helpers::checked(&mut g, Action::AnnounceFlashback { card: dig });
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(dig).zone, Zone::Exile);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 2);
        let acolyte = put(&mut g, p, "Shattered Acolyte", Zone::Hand);
        paid(&mut g, acolyte, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, acolyte), (2, 2));

        let enchant = put(&mut g, p.other(), "Graduation Day", Zone::Battlefield);
        activate(&mut g, acolyte);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enchant),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(acolyte).zone, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.object_view(enchant).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_audit_wu_procrastinate_x_zero_and_twice_stun_actual_untap() {
    {
        let p = Player::P0;
        for x in [0, 3] {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Procrastinate", Zone::Hand);
            big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
            big_helpers::checked(&mut g, Action::ChooseX { value: x });
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            pay(&mut g);
            finish(&mut g);
            assert!(g.object_view(enemy).tapped);
            assert_eq!(g.counter_count(enemy, CounterKind::Stun), u64::from(2 * x));
            big_helpers::next_turn(&mut g);
            assert_eq!(g.state.active, p.other());
            assert_eq!(g.object_view(enemy).tapped, x > 0);
            assert_eq!(
                g.counter_count(enemy, CounterKind::Stun),
                u64::from((2 * x).saturating_sub(1))
            );
        }
    }
}
#[test]
fn sos_audit_wu_run_behind_attacking_discount_owner_top_or_bottom() {
    {
        let p = Player::P0;
        for attacking in [false, true] {
            for bottom in [false, true] {
                let mut g = scenario(p);
                let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.state.objects[bear as usize].controller = p;
                if attacking {
                    g.state.step = Step::Attackers;
                    g.state.pending = Pending::Attackers;
                    big_helpers::checked(&mut g, Action::Attack { card: bear });
                    big_helpers::checked(&mut g, Action::FinishAttackers);
                }
                let spell = put(&mut g, p, "Run Behind", Zone::Hand);
                pool(&mut g, p, [0, 1, 0, 0, 0, if attacking { 2 } else { 3 }]);
                paid(&mut g, spell, &[Target::Card(bear)]);
                assert_eq!(
                    g.state.stack.last().unwrap().cast.unwrap().mana_spent(),
                    if attacking { 3 } else { 4 }
                );
                choice(&mut g);
                assert_eq!(g.actor(), Some(p.other()));
                let object = g.object_ref(bear);
                big_helpers::checked(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination: if bottom {
                            LibraryDestination::Bottom
                        } else {
                            LibraryDestination::Top
                        },
                    },
                );
                finish(&mut g);
                assert_eq!(g.object_view(bear).zone, Zone::Library);
                let lib = &g.state.players[p.other().index()].library;
                assert_eq!(if bottom { lib.first() } else { lib.last() }, Some(&bear));
            }
        }
    }
}
#[test]
fn sos_audit_wu_quick_study_two_draws_and_essence_scatter_creatures_only() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let study = put(&mut g, p, "Quick Study", Zone::Hand);
        paid(&mut g, study, &[]);
        let counter = put(&mut g, p, "Essence Scatter", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: counter });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        paid(&mut g, bear, &[]);
        paid(&mut g, counter, &[Target::Spell(bear)]);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
    }
}
