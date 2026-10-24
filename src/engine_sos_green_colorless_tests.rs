//! Paid SOS green/colorless interactions, including exact token continuations.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
fn paid(g: &mut Game, p: Player, name: &str, mode: Option<u16>, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, mode, targets);
    id
}
fn tokens(g: &Game, p: Player, card: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == card)
                .then_some(i as CardId)
        })
        .collect()
}
fn target_settle(g: &mut Game, target: CardId) {
    for _ in 0..100 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let target = Action::ChooseTarget {
            target: Target::Card(target),
        };
        let action = if actions.contains(&target) {
            target
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, action);
    }
    panic!("unsettled target {:?}", g.state.pending);
}

#[test]
fn sos_oracles_restoration_own_target_paid_draw_life_cleanup_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, "Oracle's Restoration", Zone::Hand);
            let life = g.state.players[p.index()].life;
            let hand = g.hand_count(p);
            checked(&mut g, Action::AnnounceCast { card: id });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, payment);
            if leaves {
                g.move_object(own, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, life + i128::from(!leaves));
            assert_eq!(g.hand_count(p), hand - usize::from(leaves));
            if !leaves {
                assert_eq!((g.stats(own).power, g.stats(own).toughness), (3, 3));
                g.finish_cleanup();
                assert_eq!((g.stats(own).power, g.stats(own).toughness), (2, 2));
            }
        }
    }
}

#[test]
fn sos_chelonian_tackle_optional_opponent_fight_and_partial_illegal_operands() {
    for p in [Player::P0, Player::P1] {
        for opponent in [false, true] {
            for leaves in [false, true] {
                let mut g = setup(p);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
                let mut targets = vec![Target::Card(own)];
                if opponent {
                    targets.push(Target::Card(enemy));
                }
                paid(&mut g, p, "Chelonian Tackle", None, &targets);
                if leaves {
                    g.move_object(own, Zone::Graveyard);
                }
                settle(&mut g);
                assert_eq!(g.on_battlefield(enemy), !opponent || leaves);
                if !leaves {
                    assert_eq!(g.stats(own).toughness, 12);
                    assert_eq!(g.object_view(own).damage, if opponent { 2 } else { 0 });
                    g.finish_cleanup();
                    assert_eq!(g.stats(own).toughness, 2);
                }
            }
        }
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Chelonian Tackle",
            None,
            &[Target::Card(own), Target::Card(enemy)],
        );
        g.move_object(enemy, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.stats(own).toughness, 12);
        assert_eq!(g.object_view(own).damage, 0);
    }
}

#[test]
fn sos_glorious_decay_each_mode_and_flying_restriction_and_grave_fizzle() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for leaves in [false, true] {
                let mut g = setup(p);
                let (name, zone) = match mode {
                    0 => ("Gilded Lotus", Zone::Battlefield),
                    1 => ("Serra Angel", Zone::Battlefield),
                    _ => ("Forest", Zone::Graveyard),
                };
                let target = named(&mut g, p.other(), name, zone);
                let hand = g.hand_count(p);
                paid(
                    &mut g,
                    p,
                    "Glorious Decay",
                    Some(mode),
                    &[Target::Card(target)],
                );
                if leaves {
                    g.move_object(target, Zone::Hand);
                }
                settle(&mut g);
                if !leaves {
                    assert_eq!(
                        g.object_view(target).zone,
                        if mode == 2 {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        }
                    );
                }
                assert_eq!(g.hand_count(p), hand + usize::from(mode == 2 && !leaves));
            }
        }
        let mut g = setup(p);
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let spell = named(&mut g, p, "Glorious Decay", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(ground),
            },
        );
    }
}

#[test]
fn sos_together_as_one_counts_colors_not_colorless_and_separates_targets() {
    for p in [Player::P0, Player::P1] {
        for colored in [false, true] {
            let mut g = setup(p);
            let q = p.other();
            let hand = g.hand_count(q);
            let life = g.state.players[p.index()].life;
            let qlife = g.state.players[q.index()].life;
            g.state.players[p.index()].mana = if colored { [1; 6] } else { [0, 0, 0, 0, 0, 6] };
            paid(
                &mut g,
                p,
                "Together as One",
                None,
                &[Target::Player(q), Target::Player(q)],
            );
            settle(&mut g);
            let amount = if colored { 5 } else { 0 };
            assert_eq!(g.hand_count(q), hand + amount);
            assert_eq!(g.state.players[p.index()].life, life + amount as i128);
            assert_eq!(g.state.players[q.index()].life, qlife - amount as i128);
        }
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let hand = g.hand_count(p);
        g.state.players[p.index()].mana = [1; 6];
        paid(
            &mut g,
            p,
            "Together as One",
            None,
            &[Target::Player(p), Target::Card(target)],
        );
        g.move_object(target, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 5);
        assert_eq!(g.state.players[p.index()].life, 25);
    }
}

