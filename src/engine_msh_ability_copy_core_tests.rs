//! Whole-card paid scenarios for source-qualified independent ability copies.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const ECHO: &str = "Echo, Perceptive Prodigy";
const SCIENTIST: &str = "Scientist Supreme of A.I.M.";
fn prepared(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Hand)
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str) -> CardId {
    priority(g, p);
    let id = prepared(g, p, name);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = packet(g, p, name);
    settle(g);
    id
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
        if done(g) {
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
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("ability copy did not reach requested state");
}
fn age(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    priority(g, p);
}
fn pay(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
}
fn activate(g: &mut Game, source: CardId, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    pay(g);
}
fn stack_ability(g: &Game, source: CardId) -> u64 {
    g.state
        .stack
        .iter()
        .find(|s| {
            s.card == source
                && matches!(
                    s.kind,
                    StackKind::ActivatedAbility { .. } | StackKind::TriggeredAbility { .. }
                )
        })
        .unwrap()
        .id
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let bytes = g.state_bytes();
    let observations =
        serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap(),
        observations
    );
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        // Restore advances only the branch revision, not its rules contents.
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let before = g.snapshot();
    let before_views = views(&g);
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
#[test]
fn msh_ability_copy_core_paid_echo_retargets_raft_without_source_allocation_or_second_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let raft = paid(&mut g, p, "Raft Security Officer");
        age(&mut g, p);
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, raft, Some(Target::Card(a)));
        assert_eq!(g.state.players[p.index()].mana[5], 1);
        let original = g.state.stack.last().unwrap().clone();
        assert!(g.object_view(raft).tapped);
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(echo).tapped);
        let count = g.state.objects.len();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        assert_eq!(g.state.objects.len(), count);
        let copy = &g.retarget_view().unwrap().item;
        assert_eq!(
            (copy.card, copy.object, copy.kind),
            (original.card, original.object, original.kind)
        );
        assert!(copy.is_copy);
        assert_ne!(copy.id, original.id);
        g = orphan(
            g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        assert_eq!(g.state.objects.len(), count);
        assert_eq!(
            g.state.stack.iter().find(|s| s.id == original.id).unwrap(),
            &original
        );
        settle(&mut g);
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
        assert_eq!(g.object_view(raft).zone, Zone::Battlefield);
        assert_eq!(g.object_view(echo).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_ability_copy_core_paid_x_is_preserved_and_original_power_up_use_not_repeated() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let stature = paid(&mut g, p, "Stature, Size Shifter");
        age(&mut g, p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: stature,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::ChooseX { value: 3 });
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        assert_eq!(original.x, Some(3));
        let before = g.object_view(stature).power_up_uses;
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        let count = g.state.objects.len();
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.is_copy && s.object == original.object)
        });
        let copy = g
            .state
            .stack
            .iter()
            .find(|s| s.is_copy && s.object == original.object)
            .unwrap();
        assert_eq!(copy.x, original.x);
        assert_eq!(copy.source_ability_origin, original.source_ability_origin);
        assert_eq!(g.object_view(stature).power_up_uses, before);
        settle(&mut g);
        assert_eq!(g.counter_count(stature, CounterKind::PlusOne), 6);
        assert_eq!(g.state.objects.len(), count);
    }
}
#[test]
fn msh_ability_copy_core_actual_kree_cycling_has_independent_private_searches_for_both_copiers() {
    for p in [Player::P0, Player::P1] {
        for copier in [ECHO, SCIENTIST] {
            let mut g = setup(p);
            let source = paid(&mut g, p, copier);
            age(&mut g, p);
            let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
            let hand = g.hand_count(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            activate(&mut g, kree, None);
            assert_eq!(g.object_view(kree).zone, Zone::Graveyard);
            assert_eq!(g.hand_count(p), hand - 1);
            let id = stack_ability(&g, kree);
            let count = g.state.objects.len();
            let life = g.state.players[p.index()].life;
            activate(&mut g, source, Some(Target::Stack(id)));
            assert_eq!(
                g.state.players[p.index()].life,
                life - if copier == SCIENTIST { 2 } else { 0 }
            );
            for _ in 0..2 {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectSearch { .. })
                });
                assert!(g.observe(p).search_choice.is_some());
                assert!(g.observe(p.other()).search_choice.is_none());
                let chosen = g.state.players[p.index()].library[0];
                checked(&mut g, Action::ChooseSearch { card: chosen });
                checked(&mut g, Action::FinishSearch);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert_eq!(g.object_view(kree).zone, Zone::Graveyard);
            assert_eq!(g.state.objects.len(), count);
        }
    }
}
#[test]
fn msh_ability_copy_core_live_type_departure_lki_and_new_incarnation_do_not_rebind_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let scientist = paid(&mut g, p, SCIENTIST);
        let raft = paid(&mut g, p, "Raft Security Officer");
        age(&mut g, p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, raft, Some(Target::Card(bear)));
        let id = stack_ability(&g, raft);
        let target = Target::Stack(id);
        let rule = TargetRule::YourAbilityFromSourceKinds { kinds: 1 };
        assert!(g.stack_target_rule_matches(
            rule,
            g.state.stack.iter().find(|s| s.id == id).unwrap(),
            p
        ));
        // Bounded copy-layer setup isolates current-kind/LKI behavior while the
        // causing activation above paid its ordinary actual source cost.
        let old = g.object_ref(raft);
        g.apply_temporary_copy(
            old,
            CopyRecipe::new(Card::from_name("Arc Reactor").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let packet = g.state.stack.iter().find(|s| s.id == id).unwrap();
        assert!(!g.stack_target_rule_matches(rule, packet, p));
        assert!(g.stack_target_rule_matches(
            TargetRule::YourAbilityFromSourceKinds { kinds: 16 },
            packet,
            p
        ));
        g.move_object(raft, Zone::Graveyard);
        g.move_object(raft, Zone::Battlefield);
        assert_ne!(old, g.object_ref(raft));
        assert!(g.is_creature(raft));
        let packet = g.state.stack.iter().find(|s| s.id == id).unwrap();
        assert!(!g.stack_target_rule_matches(rule, packet, p));
        assert!(g.stack_target_rule_matches(
            TargetRule::YourAbilityFromSourceKinds { kinds: 16 },
            packet,
            p
        ));
        assert!(!g.stack_target_rule_matches(
            TargetRule::YourAbilityFromSourceKinds { kinds: 16 },
            packet,
            p.other()
        ));
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let life = g.state.players[p.index()].life;
        activate(&mut g, scientist, Some(target));
        assert_eq!(g.state.players[p.index()].life, life - 2);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert!(g.object_view(bear).tapped);
        assert_eq!(g.object_view(raft).zone, Zone::Battlefield);
    }
}

