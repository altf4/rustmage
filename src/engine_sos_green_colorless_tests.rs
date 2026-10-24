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

#[test]
fn sos_comforting_counsel_counts_gain_events_live_threshold_control_and_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Comforting Counsel", None, &[]);
        settle(&mut g);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.gain_life(p, 20);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Growth), 1);
        g.gain_life(p, 0);
        g.gain_life(p.other(), 5);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Growth), 1);
        for counter in 2..=5 {
            g.gain_life(p, 1);
            settle(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::Growth), counter);
            assert_eq!(g.stats(own).power, if counter == 5 { 5 } else { 2 });
            assert_eq!(g.stats(enemy).power, 2);
        }
        assert_eq!(g.stats(own).toughness, 5);
        let reference = g.object_ref(source);
        g.change_controller(reference, p.other());
        assert_eq!(g.stats(own).power, 2);
        assert_eq!(g.stats(enemy).power, 5);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.stats(enemy).power, 2);
        g.move_object(source, Zone::Battlefield);
        assert_eq!(g.counter_count(source, CounterKind::Growth), 0);
        assert_eq!(g.stats(own).power, 2);
        assert_eq!(g.stats(enemy).power, 2);
    }
}

fn prep_copy(g: &Game, source: CardId) -> CardId {
    g.state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == g.object_ref(source))
        .unwrap()
        .spell_copy
        .card
}
fn paid_prep(g: &mut Game, source: CardId, x: Option<u32>, targets: &[Target]) -> CardId {
    let copy = prep_copy(g, source);
    checked(g, Action::AnnounceExile { card: copy });
    if let Some(value) = x {
        checked(g, Action::ChooseX { value });
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
    assert!(!g.object_view(source).prepared);
    copy
}
fn until_pending(g: &mut Game, stop: impl Fn(&Pending) -> bool) {
    for _ in 0..60 {
        if stop(&g.state.pending) {
            return;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, action);
    }
    panic!("missing pending {:?}", g.state.pending);
}

#[test]
fn sos_infirmary_healer_paid_prepared_x_life_and_exile_copy_cleanup() {
    for p in [Player::P0, Player::P1] {
        for recipient in [p, p.other()] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Infirmary Healer", None, &[]);
            settle(&mut g);
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (2, 3));
            assert!(g.object_view(source).prepared);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 4, 0];
            let life = g.state.players[recipient.index()].life;
            let copy = paid_prep(&mut g, source, Some(3), &[Target::Player(recipient)]);
            settle(&mut g);
            assert_eq!(g.state.players[recipient.index()].life, life + 3);
            assert_eq!(g.object_view(copy).zone, Zone::Outside);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn sos_emeritus_regrowth_and_attack_reprepare_rechecks_eight_lands() {
    for p in [Player::P0, Player::P1] {
        for (lands, lose_land) in [(7, false), (8, false), (8, true)] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Emeritus of Abundance", None, &[]);
            settle(&mut g);
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (3, 4));
            assert!(g.keywords(source).contains(Keywords::VIGILANCE));
            let old = named(&mut g, p, "Forest", Zone::Graveyard);
            paid_prep(&mut g, source, None, &[Target::Card(old)]);
            settle(&mut g);
            assert_eq!(g.object_view(old).zone, Zone::Hand);
            let land_ids = (0..lands)
                .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
                .collect::<Vec<_>>();
            g.state.objects[source as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: source });
            checked(&mut g, Action::FinishAttackers);
            if lose_land {
                g.move_object(land_ids[0], Zone::Graveyard);
            }
            settle(&mut g);
            assert!(!g.object_view(source).tapped);
            assert_eq!(g.object_view(source).prepared, lands == 8 && !lose_land);
        }
    }
}

#[test]
fn sos_studious_first_year_search_basic_tapped_or_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Studious First-Year", None, &[]);
            settle(&mut g);
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (1, 1));
            let land = named(&mut g, p, "Island", Zone::Library);
            let bad = named(&mut g, p, "Bear Cub", Zone::Library);
            paid_prep(&mut g, source, None, &[]);
            until_pending(&mut g, |pending| {
                matches!(pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: bad });
            if find {
                checked(&mut g, Action::ChooseSearch { card: land });
            }
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.on_battlefield(land), find);
            assert_eq!(g.object_view(land).tapped, find);
        }
    }
}

