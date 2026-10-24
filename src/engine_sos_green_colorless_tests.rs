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

#[test]
fn sos_lumarets_favor_infusion_checks_life_at_resolution_and_may_retarget() {
    for p in [Player::P0, Player::P1] {
        for gained in [false, true] {
            let mut g = setup(p);
            let first = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            paid(&mut g, p, "Lumaret's Favor", None, &[Target::Card(first)]);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            if gained {
                g.gain_life(p, 1);
            }
            if gained {
                for _ in 0..16 {
                    g.prepare_triggers();
                    if matches!(g.state.pending, Pending::Retarget { .. }) {
                        break;
                    }
                    checked(&mut g, Action::Pass);
                }
                assert!(matches!(g.state.pending, Pending::Retarget { .. }));
                checked(
                    &mut g,
                    Action::SetStackTarget {
                        index: 0,
                        target: Target::Card(second),
                    },
                );
                checked(&mut g, Action::FinishStackTargets);
            }
            settle(&mut g);
            assert_eq!((g.stats(first).power, g.stats(first).toughness), (4, 6));
            assert_eq!(
                (g.stats(second).power, g.stats(second).toughness),
                if gained { (4, 6) } else { (2, 2) }
            );
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
        }
    }
}

#[test]
fn sos_lumarets_favor_source_copy_survives_countered_original_and_keeps_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.gain_life(p, 1);
        let original = paid(&mut g, p, "Lumaret's Favor", None, &[Target::Card(target)]);
        let spell = g
            .state
            .stack
            .iter()
            .find(|s| s.card == original && s.kind == StackKind::Spell)
            .unwrap()
            .id;
        g.counter_stack_item(spell, false);
        settle(&mut g);
        assert_eq!((g.stats(target).power, g.stats(target).toughness), (4, 6));
        assert_eq!(g.object_view(original).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
    }
}

fn until_optional(g: &mut Game) {
    for _ in 0..30 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing optional effect {:?}", g.state.pending);
}

#[test]
fn sos_transcendent_converge_optional_actual_draw_count_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            for (payment, colors) in [
                ([0, 0, 0, 0, 0, 7], 0),
                ([0, 0, 0, 0, 7, 0], 1),
                ([1, 1, 1, 1, 1, 2], 5),
            ] {
                let mut g = setup(p);
                g.state.players[p.index()].mana = payment;
                let before = g.state.players[p.index()].cards_drawn_this_turn;
                let source = paid(&mut g, p, "Transcendent Archaic", None, &[]);
                until_optional(&mut g);
                assert!(g.keywords(source).contains(Keywords::VIGILANCE));
                g.move_object(source, Zone::Graveyard);
                checked(&mut g, Action::ChooseOptional { accept });
                settle(&mut g);
                let drawn = if accept { colors } else { 0 };
                assert_eq!(
                    g.state.players[p.index()].cards_drawn_this_turn,
                    before + drawn
                );
                assert_eq!(g.hand_count(p), drawn.saturating_sub(2) as usize);
                assert_eq!(
                    g.state.players[p.index()].cards_discarded_this_turn,
                    drawn.min(2)
                );
            }
        }
    }
}

#[test]
fn sos_converge_reanimation_zero_and_cast_event_caster_copy_distinction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Transcendent Archaic", Zone::Graveyard);
        paid(&mut g, p, "Zombify", None, &[Target::Card(source)]);
        until_optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 0);
        g.state.players[p.index()].mana = [1, 1, 1, 1, 1, 2];
        paid(&mut g, p, "Transcendent Archaic", None, &[]);
        let mut item = g.state.stack.last().unwrap().clone();
        item.controller = p.other();
        let event = item.actual_cast_event().unwrap();
        assert_eq!(event.controller, p);
        assert_eq!(event.payment, [1, 1, 1, 1, 1, 2]);
        item.is_copy = true;
        assert!(item.actual_cast_event().is_none());
    }
}

#[test]
fn sos_sundering_converge_target_ceiling_nonland_opponent_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for (payment, legal_name, illegal_name) in [
            ([0, 0, 0, 0, 0, 6], "Mox Jasper", "Bear Cub"),
            ([0, 0, 0, 0, 6, 0], "Ambitious Augmenter", "Bear Cub"),
            ([1, 1, 1, 0, 0, 3], "Bear Cub", "Transcendent Archaic"),
        ] {
            let mut g = setup(p);
            let legal = named(&mut g, p.other(), legal_name, Zone::Battlefield);
            let illegal = named(&mut g, p.other(), illegal_name, Zone::Battlefield);
            let own = named(&mut g, p, legal_name, Zone::Battlefield);
            let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
            g.state.players[p.index()].mana = payment;
            let source = paid(&mut g, p, "Sundering Archaic", None, &[]);
            for _ in 0..20 {
                g.prepare_triggers();
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::Targets { .. }));
            for target in [illegal, own, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(legal),
                },
            );
            if g.state.announcing.is_some() {
                checked(&mut g, Action::PayCosts { payment: [0; 6] });
            }
            g.move_object(source, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.object_view(legal).zone, Zone::Exile);
            assert!(g.on_battlefield(illegal));
        }
    }
}

#[test]
fn sos_sundering_paid_graveyard_bottom_uses_owner_and_rejects_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Sundering Archaic", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, payment);
            g.move_object(source, Zone::Graveyard);
            if leaves {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Graveyard);
            }
            settle(&mut g);
            if leaves {
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            } else {
                assert_eq!(
                    g.state.players[p.other().index()].library.first(),
                    Some(&target)
                );
            }
        }
    }
}

