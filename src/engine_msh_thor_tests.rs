//! Whole Thor and instruction-level tapped returns through owned continuations.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const THOR: &str = "The Mighty Thor, Jane Foster";
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
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
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected choice {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("Thor scenario stalled {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn thor(g: &mut Game, p: Player) -> CardId {
    g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 1];
    let id = paid(g, p, THOR, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.state.players[p.index()].mana = [30; 6];
    id
}
fn ready(g: &mut Game, p: Player) -> CardId {
    let id = thor(g, p);
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    assert_eq!(g.state.active, p);
    assert!(!g.object_view(id).summoning_sick);
    g.state.players[p.index()].mana = [30; 6];
    id
}
fn pay(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn attack(g: &mut Game, id: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    until(g, |g| {
        g.state.announcing.as_ref().is_some_and(|a| {
            a.item.card == id && a.item.kind == (StackKind::TriggeredAbility { ability: 0 })
        })
    });
}
fn choose(g: &mut Game, id: CardId) {
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    pay(g);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
#[test]
fn msh_thor_paid_body_no_self_draw_sickness_flying_and_optional_zero() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let library = g.state.players[p.index()].library.len();
        let id = thor(&mut g, p);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (3, 3));
        assert!(g.keywords(id).contains(Keywords::FLYING));
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        reject(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, id);
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(id).tapped);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                attacker: id,
                blocker: ground,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
#[test]
fn msh_thor_optional_targets_own_opposing_creature_and_noncreature_artifact() {
    for p in [Player::P0, Player::P1] {
        for (owner, name) in [
            (p, "Bear Cub"),
            (p.other(), "Bear Cub"),
            (p, "Vibranium Energy Daggers"),
        ] {
            let mut g = setup(p);
            let id = ready(&mut g, p);
            let target = if owner == p {
                paid(&mut g, p, name, &[])
            } else {
                named(&mut g, owner, name, Zone::Battlefield)
            };
            let old = g.object_ref(target);
            let library = g.state.players[p.index()].library.len();
            attack(&mut g, id);
            choose(&mut g, target);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).controller, owner);
            assert!(g.object_view(target).tapped);
            assert_ne!(g.object_ref(target), old);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                library - usize::from(name == "Vibranium Energy Daggers")
            );
        }
    }
}
#[test]
fn msh_thor_rejects_tokens_lands_enchantments_and_second_target_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        paid(&mut g, p, "Hire a Crew", &[]);
        let token = g
            .state
            .objects
            .iter()
            .position(|o| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
            .unwrap() as CardId;
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let enchantment = paid(&mut g, p, "Rest in Peace", &[]);
        let first = paid(&mut g, p, "Bear Cub", &[]);
        let second = paid(&mut g, p, "Bear Cub", &[]);
        attack(&mut g, id);
        for target in [token, land, enchantment] {
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
                target: Target::Card(first),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(first).tapped);
        assert!(!g.object_view(second).tapped);
    }
}
#[test]
fn msh_thor_stolen_target_returns_to_owner_with_new_incarnation_and_cleared_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Involuntary Employment", &[Target::Card(target)]);
        paid(&mut g, p, "Giant Growth", &[Target::Card(target)]);
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(target)]);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
        let old = g.object_ref(target);
        assert_eq!(g.object_view(target).controller, p);
        attack(&mut g, id);
        choose(&mut g, target);
        settle(&mut g);
        assert_eq!(g.object_view(target).owner, p.other());
        assert_eq!(g.object_view(target).controller, p.other());
        assert_ne!(g.object_ref(target), old);
        assert!(g.object_view(target).tapped);
        assert_eq!((g.stats(target).power, g.stats(target).toughness), (2, 2));
        assert!(!g.keywords(target).contains(Keywords::HASTE));
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_thor_queued_blink_survives_source_removal_suppression_or_control_change() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let id = ready(&mut g, p);
            let target = paid(&mut g, p, "Bear Cub", &[]);
            let old = g.object_ref(target);
            attack(&mut g, id);
            choose(&mut g, target);
            g.state.players[p.index()].mana = [30; 6];
            match mode {
                0 => {
                    let s = named(&mut g, p, "Murder", Zone::Hand);
                    cast(&mut g, s, None, &[Target::Card(id)]);
                }
                1 => {
                    let s = named(&mut g, p, "Eaten by Piranhas", Zone::Hand);
                    cast(&mut g, s, None, &[Target::Card(id)]);
                }
                _ => g.record_control(g.object_ref(id), p.other(), None),
            }
            settle(&mut g);
            assert_ne!(g.object_ref(target), old);
            assert!(g.object_view(target).tapped);
            assert_eq!(g.object_view(target).controller, p);
        }
    }
}
#[test]
fn msh_thor_suppression_before_declaration_prevents_attack_and_equipment_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        paid(&mut g, p, "Witness Protection", &[Target::Card(id)]);
        let library = g.state.players[p.index()].library.len();
        paid(&mut g, p, "Vibranium Energy Daggers", &[]);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .any(|a| a.card == id)
        );
    }
}
#[test]
fn msh_thor_stale_target_does_not_blink_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        let target = paid(&mut g, p, "Bear Cub", &[]);
        paid(&mut g, p, "Captain Mar-Vell, Space-Born", &[]);
        attack(&mut g, id);
        choose(&mut g, target);
        g.state.players[p.index()].mana = [30; 6];
        checked(&mut g, Action::Pass);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 0];
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p)]);
        checked(&mut g, Action::Pass);
        let spell = named(&mut g, p, "Daydream", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(target)]);
        until(&mut g, |g| {
            g.object_view(spell).zone == Zone::Graveyard && g.state.pending == Pending::Priority
        });
        let fresh = g.object_ref(target);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
        settle(&mut g);
        assert_eq!(g.object_ref(target), fresh);
        assert!(!g.object_view(target).tapped);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_thor_tapped_entry_is_not_a_tap_event_for_captain() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Captain America, Living Legend", &[]);
        let id = ready(&mut g, p);
        let target = paid(&mut g, p, "Bear Cub", &[]);
        attack(&mut g, id);
        choose(&mut g, target);
        settle(&mut g);
        assert!(g.object_view(target).tapped);
        assert!(!g.first_tap(g.object_ref(target)));
        // The Captain untaps the real attacking Thor; it never untaps an entry-tapped Bear.
        assert!(!g.object_view(id).tapped);
    }
}
#[test]
fn msh_thor_equipment_entries_include_token_copies_and_follow_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = thor(&mut g, p);
        let before = g.state.players[p.index()].library.len();
        let equipment = paid(&mut g, p, "Vibranium Energy Daggers", &[]);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        paid(&mut g, p, "Molten Duplication", &[Target::Card(equipment)]);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert!(
            g.state
                .objects
                .iter()
                .enumerate()
                .any(|(i, o)| o.zone == Zone::Battlefield
                    && o.is_token()
                    && g.has_subtype(i as CardId, "Equipment"))
        );
        paid(&mut g, p, "Bear Cub", &[]);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        g.record_control(g.object_ref(id), p.other(), None);
        paid(&mut g, p, "Vibranium Energy Daggers", &[]);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.active, p.other());
        g.state.players[p.other().index()].mana = [30; 6];
        let other_before = g.state.players[p.other().index()].library.len();
        paid(&mut g, p.other(), "Vibranium Energy Daggers", &[]);
        assert_eq!(
            g.state.players[p.other().index()].library.len(),
            other_before - 1
        );
    }
}
#[test]
fn msh_thor_owned_tapped_continuation_survives_choice_bearing_departure_and_parent_drop() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        let automaton = named(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
        g.state.objects[automaton as usize].tapped = true;
        let henchbots = named(&mut g, p, "Henchbots", Zone::Hand);
        cast(&mut g, henchbots, None, &[]);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == henchbots)
        });
        choose(&mut g, automaton);
        settle(&mut g);
        assert_eq!(g.object_view(automaton).zone, Zone::Exile);
        attack(&mut g, id);
        choose(&mut g, henchbots);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        let view = g.observe(p);
        let row = &view.immediate_blink_return.as_ref().unwrap()[0];
        assert_eq!(row.object, g.object_ref(henchbots));
        assert!(row.tapped);
        assert_eq!(row.controller, p);
        assert_eq!(serde_json::to_value(row).unwrap()["tapped"], true);
        assert_eq!(
            serde_json::to_value(&view.immediate_blink_return).unwrap(),
            serde_json::to_value(g.observe(p.other()).immediate_blink_return).unwrap()
        );
        let human =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Human").unwrap());
        g = orphan(g, Action::ChooseEntryValue { value: human });
        settle(&mut g);
        assert_eq!(g.object_view(automaton).controller, p.other());
        assert_eq!(g.object_view(automaton).entry_choice, Some(human));
        assert_eq!(g.object_view(henchbots).zone, Zone::Battlefield);
        assert!(g.object_view(henchbots).tapped);
        assert!(g.observe(p).immediate_blink_return.is_none());
    }
}
#[test]
fn msh_thor_choice_bearing_entry_retains_tapped_instruction_and_old_daydream_untapped_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        let target = named(&mut g, p, "Adaptive Automaton", Zone::Battlefield);
        attack(&mut g, id);
        choose(&mut g, target);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        let robot =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Robot").unwrap());
        g = orphan(g, Action::ChooseEntryValue { value: robot });
        settle(&mut g);
        assert!(g.object_view(target).tapped);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        g.state.players[p.index()].mana = [30; 6];
        let spell = named(&mut g, p, "Daydream", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(target)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        checked(&mut g, Action::ChooseEntryValue { value: robot });
        settle(&mut g);
        assert!(!g.object_view(target).tapped);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_thor_animated_land_returns_as_land_with_real_untapped_entry_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        paid(&mut g, p, "Spelunking", &[]);
        let land = named(&mut g, p, "Crawling Barrens", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert!(g.object_view(land).is_creature());
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 2);
        let old = g.object_ref(land);
        attack(&mut g, id);
        choose(&mut g, land);
        settle(&mut g);
        assert_ne!(g.object_ref(land), old);
        assert!(!g.object_view(land).is_creature());
        assert_eq!(g.counter_count(land, CounterKind::PlusOne), 0);
        assert!(!g.object_view(land).tapped);
    }
}
#[test]
fn msh_thor_blink_of_actual_transformed_photon_restores_front_face_tapped() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = ready(&mut g, p);
        let target = paid(&mut g, p, "Monica Rambeau // Photon, Living Light", &[]);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: target,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(target).face.unwrap().back);
        let old = g.object_ref(target);
        attack(&mut g, id);
        choose(&mut g, target);
        settle(&mut g);
        assert_ne!(g.object_ref(target), old);
        assert!(!g.object_view(target).face.unwrap().back);
        assert_eq!(g.object_view(target).name(), "Monica Rambeau");
        assert!(g.object_view(target).tapped);
        assert!(!g.keywords(target).contains(Keywords::HEXPROOF));
        assert_eq!((g.stats(target).power, g.stats(target).toughness), (3, 3));
    }
}

