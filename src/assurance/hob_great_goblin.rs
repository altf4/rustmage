//! The Great Goblin: actual placement and frozen-death observations.
use super::*;
use crate::engine::replacements::ResumeMode;

fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn finish(g: &mut Game) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseTarget {
                        target: Target::Player(_)
                    }
                    | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("Great Goblin decision budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Great Goblin", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 1];
    cast(g, id, None);
    finish(g);
    assert_eq!(stats(g, id), (3, 2));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
#[test]
fn great_goblin_mixed_placement_per_recipient_both_seats_and_placer() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        types(&mut g, a, &["Goblin", "Orc", "Army"]);
        types(&mut g, b, &["Army"]);
        types(&mut g, enemy, &["Goblin"]);
        let ar = g.object_ref(a);
        let br = g.object_ref(b);
        g.effect_counters(
            vec![
                (
                    ar,
                    Counters::of(CounterKind::PlusOne, 2).with(CounterKind::Stun, 1),
                ),
                (br, Counters::of(CounterKind::Stun, 1)),
            ],
            ResumeMode::Pending,
        );
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        g.effect_counter_by(a, CounterKind::Stun, 0, p, ResumeMode::Pending);
        g.effect_counter_by(a, CounterKind::Stun, 1, p.other(), ResumeMode::Pending);
        g.effect_counter_by(enemy, CounterKind::Stun, 1, p, ResumeMode::Pending);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        g.effect_counter_by(source, CounterKind::Stun, 1, p, ResumeMode::Pending);
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn great_goblin_other_deaths_use_lki_and_public_exact_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, victim, &["Goblin", "Orc"]);
        let top = put(&mut g, p, "Shock", Zone::Library);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
        g.move_object(victim, Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        assert!(g.observe(p.other()).objects.iter().any(|o| o.id == top));
        assert_eq!(g.state.exile_permissions.len(), 1);
    }
}

#[test]
fn great_goblin_noncreature_kindred_and_simultaneous_source_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        // Boggart Mischief's unsupported abilities are irrelevant to this fixture:
        // its printed kindred Goblin enchantment characteristics are the event subject.
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].card = Card::from_name("Boggart Mischief").unwrap();
        assert!(g.has_subtype(victim, "Goblin"));
        assert!(!g.is_creature(victim));
        let top = put(&mut g, p, "Shock", Zone::Library);
        let observers = g.death_trigger_sources();
        let events = g.move_objects_raw(&[source, victim], Zone::Graveyard);
        g.finish_zone_events(&events, &observers);
        finish(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        assert!(g.can_play_exiled(p, top));
        assert!(!g.can_play_exiled(p.other(), top));
    }
}
#[test]
fn great_goblin_actual_opponent_amass_uses_placing_player_and_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "The Great Goblin", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let azog = put(&mut g, p.other(), "Azog, Moria's Ruin", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0, 0, 1, 0, 0, 2];
        cast(&mut g, azog, None);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(
            g.state
                .objects
                .iter()
                .enumerate()
                .any(|(id, o)| o.zone == Zone::Battlefield
                    && o.controller == p
                    && g.has_subtype(id as CardId, "Army")
                    && g.counter_count(id as CardId, CounterKind::PlusOne) == 2)
        );
    }
}

fn next_main(g: &mut Game) {
    let turn = g.state.turn;
    for _ in 0..100 {
        if g.state.turn != turn && g.state.step == Step::Main1 && quiet(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("Great Goblin next turn budget");
}
#[test]
fn great_goblin_replacement_multiplicity_targeting_and_owned_branch() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p);
        let recipient = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, source, &["Goblin", "Dwarf"]);
        types(&mut g, recipient, &["Orc"]);
        put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(
            &mut g,
            p,
            "Doc Samson, Super Psychiatrist",
            Zone::Battlefield,
        );
        g.refresh_enduring_stories();
        assert_eq!(g.object_view(source).additional_triggers, 1);
        g.effect_counter_by(recipient, CounterKind::PlusOne, 1, p, ResumeMode::Pending);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
                )
            });
        }
        assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 2);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        let mut blocked = scenario(p);
        put(&mut blocked, p, "The Great Goblin", Zone::Battlefield);
        let goblin = put(&mut blocked, p, "Bear Cub", Zone::Battlefield);
        types(&mut blocked, goblin, &["Goblin"]);
        blocked.state.players[p.other().index()].hexproof_until_end_turn = true;
        blocked.effect_counter_by(goblin, CounterKind::Stun, 1, p, ResumeMode::Pending);
        finish(&mut blocked);
        assert_eq!(blocked.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn great_goblin_permission_next_turn_stale_and_token_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(&mut g, p);
        let victim = put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        types(&mut g, victim, &["Goblin"]);
        let top = put(&mut g, p, "Bear Cub", Zone::Library);
        g.move_object(victim, Zone::Graveyard);
        finish(&mut g);
        assert!(g.can_play_exiled(p, top));
        let mut stale = g.clone();
        stale.move_object(top, Zone::Hand);
        stale.move_object(top, Zone::Exile);
        assert!(!stale.can_play_exiled(p, top));
        next_main(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(g.can_play_exiled(p, top));
        next_main(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.can_play_exiled(p, top));
        next_main(&mut g);
        assert!(!g.can_play_exiled(p, top));
    }
}

#[test]
fn great_goblin_replaced_death_does_not_exile_library_and_paid_land_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(&mut g, p);
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, victim, &["Orc"]);
        let top = put(&mut g, p, "Forest", Zone::Library);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, murder, Some(Target::Card(victim)));
        finish(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert_eq!(g.object_view(top).zone, Zone::Library);
        assert!(!g.can_play_exiled(p, top));

        let mut g = scenario(p);
        paid(&mut g, p);
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, victim, &["Army"]);
        let top = put(&mut g, p, "Forest", Zone::Library);
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, murder, Some(Target::Card(victim)));
        finish(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert!(g.can_play_exiled(p, top));
        act(&mut g, Action::PlayLand { card: top });
        assert_eq!(g.object_view(top).zone, Zone::Battlefield);
        let other = put(&mut g, p, "Forest", Zone::Hand);
        reject(&mut g, Action::PlayLand { card: other });
    }
}

#[test]
fn great_goblin_hybrid_costs_and_zero_post_replacement_event() {
    for p in [Player::P0, Player::P1] {
        for mana in [[0, 0, 2, 0, 0, 1], [0, 0, 0, 2, 0, 1], [0, 0, 1, 1, 0, 1]] {
            let mut g = scenario(p);
            let id = put(&mut g, p, "The Great Goblin", Zone::Hand);
            g.state.players[p.index()].mana = mana;
            cast(&mut g, id, None);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(stats(&g, id), (3, 2));
            let zero = Counters::default();
            let reference = g.object_ref(id);
            g.counter_batch_added(std::iter::once((reference, &zero, p)));
            assert!(g.state.pending_triggers.is_empty());
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