#[test]
fn sos_zimone_private_up_to_two_split_revealed_hand_and_tapped_lands() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1, 2] {
            let mut g = setup(p);
            let untouched = g.state.players[p.index()].library.clone();
            let first = named(&mut g, p, "Forest", Zone::Library);
            let second = named(&mut g, p, "Bear Cub", Zone::Library);
            let illegal = named(&mut g, p, "Oracle's Restoration", Zone::Library);
            let third = named(&mut g, p, "Island", Zone::Library);
            let fourth = named(&mut g, p, "Bear Cub", Zone::Library);
            paid(&mut g, p, "Zimone's Experiment", None, &[]);
            for _ in 0..16 {
                if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert_eq!(
                g.observe(p)
                    .library_selection
                    .as_ref()
                    .unwrap()
                    .remaining
                    .len(),
                5
            );
            assert!(g.observe(p.other()).library_selection.is_none());
            reject(&mut g, Action::SelectLibrary { card: illegal });
            if count >= 1 {
                checked(&mut g, Action::SelectLibrary { card: first });
            }
            if count >= 2 {
                checked(&mut g, Action::SelectLibrary { card: second });
            }
            if count == 2 {
                reject(&mut g, Action::SelectLibrary { card: third });
            }
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            assert_eq!(g.on_battlefield(first), count >= 1);
            if count >= 1 {
                assert!(g.object_view(first).tapped);
            }
            assert_eq!(g.object_view(second).zone == Zone::Hand, count == 2);
            assert_eq!(
                g.observe(p.other()).objects.iter().any(|o| o.id == second),
                count == 2
            );
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .all(|o| o.id != illegal && o.id != third && o.id != fourth)
            );
            let library = &g.state.players[p.index()].library;
            let remainder = 5 - count;
            assert_eq!(&library[remainder..], untouched.as_slice());
            let mut actual = library[..remainder].to_vec();
            actual.sort_unstable();
            let mut expected: Vec<_> = [first, second, illegal, third, fourth]
                .into_iter()
                .filter(|id| !(count >= 1 && *id == first || count == 2 && *id == second))
                .collect();
            expected.sort_unstable();
            assert_eq!(actual, expected);
        }
    }
}

#[test]
fn sos_zimone_reveals_selected_only_during_land_entry_choices_and_finishes_creature_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Secluded Courtyard", Zone::Library);
        let creature = named(&mut g, p, "Bear Cub", Zone::Library);
        let private = named(&mut g, p, "Oracle's Restoration", Zone::Library);
        paid(&mut g, p, "Zimone's Experiment", None, &[]);
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::SelectLibrary { card: land });
        checked(&mut g, Action::SelectLibrary { card: creature });
        checked(&mut g, Action::FinishLibrarySelection);
        let public = g.observe(p.other()).library_selection.unwrap();
        assert!(public.public);
        assert!(public.remaining.is_empty());
        assert!(public.bottom.is_empty());
        assert_eq!(public.selected.len(), 2);
        assert!(public.selected.iter().all(|o| o.id != private));
        settle(&mut g);
        assert!(g.on_battlefield(land));
        assert!(g.object_view(land).tapped);
        assert_eq!(g.object_view(creature).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == creature)
        );
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != private));
    }
}

fn paid_wildgrowth(g: &mut Game, p: Player, cost: u16, payment: Mana) -> CardId {
    g.state.players[p.index()].mana = payment;
    let card = named(g, p, "Wildgrowth Archaic", Zone::Hand);
    checked(g, Action::AnnounceCast { card });
    checked(g, Action::ChooseSpellCost { cost });
    checked(g, Action::PayCosts { payment });
    card
}

#[test]
fn sos_wildgrowth_hybrid_costs_intrinsic_converge_keywords_and_zero_color_death() {
    for p in [Player::P0, Player::P1] {
        for (cost, payment, count) in [
            (0, [0, 0, 0, 0, 0, 4], 0),
            (0, [1, 1, 0, 0, 0, 2], 2),
            (1, [1, 0, 0, 0, 2, 0], 2),
            (2, [0, 0, 0, 0, 2, 0], 1),
        ] {
            let mut g = setup(p);
            let source = paid_wildgrowth(&mut g, p, cost, payment);
            settle(&mut g);
            if count == 0 {
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            } else {
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), count);
                assert!(
                    g.keywords(source)
                        .contains(Keywords::REACH.union(Keywords::TRAMPLE))
                );
                assert_eq!(g.stats(source).power, i128::from(count));
            }
        }
    }
}

#[test]
fn sos_wildgrowth_multiple_departed_sources_stack_entry_counters_and_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = paid_wildgrowth(&mut g, p, 2, [0, 0, 0, 0, 2, 0]);
        settle(&mut g);
        let second = paid_wildgrowth(&mut g, p, 2, [0, 0, 0, 0, 2, 0]);
        settle(&mut g);
        assert_eq!(g.counter_count(first, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(second, CounterKind::PlusOne), 2);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 1, 0];
        let creature = paid(&mut g, p, "Bear Cub", None, &[]);
        g.move_object(first, Zone::Graveyard);
        g.move_object(second, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 8);
        assert_eq!(g.stats(creature).power, 10);
        assert!(g.state.cast_entry_counters.is_none());
    }
}

#[test]
fn sos_wildgrowth_incoming_controller_places_counters_after_spell_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid_wildgrowth(&mut g, p, 2, [0, 0, 0, 0, 2, 0]);
        settle(&mut g);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 3];
        let source = paid(&mut g, p, "Fractal Tender", None, &[]);
        let spell = g
            .state
            .stack
            .iter_mut()
            .find(|s| s.card == source && s.kind == StackKind::Spell)
            .unwrap();
        spell.controller = p.other();
        g.state.objects[source as usize].controller = p.other();
        settle(&mut g);
        assert_eq!(g.object_view(source).controller, p.other());
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
        assert!(g.put_counter_on_source_this_turn(g.object_ref(source), p.other()));
        assert!(!g.put_counter_on_source_this_turn(g.object_ref(source), p));
    }
}

