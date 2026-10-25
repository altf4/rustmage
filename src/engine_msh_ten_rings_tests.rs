//! Paid whole-card and cross-card rules for finite/infinite hand limits.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const RINGS: &str = "The Ten Rings";
const NIV: &str = "Niv-Mizzet, Visionary";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = g.object_view(id).card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, targets);
    settle(g);
    id
}
fn fill(g: &mut Game, p: Player, n: usize) {
    assert!(g.hand_count(p) <= n);
    for _ in g.hand_count(p)..n {
        named(g, p, "Forest", Zone::Hand);
    }
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::FinishTargets
                        | Action::PutTrigger { .. }
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .expect("hand rules continuation")
            .clone();
        checked(g, a);
    }
    panic!("hand rules stalled {:?}", g.state.pending);
}
fn end(g: &mut Game) {
    until(g, |g| g.state.step == Step::End);
}
fn time(g: &Game, id: CardId) -> u64 {
    g.state
        .hand_size
        .as_ref()
        .unwrap()
        .sources
        .iter()
        .find(|v| v.source == g.object_ref(id))
        .unwrap()
        .timestamp
}
fn packet(g: &Game, id: CardId) -> bool {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .any(|v| v.card == id)
}
fn move_to(g: &mut Game, id: CardId, zone: Zone) {
    g.move_object(id, zone);
}
fn discard_all(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Discard {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("cleanup discard stalled");
}
#[test]
fn msh_ten_rings_paid_eight_body_and_atomic_illegal_announcements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = named(&mut g, p, RINGS, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 7];
        reject(&mut g, Action::AnnounceCast { card: r });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
        cast(&mut g, r, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.hand_count(p), 0);
        assert!(g.on_battlefield(r));
        assert!(!g.is_creature(r));
        assert_eq!(g.maximum_hand_size(p), Some(10));
        assert_eq!(g.maximum_hand_size(p.other()), Some(7));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        let second = named(&mut g, p, RINGS, Zone::Hand);
        g.state.players[p.index()].mana = [8; 6];
        g.state.active = p.other();
        reject(&mut g, Action::AnnounceCast { card: second });
    }
}
#[test]
fn msh_ten_rings_actual_land_funding_consumes_eight_sources() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let r = named(&mut g, p, RINGS, Zone::Hand);
        let lands: Vec<_> = (0..8)
            .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
            .collect();
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: r })
        );
        for &card in &lands {
            checked(&mut g, Action::TapMana { card });
        }
        cast(&mut g, r, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(lands.iter().all(|&id| g.object_view(id).tapped));
        assert_eq!(g.maximum_hand_size(p), Some(10));
    }
}
#[test]
fn msh_ten_rings_paid_second_copy_legend_cleanup_keeps_exact_provider() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = paid(&mut g, p, RINGS, &[]);
        let id = named(&mut g, p, RINGS, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
        cast(&mut g, id, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        let object = g.object_ref(id);
        checked(&mut g, Action::KeepLegend { object });
        settle(&mut g);
        assert!(!g.on_battlefield(old));
        assert!(g.on_battlefield(id));
        assert_eq!(g.state.hand_size.as_ref().unwrap().sources.len(), 1);
    }
}
#[test]
fn msh_ten_rings_end_step_boundaries_and_opponent_step() {
    for p in [Player::P0, Player::P1] {
        for n in [0, 1, 9, 10, 11] {
            let mut g = setup(p);
            let r = paid(&mut g, p, RINGS, &[]);
            fill(&mut g, p, n);
            end(&mut g);
            assert_eq!(packet(&g, r), n < 10);
            settle(&mut g);
            assert_eq!(g.hand_count(p), n.max(10));
            let mut g = setup(p);
            let r = named(&mut g, p.other(), RINGS, Zone::Battlefield);
            end(&mut g);
            assert!(!packet(&g, r));
        }
    }
}
#[test]
fn msh_ten_rings_intervening_hand_growth_cancels_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = paid(&mut g, p, RINGS, &[]);
        fill(&mut g, p, 9);
        end(&mut g);
        assert!(packet(&g, r));
        paid(&mut g, p, "Quick Study", &[]);
        assert_eq!(g.hand_count(p), 11);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