#[test]
fn msh_thor_queued_equipment_draw_keeps_captured_controller_after_source_changes() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let id = thor(&mut g, p);
            let before = g.state.players[p.index()].library.len();
            let equipment = named(&mut g, p, "Vibranium Energy Daggers", Zone::Hand);
            cast(&mut g, equipment, None, &[]);
            until(&mut g, |g| {
                g.state.pending == Pending::Priority
                    && g.state.stack.iter().any(|a| {
                        a.card == id && a.kind == (StackKind::TriggeredAbility { ability: 1 })
                    })
            });
            match mode {
                0 => {
                    let spell = named(&mut g, p, "Murder", Zone::Hand);
                    cast(&mut g, spell, None, &[Target::Card(id)]);
                }
                1 => {
                    let spell = named(&mut g, p, "Eaten by Piranhas", Zone::Hand);
                    cast(&mut g, spell, None, &[Target::Card(id)]);
                }
                _ => g.record_control(g.object_ref(id), p.other(), None),
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), before - 1);
            assert_eq!(g.state.players[p.other().index()].library.len(), 20);
        }
    }
}
#[test]
fn msh_thor_equipment_draw_does_not_disclose_the_private_top_card_to_opponent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        thor(&mut g, p);
        let equipment = named(&mut g, p, "Vibranium Energy Daggers", Zone::Hand);
        let top = *g.state.players[p.index()].library.last().unwrap();
        let (mut left, mut right) = (g.clone(), g.clone());
        drop(g);
        right.state.objects[top as usize].card = Card::from_name("Island").unwrap();
        assert_eq!(
            serde_json::to_vec(&left.observe(p.other())).unwrap(),
            serde_json::to_vec(&right.observe(p.other())).unwrap()
        );
        for branch in [&mut left, &mut right] {
            cast(branch, equipment, None, &[]);
            settle(branch);
            assert_eq!(branch.object_view(top).zone, Zone::Hand);
        }
        assert_ne!(left.object_view(top).card, right.object_view(top).card);
        assert_eq!(
            serde_json::to_vec(&left.observe(p.other())).unwrap(),
            serde_json::to_vec(&right.observe(p.other())).unwrap()
        );
        assert_ne!(
            serde_json::to_vec(&left.observe(p)).unwrap(),
            serde_json::to_vec(&right.observe(p)).unwrap()
        );
    }
}