fn trigger_stack(g: &mut Game, source: CardId) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state
                .stack
                .iter()
                .any(|s| s.card == source && matches!(s.kind, StackKind::TriggeredAbility { .. }))
    });
}
#[test]
fn msh_ability_copy_core_triggered_surveil_choices_are_private_and_independent() {
    for p in [Player::P0, Player::P1] {
        for copier in [ECHO, SCIENTIST] {
            let mut g = setup(p);
            let source = paid(&mut g, p, copier);
            age(&mut g, p);
            let low = named(&mut g, p, "Island", Zone::Library);
            let top = named(&mut g, p, "Hill Giant", Zone::Library);
            let synth = packet(&mut g, p, "A.I.M. Synthoids");
            trigger_stack(&mut g, synth);
            let original = g.state.stack.last().unwrap().clone();
            let count = g.state.objects.len();
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            activate(&mut g, source, Some(Target::Stack(original.id)));
            until(&mut g, |g| g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            assert_eq!(g.observe(p).library_choice.unwrap().remaining.len(), 2);
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
            g = orphan(
                g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: LibraryDestination::Graveyard,
                },
            );
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: low,
                    destination: LibraryDestination::Top,
                },
            );
            until(&mut g, |g| g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            assert_eq!(g.observe(p).library_choice.unwrap().remaining[0].id, low);
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: low,
                    destination: LibraryDestination::Top,
                },
            );
            let remaining = g.observe(p).library_choice.unwrap().remaining[0].id;
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: remaining,
                    destination: LibraryDestination::Top,
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(top).zone, Zone::Graveyard);
            assert_eq!(g.object_view(synth).zone, Zone::Battlefield);
            assert_eq!(g.state.objects.len(), count);
        }
    }
}
#[test]
fn msh_ability_copy_core_modal_targets_are_frozen_with_transactional_distinct_retargeting() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        age(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let c = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let widow = packet(&mut g, p, "White Widow, Free Agent");
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
        });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        pay(&mut g);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.card == widow)
            .unwrap()
            .clone();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        assert_eq!(g.retarget_view().unwrap().item.mode, original.mode);
        assert_eq!(g.retarget_view().unwrap().item.targets, original.targets);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(c),
            },
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 1,
                target: Target::Card(c),
            },
        );
        reject(&mut g, Action::FinishStackTargets);
        checked(&mut g, Action::RestoreStackTarget { index: 1 });
        reject(&mut g, Action::ChooseMode { mode: 1 });
        g = orphan(g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(c, CounterKind::PlusOne), 1);
        assert_eq!(g.object_view(widow).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_ability_copy_core_committed_copy_acquires_retained_or_changed_targets_for_loki_once() {
    for p in [Player::P0, Player::P1] {
        for change in [false, true] {
            let mut g = setup(p);
            let echo = paid(&mut g, p, ECHO);
            let raft = paid(&mut g, p, "Raft Security Officer");
            age(&mut g, p);
            let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            activate(&mut g, raft, Some(Target::Card(a)));
            let original = g.state.stack.last().unwrap().clone();
            // The physical observer enters after the original acquisition; its
            // actual copy-acquisition interaction is isolated from its own cast.
            let loki = named(&mut g, p, "Loki, God of Mischief", Zone::Battlefield);
            let hand = g.hand_count(p);
            activate(&mut g, echo, Some(Target::Stack(original.id)));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Retarget { .. })
            });
            assert!(!g.state.pending_triggers.iter().any(|s| s.card == loki));
            if change {
                checked(
                    &mut g,
                    Action::SetStackTarget {
                        index: 0,
                        target: Target::Card(b),
                    },
                );
            }
            assert!(!g.state.pending_triggers.iter().any(|s| s.card == loki));
            checked(&mut g, Action::FinishStackTargets);
            assert_eq!(
                g.state
                    .pending_triggers
                    .iter()
                    .chain(g.state.stack.iter())
                    .filter(|s| s.card == loki)
                    .count(),
                1
            );
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert!(g.object_view(a).tapped);
            assert_eq!(g.object_view(b).tapped, change);
        }
    }
}
#[test]
fn msh_ability_copy_core_type_revalidation_fizzles_without_refunding_paid_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let stature = paid(&mut g, p, "Stature, Size Shifter");
        age(&mut g, p);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 4];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: stature,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::ChooseX { value: 3 });
        pay(&mut g);
        let id = stack_ability(&g, stature);
        activate(&mut g, echo, Some(Target::Stack(id)));
        assert!(g.object_view(echo).tapped);
        // Bounded live layer change after real target declaration and payment.
        g.apply_temporary_copy(
            g.object_ref(stature),
            CopyRecipe::new(Card::from_name("Arc Reactor").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let count = g.state.objects.len();
        settle(&mut g);
        assert_eq!(
            g.counter_count(stature, CounterKind::PlusOne),
            3,
            "only the original resolves; an erroneous copy would produce six"
        );
        assert!(g.object_view(echo).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.objects.len(), count);
        assert_eq!(g.object_view(stature).name(), "Arc Reactor");
    }
}
#[test]
fn msh_ability_copy_core_stack_source_cascade_copy_never_moves_physical_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        age(&mut g, p);
        let library_before = g.state.players[p.index()].library.len();
        let dragon = packet(&mut g, p, "Quandrix, the Proof");
        assert_eq!(g.object_view(dragon).zone, Zone::Stack);
        trigger_stack(&mut g, dragon);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.card == dragon && s.kind != StackKind::Spell)
            .unwrap()
            .clone();
        let physical = g.object_ref(dragon);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        let count = g.state.objects.len();
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.is_copy && s.object == physical)
        });
        assert_eq!(g.object_ref(dragon), physical);
        assert_eq!(g.object_view(dragon).zone, Zone::Stack);
        settle(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Battlefield);
        assert_eq!(g.state.objects.len(), count);
        assert_eq!(g.state.players[p.index()].library.len(), library_before);
    }
}