#[test]
fn msh_ten_rings_resolution_resamples_spent_card_and_no_late_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = paid(&mut g, p, RINGS, &[]);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        fill(&mut g, p, 9);
        end(&mut g);
        assert!(packet(&g, r));
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 10);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let mut g = setup(p);
        let r = paid(&mut g, p, RINGS, &[]);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        fill(&mut g, p, 10);
        end(&mut g);
        assert!(!packet(&g, r));
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 9);
    }
}
#[test]
fn msh_ten_rings_packet_controller_survives_departure_suppression_and_theft() {
    for p in [Player::P0, Player::P1] {
        for kind in 0..4 {
            let mut g = setup(p);
            let r = paid(&mut g, p, RINGS, &[]);
            fill(&mut g, p, 5);
            end(&mut g);
            assert!(packet(&g, r));
            match kind {
                0 => {
                    paid(&mut g, p, "Disenchant", &[Target::Card(r)]);
                }
                1 => {
                    g.change_controller(g.object_ref(r), p.other());
                    settle(&mut g);
                }
                2 => {
                    paid(&mut g, p, "Stop Cold", &[Target::Card(r)]);
                }
                _ => {
                    move_to(&mut g, r, Zone::Hand);
                    settle(&mut g);
                }
            }
            assert_eq!(g.hand_count(p), 10);
            assert_eq!(g.hand_count(p.other()), 0);
            assert_eq!(g.maximum_hand_size(p), Some(7));
            assert_eq!(
                g.maximum_hand_size(p.other()),
                if kind == 1 { Some(10) } else { Some(7) }
            );
        }
    }
}
#[test]
fn msh_ten_rings_no_retroactive_end_step_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        end(&mut g);
        let r = named(&mut g, p, RINGS, Zone::Graveyard);
        move_to(&mut g, r, Zone::Battlefield);
        settle(&mut g);
        assert!(!packet(&g, r));
        assert_eq!(g.hand_count(p), 0);
    }
}
#[test]
fn msh_ten_rings_cleanup_ten_eleven_thirteen_and_inactive_hand() {
    for p in [Player::P0, Player::P1] {
        for n in [10, 11, 13] {
            let mut g = setup(p);
            paid(&mut g, p, RINGS, &[]);
            fill(&mut g, p, n);
            fill(&mut g, p.other(), 13);
            let turn = g.state.turn;
            assert_eq!(g.state.step, Step::Main1);
            until(&mut g, |g| {
                g.state.pending == Pending::Discard || g.state.turn != turn
            });
            if n > 10 {
                assert_eq!(g.state.step, Step::Cleanup);
                discard_all(&mut g);
            }
            assert_eq!(g.hand_count(p), 10);
            assert_eq!(g.hand_count(p.other()), 13);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 0); // reset only when actual turn advances
        }
    }
}
#[test]
fn msh_ten_rings_cleanup_illegal_choices_are_atomic_and_private() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, RINGS, &[]);
        fill(&mut g, p, 13);
        let foreign = named(&mut g, p.other(), "Murder", Zone::Hand);
        let grave = named(&mut g, p, "Murder", Zone::Graveyard);
        until(&mut g, |g| g.state.pending == Pending::Discard);
        reject(&mut g, Action::Discard { card: foreign });
        reject(&mut g, Action::Discard { card: grave });
        reject(&mut g, Action::Pass);
        let mut alternative = g.clone();
        let card = g
            .legal_actions()
            .into_iter()
            .find_map(|a| {
                if let Action::Discard { card } = a {
                    Some(card)
                } else {
                    None
                }
            })
            .unwrap();
        let other = alternative
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { card: c } if *c != card))
            .unwrap();
        checked(&mut alternative, other);
        checked(&mut g, Action::Discard { card });
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(alternative.observe(p.other())).unwrap()
        );
        reject(&mut g, Action::Discard { card });
        discard_all(&mut g);
        assert_eq!(g.hand_count(p), 10);
    }
}
#[test]
fn msh_ten_rings_static_infinity_timestamp_order_and_fallback() {
    for p in [Player::P0, Player::P1] {
        for later in [false, true] {
            let mut g = setup(p);
            let (r, n) = if later {
                let r = paid(&mut g, p, RINGS, &[]);
                let n = paid(&mut g, p, NIV, &[]);
                (r, n)
            } else {
                let n = paid(&mut g, p, NIV, &[]);
                let r = paid(&mut g, p, RINGS, &[]);
                (r, n)
            };
            assert_eq!(g.maximum_hand_size(p), if later { None } else { Some(10) });
            move_to(&mut g, if later { n } else { r }, Zone::Exile);
            assert_eq!(g.maximum_hand_size(p), if later { Some(10) } else { None });
        }
    }
}
#[test]
fn msh_ten_rings_lasting_infinity_order_and_repeat_real_wisdom() {
    for p in [Player::P0, Player::P1] {
        for later in [false, true] {
            let mut g = setup(p);
            let r = if later {
                let r = paid(&mut g, p, RINGS, &[]);
                paid(&mut g, p, "Wisdom of Ages", &[]);
                r
            } else {
                paid(&mut g, p, "Wisdom of Ages", &[]);
                paid(&mut g, p, RINGS, &[])
            };
            assert_eq!(g.maximum_hand_size(p), if later { None } else { Some(10) });
            let old = g.state.hand_size.as_ref().unwrap().lasting[p.index()].unwrap();
            paid(&mut g, p, "Wisdom of Ages", &[]);
            assert!(g.state.hand_size.as_ref().unwrap().lasting[p.index()].unwrap() > old);
            assert_eq!(g.maximum_hand_size(p), None);
            move_to(&mut g, r, Zone::Graveyard);
            assert_eq!(g.maximum_hand_size(p), None);
        }
    }
}
#[test]
fn msh_ten_rings_finale_x_ten_creates_later_lasting_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, RINGS, &[]);
        let f = named(&mut g, p, "Finale of Revelation", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 10];
        checked(&mut g, Action::AnnounceCast { card: f });
        checked(&mut g, Action::ChooseX { value: 10 });
        for _ in 0..8 {
            if g.state.announcing.is_none() {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.maximum_hand_size(p), None);
        assert_eq!(g.state.objects[f as usize].zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 10);
    }
}
#[test]
fn msh_ten_rings_control_transfer_keeps_timestamp_blink_gets_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = paid(&mut g, p, NIV, &[]);
        let r = paid(&mut g, p, RINGS, &[]);
        let rt = time(&g, r);
        let nt = time(&g, n);
        g.change_controller(g.object_ref(n), p.other());
        g.change_controller(g.object_ref(r), p.other());
        assert_eq!(g.maximum_hand_size(p.other()), Some(10));
        assert_eq!(time(&g, r), rt);
        assert_eq!(time(&g, n), nt);
        g.change_controller(g.object_ref(n), p);
        g.change_controller(g.object_ref(r), p);
        assert_eq!(g.maximum_hand_size(p), Some(10));
        let old = g.object_ref(n);
        move_to(&mut g, n, Zone::Hand);
        move_to(&mut g, n, Zone::Battlefield);
        assert_ne!(g.object_ref(n), old);
        assert!(time(&g, n) > rt);
        assert_eq!(g.maximum_hand_size(p), None);
    }
}
#[test]
fn msh_ten_rings_simultaneous_entries_choose_both_orders_owned_and_private() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let r = named(&mut g, p, RINGS, Zone::Hand);
            let n = named(&mut g, p, NIV, Zone::Hand);
            let moves = [r, n]
                .map(|id| g.entry_move(id, Zone::Battlefield, false))
                .to_vec();
            g.stage_entries(moves, None);
            g.advance_entries();
            assert!(matches!(g.state.pending, Pending::EntryTimestamp { .. }));
            assert!(g.observe(p.other()).entry_timestamps.is_none());
            assert!(g.observe(p.other()).hand_size_timeline.is_none());
            let first = if reverse { n } else { r };
            let object = g.object_ref(first);
            checked(&mut g, Action::ChooseEntryTimestamp { object });
            settle(&mut g);
            assert_eq!(
                g.maximum_hand_size(p),
                if reverse { Some(10) } else { None }
            );
            assert!(g.observe(p).ability_source_timestamps.is_empty());
        }
    }
}
#[test]
fn msh_ten_rings_simultaneous_apnap_then_theft_uses_original_times() {
    for p in [Player::P0, Player::P1] {
        for active_rings in [false, true] {
            let mut g = setup(p);
            let r = named(
                &mut g,
                if active_rings { p } else { p.other() },
                RINGS,
                Zone::Graveyard,
            );
            let n = named(
                &mut g,
                if active_rings { p.other() } else { p },
                NIV,
                Zone::Graveyard,
            );
            let moves = [n, r]
                .map(|id| g.entry_move(id, Zone::Battlefield, false))
                .to_vec();
            g.stage_entries(moves, None);
            g.advance_entries();
            settle(&mut g);
            g.change_controller(g.object_ref(r), p);
            g.change_controller(g.object_ref(n), p);
            assert_eq!(
                g.maximum_hand_size(p),
                if active_rings { None } else { Some(10) }
            );
        }
    }
}
#[test]
fn msh_ten_rings_old_infinity_only_rows_do_not_enter_public_ability_clock() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = paid(&mut g, p, NIV, &[]);
        assert!(g.observe(p).ability_source_timestamps.is_empty());
        assert!(g.observe(p).hand_size_timeline.is_none());
        assert!(g.state.roles.is_none());
        paid(&mut g, p, "Wisdom of Ages", &[]);
        assert!(g.state.roles.is_none());
        assert!(g.observe(p).hand_size_timeline.is_none());
        let old = time(&g, n);
        paid(&mut g, p, RINGS, &[]);
        let timeline = g.observe(p).hand_size_timeline.unwrap();
        assert_eq!(
            timeline
                .sources
                .iter()
                .find(|v| v.source == g.object_ref(n))
                .unwrap()
                .timestamp,
            old
        );
        assert!(timeline.lasting[p.index()].is_some());
    }
}
#[test]
fn msh_ten_rings_actual_suppression_restoration_does_not_retimestamp() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = paid(&mut g, p, NIV, &[]);
        let r = paid(&mut g, p, RINGS, &[]);
        let old = time(&g, r);
        let cold = paid(&mut g, p, "Stop Cold", &[Target::Card(r)]);
        assert_eq!(g.maximum_hand_size(p), None);
        assert_eq!(time(&g, r), old);
        paid(&mut g, p, "Disenchant", &[Target::Card(cold)]);
        assert_eq!(g.maximum_hand_size(p), Some(10));
        assert_eq!(time(&g, r), old);
        let cold = paid(&mut g, p, "Stop Cold", &[Target::Card(n)]);
        move_to(&mut g, r, Zone::Exile);
        assert_eq!(g.maximum_hand_size(p), Some(7));
        move_to(&mut g, cold, Zone::Graveyard);
        assert_eq!(g.maximum_hand_size(p), None);
    }
}
#[test]
fn msh_ten_rings_time_stop_skips_end_trigger_but_discards_to_ten() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, RINGS, &[]);
        fill(&mut g, p, 13);
        let t = named(&mut g, p, "Time Stop", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        cast(&mut g, t, None, &[]);
        until(&mut g, |g| g.state.pending == Pending::Discard);
        assert_eq!(g.state.step, Step::Cleanup);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        discard_all(&mut g);
        assert_eq!(g.hand_count(p), 10);
    }
}
#[test]
fn msh_ten_rings_repeated_cleanup_recomputes_after_actual_discard_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let r = paid(&mut g, p, RINGS, &[]);
        let mako = paid(&mut g, p, "Marauding Mako", &[]);
        fill(&mut g, p, 11);
        until(&mut g, |g| g.state.pending == Pending::Discard);
        discard_all(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 10);
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 1);
        assert_eq!(g.state.step, Step::Cleanup);
        paid(&mut g, p, "Disenchant", &[Target::Card(r)]);
        assert_eq!(g.maximum_hand_size(p), Some(7));
        until(&mut g, |g| g.state.pending == Pending::Discard);
        discard_all(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 7);
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 4);
    }
}
#[test]
fn msh_ten_rings_interruptible_first_draw_freezes_deficit_and_hides_eligibility() {
    for p in [Player::P0, Player::P1] {
        let mut public = None;
        for eligible in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, RINGS, &[]);
            named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
            fill(&mut g, p, 5);
            let top = *g.state.players[p.index()].library.last().unwrap();
            g.state.objects[top as usize].card =
                Card::from_name(if eligible { "Shock" } else { "Forest" }).unwrap();
            end(&mut g);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::MiracleReveal { .. })
            });
            assert_eq!(g.hand_count(p), 6);
            assert!(g.observe(p.other()).miracle.is_none());
            let view = serde_json::to_value(g.observe(p.other())).unwrap();
            if let Some(expected) = &public {
                assert_eq!(&view, expected);
            } else {
                public = Some(view);
            }
            assert_eq!(g.observe(p).miracle.unwrap().can_reveal, eligible);
            reject(&mut g, Action::ChooseX { value: 99 });
            checked(&mut g, Action::DeclineMiracle);
            settle(&mut g);
            assert_eq!(g.hand_count(p), 10);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
        }
    }
}
#[test]
fn msh_ten_rings_partial_library_uses_successful_ordinals_and_empty_draw_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, RINGS, &[]);
        fill(&mut g, p, 5);
        let ids = g.state.players[p.index()].library.clone();
        for &id in ids.iter().skip(3) {
            move_to(&mut g, id, Zone::Exile);
        }
        end(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 8);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}
