//! Keyword-counter chronology and complete paid representative consumers.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &target.into_iter().collect::<Vec<_>>());
    settle(g);
    id
}
fn announce(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
    panic!("unfinished payment");
}
fn activate(g: &mut Game, id: CardId) {
    checked(g, announce(id));
    pay(g);
    settle(g);
}
fn place(g: &mut Game, id: CardId, kind: CounterKind, n: u64) {
    g.effect_counter(id, kind, n, replacements::ResumeMode::Pending);
    settle(g);
}
fn has(g: &Game, id: CardId, keyword: Keywords) -> bool {
    g.object_view(id).keywords.contains(keyword)
}
fn counter_effect(g: &Game, id: CardId, kind: CounterKind) -> Option<KeywordEffectView> {
    g.state
        .roles
        .as_ref()
        .and_then(|s| {
            s.keyword_effects.iter().find(|r| {
                r.recipient == g.object_ref(id) && r.origin == KeywordEffectOrigin::Counter { kind }
            })
        })
        .copied()
}
fn next(g: &mut Game) {
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
fn await_target(g: &mut Game, ability: u16) {
    g.prepare_triggers();
    for _ in 0..50 {
        if g.state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.kind == (StackKind::TriggeredAbility { ability }))
        {
            return;
        }
        next(g);
    }
    panic!("missing trigger{ability} at {:?}", g.state.pending);
}
fn target(g: &mut Game, id: CardId) {
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
}

#[test]
fn keyword_counters_all_eleven_append_stably_and_obey_loss_refresh_partial_and_final_removal() {
    assert_eq!(CounterKind::Invasion as usize, 17);
    assert_eq!(CounterKind::ALL.len(), 29);
    for p in [Player::P0, Player::P1] {
        for (kind, keyword) in [
            (CounterKind::Haste, Keywords::HASTE),
            (CounterKind::Flying, Keywords::FLYING),
            (CounterKind::FirstStrike, Keywords::FIRST_STRIKE),
            (CounterKind::DoubleStrike, Keywords::DOUBLE_STRIKE),
            (CounterKind::Deathtouch, Keywords::DEATHTOUCH),
            (CounterKind::Indestructible, Keywords::INDESTRUCTIBLE),
            (CounterKind::Lifelink, Keywords::LIFELINK),
            (CounterKind::Menace, Keywords::MENACE),
            (CounterKind::Reach, Keywords::REACH),
            (CounterKind::Trample, Keywords::TRAMPLE),
            (CounterKind::Vigilance, Keywords::VIGILANCE),
        ] {
            assert!(kind as usize > CounterKind::Invasion as usize);
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            place(&mut g, bear, kind, 2);
            let original = counter_effect(&g, bear, kind).unwrap();
            assert!(has(&g, bear, keyword));
            let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(bear)));
            assert!(!has(&g, bear, keyword));
            g.pay_counters(bear, &Counters::of(kind, 1));
            assert_eq!(
                counter_effect(&g, bear, kind).unwrap().timestamp,
                original.timestamp
            );
            assert!(!has(&g, bear, keyword));
            place(&mut g, bear, kind, 1);
            assert!(counter_effect(&g, bear, kind).unwrap().timestamp > original.timestamp);
            assert!(has(&g, bear, keyword));
            g.pay_counters(bear, &Counters::of(kind, 1));
            assert!(has(&g, bear, keyword));
            g.pay_counters(bear, &Counters::of(kind, 1));
            assert!(counter_effect(&g, bear, kind).is_none());
            assert!(!has(&g, bear, keyword));
            g.move_object(aura, Zone::Graveyard);
            assert!(!has(&g, bear, keyword));
        }
    }
    let mixed = Counters::of(CounterKind::Flying, 2).with(CounterKind::FirstStrike, 1);
    let wire = serde_json::to_string(&mixed).unwrap();
    assert_eq!(serde_json::from_str::<Counters>(&wire).unwrap(), mixed);
}

#[test]
fn keyword_counters_preserve_ordinary_grants_zero_placement_and_cold_owned_storage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let clone = g.clone();
        let bytes = g.state_bytes();
        place(&mut g, bear, CounterKind::Flying, 1);
        assert!(g.state.objects.ptr_eq(&clone.state.objects));
        assert!(g.state_bytes() > bytes);
        assert!(!has(&clone, bear, Keywords::FLYING));
        let snapshot = g.snapshot();
        g.effect_counter(
            bear,
            CounterKind::Flying,
            0,
            replacements::ResumeMode::Pending,
        );
        assert_eq!(g.snapshot(), snapshot);
        let object = g.object_ref(bear);
        g.grant_keyword_effects(&[bear], object, Keywords::FLYING);
        let ordinary = g
            .state
            .roles
            .as_ref()
            .unwrap()
            .keyword_effects
            .iter()
            .filter(|r| matches!(r.origin, KeywordEffectOrigin::Resolution { .. }))
            .count();
        g.pay_counters(bear, &Counters::of(CounterKind::Flying, 1));
        assert!(has(&g, bear, Keywords::FLYING));
        assert_eq!(
            g.state
                .roles
                .as_ref()
                .unwrap()
                .keyword_effects
                .iter()
                .filter(|r| matches!(r.origin, KeywordEffectOrigin::Resolution { .. }))
                .count(),
            ordinary
        );
        assert!(g.state.named_counters.is_none());
        g.restore(&snapshot).unwrap();
        assert!(counter_effect(&g, bear, CounterKind::Flying).is_some());
        assert!(has(&g, bear, Keywords::FLYING));
    }
}