#[test]
fn sos_wildgrowth_countered_spell_and_new_incarnation_cannot_inherit_pending_bonus() {
    for p in [Player::P0, Player::P1] {
        for resolve_trigger in [false, true] {
            let mut g = setup(p);
            paid_wildgrowth(&mut g, p, 2, [0, 0, 0, 0, 2, 0]);
            settle(&mut g);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 1, 0];
            let creature = paid(&mut g, p, "Bear Cub", None, &[]);
            let spell = g
                .state
                .stack
                .iter()
                .find(|s| s.card == creature && s.kind == StackKind::Spell)
                .unwrap()
                .id;
            if resolve_trigger {
                for _ in 0..12 {
                    g.prepare_triggers();
                    if g.has_cast_entry_counters(g.object_ref(creature)) {
                        break;
                    }
                    checked(&mut g, Action::Pass);
                }
                assert!(g.has_cast_entry_counters(g.object_ref(creature)));
                assert_eq!(
                    g.observe(p).cast_entry_counters,
                    g.observe(p.other()).cast_entry_counters
                );
            }
            g.counter_stack_item(spell, false);
            g.move_object(creature, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
            assert!(g.state.cast_entry_counters.is_none());
        }
    }
}

fn dawning_attack_offer(g: &mut Game, p: Player, source: CardId, target: CardId) {
    g.state.objects[source as usize].summoning_sick = false;
    g.state.active = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: source });
    checked(g, Action::FinishAttackers);
    g.prepare_triggers();
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.state.announcing.is_some() {
        checked(g, Action::PayCosts { payment: [0; 6] });
    }
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing graveyard cast offer {:?}", g.state.pending);
}
fn choose_free_target(g: &mut Game, target: ObjectRef) {
    checked(
        g,
        Action::CastExiledDuringResolution {
            object: target,
            face: CastFace::Normal,
        },
    );
}

#[test]
fn sos_dawning_own_graveyard_card_count_discount_and_reach() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Oracle's Restoration", Zone::Graveyard);
        named(&mut g, p, "Wild Hypothesis", Zone::Graveyard);
        named(&mut g, p.other(), "Oracle's Restoration", Zone::Graveyard);
        named(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
        let source = paid(&mut g, p, "The Dawning Archaic", None, &[]);
        settle(&mut g);
        assert!(g.keywords(source).contains(Keywords::REACH));
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (7, 7));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn sos_dawning_decline_preserves_exact_graveyard_target_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Wild Hypothesis", Zone::Graveyard);
        let object = g.object_ref(target);
        let source = paid(&mut g, p, "The Dawning Archaic", None, &[]);
        settle(&mut g);
        dawning_attack_offer(&mut g, p, source, target);
        let public = g.observe(p.other()).resolution_free_casts.unwrap();
        assert_eq!(public.origin, Zone::Graveyard);
        assert!(public.exile_on_graveyard);
        assert_eq!(public.remaining, vec![object]);
        g.move_object(source, Zone::Graveyard);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.object_ref(target), object);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_dawning_free_sorcery_during_combat_x_zero_and_exile_after_resolution_or_counter() {
    for p in [Player::P0, Player::P1] {
        for counter in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Wild Hypothesis", Zone::Graveyard);
            let source = paid(&mut g, p, "The Dawning Archaic", None, &[]);
            settle(&mut g);
            dawning_attack_offer(&mut g, p, source, target);
            g.move_object(source, Zone::Graveyard);
            g.state.players[p.index()].mana = [0; 6];
            let object = g.object_ref(target);
            choose_free_target(&mut g, object);
            checked(&mut g, Action::PayCosts { payment: [0; 6] });
            let spell = g
                .state
                .stack
                .iter()
                .find(|s| s.card == target && s.kind == StackKind::Spell)
                .unwrap();
            assert_eq!(spell.x, Some(0));
            assert_eq!(spell.cast.unwrap().origin, Zone::Graveyard);
            assert!(spell.cast.unwrap().exile_on_graveyard);
            assert_eq!(spell.cast.unwrap().payment, [0; 6]);
            let stack_id = spell.id;
            checked(&mut g, Action::FinishResolutionCasts);
            if counter {
                g.counter_stack_item(stack_id, false);
            }
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
        }
    }
}

#[test]
fn sos_dawning_free_cast_still_requires_additional_sacrifice_and_has_no_stale_target() {
    for p in [Player::P0, Player::P1] {
        for sacrifice in [false, true] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Worthy Cost", Zone::Graveyard);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = paid(&mut g, p, "The Dawning Archaic", None, &[]);
            settle(&mut g);
            dawning_attack_offer(&mut g, p, source, target);
            let object = g.object_ref(target);
            if !sacrifice {
                g.move_object(source, Zone::Graveyard);
                reject(
                    &mut g,
                    Action::CastExiledDuringResolution {
                        object,
                        face: CastFace::Normal,
                    },
                );
                checked(&mut g, Action::FinishResolutionCasts);
                settle(&mut g);
                assert_eq!(g.object_ref(target), object);
                continue;
            }
            choose_free_target(&mut g, object);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            let source_ref = g.object_ref(source);
            checked(&mut g, Action::ChooseAdditionalCost { object: source_ref });
            checked(&mut g, Action::PayCosts { payment: [0; 6] });
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.object_view(enemy).zone, Zone::Exile);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
        }
    }
}

