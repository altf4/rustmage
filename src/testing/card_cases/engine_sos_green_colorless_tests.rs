use crate::engine::ability_world::CharacteristicWorld;
// Focused printed outcomes; shared engine invariants live in core tests.
// Paid SOS green/colorless interactions, including exact token continuations.
use super::*;
use crate::engine::testing::card_cases::big_helpers::{cast, checked, named, settle, setup};

fn reject(g: &mut Game, action: Action) {
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
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
fn sos_chelonian_tackle_optional_opponent_fight_and_partial_illegal_operands() {
    {
        let p = Player::P0;
        for opponent in [false, true] {
            {
                let leaves = false;
                let mut g = setup(p);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
                let mut targets = vec![Target::Card(own)];
                if opponent {
                    targets.push(Target::Card(enemy));
                }
                paid(&mut g, p, "Chelonian Tackle", None, &targets);

                settle(&mut g);
                assert_eq!(g.on_battlefield(enemy), !opponent || leaves);
                {
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
fn sos_together_as_one_counts_colors_not_colorless_and_separates_targets() {
    {
        let p = Player::P0;
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
fn sos_fractal_counter_completion_replacements_apply_after_token_doubling() {
    {
        let p = Player::P0;
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
fn sos_pestbrood_sloth_reach_death_tokens_and_attack_life() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_ambitious_augmenter_increment_threshold_and_historical_all_counter_transfer() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_skycoach_waypoint_mana_paid_prepare_and_noop_on_ordinary_creature() {
    {
        let p = Player::P0;
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
fn sos_slumbering_trudge_x_entry_counters_tapped_threshold_and_stun_untaps() {
    {
        let p = Player::P0;
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
fn sos_tender_increment_ward_and_counter_removal_end_step_history() {
    {
        let p = Player::P0;
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
fn sos_scolding_repartee_optional_historical_all_counter_transfer_and_replacements() {
    {
        let p = Player::P0;
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
fn sos_lumarets_favor_infusion_checks_life_at_resolution_and_may_retarget() {
    {
        let p = Player::P0;
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
fn sos_converge_reanimation_zero_and_cast_event_caster_copy_distinction() {
    {
        let p = Player::P0;
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
fn sos_sundering_paid_graveyard_bottom_uses_owner_and_rejects_new_incarnation() {
    {
        let p = Player::P0;
        {
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

            settle(&mut g);
            {
                assert_eq!(
                    g.state.players[p.other().index()].library.first(),
                    Some(&target)
                );
            }
        }
    }
}

#[test]
fn sos_zimone_reveals_selected_only_during_land_entry_choices_and_finishes_creature_hand() {
    {
        let p = Player::P0;
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

#[test]
fn sos_dawning_own_graveyard_card_count_discount_and_reach() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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

#[test]
fn sos_source_power_mana_of_animated_noncreature_reads_current_base_setter() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_diary_discount_is_locked_before_payments_and_borrowing_uses_recipient_pages() {
    {
        let p = Player::P0;
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
fn sos_page_grandeur_reveals_until_first_spell_publicly_and_random_bottoms_rest() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_great_hall_granted_cast_trigger_tracks_new_controller_and_ability_removal() {
    {
        let p = Player::P0;
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

#[test]
fn sos_hamlet_old_trigger_choice_does_not_bind_reentered_source() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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

fn await_mind_roots_discard(g: &mut Game, player: Player) {
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::EffectDiscard { player: actor, .. } if actor == player)
        {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing discard phase: {:?}", g.state.pending);
}

#[test]
fn sos_mind_roots_discard_replacement_to_exile_and_optional_decline_preserves_identity() {
    {
        let p = Player::P0;
        for decline in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, "Rest in Peace", None, &[]);
            settle(&mut g);
            let land = named(&mut g, p.other(), "Forest", Zone::Hand);
            let spell = named(&mut g, p.other(), "Shock", Zone::Hand);
            let old = g.object_ref(land);
            paid(&mut g, p, "Mind Roots", None, &[Target::Player(p.other())]);
            await_mind_roots_discard(&mut g, p.other());
            checked(&mut g, Action::Discard { card: land });
            checked(&mut g, Action::Discard { card: spell });
            assert_eq!(g.object_view(land).zone, Zone::Exile);
            let discarded = g.object_ref(land);
            assert_ne!(old, discarded);
            reject(&mut g, Action::SelectLand { object: old });
            if decline {
                checked(&mut g, Action::FinishLandSelection);
            } else {
                checked(&mut g, Action::SelectLand { object: discarded });
            }
            settle(&mut g);
            if decline {
                assert_eq!(g.object_ref(land), discarded);
                assert_eq!(g.object_view(land).zone, Zone::Exile);
            } else {
                assert_eq!(g.object_view(land).zone, Zone::Battlefield);
                assert_eq!(g.object_view(land).controller, p);
                assert!(g.object_view(land).tapped);
            }
        }
    }
}
#[test]
fn sos_mind_roots_staged_discard_entry_and_later_incarnation_are_not_unrelated_cards() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        let liege = named(&mut g, p.other(), "Wilt-Leaf Liege", Zone::Hand);
        paid(&mut g, p, "Mind Roots", None, &[Target::Player(p.other())]);
        await_mind_roots_discard(&mut g, p.other());
        checked(&mut g, Action::Discard { card: land });
        checked(&mut g, Action::Discard { card: liege });
        assert_eq!(g.object_view(liege).zone, Zone::Battlefield);
        assert_eq!(g.object_view(liege).controller, p.other());
        assert!(matches!(g.state.pending,Pending::EffectChoice{player} if player==p));
        let exact = g.object_ref(land);
        g.move_object(land, Zone::Hand);
        g.move_object(land, Zone::Graveyard);
        reject(&mut g, Action::SelectLand { object: exact });
        let changed = g.object_ref(land);
        reject(&mut g, Action::SelectLand { object: changed });
        checked(&mut g, Action::FinishLandSelection);
        settle(&mut g);
        assert_eq!(g.object_ref(land), changed);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
    }
}

fn original_until(g: &mut Game, predicate: impl Fn(&Pending) -> bool) {
    for _ in 0..80 {
        g.prepare_triggers();
        if predicate(&g.state.pending) {
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
    panic!("original SOS phase missing: {:?}", g.state.pending);
}
fn original_activate(g: &mut Game, card: CardId, ability: u16, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    if g.state.announcing.is_some() {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
}
#[test]
fn sos_original_basic_and_slow_land_play_thresholds_and_exact_colors() {
    {
        let p = Player::P0;
        for (name, color) in [
            ("Plains", 0),
            ("Island", 1),
            ("Swamp", 2),
            ("Mountain", 3),
            ("Forest", 4),
        ] {
            let mut g = setup(p);
            let land = named(&mut g, p, name, Zone::Hand);
            checked(&mut g, Action::PlayLand { card: land });
            assert!(!g.object_view(land).tapped);
            let before = g.state.players[p.index()].mana;
            checked(&mut g, Action::TapMana { card: land });
            assert_eq!(g.state.players[p.index()].mana[color], before[color] + 1);
            reject(&mut g, Action::TapMana { card: land });
        }
        for (name, colors) in [
            ("Deathcap Glade", [2, 4]),
            ("Dreamroot Cascade", [1, 4]),
            ("Shattered Sanctum", [0, 2]),
            ("Stormcarved Coast", [1, 3]),
            ("Sundown Pass", [0, 3]),
        ] {
            for own in 0..=2 {
                let mut g = setup(p);
                for _ in 0..own {
                    named(&mut g, p, "Forest", Zone::Battlefield);
                }
                for _ in 0..3 {
                    named(&mut g, p.other(), "Forest", Zone::Battlefield);
                }
                let land = named(&mut g, p, name, Zone::Hand);
                checked(&mut g, Action::PlayLand { card: land });
                assert_eq!(g.object_view(land).tapped, own < 2);
                for color in colors {
                    g.state.objects[land as usize].tapped = false;
                    let mut production = [0; 6];
                    production[color] = 1;
                    checked(
                        &mut g,
                        Action::ActivateMana {
                            card: land,
                            ability: 0,
                            payment: [0; 6],
                            counter_payment: Counters::default(),
                            production,
                        },
                    );
                }
                g.state.objects[land as usize].tapped = false;
                reject(
                    &mut g,
                    Action::ActivateMana {
                        card: land,
                        ability: 0,
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                        production: [0, 0, 0, 0, 0, 1],
                    },
                );
            }
        }
    }
}

#[test]
fn sos_original_skycoach_crew_sick_creatures_exact_power_and_cleanup() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let vehicle = paid(&mut g, p, "Strixhaven Skycoach", None, &[]);
        until_optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        let first = paid(&mut g, p, "Noxious Newt", None, &[]);
        settle(&mut g);
        let second = paid(&mut g, p, "Llanowar Elves", None, &[]);
        settle(&mut g);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(!g.is_creature(vehicle));
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(&mut g, Action::FinishCrew);
        let enemy_ref = g.object_ref(enemy);
        reject(&mut g, Action::ChooseCrewer { object: enemy_ref });
        let first_ref = g.object_ref(first);
        checked(&mut g, Action::ChooseCrewer { object: first_ref });
        reject(&mut g, Action::ChooseCrewer { object: first_ref });
        reject(&mut g, Action::FinishCrew);
        let second_ref = g.object_ref(second);
        checked(&mut g, Action::ChooseCrewer { object: second_ref });
        checked(&mut g, Action::FinishCrew);
        if g.state.announcing.is_some() {
            checked(&mut g, Action::PayCosts { payment: [0; 6] });
        }
        assert!(g.object_view(first).tapped && g.object_view(second).tapped);
        assert!(!g.is_creature(vehicle));
        settle(&mut g);
        assert_eq!((g.stats(vehicle).power, g.stats(vehicle).toughness), (3, 2));
        assert!(g.keywords(vehicle).contains(Keywords::FLYING));
        g.finish_cleanup();
        assert!(!g.is_creature(vehicle));
    }
}

#[test]
fn sos_original_graffalon_increment_strict_trigger_and_resolution_checks() {
    {
        let p = Player::P0;
        for buff in [false, true] {
            let mut g = setup(p);
            let graffalon = paid(&mut g, p, "Hungry Graffalon", None, &[]);
            settle(&mut g);
            assert!(g.keywords(graffalon).contains(Keywords::REACH));
            paid(&mut g, p, "Shopkeeper's Bane", None, &[]);
            settle(&mut g);
            assert_eq!(
                g.object_view(graffalon).counters.plus_one,
                0,
                "three is not greater than three or four"
            );
            paid(&mut g, p, "Rancorous Archaic", None, &[]);
            g.prepare_triggers();
            if buff {
                paid(&mut g, p, "Giant Growth", None, &[Target::Card(graffalon)]);
            }
            settle(&mut g);
            assert_eq!(g.object_view(graffalon).counters.plus_one, u64::from(!buff));
        }
    }
}

#[test]
fn sos_original_biomancer_once_each_turn_and_newt_sickness_mana_deathtouch_fight() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let life = g.state.players[p.index()].life;
        let biomancer = paid(&mut g, p, "Mindful Biomancer", None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        original_activate(&mut g, biomancer, 0, &[]);
        settle(&mut g);
        assert_eq!(
            (g.stats(biomancer).power, g.stats(biomancer).toughness),
            (4, 4)
        );
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: biomancer,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.finish_cleanup();
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        g.state.step = Step::Main1;
        original_activate(&mut g, biomancer, 0, &[]);
        settle(&mut g);
        assert_eq!(g.stats(biomancer).power, 4);
        let mut g = setup(p);
        let newt = paid(&mut g, p, "Noxious Newt", None, &[]);
        settle(&mut g);
        let mana = Action::ActivateMana {
            card: newt,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        reject(&mut g, mana.clone());
        g.state.objects[newt as usize].summoning_sick = false;
        let before = g.state.players[p.index()].mana[4];
        checked(&mut g, mana);
        assert_eq!(g.state.players[p.index()].mana[4], before + 1);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Chelonian Tackle",
            None,
            &[Target::Card(newt), Target::Card(enemy)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert!(g.on_battlefield(newt));
    }
}
#[test]
fn sos_original_manawurm_referee_and_witherbloom_distinguish_actual_payment_and_color() {
    {
        let p = Player::P0;
        for provider_leaves in [false, true] {
            let mut g = setup(p);
            let wurm = paid(&mut g, p, "Aberrant Manawurm", None, &[]);
            settle(&mut g);
            let referee = paid(&mut g, p, "Mage Tower Referee", None, &[]);
            settle(&mut g);
            for _ in 0..6 {
                named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            }
            g.state.players[p.index()].mana = [0, 0, 1, 0, 1, 4];
            let dragon = paid(&mut g, p, "Witherbloom, the Balancer", None, &[]);
            assert_eq!(
                g.state
                    .stack
                    .iter()
                    .find(|item| item.source.is_some_and(|source| source.id == dragon))
                    .unwrap()
                    .cast
                    .unwrap()
                    .payment
                    .iter()
                    .sum::<u32>(),
                6
            );
            settle(&mut g);
            assert_eq!(g.object_view(referee).counters.plus_one, 1);
            assert_eq!(g.stats(wurm).power, 2);
            assert!(
                g.keywords(dragon).contains(Keywords::FLYING)
                    && g.keywords(dragon).contains(Keywords::DEATHTOUCH)
            );
            if provider_leaves {
                g.move_object(dragon, Zone::Graveyard);
            }
            g.state.players[p.index()].mana = [30; 6];
            let target = named(&mut g, p.other(), "Mox Jasper", Zone::Battlefield);
            let spell = paid(
                &mut g,
                p,
                "Grapple with Death",
                None,
                &[Target::Card(target)],
            );
            let amount = if provider_leaves { 3 } else { 2 };
            assert_eq!(
                g.state
                    .stack
                    .iter()
                    .find(|item| item.source.is_some_and(|source| source.id == spell))
                    .unwrap()
                    .cast
                    .unwrap()
                    .payment
                    .iter()
                    .sum::<u32>(),
                amount
            );
            settle(&mut g);
            assert_eq!(g.stats(wurm).power, 2 + i128::from(amount));
            assert_eq!(g.object_view(referee).counters.plus_one, 2);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            g.state.players[p.index()].mana = [1, 1, 1, 1, 1, 0];
            paid(&mut g, p, "Rancorous Archaic", None, &[]);
            settle(&mut g);
            assert_eq!(
                g.object_view(referee).counters.plus_one,
                2,
                "five payment colors do not make a colorless spell multicolored"
            );
            g.finish_cleanup();
            assert_eq!(g.stats(wurm).power, 2);
        }
    }
}

#[test]
fn sos_original_shopkeeper_attack_life_and_live_infusion_control_and_removal() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let striker = paid(&mut g, p, "Thornfist Striker", None, &[]);
        settle(&mut g);
        let concocter = paid(&mut g, p, "Tenured Concocter", None, &[]);
        settle(&mut g);
        let bane = paid(&mut g, p, "Shopkeeper's Bane", None, &[]);
        settle(&mut g);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(g.stats(concocter).power, 4);
        assert_eq!(g.stats(bane).power, 4);
        let life = g.state.players[p.index()].life;
        g.state.objects[bane as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: bane });
        checked(&mut g, Action::FinishAttackers);
        original_until(&mut g, |pending| matches!(pending, Pending::Blockers));
        assert_eq!(g.state.players[p.index()].life, life + 2);
        assert_eq!(g.stats(concocter).power, 7);
        assert_eq!(g.stats(bane).power, 5);
        assert!(g.keywords(bane).contains(Keywords::TRAMPLE));
        for player in &mut g.state.players {
            player.mana = [30; 6];
        }
        g.state.step = Step::Main2;
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        paid(
            &mut g,
            p,
            "Harmless Offering",
            None,
            &[Target::Player(p.other()), Target::Card(striker)],
        );
        settle(&mut g);
        assert_eq!(g.stats(bane).power, 4);
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
        g.state.priority = p.other();
        paid(
            &mut g,
            p.other(),
            "Lightning Helix",
            None,
            &[Target::Player(p)],
        );
        settle(&mut g);
        assert_eq!(g.stats(enemy).power, 3);
        assert!(g.keywords(enemy).contains(Keywords::TRAMPLE));
        let aura = paid(
            &mut g,
            p,
            "Witness Protection",
            None,
            &[Target::Card(striker)],
        );
        original_until(&mut g, |pending| {
            matches!(pending, Pending::ResolutionMana { .. })
        });
        let ward = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap();
        checked(&mut g, ward);
        settle(&mut g);
        assert_eq!(g.stats(enemy).power, 2);
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.stats(enemy).power, 3);
        g.finish_cleanup();
        assert_eq!(g.stats(concocter).power, 4);
        assert_eq!(g.stats(enemy).power, 2);
    }
}

#[test]
fn sos_original_bg_entry_death_life_events_and_mascot_count_events_not_amount() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let start = g.state.players[p.index()].life;
        let lumaret = paid(&mut g, p, "Bogwater Lumaret", None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, start + 1);
        let mascot = paid(&mut g, p, "Pest Mascot", None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(mascot).counters.plus_one, 1);
        assert!(g.keywords(mascot).contains(Keywords::TRAMPLE));
        paid(&mut g, p, "Cauldron of Essence", None, &[]);
        settle(&mut g);
        let before = g.state.players[p.index()].life;
        g.state.priority = p.other();
        paid(&mut g, p.other(), "Spectral Sailor", None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, before);
        g.state.priority = p;
        let first = paid(&mut g, p, "Noxious Newt", None, &[]);
        settle(&mut g);
        let second = paid(&mut g, p, "Mindful Biomancer", None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(mascot).counters.plus_one, 4);
        let own = g.state.players[p.index()].life;
        let enemy = g.state.players[p.other().index()].life;
        g.sacrifice_batch(&[(p, first), (p, second)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, own + 2);
        assert_eq!(g.state.players[p.other().index()].life, enemy - 2);
        assert_eq!(g.object_view(mascot).counters.plus_one, 6);
        paid(
            &mut g,
            p,
            "Lightning Helix",
            None,
            &[Target::Player(p.other())],
        );
        settle(&mut g);
        assert_eq!(g.object_view(mascot).counters.plus_one, 7);
        assert!(g.on_battlefield(lumaret));
    }
}
#[test]
fn sos_original_old_growth_educator_checks_life_at_trigger_resolution() {
    {
        let p = Player::P0;
        for late_gain in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Old-Growth Educator", None, &[]); // Resolve only the creature, leaving its ETB trigger pending.
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert!(g.on_battlefield(source));
            g.prepare_triggers();
            if late_gain {
                paid(
                    &mut g,
                    p,
                    "Lightning Helix",
                    None,
                    &[Target::Player(p.other())],
                );
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(source).counters.plus_one,
                if late_gain { 2 } else { 0 }
            );
            assert!(
                g.keywords(source)
                    .contains(Keywords::REACH.union(Keywords::VIGILANCE))
            );
        }
    }
}

#[test]
fn sos_original_teachers_pest_paid_grave_return_tapped_exact_incarnation_and_menace() {
    {
        let p = Player::P0;
        for changed in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Teacher's Pest", None, &[]);
            settle(&mut g);
            g.move_object(source, Zone::Graveyard);
            original_activate(&mut g, source, 0, &[]);
            if changed {
                g.move_object(source, Zone::Hand);
                g.move_object(source, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if changed {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if changed {
                continue;
            }
            assert!(g.object_view(source).tapped);
            assert!(g.keywords(source).contains(Keywords::MENACE));
            g.state.objects[source as usize].tapped = false;
            g.state.objects[source as usize].summoning_sick = false;
            let first = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let second = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let life = g.state.players[p.index()].life;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: source });
            checked(&mut g, Action::FinishAttackers);
            original_until(&mut g, |pending| matches!(pending, Pending::Blockers));
            assert_eq!(g.state.players[p.index()].life, life + 1);
            checked(
                &mut g,
                Action::Block {
                    blocker: first,
                    attacker: source,
                },
            );
            reject(&mut g, Action::FinishBlockers);
            checked(
                &mut g,
                Action::Block {
                    blocker: second,
                    attacker: source,
                },
            );
            checked(&mut g, Action::FinishBlockers);
        }
    }
}

#[test]
fn meltstrider_fight_revalidates_target_and_preserves_attachment_incarnations() {
    {
        let p = Player::P0;
        for change in 0..4 {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let aura = paid(
                &mut g,
                p,
                "Meltstrider's Resolve",
                None,
                &[Target::Card(own)],
            );
            original_until(&mut g, |pending| matches!(pending, Pending::Targets { .. }));
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            match change {
                0 => {
                    g.move_object(enemy, Zone::Graveyard);
                    g.move_object(enemy, Zone::Battlefield);
                }
                1 => {
                    g.move_object(own, Zone::Graveyard);
                    g.move_object(own, Zone::Battlefield);
                }
                2 => {
                    g.state.objects[enemy as usize].controller = p;
                }
                _ => {
                    g.move_object(aura, Zone::Graveyard);
                    g.move_object(aura, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(g.object_view(enemy).damage, if change == 3 { 2 } else { 0 });
        }
    }
}

#[test]
fn starting_town_offers_colorless_and_each_color_for_one_life() {
    {
        let p = Player::P0;
        let mut base = setup(p);
        base.state.players[p.index()].turns_started = 1;
        let town = named(&mut base, p, "Starting Town", Zone::Hand);
        checked(&mut base, Action::PlayLand { card: town });
        for color in 0..6 {
            let mut g = base.clone();
            let mut production = [0; 6];
            production[color] = 1;
            let ability = u16::from(color != 5);
            let action = g
                .legal_actions()
                .into_iter()
                .find(|action| {
                    matches!(action,
                Action::ActivateMana { card, ability: n, production: got, .. }
                    if *card == town && *n == ability && *got == production)
                })
                .unwrap();
            let mana = g.state.players[p.index()].mana[color];
            let life = g.state.players[p.index()].life;
            checked(&mut g, action);
            assert_eq!(g.state.players[p.index()].mana[color], mana + 1);
            assert_eq!(
                g.state.players[p.index()].life,
                life - if color != 5 { 1 } else { 0 }
            );
        }
        let mut no_life = base.clone();
        no_life.state.players[p.index()].life = 0;
        assert!(
            !no_life.legal_actions().iter().any(|action| matches!(action,
            Action::ActivateMana { card, ability: 1, .. } if *card == town))
        );
    }
}

#[test]
fn multiversal_chooses_type_before_competing_entry_replacements() {
    {
        let p = Player::P0;
        for intrinsic_first in [false, true] {
            let mut g = setup(p);
            let spelunking = named(&mut g, p, "Spelunking", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            g.state.objects[land as usize].card = Card::from_name("Multiversal Passage").unwrap();
            checked(&mut g, Action::PlayLand { card: land });
            assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
            checked(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::BasicLandType(crate::cards::BasicLandType::Forest),
                },
            );
            assert!(matches!(g.state.pending, Pending::EntryReplacement { .. }));
            let source = if intrinsic_first {
                g.object_ref(land)
            } else {
                g.object_ref(spelunking)
            };
            checked(
                &mut g,
                Action::ChooseEntryReplacement {
                    source,
                    intrinsic: intrinsic_first,
                },
            );
            assert!(matches!(g.state.pending, Pending::EntryLife { .. }));
            checked(&mut g, Action::ChooseEntryLife { pay: false });
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            assert_eq!(g.object_view(land).tapped, !intrinsic_first);
            assert!(g.has_subtype(land, "Forest"));
        }
    }
}