#[test]
fn keyword_counters_control_exact_incarnation_private_zones_and_copy_do_not_copy_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        place(&mut g, bear, CounterKind::Flying, 1);
        let original = g.object_ref(bear);
        let time = counter_effect(&g, bear, CounterKind::Flying)
            .unwrap()
            .timestamp;
        g.change_controller(original, p.other());
        assert!(has(&g, bear, Keywords::FLYING));
        assert_eq!(
            counter_effect(&g, bear, CounterKind::Flying)
                .unwrap()
                .timestamp,
            time
        );
        let definition = g.state.objects[bear as usize].card;
        let copies = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        );
        g.advance_entries();
        settle(&mut g);
        let copied = copies[0].card;
        assert_eq!(g.counter_count(copied, CounterKind::Flying), 0);
        assert!(!has(&g, copied, Keywords::FLYING));
        g.move_object(bear, Zone::Hand);
        assert!(!has(&g, bear, Keywords::FLYING));
        assert!(
            !g.state
                .roles
                .as_ref()
                .unwrap()
                .keyword_effects
                .iter()
                .any(|r| r.recipient == original)
        );
        g.set_counters(bear, Counters::of(CounterKind::Lifelink, 1));
        assert!(has(&g, bear, Keywords::LIFELINK));
        assert!(
            g.observe(p)
                .keyword_effects
                .iter()
                .any(|r| r.recipient == g.object_ref(bear))
        );
        // A publicly returned card stays known in hand. A fresh hidden card does not.
        assert!(
            g.observe(p.other())
                .keyword_effects
                .iter()
                .any(|r| r.recipient == g.object_ref(bear))
        );
        let hidden = named(&mut g, p, "Bear Cub", Zone::Hand);
        g.set_counters(hidden, Counters::of(CounterKind::Lifelink, 1));
        assert!(
            g.observe(p)
                .keyword_effects
                .iter()
                .any(|r| r.recipient == g.object_ref(hidden))
        );
        assert!(
            !g.observe(p.other())
                .keyword_effects
                .iter()
                .any(|r| r.recipient == g.object_ref(hidden))
        );
        g.move_object(hidden, Zone::Exile);
        g.move_object(bear, Zone::Library);
        g.set_counters(bear, Counters::of(CounterKind::Flying, 1));
        assert!(has(&g, bear, Keywords::FLYING));
        for viewer in [p, p.other()] {
            assert!(
                !g.observe(viewer)
                    .keyword_effects
                    .iter()
                    .any(|r| r.recipient == g.object_ref(bear))
            );
        }
        g.move_object(bear, Zone::Battlefield);
        assert!(!has(&g, bear, Keywords::FLYING));
        assert!(g.state.named_counters.is_none());
    }
}

#[test]
fn keyword_counters_replaced_entry_counts_get_chosen_positive_timestamps_and_survive_cleanup() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Doubling Season", Zone::Battlefield);
            let ids = [
                named(&mut g, p, "Bear Cub", Zone::Hand),
                named(&mut g, p, "Bear Cub", Zone::Hand),
            ];
            let moves = ids
                .into_iter()
                .map(|id| {
                    let mut m = g.entry_move(id, Zone::Battlefield, false);
                    m.effect_counter = Some((CounterKind::Flying, 1));
                    m
                })
                .collect();
            g.stage_entries(moves, None);
            g.advance_entries();
            for _ in 0..20 {
                if matches!(g.state.pending, Pending::EntryTimestamp { .. }) {
                    break;
                }
                next(&mut g);
            }
            let ordered = if reverse { [ids[1], ids[0]] } else { ids };
            for id in ordered {
                let action = Action::ChooseEntryTimestamp {
                    object: g.object_ref(id),
                };
                if g.legal_actions().contains(&action) {
                    checked(&mut g, action);
                }
            }
            settle(&mut g);
            let first = counter_effect(&g, ordered[0], CounterKind::Flying).unwrap();
            let last = counter_effect(&g, ordered[1], CounterKind::Flying).unwrap();
            assert!(first.timestamp > 0 && last.timestamp > first.timestamp);
            for id in ids {
                assert_eq!(g.counter_count(id, CounterKind::Flying), 2);
                assert!(has(&g, id, Keywords::FLYING));
            }
            next_turn(&mut g);
            for id in ids {
                assert!(has(&g, id, Keywords::FLYING));
            }
        }
    }
}