#[test]
fn sos_vastlands_scavenger_exact_milled_creature_mandatory_and_source_independent() {
    for p in [Player::P0, Player::P1] {
        for have_creatures in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Vastlands Scavenger", None, &[]);
            settle(&mut g);
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (4, 4));
            assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
            let old = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let first = if have_creatures {
                Some(named(&mut g, p, "Serra Angel", Zone::Library))
            } else {
                None
            };
            if have_creatures {
                named(&mut g, p, "Bear Cub", Zone::Library);
            }
            let count = g.state.players[p.index()].library.len();
            paid_prep(&mut g, source, None, &[]);
            g.move_object(source, Zone::Graveyard);
            if let Some(first) = first {
                until_pending(&mut g, |pending| {
                    matches!(pending, Pending::MilledSelection { .. })
                });
                assert!(g.observe(p.other()).milled_choice.is_none());
                let old = g.object_ref(old);
                reject(&mut g, Action::SelectMilled { object: old });
                reject(&mut g, Action::FinishMilledSelection);
                let first = g.object_ref(first);
                checked(&mut g, Action::SelectMilled { object: first });
                assert!(!matches!(g.state.pending, Pending::MilledSelection { .. }));
                settle(&mut g);
                assert!(g.on_battlefield(first.card));
            } else {
                settle(&mut g);
            }
            assert_eq!(g.state.players[p.index()].library.len(), count - 7);
            assert_eq!(g.object_view(old).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn sos_skycoach_waypoint_mana_paid_prepare_and_noop_on_ordinary_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let waypoint = named(&mut g, p, "Skycoach Waypoint", Zone::Battlefield);
        let mana = g.state.players[p.index()].mana[5];
        let tap = g
            .legal_actions()
            .into_iter()
            .find(
                |a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == waypoint),
            )
            .unwrap();
        checked(&mut g, tap);
        assert_eq!(g.state.players[p.index()].mana[5], mana + 1);
        assert!(g.object_view(waypoint).tapped);
        g.state.objects[waypoint as usize].tapped = false;
        let healer = paid(&mut g, p, "Infirmary Healer", None, &[]);
        settle(&mut g);
        paid_prep(&mut g, healer, Some(0), &[Target::Player(p)]);
        settle(&mut g);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: waypoint,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(healer),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(waypoint, Zone::Graveyard);
        settle(&mut g);
        assert!(g.object_view(healer).prepared);
        let ordinary = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let next = named(&mut g, p, "Skycoach Waypoint", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: next,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(ordinary),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert!(!g.object_view(ordinary).prepared);
    }
}

#[test]
fn sos_biblioplex_tomekeeper_optional_entry_modes_prepare_unprepare_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Infirmary Healer", None, &[]);
            settle(&mut g);
            if mode == 1 {
                paid_prep(&mut g, source, Some(0), &[Target::Player(p)]);
                settle(&mut g);
            }
            let tome = paid(&mut g, p, "Biblioplex Tomekeeper", None, &[]);
            until_pending(&mut g, |pending| matches!(pending, Pending::Modes { .. }));
            assert_eq!((g.stats(tome).power, g.stats(tome).toughness), (3, 4));
            g.move_object(tome, Zone::Graveyard);
            checked(&mut g, Action::ChooseMode { mode });
            if mode != 0 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
            }
            settle(&mut g);
            assert_eq!(g.object_view(source).prepared, mode != 2);
        }
    }
}

#[test]
fn sos_slumbering_trudge_x_entry_counters_tapped_threshold_and_stun_untaps() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 1, 2, 3, 5] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, x + 1, 0];
            let source = named(&mut g, p, "Slumbering Trudge", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: source });
            checked(&mut g, Action::ChooseX { value: x });
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            settle(&mut g);
            let counters = 3u32.saturating_sub(x);
            assert_eq!((g.stats(source).power, g.stats(source).toughness), (6, 6));
            assert_eq!(
                g.counter_count(source, CounterKind::Stun),
                u64::from(counters)
            );
            assert_eq!(g.object_view(source).tapped, x <= 2);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            for remaining in (0..counters).rev() {
                g.untap_objects(&[source], untap_rules::UntapCause::Effect);
                assert!(g.object_view(source).tapped);
                assert_eq!(
                    g.counter_count(source, CounterKind::Stun),
                    u64::from(remaining)
                );
            }
            g.untap_objects(&[source], untap_rules::UntapCause::Effect);
            assert!(!g.object_view(source).tapped);
        }
    }
}

#[test]
fn sos_slumbering_trudge_counter_replacement_and_reanimation_uses_x_zero() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = named(&mut g, p, "Slumbering Trudge", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
        checked(&mut g, Action::AnnounceCast { card: source });
        checked(&mut g, Action::ChooseX { value: 1 });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Stun), 4);
        assert!(g.object_view(source).tapped);
        g.move_object(source, Zone::Graveyard);
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Zombify", None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Stun), 6);
        assert!(g.object_view(source).tapped);
    }
}

