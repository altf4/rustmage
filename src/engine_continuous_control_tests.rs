//! Actual layer-two dependency/timestamp interactions; gated printed definitions
//! are used only as immutable static providers, never admitted deck cards.
use super::ability_timeline_tests::{checked, setup};
use super::tests::spawn;
use super::*;
fn aura(g: &mut Game, player: Player, target: CardId) -> CardId {
    let source = spawn(
        g,
        player,
        Card::from_name("Confiscate").unwrap(),
        Zone::Battlefield,
    );
    g.state.objects[source as usize].attached_to = Some(g.object_ref(target));
    g.retimestamp_live_static_source(g.object_ref(source));
    g.refresh_controllers();
    source
}
fn transfer(g: &mut Game, source: CardId, target: CardId, temporary: bool) {
    let player = g.state.objects[source as usize].controller;
    let mut item = g.activation_item(player, source, 0);
    item.targets.push(g.bind_target(Target::Card(target)));
    if temporary {
        g.gain_control_until_end_turn(&item);
    } else {
        g.gain_control(&item, Subject::Targets, PlayerSet::Controller);
    }
}
#[test]
fn control_aura_removal_restores_base_owner_is_unchanged_and_old_incarnation_isolated() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let source = aura(&mut g, p, victim);
        assert_eq!(g.object_view(victim).controller, p);
        assert_eq!(g.object_view(victim).owner, p.other());
        assert!(g.object_view(victim).summoning_sick);
        checked(&mut g, Action::Pass);
        g.move_object(source, Zone::Hand);
        assert_eq!(g.object_view(victim).controller, p.other());
        g.move_object(victim, Zone::Graveyard);
        g.move_object(victim, Zone::Battlefield);
        g.refresh_controllers();
        assert_eq!(g.object_view(victim).controller, p.other());
        assert!(g.state.controls.is_none());
    }
}
#[test]
fn layered_control_preserves_fixed_and_temporary_effects_on_both_sides_of_aura() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let actor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        transfer(&mut g, actor, victim, true);
        let source = aura(&mut g, p.other(), victim);
        assert_eq!(g.object_view(victim).controller, p.other());
        g.expire_temporary_controls();
        assert_eq!(g.object_view(victim).controller, p.other());
        transfer(&mut g, actor, victim, false);
        assert_eq!(g.object_view(victim).controller, p);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.object_view(victim).controller, p);
        let second = aura(&mut g, p.other(), victim);
        transfer(&mut g, actor, victim, true);
        assert_eq!(g.object_view(victim).controller, p);
        g.expire_temporary_controls();
        assert_eq!(g.object_view(victim).controller, p.other());
        g.move_object(second, Zone::Graveyard);
        assert_eq!(g.object_view(victim).controller, p);
    }
}
#[test]
fn controlling_a_control_aura_is_a_dependency_regardless_of_timestamp() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let first = aura(&mut g, p, victim);
        let second = aura(&mut g, p.other(), first);
        assert_eq!(g.object_view(first).controller, p.other());
        assert_eq!(g.object_view(victim).controller, p.other());
        // Retimestamp the recipient's effect to make the opposite ordering;
        // dependency must still apply the source's control change first.
        g.retimestamp_live_static_source(g.object_ref(first));
        g.refresh_controllers();
        assert_eq!(g.object_view(victim).controller, p.other());
        let snapshot = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.move_object(second, Zone::Graveyard);
        assert_eq!(child.object_view(victim).controller, p);
        assert_eq!(g.snapshot(), snapshot);
        g.move_object(second, Zone::Graveyard);
        assert_eq!(g.snapshot(), child.snapshot());
        checked(&mut g, Action::Pass);
    }
}
#[test]
fn layer_two_survives_later_ability_removal_and_dependency_loops_use_timestamps() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let first = aura(&mut g, p, victim);
        let blank = spawn(
            &mut g,
            p.other(),
            Card::from_name("Imprisoned in the Moon").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[blank as usize].attached_to = Some(g.object_ref(first));
        g.retimestamp_live_static_source(g.object_ref(blank));
        g.refresh_controllers();
        assert!(!g.printed_abilities_available(first));
        assert_eq!(g.object_view(victim).controller, p);
        g.move_object(blank, Zone::Hand);
        let second = aura(&mut g, p.other(), first);
        g.state.objects[first as usize].attached_to = Some(g.object_ref(second));
        g.retimestamp_live_static_source(g.object_ref(first));
        g.refresh_controllers();
        assert_eq!(g.object_view(first).controller, p.other());
        assert_eq!(g.object_view(second).controller, p.other());
        // Repeated evaluation must not derive the baseline from its own cache.
        g.refresh_controllers();
        assert_eq!(g.object_view(first).controller, p.other());
        assert_eq!(g.object_view(second).controller, p.other());
    }
}
#[test]
fn simultaneous_aura_departure_materializes_only_final_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let older = aura(&mut g, p.other(), victim);
        let newer = aura(&mut g, p, victim);
        g.state.objects[victim as usize].summoning_sick = false;
        g.state.attackers.push(Attacker {
            card: victim,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        let events = g.move_objects_raw(&[newer, older], Zone::Graveyard);
        assert_eq!(events[0].before.controller, p);
        assert_eq!(events[1].before.controller, p.other());
        assert_eq!(g.object_view(victim).controller, p);
        assert!(!g.object_view(victim).summoning_sick);
        assert!(g.state.attackers.iter().any(|a| a.card == victim));
    }
}
fn destiny(g: &mut Game, owner: Player, target: CardId) -> CardId {
    let source = spawn(
        g,
        owner,
        Card::from_name("Angelic Destiny").unwrap(),
        Zone::Battlefield,
    );
    g.state.objects[source as usize].attached_to = Some(g.object_ref(target));
    g.retimestamp_live_static_source(g.object_ref(source));
    source
}
#[test]
fn destiny_adds_angel_stats_and_keywords_then_returns_to_its_owner_after_aura_sba() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let source = destiny(&mut g, p.other(), victim);
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert!(
            g.keywords(victim)
                .contains(Keywords::FLYING.union(Keywords::FIRST_STRIKE))
        );
        assert!(g.has_subtype(victim, "Angel"));
        assert!(g.has_subtype(victim, "Bear"));
        let actor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        transfer(&mut g, actor, source, false);
        g.move_object(victim, Zone::Graveyard);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.state.pending_triggers[0].controller, p);
        assert_eq!(g.state.pending_triggers[0].source_move, None);
        g.state_based_actions();
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(
            g.state.pending_triggers[0]
                .source_move
                .as_ref()
                .unwrap()
                .after,
            g.object_ref(source)
        );
        super::ability_timeline_tests::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
        assert_eq!(g.object_view(source).owner, p.other());
        assert_eq!(g.object_view(source).controller, p.other());
    }
}
#[test]
fn destiny_simultaneous_death_binds_actual_source_move_in_either_event_order() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let victim = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            let source = destiny(&mut g, p, victim);
            let sources = g.death_trigger_sources();
            let ids = if reverse {
                [source, victim]
            } else {
                [victim, source]
            };
            let events = g.move_objects_raw(&ids, Zone::Graveyard);
            g.finish_zone_events(&events, &sources);
            assert_eq!(g.state.pending_triggers.len(), 1);
            let captured = g.state.pending_triggers[0].clone();
            assert_eq!(
                captured.source_attachment,
                Some(
                    events
                        .iter()
                        .find(|e| e.before.id == victim)
                        .unwrap()
                        .before
                        .reference
                )
            );
            assert_eq!(
                captured.source_move.as_ref().unwrap().after,
                g.object_ref(source)
            );
            super::ability_timeline_tests::settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Hand);
        }
    }
}
#[test]
fn destiny_does_not_rebind_a_second_source_move_even_if_it_returns_to_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let source = destiny(&mut g, p, victim);
        g.move_object(victim, Zone::Graveyard);
        g.state_based_actions();
        let captured = g.state.pending_triggers[0].source_move.clone();
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.state.pending_triggers[0].source_move, captured);
        super::ability_timeline_tests::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}