fn paid_target(g: &mut Game, p: Player, name: &str, target: Target) -> CardId {
    priority(g, p);
    let id = prepared(g, p, name);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, &[target]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn exact_copy(copy: &SpellView, original: &SpellView, p: Player) {
    let mut expected = original.clone();
    expected.id = copy.id;
    expected.controller = p;
    expected.is_copy = true;
    assert_ne!(copy.id, original.id);
    assert_eq!(copy, &expected);
}
#[test]
fn msh_ability_copy_core_copy_of_copy_keeps_full_paid_packet_and_three_private_searches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let scientist = paid(&mut g, p, SCIENTIST);
        age(&mut g, p);
        let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
        let hand = g.hand_count(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        activate(&mut g, kree, None);
        let original = g.state.stack.last().unwrap().clone();
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.is_copy && s.object == original.object)
        });
        let first = g
            .state
            .stack
            .iter()
            .find(|s| s.is_copy && s.object == original.object)
            .unwrap()
            .clone();
        exact_copy(&first, &original, p);
        priority(&mut g, p);
        activate(&mut g, scientist, Some(Target::Stack(first.id)));
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .filter(|s| s.is_copy && s.object == original.object)
                .count()
                == 2
        });
        exact_copy(g.state.stack.last().unwrap(), &first, p);
        for _ in 0..3 {
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p.other()).search_choice.is_none());
            let card = g.state.players[p.index()].library[0];
            checked(&mut g, Action::ChooseSearch { card });
            checked(&mut g, Action::FinishSearch);
        }
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 2);
        assert_eq!(g.object_view(kree).zone, Zone::Graveyard);
        assert_eq!(g.life(p), 18);
    }
}
#[test]
fn msh_ability_copy_core_granted_ability_qualifies_recipient_and_keeps_exact_equipment_link() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let scientist = paid(&mut g, p, SCIENTIST);
        let bear = paid(&mut g, p, "Bear Cub");
        let pole = paid(&mut g, p, "Fishing Pole");
        age(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, pole, Some(Target::Card(bear)));
        settle(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let grantor = g.object_ref(pole);
        checked(
            &mut g,
            Action::AnnounceAttachedAbility {
                grantor,
                ability: 1,
            },
        );
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        assert_eq!(original.object, g.object_ref(bear));
        assert_eq!(
            original.attachment_trigger.as_ref().unwrap().grantor,
            g.object_ref(pole)
        );
        assert!(g.object_view(pole).tapped && g.object_view(bear).tapped);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: scientist,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        until(&mut g, |g| g.state.stack.iter().any(|s| s.is_copy));
        exact_copy(g.state.stack.last().unwrap(), &original, p);
        settle(&mut g);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 2);
        assert_eq!(g.counter_count(bear, CounterKind::Bait), 0);
        assert_eq!(g.life(p), 20);
    }
}
#[test]
fn msh_ability_copy_core_linked_exile_copies_share_physical_source_but_choose_independently() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        age(&mut g, p);
        let a = named(&mut g, p, "Arc Reactor", Zone::Library);
        let b = named(&mut g, p, "Fishing Pole", Zone::Library);
        let dragon = packet(&mut g, p, "Hoarding Dragon");
        trigger_stack(&mut g, dragon);
        let original = g.state.stack.last().unwrap().clone();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        for card in [a, b] {
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            checked(&mut g, Action::ChooseOptional { accept: true });
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p.other()).search_choice.is_none());
            checked(&mut g, Action::ChooseSearch { card });
            checked(&mut g, Action::FinishSearch);
        }
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        paid_target(&mut g, p, "Murder", Target::Card(dragon));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Graveyard);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
    }
}
#[test]
fn msh_ability_copy_core_source_control_is_independent_from_ability_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let raft = paid(&mut g, p, "Raft Security Officer");
        age(&mut g, p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, raft, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        g.record_control(g.object_ref(raft), p.other(), None);
        assert_eq!(g.object_view(raft).controller, p.other());
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        exact_copy(&g.retarget_view().unwrap().item, &original, p);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert!(g.object_view(target).tapped);
        assert_eq!(g.object_view(raft).controller, p.other());
    }
}
#[test]
fn msh_ability_copy_core_copied_modal_targets_may_retain_illegal_but_cannot_select_it_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        age(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let widow = packet(&mut g, p, "White Widow, Free Agent");
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
        });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        for card in [a, b] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        pay(&mut g);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.card == widow)
            .unwrap()
            .clone();
        g.move_object(a, Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 1,
                target: Target::Card(a),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_ability_copy_core_spell_foreign_resolved_and_mana_targets_are_rejected_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let raft = paid(&mut g, p, "Raft Security Officer");
        age(&mut g, p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, raft, Some(Target::Card(target)));
        let own = g.state.stack.last().unwrap().clone();
        let spell = packet(&mut g, p, "Think Twice");
        let spell_packet = g.state.stack.iter().find(|s| s.card == spell).unwrap().id;
        let foreign = named(
            &mut g,
            p.other(),
            "Raft Security Officer",
            Zone::Battlefield,
        );
        priority(&mut g, p.other());
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, foreign, Some(Target::Card(target)));
        let foreign_packet = g.state.stack.last().unwrap().id;
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for bad in [
            Target::Stack(spell_packet),
            Target::Spell(spell),
            Target::Stack(foreign_packet),
            Target::Stack(u64::MAX),
        ] {
            reject(&mut g, Action::ChooseTarget { target: bad });
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(own.id),
            },
        );
        pay(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a, Action::ChooseTarget { target: Target::Stack(id) } if *id == own.id)
        ));
        // Mana abilities resolve immediately and never create a targetable stack
        // packet; this boundary is checked through an actual land activation.
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        priority(&mut g, p);
        let before = g.state.stack.len();
        let mana_action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::TapMana { card } if *card == land))
            .unwrap();
        checked(&mut g, mana_action);
        assert_eq!(g.state.stack.len(), before);
    }
}
#[test]
fn msh_ability_copy_core_copied_red_parent_creates_fresh_uncopied_reflexive_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let red = paid(&mut g, p, "Red Hulk");
        age(&mut g, p);
        paid_target(&mut g, p, "Shock", Target::Card(red));
        trigger_stack(&mut g, red);
        let original = g.state.stack.last().unwrap().clone();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        for n in [1, 2] {
            until(&mut g, |g| {
                g.state.announcing.as_ref().is_some_and(|a| {
                    a.item.card == red && a.item.kind == StackKind::TriggeredAbility { ability: 1 }
                })
            });
            let child = &g.state.announcing.as_ref().unwrap().item;
            assert!(!child.is_copy);
            assert_eq!(child.object, original.object);
            assert_eq!(g.counter_count(red, CounterKind::PlusOne), n);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
        }
        settle(&mut g);
        assert_eq!(g.life(p.other()), 17);
        assert_eq!(g.counter_count(red, CounterKind::PlusOne), 2);
    }
}
#[test]
fn msh_ability_copy_core_actual_countered_stack_source_uses_lki_without_copy_cleanup_move() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        age(&mut g, p);
        let dragon = packet(&mut g, p, "Quandrix, the Proof");
        trigger_stack(&mut g, dragon);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.card == dragon && s.kind != StackKind::Spell)
            .unwrap()
            .clone();
        let reference = original.object;
        paid_target(&mut g, p.other(), "Essence Scatter", Target::Spell(dragon));
        until(&mut g, |g| {
            g.object_view(dragon).zone == Zone::Graveyard && g.state.pending == Pending::Priority
        });
        assert_ne!(g.object_ref(dragon), reference);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        let count = g.state.objects.len();
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.is_copy && s.object == reference)
        });
        exact_copy(g.state.stack.last().unwrap(), &original, p);
        settle(&mut g);
        assert_eq!(g.object_view(dragon).zone, Zone::Graveyard);
        assert_eq!(g.state.objects.len(), count);
    }
}