#[test]
fn msh_ten_rings_owned_orphan_trigger_and_lasting_rows_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Wisdom of Ages", &[]);
        let r = paid(&mut g, p, RINGS, &[]);
        fill(&mut g, p, 5);
        end(&mut g);
        let before = g.snapshot();
        let mut orphan = {
            let parent = g.clone();
            parent.clone()
        };
        move_to(&mut orphan, r, Zone::Exile);
        assert_eq!(orphan.maximum_hand_size(p), None);
        assert_eq!(g.maximum_hand_size(p), Some(10));
        assert_eq!(g.snapshot(), before);
        let departure = orphan.snapshot();
        settle(&mut orphan);
        let after = orphan.snapshot();
        orphan.restore(&departure).unwrap();
        settle(&mut orphan);
        assert_eq!(orphan.snapshot(), after);
        assert_eq!(orphan.hand_count(p), 10);
    }
}
#[test]
fn msh_ten_rings_owned_cleanup_selection_nonactor_privacy_and_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, RINGS, &[]);
        fill(&mut g, p, 12);
        let secret = named(&mut g, p, "Shock", Zone::Hand);
        until(&mut g, |g| g.state.pending == Pending::Discard);
        let before = g.snapshot();
        let mut different = g.clone();
        different.state.objects[secret as usize].card = Card::from_name("Murder").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(different.observe(p.other())).unwrap()
        );
        let mut orphan = {
            let parent = g.clone();
            parent.clone()
        };
        checked(&mut orphan, Action::Discard { card: secret });
        let selected = orphan.snapshot();
        discard_all(&mut orphan);
        let after = orphan.snapshot();
        orphan.restore(&selected).unwrap();
        discard_all(&mut orphan);
        assert_eq!(orphan.snapshot(), after);
        assert_eq!(g.snapshot(), before);
        assert!(g.state_bytes() > std::mem::size_of::<State>());
    }
}