#[test]
fn destiny_first_public_exile_destination_uses_actual_event_and_owned_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let source = destiny(&mut g, p.other(), victim);
        let observers = g.death_trigger_sources();
        let old_source = g.object_view(source);
        let old_victim = g.object_view(victim);
        // Frozen simultaneous movement fixture, including a replaced Aura
        // destination. Bind the actual event, never a guessed generation.
        let first = g.transition_object(old_source, Zone::Exile, false);
        let second = g.transition_object(old_victim, Zone::Graveyard, false);
        g.finish_zone_events(&[first, second], &observers);
        assert_eq!(
            g.state.pending_triggers[0]
                .source_move
                .as_ref()
                .unwrap()
                .destination,
            Zone::Exile
        );
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        super::ability_timeline_tests::settle(&mut branch);
        super::ability_timeline_tests::settle(&mut g);
        assert_eq!(g.snapshot(), branch.snapshot());
        assert_eq!(g.object_view(source).zone, Zone::Hand);
    }
}
#[test]
fn destiny_departed_early_or_creature_exiled_does_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let source = destiny(&mut g, p, victim);
        g.move_object(source, Zone::Graveyard);
        g.move_object(victim, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
        let other = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let aura = destiny(&mut g, p, other);
        g.move_object(other, Zone::Exile);
        g.state_based_actions();
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn no_op_assignment_does_not_invent_a_three_aura_dependency_cycle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = Card::from_name("Confiscate").unwrap();
        let a = spawn(&mut g, p, card, Zone::Battlefield);
        let b = spawn(&mut g, p.other(), card, Zone::Battlefield);
        let c = spawn(&mut g, p.other(), card, Zone::Battlefield);
        for (source, target) in [(a, b), (b, c), (c, a)] {
            g.state.objects[source as usize].attached_to = Some(g.object_ref(target));
            g.retimestamp_live_static_source(g.object_ref(source));
        }
        // A(P)->B, B(Q)->C, C(Q)->A, timestamps A/B/C. B assigning Q
        // to C(Q) changes nothing: C is independent of B (613.8a), so
        // C applies before A, then B. Reevaluate after every application
        // (613.8c); a pointer-only graph incorrectly calls this a loop.
        g.refresh_controllers();
        for id in [a, b, c] {
            assert_eq!(g.object_view(id).controller, p.other());
        }
        g.refresh_controllers();
        for id in [a, b, c] {
            assert_eq!(g.object_view(id).controller, p.other());
        }
        checked(&mut g, Action::Pass);
    }
}
#[test]
fn genuine_four_aura_cycle_uses_timestamp_then_reevaluates_changed_dependencies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = Card::from_name("Confiscate").unwrap();
        let a = spawn(&mut g, p, card, Zone::Battlefield);
        let b = spawn(&mut g, p.other(), card, Zone::Battlefield);
        let c = spawn(&mut g, p, card, Zone::Battlefield);
        let d = spawn(&mut g, p.other(), card, Zone::Battlefield);
        for (source, target) in [(a, b), (d, a), (c, d), (b, c)] {
            g.state.objects[source as usize].attached_to = Some(g.object_ref(target));
            g.retimestamp_live_static_source(g.object_ref(source));
        }
        // All four initial assignments change the next source's controller.
        // 613.8b breaks that real loop with earliest timestamp A. Once B's
        // controller becomes P, its assignment to C(P) is now a no-op; the
        // remaining dependency relations must change under 613.8c.
        g.refresh_controllers();
        for id in [a, b, c, d] {
            assert_eq!(g.object_view(id).controller, p);
        }
        let snapshot = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.refresh_controllers();
        assert_eq!(branch.snapshot(), snapshot);
        checked(&mut g, Action::Pass);
    }
}
