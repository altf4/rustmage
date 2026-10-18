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