#[test]
fn keyword_counters_captain_paid_flash_discount_full_price_once_control_and_blink() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        checked(&mut g, Action::Pass);
        let captain = paid(&mut g, p, "Captain Marvel, Earth's Protector", None);
        assert_eq!(g.object_view(captain).power, 5);
        assert!(has(
            &g,
            captain,
            Keywords::FLASH
                .union(Keywords::FLYING)
                .union(Keywords::LIFELINK)
        ));
        checked(&mut g, Action::Pass);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, announce(captain));
        g.state.players[p.index()].mana[5] = 2;
        checked(&mut g, announce(captain));
        let request = g.announcement_payment_request(p).unwrap();
        assert_eq!((request.required, request.generic), ([0; 6], 2));
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(captain, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(captain, CounterKind::Indestructible), 1);
        assert!(has(&g, captain, Keywords::INDESTRUCTIBLE));
        checked(&mut g, Action::Pass);
        reject(&mut g, announce(captain));
        let old = g.object_ref(captain);
        g.change_controller(old, p.other());
        g.state.priority = p.other();
        reject(&mut g, announce(captain));
        g.move_object(captain, Zone::Exile);
        g.move_object(captain, Zone::Battlefield);
        g.state.active = p;
        g.state.priority = p;
        next_turn(&mut g);
        next_turn(&mut g);
        g.state.active = p;
        g.state.priority = p;
        assert!(!g.object_view(captain).entered_this_turn);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 7];
        reject(&mut g, announce(captain));
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 5];
        activate(&mut g, captain);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(captain).power, 6);
        assert_eq!(g.object_view(captain).power_up_uses[0].count, 1);
    }
}

#[test]
fn keyword_counters_captain_actual_flying_lifelink_lethal_damage_and_destroy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let captain = paid(&mut g, p, "Captain Marvel, Earth's Protector", None);
        activate(&mut g, captain);
        checked(&mut g, Action::Pass);
        paid(&mut g, p.other(), "Murder", Some(Target::Card(captain)));
        assert_eq!(g.object_view(captain).zone, Zone::Battlefield);
        let dealer = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = g.damage_source(dealer);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Card(captain),
            amount: 99,
            combat: false,
            preventable: true,
        }]);
        g.state_based_actions();
        assert_eq!(g.object_view(captain).zone, Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[captain as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: captain });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..10 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: captain,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        for _ in 0..20 {
            if g.state.players[p.index()].life == 26 {
                break;
            }
            next(&mut g);
        }
        assert_eq!(g.state.players[p.index()].life, 26);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}

#[test]
fn keyword_counters_indestructible_does_not_stop_paid_sacrifice_or_zero_toughness() {
    for p in [Player::P0, Player::P1] {
        for sacrifice in [false, true] {
            let mut g = setup(p);
            let captain = paid(&mut g, p, "Captain Marvel, Earth's Protector", None);
            activate(&mut g, captain);
            if sacrifice {
                let spell = named(&mut g, p, "Corrupted Conviction", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: spell });
                for _ in 0..10 {
                    let action = Action::ChooseAdditionalCost {
                        object: g.object_ref(captain),
                    };
                    if g.legal_actions().contains(&action) {
                        checked(&mut g, action);
                        break;
                    }
                    let action = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| {
                            matches!(
                                a,
                                Action::PayCosts { .. }
                                    | Action::FinishTargets
                                    | Action::ChooseCastCost { .. }
                            )
                        })
                        .unwrap();
                    checked(&mut g, action);
                }
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            } else {
                checked(&mut g, Action::Pass);
                paid(
                    &mut g,
                    p.other(),
                    "Syphon Fuel",
                    Some(Target::Card(captain)),
                );
            }
            assert_eq!(g.object_view(captain).zone, Zone::Graveyard);
            assert_eq!(g.counter_count(captain, CounterKind::Indestructible), 0);
            assert!(counter_effect(&g, captain, CounterKind::Indestructible).is_none());
        }
    }
}