#[test]
fn sos_additive_evolution_entry_and_combat_are_independent_of_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Additive Evolution", None, &[]);
        // Let the enchantment enter and its mandatory trigger reach the stack.
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        g.prepare_triggers();
        g.move_object(id, Zone::Graveyard);
        settle(&mut g);
        let token = tokens(&g, p, Card::SosFractalToken)[0];
        assert_eq!(g.counter_count(token, CounterKind::PlusOne), 3);
        assert_eq!((g.stats(token).power, g.stats(token).toughness), (3, 3));
        paid(&mut g, p, "Additive Evolution", None, &[]);
        settle(&mut g);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        target_settle(&mut g, token);
        assert_eq!(g.counter_count(token, CounterKind::PlusOne), 4);
        assert!(g.keywords(token).contains(Keywords::VIGILANCE));
        g.finish_cleanup();
        assert!(!g.keywords(token).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn sos_fractal_counter_completion_replacements_apply_after_token_doubling() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Loading Zone", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        g.state.players[p.index()].mana = [1; 6];
        paid(&mut g, p, "Snarl Song", None, &[]);
        settle(&mut g);
        let fractals = tokens(&g, p, Card::SosFractalToken);
        assert_eq!(fractals.len(), 4);
        for token in fractals {
            assert_eq!(g.counter_count(token, CounterKind::PlusOne), 20);
            assert_eq!(g.object_view(token).card.colors(), 18);
        }
        assert_eq!(g.state.players[p.index()].life, life + 5);
    }
}

#[test]
fn sos_wild_hypothesis_chosen_x_zero_and_owned_private_surveil() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 3] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, x + 1, 0];
            let spell = named(&mut g, p, "Wild Hypothesis", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseX { value: x });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert!(g.observe(p.other()).library_choice.is_none());
            let choice = g.observe(p).library_choice.unwrap();
            assert_eq!(choice.remaining.len(), 2);
            assert!(choice.surveil);
            let first = choice.remaining[0].reference.card;
            let second = choice.remaining[1].reference.card;
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: first,
                    destination: LibraryDestination::Graveyard,
                },
            );
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: second,
                    destination: LibraryDestination::Top,
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(first).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].library.last(), Some(&second));
            let fractals = tokens(&g, p, Card::SosFractalToken);
            assert_eq!(fractals.len(), usize::from(x != 0));
            if x != 0 {
                assert_eq!(
                    g.counter_count(fractals[0], CounterKind::PlusOne),
                    u64::from(x)
                );
            }
        }
    }
}

#[test]
fn sos_emil_live_counter_trample_and_distinct_names_resolution_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let emil = named(&mut g, p, "Emil, Vastlands Roamer", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(own, CounterKind::PlusOne, 1);
        g.add_counter(enemy, CounterKind::PlusOne, 1);
        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Island", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: emil,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert!(g.object_view(emil).tapped);
        g.move_object(emil, Zone::Graveyard);
        g.move_object(land, Zone::Graveyard);
        named(&mut g, p, "Mountain", Zone::Battlefield);
        settle(&mut g);
        let token = tokens(&g, p, Card::SosFractalToken)[0];
        assert_eq!(g.counter_count(token, CounterKind::PlusOne), 3);
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(token).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn sos_pestbrood_sloth_reach_death_tokens_and_attack_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sloth = paid(&mut g, p, "Pestbrood Sloth", None, &[]);
        settle(&mut g);
        assert!(g.keywords(sloth).contains(Keywords::REACH));
        paid(&mut g, p, "Murder", None, &[Target::Card(sloth)]);
        settle(&mut g);
        let pests = tokens(&g, p, Card::SosPestToken);
        assert_eq!(pests.len(), 2);
        let life = g.state.players[p.index()].life;
        for &id in &pests {
            g.state.objects[id as usize].summoning_sick = false;
            assert_eq!(g.stats(id).power, 1);
        }
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        for id in pests {
            checked(&mut g, Action::Attack { card: id });
        }
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}