#[test]
fn msh_ability_copy_core_real_shuri_two_groups_keep_cardinality_and_allow_cross_group_same_object()
{
    for p in [Player::P0, Player::P1] {
        for same in [false, true] {
            let mut g = setup(p);
            let echo = paid(&mut g, p, ECHO);
            let a = paid(&mut g, p, "Arc Reactor");
            let b = paid(&mut g, p, "Fishing Pole");
            let c = paid(&mut g, p, "Arc Reactor");
            let shuri = paid(&mut g, p, "Shuri, Wakandan Inventor");
            age(&mut g, p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: shuri,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(b),
                },
            );
            pay(&mut g);
            let original = g.state.stack.last().unwrap().clone();
            assert_eq!(
                original.targets.iter().map(|b| b.group).collect::<Vec<_>>(),
                vec![0, 1]
            );
            activate(&mut g, echo, Some(Target::Stack(original.id)));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Retarget { .. })
            });
            exact_copy(&g.retarget_view().unwrap().item, &original, p);
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(c),
                },
            );
            if same {
                checked(
                    &mut g,
                    Action::SetStackTarget {
                        index: 1,
                        target: Target::Card(c),
                    },
                );
            }
            checked(&mut g, Action::FinishStackTargets);
            settle(&mut g);
            assert_eq!(g.object_view(a).name(), "Fishing Pole");
            assert_eq!(
                g.object_view(c).name(),
                if same { "Arc Reactor" } else { "Fishing Pole" }
            );
            assert!(g.object_view(shuri).tapped && g.object_view(echo).tapped);
        }
    }
}
#[test]
fn msh_ability_copy_core_bounded_artifact_chandra_keeps_declared_divisions_and_loyalty_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let scientist = paid(&mut g, p, SCIENTIST);
        let chandra = paid(&mut g, p, "Chandra, Flameshaper");
        // Explicit source-kind boundary: no claim that a supported card currently
        // makes Chandra an artifact. The real loyalty activation, division,
        // payment, copying and retargeting below use immutable authored programs.
        g.apply_temporary_copy(
            g.object_ref(chandra),
            CopyRecipe::new(Card::from_name("Chandra, Flameshaper").unwrap())
                .with_exceptions(CopyExceptions::ARTIFACT),
            CopyDeadline::EndOfTurn,
        );
        let a = named(&mut g, p.other(), "Kree Sentinel", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let c = named(&mut g, p.other(), "Kree Sentinel", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: chandra,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        for card in [a, b] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(&mut g, Action::FinishTargets);
        checked(&mut g, Action::AllocateTargetAmount { amount: 3 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 5 });
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        assert_eq!(g.counter_count(chandra, CounterKind::Loyalty), 2);
        activate(&mut g, scientist, Some(Target::Stack(original.id)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        exact_copy(&g.retarget_view().unwrap().item, &original, p);
        reject(&mut g, Action::AllocateTargetAmount { amount: 4 });
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(c),
            },
        );
        let shares = g.retarget_view().unwrap().item.target_allocations.unwrap();
        assert_eq!(
            shares.iter().map(|r| r.amount).collect::<Vec<_>>(),
            vec![3, 5]
        );
        assert_eq!(shares[0].target.target, Target::Card(c));
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(a).damage, 3);
        assert_eq!(g.object_view(c).damage, 3);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(chandra, CounterKind::Loyalty), 2);
        assert_eq!(g.life(p), 18);
    }
}
#[test]
fn msh_ability_copy_core_copied_ability_counts_as_a_second_resolution_without_new_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let venom = paid(&mut g, p, "Venom Connoisseur");
        age(&mut g, p);
        let bear = packet(&mut g, p, "Bear Cub");
        trigger_stack(&mut g, venom);
        let original = g.state.stack.last().unwrap().clone();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        settle(&mut g);
        assert_eq!(g.ability_resolution_number(&original), 2);
        assert!(g.keywords(venom).contains(Keywords::DEATHTOUCH));
        assert!(g.keywords(bear).contains(Keywords::DEATHTOUCH));
        assert!(g.keywords(echo).contains(Keywords::DEATHTOUCH));
    }
}
// Printed-card obligations use ordinary support-checked physical fixtures.
fn copy_searches(g: &mut Game, p: Player, count: usize) {
    for _ in 0..count {
        until(g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        let card = g.state.players[p.index()].library[0];
        checked(g, Action::ChooseSearch { card });
        checked(g, Action::FinishSearch);
    }
    settle(g);
}
fn announce_copier(g: &mut Game, source: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn msh_ability_copy_cards_paid_printed_bodies_mana_and_spell_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        assert_eq!((g.stats(echo).power, g.stats(echo).toughness), (1, 4));
        assert!(g.keywords(echo).contains(Keywords::VIGILANCE));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let scientist = paid(&mut g, p, SCIENTIST);
        assert_eq!(
            (g.stats(scientist).power, g.stats(scientist).toughness),
            (2, 2)
        );
        assert_eq!(g.life(p), 20);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let other_echo = prepared(&mut g, p.other(), ECHO);
        priority(&mut g, p.other());
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 2];
        reject(&mut g, Action::AnnounceCast { card: other_echo });
        let other_scientist = prepared(&mut g, p.other(), SCIENTIST);
        g.state.players[p.other().index()].mana = [0, 1, 1, 0, 0, 0];
        reject(
            &mut g,
            Action::AnnounceCast {
                card: other_scientist,
            },
        );
    }
}
#[test]
fn msh_ability_copy_cards_echo_vigilance_in_blocked_combat_preserves_postcombat_tap_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        age(&mut g, p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: echo });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(echo).tapped);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: echo,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_view(echo).damage, 2);
        assert_eq!(g.object_view(bear).damage, 1);
        assert!(!g.object_view(echo).tapped);
        let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        activate(&mut g, kree, None);
        let target = stack_ability(&g, kree);
        activate(&mut g, echo, Some(Target::Stack(target)));
        assert!(g.object_view(echo).tapped);
        copy_searches(&mut g, p, 2);
        assert_eq!(g.object_view(echo).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_ability_copy_cards_echo_sickness_mana_tap_and_real_untap_repeat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let raft = named(&mut g, p, "Raft Security Officer", Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, raft, Some(Target::Card(bear)));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        age(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, raft, Some(Target::Card(bear)));
        let original = g.state.stack.last().unwrap().clone();
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        checked(&mut g, Action::FinishStackTargets);
        paid_target(&mut g, p, "Shore Up", Target::Card(echo));
        until(&mut g, |g| {
            !g.object_view(echo).tapped && g.state.pending == Pending::Priority
        });
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, echo, Some(Target::Stack(original.id)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        checked(&mut g, Action::FinishStackTargets);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.is_copy && s.object == original.object)
                .count(),
            2
        );
        settle(&mut g);
        assert!(g.object_view(echo).tapped);
        assert!(g.object_view(bear).tapped);
    }
}
#[test]
fn msh_ability_copy_cards_echo_copies_during_opponents_turn_at_instant_speed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        age(&mut g, p);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        activate(&mut g, kree, None);
        let target = stack_ability(&g, kree);
        activate(&mut g, echo, Some(Target::Stack(target)));
        copy_searches(&mut g, p, 2);
        assert_eq!(g.state.active, p.other());
        assert!(g.object_view(echo).tapped);
    }
}
#[test]
fn msh_ability_copy_cards_scientist_own_turn_once_and_actual_turn_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let scientist = paid(&mut g, p, SCIENTIST);
        assert!(g.object_view(scientist).summoning_sick);
        let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, kree, None);
        let id = stack_ability(&g, kree);
        activate(&mut g, scientist, Some(Target::Stack(id)));
        assert!(!g.object_view(scientist).tapped);
        assert_eq!(g.life(p), 18);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: scientist,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        copy_searches(&mut g, p, 2);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        let kree2 = named(&mut g, p, "Kree Sentinel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, kree2, None);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: scientist,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.life(p), 18);
        copy_searches(&mut g, p, 1);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        let kree3 = named(&mut g, p, "Kree Sentinel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, kree3, None);
        let id = stack_ability(&g, kree3);
        activate(&mut g, scientist, Some(Target::Stack(id)));
        assert_eq!(g.life(p), 16);
        copy_searches(&mut g, p, 2);
    }
}
#[test]
fn msh_ability_copy_cards_scientist_life_payment_boundary_is_real_and_atomic() {
    for p in [Player::P0, Player::P1] {
        for life in [1, 2, 3] {
            let mut g = setup(p);
            let scientist = paid(&mut g, p, SCIENTIST);
            let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            activate(&mut g, kree, None);
            let target = stack_ability(&g, kree);
            g.state.players[p.index()].life = life;
            if life == 1 {
                reject(
                    &mut g,
                    Action::AnnounceActivate {
                        card: scientist,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
            } else {
                activate(&mut g, scientist, Some(Target::Stack(target)));
                assert_eq!(g.life(p), life - 2);
                if life == 2 {
                    assert!(g.outcome().is_some());
                } else {
                    assert!(g.outcome().is_none());
                    copy_searches(&mut g, p, 2);
                }
            }
        }
    }
}
#[test]
fn msh_ability_copy_cards_paid_source_death_after_activation_preserves_captured_controller() {
    for p in [Player::P0, Player::P1] {
        for copier in [ECHO, SCIENTIST] {
            let mut g = setup(p);
            let source = paid(&mut g, p, copier);
            age(&mut g, p);
            let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            activate(&mut g, kree, None);
            let target = stack_ability(&g, kree);
            activate(&mut g, source, Some(Target::Stack(target)));
            let old = g.object_ref(source);
            paid_target(&mut g, p.other(), "Murder", Target::Card(source));
            until(&mut g, |g| {
                g.object_view(source).zone == Zone::Graveyard
                    && g.state.pending == Pending::Priority
            });
            assert_ne!(g.object_ref(source), old);
            let hand = g.hand_count(p);
            copy_searches(&mut g, p, 2);
            assert_eq!(g.hand_count(p), hand + 2);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_ability_copy_cards_paid_suppression_removes_activations_but_not_queued_programs() {
    for p in [Player::P0, Player::P1] {
        for copier in [ECHO, SCIENTIST] {
            for after in [false, true] {
                let mut g = setup(p);
                let source = paid(&mut g, p, copier);
                age(&mut g, p);
                if !after {
                    paid_target(&mut g, p, "Eaten by Piranhas", Target::Card(source));
                    settle(&mut g);
                }
                let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
                activate(&mut g, kree, None);
                let target = stack_ability(&g, kree);
                if after {
                    activate(&mut g, source, Some(Target::Stack(target)));
                    let aura =
                        paid_target(&mut g, p.other(), "Eaten by Piranhas", Target::Card(source));
                    until(&mut g, |g| {
                        g.object_view(aura).zone == Zone::Battlefield
                            && g.state.pending == Pending::Priority
                    });
                    copy_searches(&mut g, p, 2);
                } else {
                    reject(
                        &mut g,
                        Action::AnnounceActivate {
                            card: source,
                            ability: 0,
                            counter_payment: Counters::default(),
                        },
                    );
                    copy_searches(&mut g, p, 1);
                }
                assert_eq!((g.stats(source).power, g.stats(source).toughness), (1, 1));
                assert!(!g.keywords(source).contains(Keywords::VIGILANCE));
            }
        }
    }
}
#[test]
fn msh_ability_copy_cards_real_funding_requires_third_land_for_echo_after_cycling() {
    for p in [Player::P0, Player::P1] {
        for enough in [false, true] {
            let mut g = setup(p);
            let echo = paid(&mut g, p, ECHO);
            age(&mut g, p);
            let lands: Vec<_> = (0..if enough { 3 } else { 2 })
                .map(|_| named(&mut g, p, "Forest", Zone::Battlefield))
                .collect();
            g.state.players[p.index()].mana = [0; 6];
            let kree = named(&mut g, p, "Kree Sentinel", Zone::Hand);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: kree,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            fund_lands(&mut g, &lands);
            let target = stack_ability(&g, kree);
            if enough {
                announce_copier(&mut g, echo);
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Stack(target),
                    },
                );
                fund_lands(&mut g, &lands);
                assert_eq!(
                    lands.iter().filter(|&&id| g.object_view(id).tapped).count(),
                    3
                );
                copy_searches(&mut g, p, 2);
            } else {
                reject(
                    &mut g,
                    Action::AnnounceActivate {
                        card: echo,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                copy_searches(&mut g, p, 1);
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
fn fund_lands(g: &mut Game, lands: &[CardId]) {
    while g.state.announcing.is_some() {
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::TapMana { card } if lands.contains(card)))
            })
            .unwrap();
        checked(g, a.clone());
    }
}
#[test]
fn msh_ability_copy_cards_new_target_ward_relates_to_copy_and_never_repeats_source_cost() {
    for p in [Player::P0, Player::P1] {
        for pay_ward in [false, true] {
            let mut g = setup(p);
            let echo = paid(&mut g, p, ECHO);
            let raft = paid(&mut g, p, "Raft Security Officer");
            age(&mut g, p);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let titan = named(
                &mut g,
                p.other(),
                "Titania, Rugged Rumbler",
                Zone::Battlefield,
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            activate(&mut g, raft, Some(Target::Card(bear)));
            let original = g.state.stack.last().unwrap().clone();
            activate(&mut g, echo, Some(Target::Stack(original.id)));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Retarget { .. })
            });
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(titan),
                },
            );
            checked(&mut g, Action::FinishStackTargets);
            let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap().id;
            assert_eq!(g.state.players[p.index()].mana[5], 2);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            assert_eq!(g.observe(p).resolution_payment.unwrap().victim, Some(copy));
            g = orphan(
                g,
                if pay_ward {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            settle(&mut g);
            assert!(g.object_view(bear).tapped);
            assert_eq!(g.object_view(titan).tapped, pay_ward);
            assert_eq!(g.object_view(raft).zone, Zone::Battlefield);
            assert!(g.object_view(raft).tapped && g.object_view(echo).tapped);
        }
    }
}
#[test]
fn msh_ability_copy_cards_scientist_copies_noncreature_equipment_ability_without_equip_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, ECHO);
        let scientist = paid(&mut g, p, SCIENTIST);
        let pole = paid(&mut g, p, "Fishing Pole");
        age(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        activate(&mut g, pole, Some(Target::Card(a)));
        let original = g.state.stack.last().unwrap().clone();
        assert!(!g.is_creature(pole));
        assert_eq!(g.state.players[p.index()].mana[5], 1);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        activate(&mut g, scientist, Some(Target::Stack(original.id)));
        assert_eq!(g.life(p), 18);
        assert_eq!(g.state.players[p.index()].mana[5], 1);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        exact_copy(&g.retarget_view().unwrap().item, &original, p);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        until(&mut g, |g| {
            g.object_view(pole).attached_to == Some(g.object_ref(b))
                && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.players[p.index()].mana[5], 1);
        settle(&mut g);
        assert_eq!(g.object_view(pole).attached_to, Some(g.object_ref(a)));
        assert_eq!(g.object_view(pole).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana[5], 1);
    }
}
#[test]
fn msh_ability_copy_cards_scientist_copies_real_sacrificed_clue_after_token_ceases() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let scientist = paid(&mut g, p, SCIENTIST);
        let pounce = prepared(&mut g, p, "Panther Pounce");
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        cast(
            &mut g,
            pounce,
            None,
            &[Target::Player(p), Target::Card(scientist)],
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        let clues: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(i as CardId, "Clue"))
                .then_some(i as CardId)
            })
            .collect();
        assert_eq!(clues.len(), 1);
        let clue = clues[0];
        let old = g.object_ref(clue);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, clue, None);
        let original = g.state.stack.last().unwrap().clone();
        assert_eq!(original.object, old);
        assert_ne!(g.object_ref(clue), old);
        assert!(!g.on_battlefield(clue));
        assert!(original.source_facts.token);
        assert_ne!(original.source_facts.kinds & 16, 0);
        let zone = g.object_view(clue).zone;
        let objects = g.state.objects.len();
        let hand = g.hand_count(p);
        activate(&mut g, scientist, Some(Target::Stack(original.id)));
        until(&mut g, |g| {
            g.state.stack.iter().any(|s| s.is_copy && s.object == old)
        });
        exact_copy(g.state.stack.last().unwrap(), &original, p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 2);
        assert_eq!(g.life(p), 18);
        assert_eq!(g.state.objects.len(), objects);
        assert_eq!(g.object_view(clue).zone, zone);
    }
}