#[test]
fn keyword_counters_claim_paid_landfall_invalid_targets_then_distinct_reflexive_after_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = setup(p);
            let claim = paid(&mut g, p, "Claim the Kingdom", None);
            let first = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if exile {
                named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            place(&mut g, claim, CounterKind::Plan, 3);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: land });
            await_target(&mut g, 0);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(claim),
                },
            );
            reject(&mut g, Action::FinishTargets);
            target(&mut g, first);
            await_target(&mut g, 2);
            assert_eq!(
                g.object_view(claim).zone,
                if exile { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(g.counter_count(first, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(second, CounterKind::Indestructible), 0);
            assert!(g.state.announcing.as_ref().unwrap().item.targets.is_empty());
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            target(&mut g, second);
            settle(&mut g);
            assert!(has(&g, second, Keywords::INDESTRUCTIBLE));
            assert_eq!(g.counter_count(second, CounterKind::Indestructible), 1);
            assert_eq!(g.counter_count(first, CounterKind::Indestructible), 0);
        }
    }
}

#[test]
fn keyword_counters_claim_all_illegal_landfall_and_departed_source_never_rebind() {
    for p in [Player::P0, Player::P1] {
        for source_left in [false, true] {
            let mut g = setup(p);
            let claim = paid(&mut g, p, "Claim the Kingdom", None);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: land });
            await_target(&mut g, 0);
            target(&mut g, bear);
            if source_left {
                g.move_object(claim, Zone::Exile);
                g.move_object(claim, Zone::Battlefield);
            } else {
                g.move_object(bear, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(claim, CounterKind::Plan), 0);
            assert_eq!(
                g.counter_count(bear, CounterKind::PlusOne),
                u64::from(source_left)
            );
        }
    }
}

#[test]
fn keyword_counters_claim_historical_crossing_and_failed_exact_sacrifice_or_reflexive_target() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let claim = paid(&mut g, p, "Claim the Kingdom", None);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.effect_counter(
                claim,
                CounterKind::Plan,
                5,
                replacements::ResumeMode::Pending,
            );
            g.prepare_triggers();
            if mode == 0 {
                g.set_counters(claim, Counters::default());
            }
            if mode == 1 {
                g.change_controller(g.object_ref(claim), p.other());
                settle(&mut g);
                assert_eq!(g.object_view(claim).zone, Zone::Battlefield);
                continue;
            }
            if mode == 2 {
                g.move_object(claim, Zone::Exile);
                g.move_object(claim, Zone::Battlefield);
                settle(&mut g);
                assert_eq!(g.counter_count(bear, CounterKind::Indestructible), 0);
                continue;
            }
            await_target(&mut g, 2);
            target(&mut g, bear);
            if mode == 3 {
                g.change_controller(g.object_ref(bear), p.other());
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(bear, CounterKind::Indestructible),
                u64::from(mode != 3)
            );
            assert_eq!(g.object_view(claim).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn keyword_counters_captain_failed_mixed_placement_rolls_back_hot_counters_and_timeline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let captain = paid(&mut g, p, "Captain Marvel, Earth's Protector", None);
        g.set_counters(captain, Counters::of(CounterKind::Indestructible, u64::MAX));
        checked(&mut g, announce(captain));
        pay(&mut g);
        let mut failed = false;
        for _ in 0..8 {
            let before = g.snapshot();
            let revision = g.revision();
            match g.apply(g.actor().unwrap(), Action::Pass) {
                Ok(()) => (),
                Err(Error::CapacityExceeded(_)) => {
                    assert_eq!(g.snapshot(), before);
                    assert_eq!(g.revision(), revision);
                    assert_eq!(g.counter_count(captain, CounterKind::PlusOne), 0);
                    failed = true;
                    break;
                }
                other => panic!("unexpected resolution result {other:?}"),
            }
        }
        assert!(failed);
    }
}

#[test]
fn keyword_counters_actual_haste_funding_and_double_strike_vigilance_lifelink_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[elf as usize].summoning_sick = true;
        g.state.players[p.index()].mana = [0; 6];
        let spell = named(&mut g, p, "Snakeskin Veil", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::TapMana { card: elf });
        place(&mut g, elf, CounterKind::Haste, 1);
        // Legal announcement with an empty pool requires the speculative funding
        // world to recognize the counter's haste on this sick mana creature.
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        checked(&mut g, Action::TapMana { card: elf });
        cast(&mut g, spell, None, &[Target::Card(elf)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(elf).tapped);
        assert_eq!(g.counter_count(elf, CounterKind::PlusOne), 1);

        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].summoning_sick = true;
        for kind in [
            CounterKind::Haste,
            CounterKind::Vigilance,
            CounterKind::DoubleStrike,
            CounterKind::Lifelink,
        ] {
            place(&mut g, bear, kind, 1);
        }
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: bear });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(bear).tapped);
        for _ in 0..10 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        checked(&mut g, Action::FinishBlockers);
        for _ in 0..30 {
            if g.state.players[p.other().index()].life == 16 {
                break;
            }
            next(&mut g);
        }
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert!(!g.object_view(bear).tapped);
    }
}