#[test]
fn sos_stack_graveyard_exile_permission_only_changes_graveyard_and_expires_on_departure() {
    for p in [Player::P0, Player::P1] {
        for destination in [Zone::Hand, Zone::Exile, Zone::Graveyard] {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Unsummon", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            g.state
                .announcing
                .as_mut()
                .unwrap()
                .item
                .cast
                .as_mut()
                .unwrap()
                .exile_on_graveyard = true;
            checked(
                &mut g,
                Action::PayCosts {
                    payment: [0, 1, 0, 0, 0, 0],
                },
            );
            let event = g.move_object(spell, destination);
            assert_eq!(
                event.destination,
                if destination == Zone::Graveyard {
                    Zone::Exile
                } else {
                    destination
                }
            );
            if g.object_view(spell).zone != Zone::Hand {
                g.move_object(spell, Zone::Hand);
            }
            cast(&mut g, spell, None, &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn sos_copy_does_not_inherit_permission_graveyard_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.gain_life(p, 1);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Lumaret's Favor", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        g.state
            .announcing
            .as_mut()
            .unwrap()
            .item
            .cast
            .as_mut()
            .unwrap()
            .exile_on_graveyard = true;
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        for _ in 0..16 {
            g.prepare_triggers();
            if matches!(g.state.pending, Pending::Retarget { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert!(
            !g.observe(p)
                .retarget
                .unwrap()
                .item
                .cast
                .unwrap()
                .exile_on_graveyard
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.stats(bear).power, 6);
    }
}

#[test]
fn sos_dawning_attack_target_own_instant_sorcery_and_rechecks_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Wild Hypothesis", Zone::Graveyard);
        let foreign = named(&mut g, p.other(), "Wild Hypothesis", Zone::Graveyard);
        let creature = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let source = paid(&mut g, p, "The Dawning Archaic", None, &[]);
        settle(&mut g);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: source });
        checked(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        for card in [foreign, creature] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.state.announcing.is_some() {
            checked(&mut g, Action::PayCosts { payment: [0; 6] });
        }
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Graveyard);
        settle(&mut g);
        assert!(g.state.resolution_free_casts.is_none());
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}

fn topiary_mana(card: CardId, amount: u32) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 0, 0, 0, amount, 0],
    }
}

#[test]
fn sos_topiary_paid_increment_strict_threshold_growth_cleanup_and_nonpositive_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Topiary Lecturer", None, &[]);
        settle(&mut g);
        reject(&mut g, topiary_mana(source, 1));
        g.state.objects[source as usize].summoning_sick = false;
        paid(&mut g, p, "Bear Cub", None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        paid(&mut g, p, "Bear Cub", None, &[]);
        settle(&mut g);
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            1,
            "equal mana is insufficient"
        );
        paid(&mut g, p, "Giant Growth", None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.stats(source).power, 5);
        reject(&mut g, topiary_mana(source, 2));
        let before = g.state.players[p.index()].mana[4];
        checked(&mut g, topiary_mana(source, 5));
        assert_eq!(g.state.players[p.index()].mana[4], before + 5);
        g.finish_cleanup();
        assert_eq!(g.stats(source).power, 2);
        let mut g = setup(p);
        let source = named(&mut g, p, "Topiary Lecturer", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = false;
        for growth in [-1, -4] {
            g.state.objects[source as usize].tapped = false;
            g.state.objects[source as usize].growth.power = growth;
            let before = g.state.players[p.index()].mana;
            checked(&mut g, topiary_mana(source, 0));
            assert_eq!(g.state.players[p.index()].mana, before);
            assert!(g.object_view(source).tapped);
        }
    }
}

#[test]
fn sos_topiary_funding_rejects_mana_payment_that_loses_artifact_scaling_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Topiary Lecturer", Zone::Battlefield);
        let arsenal = named(&mut g, p, "Machinist's Arsenal", Zone::Battlefield);
        let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let target = g.object_ref(source);
        assert!(g.attach_equipment(g.object_ref(arsenal), target));
        g.state.objects[source as usize].summoning_sick = false;
        assert_eq!(g.stats(source).power, 5);
        let spell = named(&mut g, p, "Snarl Song", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: spell });
        let pizza_first = Action::ActivateMana {
            card: pizza,
            ability: 0,
            payment: [0, 0, 0, 0, 0, 1],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 0, 0],
        };
        reject(&mut g, pizza_first.clone());
        checked(&mut g, topiary_mana(source, 5));
        checked(&mut g, pizza_first);
        assert_eq!(g.stats(source).power, 3);
        assert_eq!(g.object_view(pizza).zone, Zone::Graveyard);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        // Independently pay the same sacrifice before opening an announcement.
        let mut g = setup(p);
        let source = named(&mut g, p, "Topiary Lecturer", Zone::Battlefield);
        let arsenal = named(&mut g, p, "Machinist's Arsenal", Zone::Battlefield);
        let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(arsenal), g.object_ref(source)));
        g.state.objects[source as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::ActivateMana {
                card: pizza,
                ability: 0,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert_eq!(g.stats(source).power, 3);
        let spell = named(&mut g, p, "Snarl Song", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: spell });
    }
}

/// Tiny borrowed object overlay: it never clones a Game or changes the parent.
struct PowerWorld<'a> {
    game: &'a Game,
    objects: Vec<(CardId, Object)>,
    life: i128,
}
impl ability_world::CharacteristicWorld for PowerWorld<'_> {
    fn game(&self) -> &Game {
        self.game
    }
    fn object(&self, id: CardId) -> &Object {
        self.objects
            .iter()
            .find(|(key, _)| *key == id)
            .map_or(&self.game.state.objects[id as usize], |(_, o)| o)
    }
    fn life(&self, _: Player) -> i128 {
        self.life
    }
    fn gained_life(&self, _: Player) -> bool {
        false
    }
    fn attacking(&self, id: CardId) -> bool {
        self.game.state.attackers.iter().any(|a| a.card == id)
    }
}