#[test]
fn sos_trove_all_colors_and_cast_guard_survives_fizzle_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Potioner's Trove", None, &[]);
        settle(&mut g);
        let activate = Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activate.clone());
        let choices: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter(
                |a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == source),
            )
            .collect();
        assert_eq!(choices.len(), 5);
        for action in choices {
            let Action::ActivateMana { production, .. } = action else {
                unreachable!()
            };
            assert_eq!(production[5], 0);
            assert_eq!(production.iter().sum::<u32>(), 1);
            checked(&mut g, action);
            g.state.objects[source as usize].tapped = false;
        }
        paid(&mut g, p, "Bear Cub", None, &[]);
        settle(&mut g);
        reject(&mut g, activate.clone());
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Oracle's Restoration",
            None,
            &[Target::Card(bear)],
        );
        g.move_object(bear, Zone::Graveyard);
        settle(&mut g);
        let life = g.state.players[p.index()].life;
        checked(&mut g, activate);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
        g.move_object(source, Zone::Battlefield);
        g.begin_turn();
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
    }
}

#[test]
fn sos_tender_increment_ward_and_counter_removal_end_step_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Fractal Tender", None, &[]);
        settle(&mut g);
        g.begin_step(Step::End);
        settle(&mut g);
        assert!(tokens(&g, p, Card::SosFractalToken).is_empty());
        g.begin_step(Step::Main1);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Oracle's Restoration",
            None,
            &[Target::Card(source)],
        );
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        paid(&mut g, p, "Germination Practicum", None, &[]);
        settle(&mut g);
        assert!(g.counter_count(source, CounterKind::PlusOne) >= 2);
        assert!(g.put_counter_on_source_this_turn(g.object_ref(source), p));
        g.set_counters(source, Counters::default());
        g.state.active = p.other();
        g.begin_step(Step::End);
        g.prepare_triggers();
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        assert!(!g.put_counter_on_source_this_turn(g.object_ref(source), p));
        settle(&mut g);
        let made = tokens(&g, p, Card::SosFractalToken);
        assert_eq!(made.len(), 1);
        assert_eq!(g.counter_count(made[0], CounterKind::PlusOne), 3);
        assert!(g.on_battlefield(bear));
        g.begin_turn();
        assert!(g.state.counter_placement_history.is_none());
    }
}

#[test]
fn sos_counter_history_entry_placement_owner_and_exact_generation() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P1, "Fractal Tender", Zone::Hand);
    let before = g.object_view(source);
    let mut counters = Counters::default();
    counters.add(CounterKind::Stun, 1);
    g.transition_object_under_control(before, Zone::Battlefield, false, counters, Some(Player::P0));
    let old = g.object_ref(source);
    assert!(g.put_counter_on_source_this_turn(old, Player::P0));
    assert!(!g.put_counter_on_source_this_turn(old, Player::P1));
    assert_eq!(
        g.observe(Player::P0).counter_placement_history,
        g.observe(Player::P1).counter_placement_history
    );
    g.set_counters(source, Counters::default());
    assert!(g.put_counter_on_source_this_turn(old, Player::P0));
    g.move_object(source, Zone::Graveyard);
    g.move_object(source, Zone::Battlefield);
    assert!(!g.put_counter_on_source_this_turn(g.object_ref(source), Player::P0));
    g.self_counters_added(g.object_ref(source), CounterKind::Page, 0, Player::P0);
    assert!(!g.put_counter_on_source_this_turn(g.object_ref(source), Player::P0));
    g.begin_turn();
    assert!(!g.put_counter_on_source_this_turn(old, Player::P0));
}

#[test]
fn sos_scolding_repartee_optional_historical_all_counter_transfer_and_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Scolding Administrator", None, &[]);
        settle(&mut g);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Oracle's Restoration",
            None,
            &[Target::Card(target)],
        );
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert!(g.keywords(source).contains(Keywords::MENACE));
        g.add_counter(source, CounterKind::Page, 2);
        g.add_counter(source, CounterKind::Stun, 3);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 9);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        target_settle(&mut g, target);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(target, CounterKind::Page), 4);
        assert_eq!(g.counter_count(target, CounterKind::Stun), 6);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 9);
        let fresh = paid(&mut g, p, "Scolding Administrator", None, &[]);
        settle(&mut g);
        g.move_object(fresh, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
    }
}

#[test]
fn sos_tender_ward_decline_counters_opponents_targeted_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Fractal Tender", Zone::Battlefield);
        g.state.priority = p.other();
        let spell = paid(&mut g, p.other(), "Unsummon", None, &[Target::Card(source)]);
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert!(g.on_battlefield(source));
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_scolding_optional_zero_target_and_target_reentry_failure() {
    for choose in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let source = named(&mut g, p, "Scolding Administrator", Zone::Battlefield);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(source, CounterKind::Page, 2);
        g.move_object(source, Zone::Graveyard);
        g.prepare_triggers();
        if choose {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        } else {
            checked(&mut g, Action::FinishTargets);
        }
        if g.state.announcing.is_some() {
            checked(&mut g, Action::PayCosts { payment: [0; 6] });
        }
        if choose {
            g.move_object(target, Zone::Graveyard);
            g.move_object(target, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::Page), 0);
    }
}
