//! Paid printed-clause scenarios through ordinary supported card definitions.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
const BROKER: &str = "Powerful Broker";
const THRALL: &str = "Evil's Thrall";
pub(super) fn seed(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    named(g, p, name, zone)
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
pub(super) fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = seed(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
pub(super) fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..200 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn own_main(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    priority(g, p);
}
fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
}
fn activation(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn activate(g: &mut Game, p: Player, id: CardId, target: Target) {
    priority(g, p);
    checked(g, activation(id));
    checked(g, Action::ChooseTarget { target });
    finish(g);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let bytes = g.state_bytes();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    owned_checked_step(g, actor, index, a)
}
fn controls(g: &Game, id: CardId, p: Player) {
    assert_eq!(g.object_view(id).controller, p);
}
#[test]
fn msh_broker_thrall_broker_paid_body_tap_sickness_sorcery_and_stack_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let broker = paid(&mut g, p, BROKER, &[]);
        assert_eq!(
            g.stats(broker),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        reject(&mut g, activation(broker));
        own_main(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        activate(&mut g, p, broker, Target::Player(p));
        settle(&mut g);
        assert!(g.object_view(broker).tapped);
        assert_eq!(g.state.players[p.index()].poison_counters, 0);
        reject(&mut g, activation(broker));
        own_main(&mut g, p);
        packet(&mut g, p, "Shock", &[Target::Player(p.other())]);
        priority(&mut g, p);
        reject(&mut g, activation(broker));
        settle(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other()
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
        });
        priority(&mut g, p);
        reject(&mut g, activation(broker));
    }
}
#[test]
fn msh_broker_thrall_broker_each_hot_cold_keyword_kind_adds_one_not_current_quantity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let broker = paid(&mut g, p, BROKER, &[]);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        // Every represented permanent kind is legal even on a land; these counters are a bounded precondition.
        let mut counters = Counters::default();
        for kind in CounterKind::ALL {
            if kind != CounterKind::MinusOne {
                counters = counters.with(kind, 3);
            }
        }
        g.set_counters(land, counters);
        own_main(&mut g, p);
        activate(&mut g, p, broker, Target::Card(land));
        settle(&mut g);
        for kind in CounterKind::ALL {
            assert_eq!(
                g.counter_count(land, kind),
                if kind == CounterKind::MinusOne { 0 } else { 4 },
                "{kind:?}"
            );
        }
        assert_eq!(g.counter_count(land, CounterKind::MinusOne), 0);
        own_main(&mut g, p);
        g.set_counters(land, Counters::of(CounterKind::MinusOne, 7));
        activate(&mut g, p, broker, Target::Card(land));
        settle(&mut g);
        assert_eq!(g.counter_count(land, CounterKind::MinusOne), 8);
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_broker_thrall_broker_empty_artifact_and_enchantment_are_legal_no_op_targets() {
    for p in [Player::P0, Player::P1] {
        for name in ["Fishing Pole", "Doubling Season", "Forest"] {
            let mut g = setup(p);
            let broker = paid(&mut g, p, BROKER, &[]);
            let target = named(&mut g, p.other(), name, Zone::Battlefield);
            own_main(&mut g, p);
            activate(&mut g, p, broker, Target::Card(target));
            settle(&mut g);
            assert_eq!(g.object_view(target).counters, Counters::default());
        }
    }
}
#[test]
fn msh_broker_thrall_broker_player_poison_counts_increment_one_and_ten_loses() {
    for p in [Player::P0, Player::P1] {
        for (count, other) in [(0, false), (2, false), (5, true), (9, true)] {
            let mut g = setup(p);
            let broker = paid(&mut g, p, BROKER, &[]);
            own_main(&mut g, p);
            let target = if other { p.other() } else { p };
            g.state.players[target.index()].poison_counters = count;
            activate(&mut g, p, broker, Target::Player(target));
            settle(&mut g);
            assert_eq!(
                g.state.players[target.index()].poison_counters,
                if count == 0 { 0 } else { count + 1 }
            );
            assert_eq!(g.outcome().is_some(), count == 9);
        }
    }
}
#[test]
fn msh_broker_thrall_broker_source_departure_and_replaced_counter_kind_set_are_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let broker = paid(&mut g, p, BROKER, &[]);
        let target = paid(&mut g, p, "Fishing Pole", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        g.set_counters(
            target,
            Counters::of(CounterKind::Charge, 3).with(CounterKind::Flying, 2),
        );
        own_main(&mut g, p);
        activate(&mut g, p, broker, Target::Card(target));
        packet(&mut g, p.other(), "Murder", &[Target::Card(broker)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.object_view(broker).zone, Zone::Graveyard);
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        g = orphan(g, choice);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::Charge), 7);
        assert_eq!(g.counter_count(target, CounterKind::Flying), 6);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_broker_thrall_broker_live_counter_set_snapshot_and_stale_target_incarnation() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = setup(p);
            let broker = paid(&mut g, p, BROKER, &[]);
            let target = paid(&mut g, p, "Fishing Pole", &[]);
            g.set_counters(target, Counters::of(CounterKind::Charge, 5));
            own_main(&mut g, p);
            activate(&mut g, p, broker, Target::Card(target));
            if stale {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            } // Explicit exact-incarnation core boundary.
            g.set_counters(target, Counters::of(CounterKind::Soul, 2));
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::Charge), 0);
            assert_eq!(
                g.counter_count(target, CounterKind::Soul),
                if stale { 2 } else { 3 }
            );
        }
    }
}
#[test]
fn msh_broker_thrall_broker_illegal_targets_and_hexproof_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let broker = paid(&mut g, p, BROKER, &[]);
        let hand = named(&mut g, p.other(), "Shock", Zone::Hand);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        own_main(&mut g, p);
        packet(&mut g, p.other(), "Shore Up", &[Target::Card(bear)]);
        settle(&mut g);
        priority(&mut g, p);
        checked(&mut g, activation(broker));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(hand),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        reject(&mut g, Action::FinishTargets);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(broker).tapped);
    }
}
#[test]
fn msh_broker_thrall_thrall_paid_short_control_untaps_haste_allows_attack_then_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[target as usize].tapped = true;
        paid(&mut g, p, THRALL, &[Target::Card(target)]);
        controls(&g, target, p);
        assert!(!g.object_view(target).tapped);
        assert!(g.keywords(target).contains(Keywords::HASTE));
        assert_eq!(
            g.observe(p).temporary_controls[0].expires_turn,
            Some(g.state.turn)
        );
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        g = orphan(g, Action::Attack { card: target });
        checked(&mut g, Action::FinishAttackers);
        next_turn(&mut g);
        controls(&g, target, p.other());
        assert!(!g.keywords(target).contains(Keywords::HASTE));
        assert!(g.observe(p).temporary_controls.is_empty());
    }
}
#[test]
fn msh_broker_thrall_thrall_greater_villain_locks_control_through_next_own_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let villain = paid(&mut g, p, BROKER, &[]);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[target as usize].tapped = true;
        paid(&mut g, p, THRALL, &[Target::Card(target)]);
        controls(&g, target, p);
        let row = g.observe(p).temporary_controls[0];
        assert_eq!(row.expires_turn, None);
        assert_eq!(
            row.expires_after_next_turn,
            Some(NextTurnControlExpiry {
                player: p,
                created_turn: g.state.turn
            })
        );
        assert_eq!(
            serde_json::to_value(g.observe(p).temporary_controls).unwrap(),
            serde_json::to_value(g.observe(p.other()).temporary_controls).unwrap()
        );
        packet(&mut g, p.other(), "Murder", &[Target::Card(villain)]);
        settle(&mut g);
        next_turn(&mut g);
        controls(&g, target, p);
        assert!(!g.keywords(target).contains(Keywords::HASTE));
        next_turn(&mut g);
        controls(&g, target, p);
        next_turn(&mut g);
        controls(&g, target, p.other());
        assert!(g.observe(p).temporary_controls.is_empty());
    }
}
#[test]
fn msh_broker_thrall_thrall_equal_lower_foreign_and_suppressed_villain_do_not_extend() {
    for p in [Player::P0, Player::P1] {
        for variant in 0..4 {
            let mut g = setup(p);
            let vp = if variant == 2 { p.other() } else { p };
            let villain = seed(&mut g, vp, "Powerful Broker", Zone::Battlefield);
            if variant == 3 {
                paid(&mut g, p, "Witness Protection", &[Target::Card(villain)]);
            }
            let target = seed(
                &mut g,
                p.other(),
                if variant == 0 {
                    "Powerful Broker"
                } else if variant == 1 {
                    "Hill Giant"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            paid(&mut g, p, THRALL, &[Target::Card(target)]);
            controls(&g, target, p);
            assert_eq!(
                g.observe(p).temporary_controls[0].expires_turn,
                Some(g.state.turn)
            );
            next_turn(&mut g);
            controls(&g, target, p.other());
        }
    }
}
#[test]
fn msh_broker_thrall_thrall_condition_is_live_at_resolution_not_cast_and_target_itself_not_greater()
{
    for p in [Player::P0, Player::P1] {
        for before in [false, true] {
            let mut g = setup(p);
            let villain = paid(&mut g, p, BROKER, &[]);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            packet(&mut g, p, THRALL, &[Target::Card(target)]);
            if before {
                packet(&mut g, p.other(), "Murder", &[Target::Card(villain)]);
            } else {
                packet(&mut g, p, "Shore Up", &[Target::Card(villain)]);
            }
            settle(&mut g);
            assert_eq!(
                g.observe(p).temporary_controls[0]
                    .expires_after_next_turn
                    .is_some(),
                !before
            );
        }
    }
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = paid(&mut g, p, BROKER, &[]);
        paid(&mut g, p, THRALL, &[Target::Card(own)]);
        assert_eq!(
            g.observe(p).temporary_controls[0].expires_turn,
            Some(g.state.turn)
        );
    }
}
#[test]
fn msh_broker_thrall_thrall_illegal_target_on_resolution_has_no_control_or_haste() {
    for p in [Player::P0, Player::P1] {
        for protection in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, BROKER, &[]);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            packet(&mut g, p, THRALL, &[Target::Card(target)]);
            packet(
                &mut g,
                p.other(),
                if protection { "Shore Up" } else { "Unsummon" },
                &[Target::Card(target)],
            );
            settle(&mut g);
            assert!(g.observe(p).temporary_controls.is_empty());
            assert!(!g.keywords(target).contains(Keywords::HASTE));
            controls(&g, target, p.other());
        }
    }
}
#[test]
fn msh_broker_thrall_thrall_source_independent_control_competes_by_timestamps_and_departure() {
    for p in [Player::P0, Player::P1] {
        for permanent in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, BROKER, &[]);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            paid(&mut g, p, THRALL, &[Target::Card(target)]);
            if permanent {
                paid(&mut g, p, "Confiscate", &[Target::Card(target)]);
                next_turn(&mut g);
                next_turn(&mut g);
                next_turn(&mut g);
                controls(&g, target, p);
            } else {
                packet(&mut g, p, "Unsummon", &[Target::Card(target)]);
                settle(&mut g);
                assert!(g.observe(p).temporary_controls.is_empty());
                g.move_object(target, Zone::Battlefield);
                controls(&g, target, p.other());
            }
        }
    }
}
#[test]
fn msh_broker_thrall_thrall_deadline_tracks_actual_next_own_turn_not_guessed_plus_two() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, BROKER, &[]);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, THRALL, &[Target::Card(target)]);
        let start = g.state.turn;
        // Bounded cleanup clocks model intervening opponent extra turns. No extra-turn producer is admitted by this fixture.
        for turn in [start, start + 1, start + 2, start + 3] {
            g.state.turn = turn;
            g.state.active = if turn == start { p } else { p.other() };
            g.expire_temporary_controls();
            controls(&g, target, p);
        }
        g.state.turn = start + 4;
        g.state.active = p;
        g.expire_temporary_controls();
        controls(&g, target, p.other());
        assert!(g.observe(p).temporary_controls.is_empty());
    }
}
#[test]
fn msh_broker_thrall_thrall_paid_target_selection_color_timing_and_owned_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = seed(&mut g, p, THRALL, Zone::Hand);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: id });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        settle(&mut g);
        controls(&g, target, p);
        let next = seed(&mut g, p, THRALL, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        packet(&mut g, p, "Shock", &[Target::Player(p.other())]);
        priority(&mut g, p);
        reject(&mut g, Action::AnnounceCast { card: next });
        settle(&mut g);
    }
}
#[test]
fn msh_broker_thrall_thrall_later_short_control_expires_back_to_long_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, BROKER, &[]);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, THRALL, &[Target::Card(target)]);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other()
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
        });
        paid(
            &mut g,
            p.other(),
            "Involuntary Employment",
            &[Target::Card(target)],
        );
        controls(&g, target, p.other());
        assert_eq!(g.observe(p).temporary_controls.len(), 2);
        next_turn(&mut g);
        controls(&g, target, p);
        assert_eq!(g.observe(p).temporary_controls.len(), 1);
        next_turn(&mut g);
        controls(&g, target, p.other());
        assert!(g.observe(p).temporary_controls.is_empty());
    }
}
#[test]
fn msh_broker_thrall_thrall_later_fixed_control_supersedes_and_removes_long_deadline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, BROKER, &[]);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, THRALL, &[Target::Card(target)]);
        paid(
            &mut g,
            p,
            "Harmless Offering",
            &[Target::Player(p.other()), Target::Card(target)],
        );
        controls(&g, target, p.other());
        assert!(g.observe(p).temporary_controls.is_empty());
        next_turn(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        controls(&g, target, p.other());
    }
}
#[test]
fn msh_broker_thrall_thrall_uses_real_current_copy_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, BROKER, &[]);
        let target = paid(&mut g, p, "Taskmaster, Mercenary Mimic", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
                && g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == target)
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].card.fixed_mana_value(), 2);
        paid(&mut g, p, THRALL, &[Target::Card(target)]);
        assert!(
            g.observe(p)
                .temporary_controls
                .iter()
                .any(|row| row.object == g.object_ref(target)
                    && row.expires_after_next_turn.is_some())
        );
    }
}