#[test]
fn sos_borrowed_power_ignores_departed_provider_indexes_and_reads_virtual_counters_life() {
    use ability_world::CharacteristicWorld;
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Topiary Lecturer", Zone::Battlefield);
        let lord = named(&mut g, p, "Elvish Archdruid", Zone::Battlefield);
        let angel = named(&mut g, p, "Angel of Vitality", Zone::Battlefield);
        g.state.players[p.index()].life = 30;
        g.add_counter(source, CounterKind::PlusOne, 2);
        assert_eq!(g.stats(source).power, 4);
        assert_eq!(g.stats(angel).power, 4);
        let snapshot = g.snapshot();
        let mut gone = g.state.objects[lord as usize].clone();
        gone.zone = Zone::Graveyard;
        let mut creature = g.state.objects[source as usize].clone();
        creature.counters.plus_one = 0;
        creature.counters.minus_one = 1;
        let world = PowerWorld {
            game: &g,
            objects: vec![(lord, gone), (source, creature)],
            life: 20,
        };
        assert_eq!(
            world.power(source),
            0,
            "removed lord must not survive its actual-world source index"
        );
        assert_eq!(
            world.power(angel),
            2,
            "life thresholds use the virtual payer's life"
        );
        assert_eq!(g.snapshot(), snapshot);
    }
}

#[test]
fn sos_source_power_mana_of_animated_noncreature_reads_current_base_setter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        let production = ManaProduction::SourcePower { color: 4 };
        assert_eq!(
            production.in_world(&g, source),
            ManaProduction::Fixed([0; 6])
        );
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.stats(source).power, 3);
        assert_eq!(
            production.in_world(&g, source),
            ManaProduction::Fixed([0, 0, 0, 0, 3, 0])
        );
    }
}

#[test]
fn sos_diary_paid_casts_collect_pages_only_for_own_instants_and_sorceries() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let diary = paid(&mut g, p, "Diary of Dreams", None, &[]);
        settle(&mut g);
        let bear = paid(&mut g, p, "Bear Cub", None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(diary, CounterKind::Page), 0);
        paid(&mut g, p, "Giant Growth", None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.counter_count(diary, CounterKind::Page), 1);
        let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        paid(&mut g, p, "Zombify", None, &[Target::Card(dead)]);
        settle(&mut g);
        assert_eq!(g.counter_count(diary, CounterKind::Page), 2);
        g.state.priority = p.other();
        paid(
            &mut g,
            p.other(),
            "Giant Growth",
            None,
            &[Target::Card(bear)],
        );
        settle(&mut g);
        assert_eq!(g.counter_count(diary, CounterKind::Page), 2);
        g.gain_life(p, 1);
        paid(&mut g, p, "Lumaret's Favor", None, &[Target::Card(bear)]);
        for _ in 0..30 {
            g.prepare_triggers();
            if matches!(g.state.pending, Pending::Retarget { .. }) {
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
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(
            g.counter_count(diary, CounterKind::Page),
            3,
            "an uncast copy does not trigger Diary"
        );
    }
}

#[test]
fn sos_diary_discount_zero_to_saturated_and_draw_survives_departure() {
    for p in [Player::P0, Player::P1] {
        for pages in [0, 1, 4, 5, 7, u64::MAX] {
            let mut g = setup(p);
            let diary = named(&mut g, p, "Diary of Dreams", Zone::Battlefield);
            g.add_counter(diary, CounterKind::Page, pages);
            let required = 5u32.saturating_sub(u32::try_from(pages).unwrap_or(u32::MAX));
            let activate = Action::AnnounceActivate {
                card: diary,
                ability: 0,
                counter_payment: Counters::default(),
            };
            if required > 0 {
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, required - 1];
                reject(&mut g, activate.clone());
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, required];
            let hand = g.hand_count(p);
            checked(&mut g, activate.clone());
            assert_eq!(
                g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
                required
            );
            checked(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, required],
                },
            );
            assert!(g.object_view(diary).tapped);
            assert_eq!(g.counter_count(diary, CounterKind::Page), pages);
            if pages == 4 {
                g.move_object(diary, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            reject(&mut g, activate);
        }
    }
}

#[test]
fn sos_diary_discount_is_locked_before_payments_and_borrowing_uses_recipient_pages() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let diary = named(&mut g, p, "Diary of Dreams", Zone::Battlefield);
        g.add_counter(diary, CounterKind::Page, 2);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: diary,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.add_counter(diary, CounterKind::Page, 5);
        assert_eq!(
            g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
            3
        );
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 3],
            },
        );
        settle(&mut g);
        let mut g = setup(p);
        let diary = named(&mut g, p.other(), "Diary of Dreams", Zone::Battlefield);
        g.add_counter(diary, CounterKind::Page, 9);
        let forge = paid(&mut g, p, "Territory Forge", None, &[]);
        target_settle(&mut g, diary);
        assert_eq!(g.object_view(diary).zone, Zone::Exile);
        assert_eq!(g.counter_count(diary, CounterKind::Page), 0);
        g.add_counter(forge, CounterKind::Page, 3);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let descriptor = g
            .borrowed_ability_views()
            .into_iter()
            .find(|a| a.source.card == forge && a.ability == 0)
            .unwrap();
        let hand = g.hand_count(p);
        checked(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: forge,
                exiled: descriptor.exiled,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(
            g.funding_goal(g.state.announcing.as_ref().unwrap()).generic,
            2
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.counter_count(forge, CounterKind::Page), 3);
    }
}

