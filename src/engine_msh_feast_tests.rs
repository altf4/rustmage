//! Paid Food/lifelink events, captured optional target bounds and owned packets.
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;
const FEAST: &str = "Heroic Feast";
const FOOD: Card = Card(5246);
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn announce(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (c, n) = g.object_view(id).card.cost();
    g.state.players[p.index()].mana = c;
    g.state.players[p.index()].mana[5] += n;
    checked(g, Action::AnnounceCast { card: id });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    while g
        .state
        .announcing
        .as_ref()
        .is_some_and(|a| a.item.card == id && a.item.kind == StackKind::Spell)
    {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = announce(g, p, name, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..250 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("Feast unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("Feast stalled {:?}", g.state.pending);
}
fn body(p: Player) -> (Game, CardId) {
    let mut g = setup(p);
    for s in &mut g.state.players {
        s.mana = [0; 6];
    }
    let f = paid(&mut g, p, FEAST, &[]);
    (g, f)
}
fn bears(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n).map(|_| paid(g, p, "Bear Cub", &[])).collect()
}
fn food(g: &Game, p: Player) -> CardId {
    (0..g.state.objects.len() as CardId)
        .find(|&id| {
            let o = g.object_view(id);
            o.zone == Zone::Battlefield && o.controller == p && o.card == FOOD
        })
        .unwrap()
}
fn activate_food(g: &mut Game, p: Player) {
    priority(g, p);
    let id = food(g, p);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    while g
        .state
        .announcing
        .as_ref()
        .is_some_and(|a| a.item.card == id)
    {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert!(!g.on_battlefield(id));
}
fn is_feast(g: &Game, f: CardId) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == f && matches!(a.item.kind, StackKind::TriggeredAbility { ability: 1 })
    })
}
fn targets(g: &mut Game, f: CardId) -> usize {
    until(g, |g| is_feast(g, f));
    g.state.announcing.as_ref().unwrap().spec.max
}
fn finish_targets(g: &mut Game, f: CardId) {
    if is_feast(g, f) {
        checked(g, Action::FinishTargets);
    }
}
fn select(g: &mut Game, f: CardId, ids: &[CardId]) {
    for &id in ids {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    finish_targets(g, f);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn moved(g: &mut Game, ids: &[CardId], zone: Zone) {
    let e = g.move_objects_raw(ids, zone);
    g.finish_zone_events(&e, &[]);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let rev = g.revision();
    let index = g.legal_actions().iter().position(|b| *b == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
#[test]
fn msh_feast_paid_body_food_cost_and_three_distinct_creature_targets() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 4);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        priority(&mut g, p);
        let food = food(&g, p);
        assert!(g.has_subtype(food, "Food"));
        assert_eq!(g.object_view(food).card.colors(), 0);
        assert!(!g.object_view(food).tapped);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: food,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let life = g.state.players[p.index()].life;
        activate_food(&mut g, p);
        assert_eq!(targets(&mut g, f), 3);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(f),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own[0]),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own[0]),
            },
        );
        select(&mut g, f, &own[1..3]);
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(own[3])
        }));
        settle(&mut g);
        for (i, &id) in own.iter().enumerate() {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), u64::from(i < 3));
        }
    }
}
#[test]
fn msh_feast_zero_one_and_two_targets_are_optional_subsets_not_distribution() {
    for p in [Player::P0, Player::P1] {
        for n in 0..3 {
            let (mut g, f) = body(p);
            let own = bears(&mut g, p, 3);
            activate_food(&mut g, p);
            assert_eq!(targets(&mut g, f), 3);
            select(&mut g, f, &own[..n]);
            settle(&mut g);
            for (i, &id) in own.iter().enumerate() {
                assert_eq!(g.counter_count(id, CounterKind::PlusOne), u64::from(i < n));
            }
        }
    }
}
#[test]
fn msh_feast_no_creatures_still_announces_and_resolves_zero_targets() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        activate_food(&mut g, p);
        assert_eq!(targets(&mut g, f), 3);
        assert!(g.legal_actions().contains(&Action::FinishTargets));
        finish_targets(&mut g, f);
        settle(&mut g);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_feast_separate_captured_events_do_not_read_turn_sum_or_later_gain() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 6); /* Isolate two production event deliveries before putting either packet on stack. */
        g.gain_life(p, 2);
        g.gain_life(p, 5);
        let mut bounds = Vec::new();
        for _ in 0..2 {
            let max = targets(&mut g, f);
            bounds.push(max);
            assert!([2, 5].contains(&max));
            select(&mut g, f, &own[..max]);
        }
        settle(&mut g);
        bounds.sort();
        assert_eq!(bounds, [2, 5]);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 7);
        for (i, &id) in own.iter().enumerate() {
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                if i < 2 { 2 } else { u64::from(i < 5) }
            );
        }
    }
}
#[test]
fn msh_feast_actual_life_replacement_changes_target_bound() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        paid(&mut g, p, "Angel of Vitality", &[]);
        let own = bears(&mut g, p, 5);
        activate_food(&mut g, p);
        assert_eq!(targets(&mut g, f), 4);
        select(&mut g, f, &own[..4]);
        settle(&mut g);
        for (i, &id) in own.iter().enumerate() {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), u64::from(i < 4));
        }
    }
}
#[test]
fn msh_feast_prevented_zero_negative_and_opponent_gain_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        g.gain_life(p, 0);
        g.gain_life(p, -3);
        g.gain_life(p.other(), 3);
        assert!(g.state.pending_triggers.is_empty());
        paid(&mut g, p, "Giant Cindermaw", &[]);
        let life = g.state.players[p.index()].life;
        activate_food(&mut g, p);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        assert!(!is_feast(&g, f));
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn msh_feast_departed_source_does_not_cancel_paid_food_packet() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 1);
        activate_food(&mut g, p);
        assert_eq!(targets(&mut g, f), 3);
        select(&mut g, f, &own);
        announce(&mut g, p, "Disenchant", &[Target::Card(f)]);
        settle(&mut g);
        assert_eq!(g.object_view(f).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_feast_recipient_control_loss_or_reentry_rechecks_exact_target() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let (mut g, f) = body(p);
            let own = bears(&mut g, p, 2);
            activate_food(&mut g, p);
            targets(&mut g, f);
            select(&mut g, f, &own); /* Bounded production control/zone leaves isolate post-announcement legality. */
            if reenter {
                moved(&mut g, &own[..1], Zone::Hand);
                moved(&mut g, &own[..1], Zone::Battlefield);
            } else {
                g.change_controller(g.object_ref(own[0]), p.other());
            }
            settle(&mut g);
            assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), 0);
            assert_eq!(g.counter_count(own[1], CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn msh_feast_all_targets_departed_does_not_rebind_recast_card() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 1);
        activate_food(&mut g, p);
        targets(&mut g, f);
        select(&mut g, f, &own);
        let old = g.object_ref(own[0]);
        moved(&mut g, &own, Zone::Hand);
        moved(&mut g, &own, Zone::Battlefield);
        assert_ne!(g.object_ref(own[0]), old);
        settle(&mut g);
        assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), 0);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_feast_own_hexproof_remains_a_legal_optional_recipient() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 1);
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(own[0])]);
        assert!(g.keywords(own[0]).contains(Keywords::HEXPROOF));
        let old = g.counter_count(own[0], CounterKind::PlusOne);
        activate_food(&mut g, p);
        targets(&mut g, f);
        select(&mut g, f, &own);
        settle(&mut g);
        assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), old + 1);
    }
}
#[test]
fn msh_feast_source_controller_changes_preserve_old_event_controller() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 1); /* Production event and fixed control primitives isolate an already queued packet. */
        g.gain_life(p, 3);
        g.change_controller(g.object_ref(f), p.other());
        assert_eq!(targets(&mut g, f), 3);
        assert_eq!(g.actor(), Some(p));
        select(&mut g, f, &own);
        settle(&mut g);
        assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), 1);
        g.gain_life(p, 3);
        assert!(g.state.pending_triggers.is_empty());
        g.gain_life(p.other(), 1);
        assert_eq!(targets(&mut g, f), 1);
        assert_eq!(g.actor(), Some(p.other()));
        finish_targets(&mut g, f);
        settle(&mut g);
    }
}
#[test]
fn msh_feast_paid_confiscate_moves_future_life_observation_to_new_controller() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        paid(&mut g, p.other(), "Confiscate", &[Target::Card(f)]);
        assert_eq!(g.object_view(f).controller, p.other());
        activate_food(&mut g, p);
        settle(&mut g);
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_feast_token_and_counter_replacements_use_actual_creation_and_placer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        let f = paid(&mut g, p, FEAST, &[]);
        assert_eq!(
            (0..g.state.objects.len() as CardId)
                .filter(|&id| g.object_view(id).zone == Zone::Battlefield
                    && g.object_view(id).card == FOOD)
                .count(),
            2
        );
        let own = bears(&mut g, p, 1);
        activate_food(&mut g, p);
        targets(&mut g, f);
        select(&mut g, f, &own);
        settle(&mut g);
        assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), 2);
    }
}
#[test]
fn msh_feast_two_actual_lifelink_damage_sources_create_separate_bounds() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let a = paid(&mut g, p, "Healer's Hawk", &[]);
        let b = paid(&mut g, p, "Healer's Hawk", &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::Attack { card: b });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..2 {
            assert_eq!(targets(&mut g, f), 1);
            finish_targets(&mut g, f);
        }
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 2);
    }
}
#[test]
fn msh_feast_bound_is_not_truncated_to_sixteen_bits() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 1);
        g.gain_life(p, 65_536);
        assert_eq!(targets(&mut g, f), 65_536);
        select(&mut g, f, &own);
        settle(&mut g);
        assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_feast_owned_target_choice_partial_selection_and_resolution_restore() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 2);
        activate_food(&mut g, p);
        targets(&mut g, f);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(own[0]),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(own[1]),
            },
        );
        g = orphan(g, Action::FinishTargets);
        g = orphan(g, Action::Pass);
        g = orphan(g, Action::Pass);
        settle(&mut g);
        for id in own {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
    }
}
#[test]
fn msh_feast_full_observations_and_choices_ignore_unrelated_hidden_hands() {
    for p in [Player::P0, Player::P1] {
        let (mut a, f) = body(p);
        bears(&mut a, p, 2);
        activate_food(&mut a, p);
        targets(&mut a, f);
        let mut b = a.clone();
        named(&mut a, p.other(), "Murder", Zone::Hand);
        named(&mut b, p.other(), "Hill Giant", Zone::Hand);
        assert_eq!(
            serde_json::to_value(a.observe(p)).unwrap(),
            serde_json::to_value(b.observe(p)).unwrap()
        );
        assert_eq!(a.legal_actions(), b.legal_actions());
    }
}
#[test]
fn msh_feast_late_counter_overflow_rolls_back_all_recipients() {
    for p in [Player::P0, Player::P1] {
        let (mut g, f) = body(p);
        let own = bears(&mut g, p, 2);
        g.state.objects[own[1] as usize].counters.plus_one = u64::MAX;
        activate_food(&mut g, p);
        targets(&mut g, f);
        select(&mut g, f, &own);
        checked(&mut g, Action::Pass);
        let before = g.snapshot();
        let revision = g.revision();
        let result = g.apply(g.actor().unwrap(), Action::Pass);
        assert!(matches!(result, Err(Error::CapacityExceeded(_))));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        assert_eq!(g.counter_count(own[0], CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(own[1], CounterKind::PlusOne), u64::MAX);
    }
}
