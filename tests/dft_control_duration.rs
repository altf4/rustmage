//! Linked control and combat restriction share an exact continuous-control duration.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const STEAL: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::GainControlAndPreventCombatWhileSourceControlled],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const CONTROL_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Control,
        fragments: &[StaticAbilityFragment::ControlAttached],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            1, 0, 0, 0, 0, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
fn steal(g: &mut Game, source: CardId, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
fn ready_attack(g: &mut Game, id: CardId) -> bool {
    let p = g.object_view(id).controller;
    g.state.objects[id as usize].summoning_sick = false;
    g.state.objects[id as usize].tapped = false;
    g.can_declare_attacker(p, id)
}

#[test]
fn successful_link_transfers_control_and_forbids_attack_and_block_without_granting_abilities() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &STEAL)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        steal(&mut g, source, target);
        settle(&mut g);
        assert_eq!(g.object_view(target).controller, p);
        assert!(!ready_attack(&mut g, target));
        g.state.active = p.other();
        assert!(!g.can_be_blocker(target));
        assert!(g.printed_abilities_available(target));
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.object_view(target).controller, p.other());
        assert!(ready_attack(&mut g, target));
    }
}

#[test]
fn source_already_lost_or_not_controlled_at_resolution_never_starts_either_effect() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &STEAL)]);
    for leave in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        steal(&mut g, source, target);
        if leave {
            g.move_object(source, Zone::Graveyard);
        } else {
            g.record_control(g.object_ref(source), Player::P1, None);
        }
        settle(&mut g);
        assert_eq!(g.object_view(target).controller, Player::P1);
        assert!(ready_attack(&mut g, target));
    }
}

#[test]
fn source_control_loss_ends_duration_irreversibly_even_if_source_returns_to_same_controller() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &STEAL)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    steal(&mut g, source, target);
    settle(&mut g);
    g.record_control(g.object_ref(source), Player::P1, None);
    assert_eq!(g.object_view(target).controller, Player::P1);
    g.record_control(g.object_ref(source), Player::P0, None);
    assert_eq!(g.object_view(target).controller, Player::P1);
    assert!(ready_attack(&mut g, target));
    assert!(g.source_linked_effect_views().is_none());
}

#[test]
fn source_blanking_preserves_resolved_link_and_target_blink_ends_it_without_rebinding() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &STEAL)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    steal(&mut g, source, target);
    settle(&mut g);
    // Existing source-independent keyword/ability-removal effect, without changing control.
    let reference = g.object_ref(source);
    g.state
        .roles
        .as_mut()
        .unwrap()
        .source_linked_effects
        .push(SourceLinkedEffectView {
            source: reference,
            recipient: reference,
            controller: Player::P0,
            kind: SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn,
            timestamp: u64::MAX - 1,
        });
    assert_eq!(g.object_view(target).controller, Player::P0);
    assert!(!ready_attack(&mut g, target));
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    assert_eq!(g.object_view(target).controller, Player::P1);
    assert!(ready_attack(&mut g, target));
}

#[test]
fn newer_target_control_can_override_link_but_combat_lock_remains_until_source_duration_ends() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &STEAL)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    steal(&mut g, source, target);
    settle(&mut g);
    g.record_control(g.object_ref(target), Player::P1, None);
    assert_eq!(g.object_view(target).controller, Player::P1);
    assert!(!ready_attack(&mut g, target));
    g.move_object(source, Zone::Hand);
    assert_eq!(g.object_view(target).controller, Player::P1);
    assert!(ready_attack(&mut g, target));
}

#[test]
fn control_projection_applies_source_change_before_dependent_link_when_aura_is_sacrificed() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &STEAL),
        (Card::from_name("Stop Cold").unwrap(), &CONTROL_AURA),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Stop Cold", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
    g.refresh_controllers();
    assert_eq!(g.object_view(source).controller, Player::P0);
    let target = put(&mut g, Player::P1, "Llanowar Elves", Zone::Battlefield);
    steal(&mut g, source, target);
    settle(&mut g);
    assert_eq!(g.object_view(target).controller, Player::P0);
    g.state.players[0].mana = [0; 6];
    let goal = FundingGoal {
        actor: Player::P0,
        required: [1, 0, 0, 0, 0, 0],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&g, &goal);
    let step = world
        .moves(&goal)
        .into_iter()
        .find(|s| world.resources[s.resource].id == aura)
        .unwrap();
    let mut visited = false;
    world.visit_applied(&step, &mut |next| {
        assert_eq!(next.object(source).controller, Player::P1);
        assert_eq!(next.object(target).controller, Player::P1);
        assert!(
            !next
                .roles()
                .unwrap()
                .source_linked_effects
                .iter()
                .any(|r| r.kind == SourceLinkedEffectKind::ControlAndCombatLockWhileControlled)
        );
        visited = true;
        false
    });
    assert!(visited);
    choose(
        &mut g,
        |action| matches!(action, Action::ActivateMana { card, ability: 0, .. } if *card == aura),
    );
    assert_eq!(g.object_view(source).controller, Player::P1);
    assert_eq!(g.object_view(target).controller, Player::P1);
    assert_eq!(g.state.players[0].mana[0], 1);
}

#[test]
fn link_duration_state_is_owned_across_children_and_restore() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &STEAL)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    steal(&mut g, source, target);
    settle(&mut g);
    let saved = g.snapshot();
    let mut child = g.clone();
    child.record_control(child.object_ref(source), Player::P1, None);
    assert_eq!(child.object_view(target).controller, Player::P1);
    assert_eq!(g.object_view(target).controller, Player::P0);
    child.restore(&saved).unwrap();
    drop(g);
    assert_eq!(child.object_view(target).controller, Player::P0);
    assert!(!ready_attack(&mut child, target));
}