#[test]
fn sos_diary_legacy_prefunded_activation_pays_reduced_cost_without_spending_pages() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let diary = named(&mut g, p, "Diary of Dreams", Zone::Battlefield);
        g.add_counter(diary, CounterKind::Page, 2);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        let hand = g.hand_count(p);
        checked(
            &mut g,
            Action::Activate {
                card: diary,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 0, 0, 3],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.counter_count(diary, CounterKind::Page), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

fn page_grandeur(g: &mut Game, source: CardId, fuel: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    assert!(
        g.observe(g.actor().unwrap().other())
            .additional_cost
            .is_none()
    );
    checked(
        g,
        Action::ChooseAdditionalCost {
            object: g.object_ref(fuel),
        },
    );
    checked(g, Action::PayCosts { payment: [0; 6] });
    assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
}
fn page_reveal(g: &mut Game) {
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("Page reveal did not suspend");
}

#[test]
fn sos_page_paid_body_sickness_colorless_mana_and_exact_private_named_discard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let page = paid(&mut g, p, "Page, Loose Leaf", None, &[]);
        settle(&mut g);
        assert_eq!((g.stats(page).power, g.stats(page).toughness), (0, 2));
        let tap = Action::ActivateMana {
            card: page,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        };
        reject(&mut g, tap.clone());
        let activate = Action::AnnounceActivate {
            card: page,
            ability: 1,
            counter_payment: Counters::default(),
        };
        let wrong = named(&mut g, p, "Bear Cub", Zone::Hand);
        let foreign = named(&mut g, p.other(), "Page, Loose Leaf", Zone::Hand);
        reject(&mut g, activate.clone());
        let fuel = named(&mut g, p, "Page, Loose Leaf", Zone::Hand);
        checked(&mut g, activate);
        assert!(g.observe(p.other()).additional_cost.is_none());
        let view = g.observe(p).additional_cost.unwrap();
        assert_eq!(view.candidates, vec![g.object_ref(fuel)]);
        for id in [page, wrong, foreign] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(fuel);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::PayCosts { payment: [0; 6] });
        page_reveal(&mut g);
        assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        g.state.objects[page as usize].summoning_sick = false;
        let before = g.state.players[p.index()].mana[5];
        checked(&mut g, tap);
        assert_eq!(g.state.players[p.index()].mana[5], before + 1);
        assert!(g.object_view(page).tapped);
    }
}

#[test]
fn sos_page_grandeur_reveals_until_first_spell_publicly_and_random_bottoms_rest() {
    for p in [Player::P0, Player::P1] {
        for name in ["Shock", "Zombify"] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Page, Loose Leaf", Zone::Battlefield);
            let fuel = named(&mut g, p, "Page, Loose Leaf", Zone::Hand);
            let deeper = named(&mut g, p, "Giant Growth", Zone::Library);
            let spell = named(&mut g, p, name, Zone::Library);
            let bear = named(&mut g, p, "Bear Cub", Zone::Library);
            let forest = named(&mut g, p, "Forest", Zone::Library);
            page_grandeur(&mut g, source, fuel);
            g.move_object(source, Zone::Graveyard);
            page_reveal(&mut g);
            for viewer in [p, p.other()] {
                let view = g.observe(viewer).library_selection.unwrap();
                assert!(view.public);
                assert_eq!(
                    view.remaining.iter().map(|o| o.id).collect::<Vec<_>>(),
                    vec![forest, bear, spell]
                );
                assert_eq!(view.candidates, vec![g.object_ref(spell)]);
                assert_eq!((view.minimum, view.maximum), (1, 1));
            }
            reject(&mut g, Action::FinishLibrarySelection);
            reject(&mut g, Action::SelectLibrary { card: bear });
            checked(&mut g, Action::SelectLibrary { card: spell });
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            assert!(
                g.observe(p.other())
                    .known_hand
                    .contains(&g.object_ref(spell))
            );
            let library = &g.state.players[p.index()].library;
            assert_eq!(library.last(), Some(&deeper));
            let mut bottom = library[..2].to_vec();
            bottom.sort_unstable();
            let mut expected = vec![forest, bear];
            expected.sort_unstable();
            assert_eq!(bottom, expected);
        }
    }
}

#[test]
fn sos_page_no_spell_reveals_whole_library_and_empty_library_does_not_draw() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = setup(p);
            if empty {
                let cards = g.state.players[p.index()].library.to_vec();
                for id in cards {
                    g.move_object(id, Zone::Exile);
                }
            }
            let source = named(&mut g, p, "Page, Loose Leaf", Zone::Battlefield);
            let fuel = named(&mut g, p, "Page, Loose Leaf", Zone::Hand);
            let hand = g.hand_count(p);
            let count = g.state.players[p.index()].library.len();
            page_grandeur(&mut g, source, fuel);
            if !empty {
                page_reveal(&mut g);
                let view = g.observe(p.other()).library_selection.unwrap();
                assert_eq!(view.remaining.len(), count);
                assert_eq!((view.minimum, view.maximum), (0, 0));
                checked(&mut g, Action::FinishLibrarySelection);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand - 1);
            assert_eq!(g.state.players[p.index()].library.len(), count);
            assert!(!g.state.players[p.index()].attempted_empty_draw);
            assert!(g.outcome().is_none());
        }
    }
}

fn animate_hall(g: &mut Game, hall: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: hall,
            ability: 2,
            counter_payment: Counters::default(),
        },
    );
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, pay);
    settle(g);
}

