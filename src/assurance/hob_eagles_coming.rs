//! Paid ownership-sensitive returns and one frozen next-upkeep token event per spell.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
const FIXTURES: &[&str] = &[
    "The Eagles Are Coming!",
    "Bear Cub",
    "Forest",
    "Doubling Season",
    "Banishing Light",
    "Adaptive Automaton",
    "Negate",
    "@token/hob-wolf",
    "@token/hob-bird-soldier",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Eagles budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn mana(kicked: bool) -> Mana {
    if kicked {
        [3, 0, 0, 0, 0, 3]
    } else {
        [1, 0, 0, 0, 0, 1]
    }
}
fn announce_eagles(g: &mut Game, p: Player, kicked: bool, ids: &[CardId]) -> CardId {
    let spell = fixture(g, p, "The Eagles Are Coming!", Zone::Hand);
    g.state.players[p.index()].mana = mana(kicked);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseKicker { pay: kicked });
    for &id in ids {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().kicker,
        Some(kicked)
    );
    spell
}
fn birds(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("@token/hob-bird-soldier").unwrap())
            .then_some(id as CardId)
        })
        .collect()
}
fn delayed(g: &Game) -> Vec<u64> {
    g.state
        .delayed_upkeep_tokens
        .as_ref()
        .into_iter()
        .flat_map(|r| r.iter())
        .map(|r| {
            if let StackKind::DelayedTokenCreation { amount, .. } = r.kind {
                amount
            } else {
                panic!("wrong delayed kind")
            }
        })
        .collect()
}
fn step(g: &mut Game, p: Player, s: Step) {
    g.state.active = p;
    g.begin_step(s);
    g.give_priority();
    g.prepare_triggers();
}
#[test]
fn eagles_paid_instant_owns_not_controls_return_and_next_any_player_upkeep_exact_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(own), p.other(), None);
        g.refresh_controllers();
        let foreign = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(foreign), p, None);
        g.refresh_controllers();
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        let spell = fixture(&mut g, p, "The Eagles Are Coming!", Zone::Hand);
        g.state.players[p.index()].mana = mana(false);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseKicker { pay: false });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        finish(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Hand);
        assert_eq!(g.object_view(own).owner, p);
        assert_eq!(g.object_view(foreign).zone, Zone::Battlefield);
        assert_eq!(delayed(&g), vec![1]);
        assert!(birds(&g, p).is_empty());
        step(&mut g, p.other(), Step::Main1);
        finish(&mut g);
        assert_eq!(delayed(&g), vec![1]);
        step(&mut g, p.other(), Step::Upkeep);
        finish(&mut g);
        let b = birds(&g, p);
        assert_eq!(b.len(), 1);
        let v = g.object_view(b[0]);
        assert_eq!((v.power, v.toughness), (4, 4));
        assert_eq!(v.colors, 1);
        assert!(v.has_subtype("Bird") && v.has_subtype("Soldier"));
        assert!(v.keywords.contains(Keywords::FLYING));
        assert_eq!(v.owner, p);
        assert!(!v.tapped);
        assert!(g.observe(p).delayed_upkeep_tokens.is_none());
        step(&mut g, p, Step::Upkeep);
        finish(&mut g);
        assert_eq!(birds(&g, p).len(), 1);
    }
}
#[test]
fn eagles_kicked_zero_targets_still_registers_one_zero_amount_delayed_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        announce_eagles(&mut g, p, true, &[]);
        finish(&mut g);
        assert_eq!(delayed(&g), vec![0]);
        step(&mut g, p.other(), Step::Upkeep);
        until(&mut g, |g| g.state.stack.len() == 1);
        assert!(matches!(
            g.state.stack[0].kind,
            StackKind::DelayedTokenCreation { amount: 0, .. }
        ));
        finish(&mut g);
        assert!(birds(&g, p).is_empty());
        assert!(delayed(&g).is_empty());
    }
}
#[test]
fn eagles_kicked_many_and_returned_token_count_is_frozen_before_cease_and_later_hand_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let token = fixture(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        g.record_control(g.object_ref(b), p.other(), None);
        g.refresh_controllers();
        announce_eagles(&mut g, p, true, &[a, b, token]);
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
        assert_eq!(delayed(&g), vec![3]);
        g.move_object(a, Zone::Graveyard);
        g.move_object(b, Zone::Battlefield);
        step(&mut g, p.other(), Step::Upkeep);
        finish(&mut g);
        assert_eq!(birds(&g, p).len(), 3);
    }
}
#[test]
fn eagles_partial_and_all_stale_or_noncreature_targets_only_count_actual_returns() {
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        announce_eagles(&mut g, p, true, &[a, b]);
        if mode == 2 {
            let timestamp = g.next_role_timestamp();
            let form = characteristic_layers::food_form(g.object_ref(a), timestamp);
            g.state.roles.get_or_insert_default().animations.push(form);
        } else {
            g.move_object(a, Zone::Hand);
            g.move_object(a, Zone::Battlefield);
        }
        if mode == 1 {
            g.move_object(b, Zone::Hand);
            g.move_object(b, Zone::Battlefield);
        }
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        if mode == 1 {
            assert_eq!(g.object_view(b).zone, Zone::Battlefield);
            assert!(delayed(&g).is_empty());
        } else {
            assert_eq!(g.object_view(b).zone, Zone::Hand);
            assert_eq!(delayed(&g), vec![1]);
            step(&mut g, p, Step::Upkeep);
            finish(&mut g);
            assert_eq!(birds(&g, p).len(), 1);
        }
    }
}
#[test]
fn eagles_next_upkeep_is_after_current_upkeep_and_independent_spells_are_distinct_triggers() {
    let p = Player::P0;
    let mut g = scenario(p);
    step(&mut g, p, Step::Upkeep);
    finish(&mut g);
    let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let c = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    announce_eagles(&mut g, p, false, &[a]);
    finish(&mut g);
    announce_eagles(&mut g, p, true, &[b, c]);
    finish(&mut g);
    assert_eq!(delayed(&g), vec![1, 2]);
    assert!(birds(&g, p).is_empty());
    contracts::branch_contract(g.clone());
    step(&mut g, p.other(), Step::Upkeep);
    until(&mut g, |g| g.state.stack.len() == 2);
    assert_eq!(
        g.state
            .stack
            .iter()
            .filter(|s| matches!(s.kind, StackKind::DelayedTokenCreation { .. }))
            .count(),
        2
    );
    let id = g
        .state
        .stack
        .iter()
        .find(|s| matches!(s.kind, StackKind::DelayedTokenCreation { amount: 1, .. }))
        .unwrap()
        .id;
    assert!(g.counter_stack_item(id, false));
    finish(&mut g);
    assert_eq!(birds(&g, p).len(), 2);
    step(&mut g, p, Step::Upkeep);
    finish(&mut g);
    assert_eq!(birds(&g, p).len(), 2);
}
#[test]
fn eagles_delayed_token_replacements_are_sampled_at_upkeep_and_owned_resume_creates_once() {
    let p = Player::P1;
    let mut g = scenario(p);
    let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    announce_eagles(&mut g, p, false, &[a]);
    finish(&mut g);
    let saved = g.snapshot();
    let mut left = g.clone();
    drop(g);
    left.restore(&saved).unwrap();
    fixture(&mut left, p, "Doubling Season", Zone::Battlefield);
    fixture(&mut left, p, "Doubling Season", Zone::Battlefield);
    assert_eq!(delayed(&left), vec![1]);
    step(&mut left, p.other(), Step::Upkeep);
    until(&mut left, |g| {
        matches!(g.state.pending, Pending::Replacement { .. })
    });
    contracts::branch_contract(left.clone());
    let saved = left.snapshot();
    let mut raw = left.clone();
    let mut indexed = left.clone();
    drop(left);
    let action = raw
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseReplacement { .. }))
        .unwrap();
    let actor = raw.actor().unwrap();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    act(&mut raw, action.clone());
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    for game in [&mut raw, &mut indexed] {
        for _ in 0..8 {
            if !matches!(game.state.pending, Pending::Replacement { .. }) {
                break;
            }
            choose(game, |a| matches!(a, Action::ChooseReplacement { .. }));
        }
        finish(game);
        assert_eq!(birds(game, p).len(), 4);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    for _ in 0..8 {
        if !matches!(indexed.state.pending, Pending::Replacement { .. }) {
            break;
        }
        choose(&mut indexed, |a| {
            matches!(a, Action::ChooseReplacement { .. })
        });
    }
    finish(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn eagles_return_batch_count_survives_linked_entry_choice_and_owned_parent_drop() {
    let p = Player::P0;
    let mut g = scenario(p);
    let auto = fixture(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
    let prison = fixture(&mut g, p, "Banishing Light", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
    cast(&mut g, prison, None);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(auto),
        })
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(auto),
        },
    );
    finish(&mut g);
    assert_eq!(g.object_view(auto).zone, Zone::Exile);
    g.change_object_form(
        g.object_ref(prison),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((1, 1)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    let second = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    announce_eagles(&mut g, p, true, &[prison, second]);
    until(&mut g, |g| {
        g.actor()
            .is_some_and(|p| g.observe(p).entry_value.is_some())
    });
    assert_eq!(g.object_view(prison).zone, Zone::Hand);
    assert_eq!(g.object_view(second).zone, Zone::Hand);
    assert_eq!(delayed(&g), vec![2]);
    contracts::branch_contract(g.clone());
    let saved = g.snapshot();
    let mut child = g.clone();
    drop(g);
    let action = Action::ChooseEntryValue {
        value: EntryChoiceValue::CreatureType(
            crate::cards::CreatureType::from_name("Bird").unwrap(),
        ),
    };
    act(&mut child, action.clone());
    finish(&mut child);
    let expected = child.snapshot();
    child.restore(&saved).unwrap();
    act(&mut child, action);
    finish(&mut child);
    assert_eq!(child.snapshot(), expected);
    step(&mut child, p.other(), Step::Upkeep);
    finish(&mut child);
    assert_eq!(birds(&child, p).len(), 2);
}
#[test]
fn eagles_countered_spell_does_not_return_or_schedule_and_failed_kicker_payment_rolls_back() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let spell = announce_eagles(&mut g, p, false, &[a]);
    act(&mut g, Action::Pass);
    let negate = fixture(&mut g, p.other(), "Negate", Zone::Hand);
    g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
    cast(&mut g, negate, Some(Target::Spell(spell)));
    finish(&mut g);
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert!(delayed(&g).is_empty());
    let spell = fixture(&mut g, p, "The Eagles Are Coming!", Zone::Hand);
    g.state.players[p.index()].mana = mana(true);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseKicker { pay: true });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    act(&mut g, Action::FinishTargets);
    g.state.players[p.index()].mana = [0; 6];
    reject(
        &mut g,
        Action::PayCosts {
            payment: mana(true),
        },
    );
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert!(delayed(&g).is_empty());
}
#[test]
fn eagles_late_schedule_capacity_failure_rolls_back_return_zone_generation_and_views() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    announce_eagles(&mut g, p, false, &[a]);
    act(&mut g, Action::Pass);
    g.state.next_stack_id = u64::MAX;
    let saved = g.snapshot();
    let revision = g.revision();
    let views = [
        serde_json::to_value(g.observe(p)).unwrap(),
        serde_json::to_value(g.observe(p.other())).unwrap(),
    ];
    let actor = g.actor().unwrap();
    assert!(matches!(
        g.apply(actor, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert!(delayed(&g).is_empty());
    assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
    assert_eq!(
        serde_json::to_value(g.observe(p.other())).unwrap(),
        views[1]
    );
}