#[test]
fn sos_fractal_counter_instruction_includes_replacement_added_tokens_and_not_entry_stats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Quina, Qu Gourmet", Zone::Battlefield);
        named(&mut g, p, "Vaultborn Tyrant", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        let hand = g.hand_count(p);
        g.state.players[p.index()].mana = [1; 6];
        paid(&mut g, p, "Snarl Song", None, &[]);
        settle(&mut g);
        let frog = Card::from_name("@token/5505438c39928760ffd3d4a1").unwrap();
        let frogs = tokens(&g, p, frog);
        assert_eq!(frogs.len(), 1);
        assert_eq!(g.counter_count(frogs[0], CounterKind::PlusOne), 5);
        // Fractals entered as 0/0 and the replacement's Frog as 1/1, so the
        // counters added later in this effect cannot trigger the Tyrant.
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.state.players[p.index()].life, life + 5);
    }
}

#[test]
fn sos_planar_engineering_sacrifices_available_lands_then_private_tapped_basic_search() {
    for p in [Player::P0, Player::P1] {
        for lands in [0, 1, 3] {
            let mut g = setup(p);
            let own = (0..lands)
                .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
                .collect::<Vec<_>>();
            let enemy = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            let creature = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let basics = ["Forest", "Island", "Swamp", "Mountain"]
                .map(|name| named(&mut g, p, name, Zone::Library));
            let bad = named(&mut g, p, "Bear Cub", Zone::Library);
            paid(&mut g, p, "Planar Engineering", None, &[]);
            for _ in 0..30 {
                if matches!(g.state.pending, Pending::EffectSearch { .. }) {
                    break;
                }
                let action = if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                    let enemy = g.object_ref(enemy);
                    let creature = g.object_ref(creature);
                    reject(&mut g, Action::Sacrifice { object: enemy });
                    reject(&mut g, Action::Sacrifice { object: creature });
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Sacrifice { .. }))
                        .unwrap()
                } else {
                    Action::Pass
                };
                checked(&mut g, action);
            }
            assert!(matches!(g.state.pending, Pending::EffectSearch { .. }));
            assert_eq!(
                own.iter()
                    .filter(|&&id| g.object_view(id).zone == Zone::Graveyard)
                    .count(),
                lands.min(2)
            );
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: bad });
            for id in basics {
                checked(&mut g, Action::ChooseSearch { card: id });
            }
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            for id in basics {
                assert!(g.on_battlefield(id));
                assert!(g.object_view(id).tapped);
            }
            assert!(g.on_battlefield(enemy));
            assert!(g.on_battlefield(creature));
        }
    }
}

#[test]
fn sos_ambitious_augmenter_increment_threshold_and_historical_all_counter_transfer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Ambitious Augmenter", None, &[]);
        settle(&mut g);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Oracle's Restoration",
            None,
            &[Target::Card(bear)],
        );
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        paid(&mut g, p, "Chelonian Tackle", None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        g.add_counter(source, CounterKind::PlusOne, 1);
        g.add_counter(source, CounterKind::Stun, 2);
        g.add_counter(source, CounterKind::Page, 3);
        g.add_counter(source, CounterKind::Soul, 4);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 9);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        settle(&mut g);
        let fractals = tokens(&g, p, Card::SosFractalToken);
        assert_eq!(fractals.len(), 2);
        for token in fractals {
            for (kind, amount) in [
                (CounterKind::PlusOne, 4),
                (CounterKind::Stun, 4),
                (CounterKind::Page, 6),
                (CounterKind::Soul, 8),
            ] {
                assert_eq!(g.counter_count(token, kind), amount);
            }
            assert_eq!(g.counter_count(token, CounterKind::MinusOne), 0);
        }
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 9);
        let fresh = paid(&mut g, p, "Ambitious Augmenter", None, &[]);
        settle(&mut g);
        let before = tokens(&g, p, Card::SosFractalToken).len();
        g.move_object(fresh, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(tokens(&g, p, Card::SosFractalToken).len(), before);
    }
}

#[test]
fn sos_germination_practicum_counters_only_own_creatures_then_recurs_after_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = paid(&mut g, p, "Germination Practicum", None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.observe(p).paradigms.len(), 1);
        g.move_object(spell, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        g.state.active = p;
        g.state.priority = p;
        g.state.pending = Pending::Priority;
        g.begin_step(Step::Main1);
        g.prepare_triggers();
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
                break;
            }
            let action = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap()
            };
            checked(&mut g, action);
        }
        let copy = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
        checked(
            &mut g,
            Action::CastExiledDuringResolution {
                object: copy,
                face: CastFace::Normal,
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 4);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.observe(p).paradigms.len(), 1);
    }
}