#[test]
fn sos_great_hall_land_play_colorless_and_paid_life_restricted_all_color_mana() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: hall });
            settle(&mut g);
            assert!(!g.object_view(hall).tapped);
            checked(
                &mut g,
                Action::ActivateMana {
                    card: hall,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 0, 0, 1],
                },
            );
            g.state.objects[hall as usize].tapped = false;
            g.state.players[p.index()].mana = [0; 6];
            let life = g.state.players[p.index()].life;
            let mut production = [0; 6];
            production[color] = 1;
            checked(
                &mut g,
                Action::ActivateMana {
                    card: hall,
                    ability: 1,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production,
                },
            );
            assert_eq!(g.state.players[p.index()].life, life - 1);
            assert_eq!(
                g.mana_lots(p)[0].quality.restriction,
                ManaRestrictionView::InstantSorcery
            );
            if color == 4 {
                let elves = named(&mut g, p, "Llanowar Elves", Zone::Hand);
                reject(&mut g, Action::AnnounceCast { card: elves });
                let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                paid(&mut g, p, "Giant Growth", None, &[Target::Card(bear)]);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert!(
                    g.state.temporary_abilities.is_none(),
                    "unanimated Hall has no cast trigger"
                );
            }
        }
    }
}

#[test]
fn sos_great_hall_conditional_animation_persists_and_repeated_activation_does_not_stack_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: hall });
        animate_hall(&mut g, hall);
        let view = g.object_view(hall);
        assert_eq!(view.kinds & 3, 3);
        assert!(g.has_subtype(hall, "Wizard"));
        assert_eq!((view.power, view.toughness), (2, 4));
        reject(
            &mut g,
            Action::ActivateMana {
                card: hall,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        animate_hall(&mut g, hall);
        assert_eq!(g.state.temporary_abilities.as_ref().unwrap().len(), 1);
        let bear = paid(&mut g, p, "Bear Cub", None, &[]);
        settle(&mut g);
        assert_eq!(g.stats(hall).power, 2);
        paid(&mut g, p, "Giant Growth", None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(hall).power, g.stats(hall).toughness), (3, 4));
        g.finish_cleanup();
        assert_eq!((g.stats(hall).power, g.stats(hall).toughness), (2, 4));
        assert!(g.is_creature(hall));
        assert_eq!(g.state.temporary_abilities.as_ref().unwrap().len(), 1);
    }
}

#[test]
fn sos_great_hall_granted_cast_trigger_tracks_new_controller_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        animate_hall(&mut g, hall);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = paid(&mut g, p, "Witness Protection", None, &[Target::Card(hall)]);
        settle(&mut g);
        assert_eq!(
            g.stats(hall).power,
            1,
            "enchantment casts do not trigger the granted instant/sorcery ability"
        );
        paid(&mut g, p, "Giant Growth", None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(
            g.stats(hall).power,
            1,
            "older granted ability is unavailable under the Aura"
        );
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.stats(hall).power, 2);
        paid(
            &mut g,
            p,
            "Harmless Offering",
            None,
            &[Target::Player(p.other()), Target::Card(hall)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(hall).controller, p.other());
        assert_eq!(g.stats(hall).power, 3);
        g.state.priority = p.other();
        paid(
            &mut g,
            p.other(),
            "Giant Growth",
            None,
            &[Target::Card(bear)],
        );
        settle(&mut g);
        assert_eq!(g.stats(hall).power, 4);
        paid(&mut g, p, "Giant Growth", None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.stats(hall).power, 4);
    }
}

#[test]
fn sos_great_hall_queued_growth_and_animation_do_not_rebind_new_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        animate_hall(&mut g, hall);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Giant Growth", None, &[Target::Card(bear)]);
        g.move_object(hall, Zone::Hand);
        g.move_object(hall, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.is_creature(hall));
        assert_eq!(g.stats(hall).power, 0);
        assert!(g.state.temporary_abilities.is_none());
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: hall,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        g.move_object(hall, Zone::Hand);
        g.move_object(hall, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.is_creature(hall));
        assert!(g.state.temporary_abilities.is_none());
    }
}

fn await_land_name(g: &mut Game) {
    for _ in 0..50 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::LandCardName { .. }) {
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
    panic!("land name did not resolve: {:?}", g.state.pending);
}
fn choose_land_name(g: &mut Game, name: &str) {
    checked(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(crate::CardName::from_name(name).unwrap()),
        },
    );
}
fn granted_land_mana(card: CardId, grantor: ObjectRef, production: [u32; 6]) -> Action {
    Action::TapGrantedMana {
        card,
        grantor,
        production,
    }
}
fn activate_draw(g: &mut Game, card: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(g, Action::PayCosts { payment: [0; 6] });
}

#[test]
fn sos_hamlet_delayed_choice_global_lock_and_independent_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Soulstone Sanctuary", Zone::Battlefield);
        let hamlet = named(&mut g, p, "Petrified Hamlet", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: hamlet });
        let animation = Action::AnnounceActivate {
            card: own,
            ability: 1,
            counter_payment: Counters::default(),
        };
        assert!(
            g.legal_actions().contains(&animation),
            "choice is an ordinary ETB trigger"
        );
        assert!(ability_world::CharacteristicWorld::granted_mana(&g, own).is_empty());
        await_land_name(&mut g);
        for viewer in [p, p.other()] {
            let view = g.observe(viewer).entry_value.unwrap();
            assert_eq!(view.kind, EntryChoiceKind::LandCardName);
            assert!(view.opponent_hand.is_none());
        }
        for wrong in ["Bear Cub", "Bala Ged Recovery"] {
            reject(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::CardName(crate::CardName::from_name(wrong).unwrap()),
                },
            );
        }
        for accepted in ["Gaea's Cradle", "Bala Ged Sanctuary", "Soulstone Sanctuary"] {
            assert!(g.legal_actions().contains(&Action::ChooseEntryValue {
                value: EntryChoiceValue::CardName(crate::CardName::from_name(accepted).unwrap())
            }));
        }
        choose_land_name(&mut g, "Soulstone Sanctuary");
        settle(&mut g);
        for (controller, card) in [(p, own), (p.other(), enemy)] {
            g.state.priority = controller;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card,
                    ability: 1,
                    counter_payment: Counters::default(),
                },
            );
            let mana = granted_land_mana(card, g.object_ref(hamlet), [0, 0, 0, 0, 0, 1]);
            checked(&mut g, mana);
        }
        g.move_object(hamlet, Zone::Graveyard);
        g.state.priority = p;
        g.state.objects[own as usize].tapped = false;
        assert!(g.legal_actions().contains(&animation));
    }
}

#[test]
fn sos_hamlet_old_trigger_choice_does_not_bind_reentered_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        let hamlet = named(&mut g, p, "Petrified Hamlet", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: hamlet });
        let old = g.object_ref(hamlet);
        g.move_object(hamlet, Zone::Hand);
        g.move_object(hamlet, Zone::Battlefield);
        // Both queued ETB abilities remain mandatory. Resolve the newer one first
        // and leave its source incarnation before the older choice resolves.
        await_land_name(&mut g);
        g.move_object(hamlet, Zone::Hand);
        choose_land_name(&mut g, "Soulstone Sanctuary");
        await_land_name(&mut g);
        choose_land_name(&mut g, "Soulstone Sanctuary");
        settle(&mut g);
        assert!(g.entry_value(old).is_none());
        assert!(ability_world::CharacteristicWorld::granted_mana(&g, land).is_empty());
    }
}

#[test]
fn sos_lute_grants_five_restricted_double_mana_choices_to_current_lands() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lute = paid(&mut g, p, "Resonating Lute", None, &[]);
        settle(&mut g);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let provider = g.object_ref(lute);
        for color in 0..5 {
            let mut production = [0; 6];
            production[color] = 2;
            assert!(
                g.legal_actions()
                    .contains(&granted_land_mana(land, provider, production))
            );
        }
        reject(
            &mut g,
            granted_land_mana(land, provider, [1, 1, 0, 0, 0, 0]),
        );
        g.state.priority = p.other();
        reject(
            &mut g,
            granted_land_mana(foreign, provider, [0, 0, 2, 0, 0, 0]),
        );
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            granted_land_mana(land, provider, [0, 0, 0, 0, 2, 0]),
        );
        g.move_object(lute, Zone::Graveyard);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: bear }),
            "restriction persists after provider leaves"
        );
        let creature = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(creature)]);
        settle(&mut g);
        assert_eq!(g.stats(creature).power, 5);
        reject(
            &mut g,
            granted_land_mana(land, provider, [0, 0, 0, 0, 2, 0]),
        );
    }
}

#[test]
fn sos_lute_borrowed_funding_and_activation_hand_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lute = paid(&mut g, p, "Resonating Lute", None, &[]);
        settle(&mut g);
        let land = named(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let creature = named(&mut g, p, "Bear Cub", Zone::Hand);
        let instant = named(&mut g, p, "Giant Growth", Zone::Hand);
        let target = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: creature })
        );
        checked(&mut g, Action::AnnounceCast { card: instant });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        let grantor = g.object_ref(lute);
        checked(&mut g, granted_land_mana(land, grantor, [0, 0, 0, 0, 2, 0]));
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 0],
            },
        );
        settle(&mut g);
        let activation = Action::AnnounceActivate {
            card: lute,
            ability: 0,
            counter_payment: Counters::default(),
        };
        while g.hand_count(p) < 6 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        reject(&mut g, activation.clone());
        named(&mut g, p, "Forest", Zone::Hand);
        activate_draw(&mut g, lute);
        // The hand condition applies to activation, not resolution.
        let hand = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| (o.zone == Zone::Hand && o.owner == p).then_some(id as CardId))
            .unwrap();
        g.move_object(hand, Zone::Graveyard);
        let before = g.hand_count(p);
        g.move_object(lute, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}

#[test]
fn sos_lute_land_type_loss_controller_change_and_new_provider_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lute = paid(&mut g, p, "Resonating Lute", None, &[]);
        settle(&mut g);
        let land = named(&mut g, p, "Soulstone Sanctuary", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Soulstone Sanctuary", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        let aura = paid(&mut g, p, "Witness Protection", None, &[Target::Card(land)]);
        settle(&mut g);
        let old = g.object_ref(lute);
        reject(&mut g, granted_land_mana(land, old, [0, 0, 2, 0, 0, 0]));
        g.move_object(aura, Zone::Graveyard);
        assert!(
            g.legal_actions()
                .contains(&granted_land_mana(land, old, [0, 0, 2, 0, 0, 0]))
        );
        paid(
            &mut g,
            p,
            "Harmless Offering",
            None,
            &[Target::Player(p.other()), Target::Card(lute)],
        );
        settle(&mut g);
        reject(&mut g, granted_land_mana(land, old, [0, 0, 2, 0, 0, 0]));
        g.state.priority = p.other();
        checked(&mut g, granted_land_mana(foreign, old, [0, 0, 2, 0, 0, 0]));
        g.move_object(lute, Zone::Hand);
        g.move_object(lute, Zone::Battlefield);
        g.state.priority = p;
        reject(&mut g, granted_land_mana(land, old, [0, 0, 2, 0, 0, 0]));
        let fresh = g.object_ref(lute);
        assert_ne!(fresh, old);
        checked(&mut g, granted_land_mana(land, fresh, [0, 0, 2, 0, 0, 0]));
    }
}
