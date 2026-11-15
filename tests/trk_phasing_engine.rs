//! Phasing changes status without a zone event or a new object incarnation.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PHASE_ALL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::PhaseOutAll(1 | 16)]),
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Any,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 11)],
        },
    ],
    ..CardRules::EMPTY
};
const STOP_PHASE_IN: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PermanentsCantPhaseIn],
    }],
    ..CardRules::EMPTY
};
const AURA: CardRules = CardRules {
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
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &PHASE_ALL),
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
        (Card::DrakeToken, &STOP_PHASE_IN),
        (Card::from_name("Pacifism").unwrap(), &AURA),
    ])
}

#[test]
fn mass_phasing_preserves_id_attachment_and_counters_but_excludes_targets_and_triggers() {
    let _rules = rules();
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let equipment = put(&mut g, p, "Short Bow", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(equipment), g.object_ref(creature)));
    g.state.objects[creature as usize].counters.plus_one = 2;
    g.state.objects[creature as usize].tapped = true;
    let before = g.object_ref(creature);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    settle(&mut g);
    for id in [creature, other, equipment] {
        assert_eq!(g.object_view(id).zone, Zone::PhasedOut);
        assert_eq!(g.object_ref(id).generation, 0);
    }
    assert_eq!(g.object_ref(creature), before);
    assert!(g.state.objects[equipment as usize].phased_out_indirectly);
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 2);
    assert_eq!(
        g.state.objects[equipment as usize].attached_to,
        Some(before)
    );
    for viewer in [p, p.other()] {
        let observed = g.observe(viewer);
        let visible = observed.objects.iter().find(|o| o.id == creature).unwrap();
        assert_eq!(visible.zone, Zone::Battlefield);
        assert!(visible.phased_out);
    }
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: shock });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 23);
    assert_eq!(g.state.players[p.other().index()].life, 18);

    g.begin_turn();
    assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(g.object_view(equipment).zone, Zone::Battlefield);
    assert_eq!(g.object_view(other).zone, Zone::PhasedOut);
    assert_eq!(g.object_ref(creature), before);
    assert_eq!(
        g.state.objects[equipment as usize].attached_to,
        Some(before)
    );
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 2);
    assert!(!g.state.objects[creature as usize].tapped);
    assert!(!g.state.objects[creature as usize].summoning_sick);
    g.state.active = p.other();
    g.begin_turn();
    assert_eq!(g.object_view(other).zone, Zone::Battlefield);
}

#[test]
fn a_live_rule_prevents_phase_in_until_the_following_actual_untap() {
    let _rules = rules();
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let reference = g.object_ref(creature);
    g.phase_out_batch(&[creature]);
    g.create_tokens(p, Card::DrakeToken, 1, false);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.card == Card::DrakeToken && o.zone == Zone::Battlefield)
        .unwrap()
        .0 as CardId;
    g.begin_turn();
    assert_eq!(g.object_view(creature).zone, Zone::PhasedOut);
    assert_eq!(g.object_ref(creature), reference);
    g.move_object(token, Zone::Graveyard);
    g.state_based_actions();
    g.begin_turn();
    assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(g.object_ref(creature), reference);
}

#[test]
fn phase_event_survives_raw_index_and_owned_restore() {
    let _rules = rules();
    let p = Player::P0;
    let mut raw = scenario(p);
    put(&mut raw, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut raw, p, "Opt", Zone::Hand);
    let action = raw
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Cast { card, .. } if *card == spell))
        .unwrap();
    let index = raw
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = raw.clone();
    indexed.apply_index(p, raw.revision(), index).unwrap();
    act(&mut raw, action);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let checkpoint = raw.snapshot();
    let mut restored = raw.clone();
    restored.begin_turn();
    restored.restore(&checkpoint).unwrap();
    assert_eq!(restored.snapshot(), checkpoint);
    raw.begin_turn();
    restored.begin_turn();
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn phased_token_survives_state_actions_and_returns_with_its_identity() {
    let _rules = rules();
    let p = Player::P0;
    let mut g = scenario(p);
    g.create_tokens(p, Card::DrakeToken, 1, false);
    let token = g
        .state
        .objects
        .iter()
        .position(|o| o.card == Card::DrakeToken && o.zone == Zone::Battlefield)
        .unwrap() as CardId;
    let reference = g.object_ref(token);
    g.phase_out_batch(&[token]);
    g.state_based_actions();
    assert_eq!(g.object_view(token).zone, Zone::PhasedOut);
    assert_eq!(g.object_ref(token), reference);
    g.begin_turn();
    assert_eq!(g.object_view(token).zone, Zone::Battlefield);
    assert_eq!(g.object_ref(token), reference);
}

#[test]
fn direct_attachments_return_unattached_after_the_host_leaves() {
    let _rules = rules();
    let p = Player::P0;
    let mut equipment_game = scenario(p);
    let host = put(&mut equipment_game, p, "Hill Giant", Zone::Battlefield);
    let equipment = put(&mut equipment_game, p, "Short Bow", Zone::Battlefield);
    let host_ref = equipment_game.object_ref(host);
    assert!(equipment_game.attach_equipment(equipment_game.object_ref(equipment), host_ref));
    equipment_game.phase_out_batch(&[equipment]);
    equipment_game.move_object(host, Zone::Graveyard);
    equipment_game.begin_turn();
    assert_eq!(
        equipment_game.object_view(equipment).zone,
        Zone::Battlefield
    );
    assert_eq!(
        equipment_game.state.objects[equipment as usize].attached_to,
        None
    );

    let mut aura_game = scenario(p);
    let host = put(&mut aura_game, p, "Hill Giant", Zone::Battlefield);
    let aura = spawn(
        &mut aura_game,
        p,
        Card::from_name("Pacifism").unwrap(),
        Zone::Battlefield,
    );
    aura_game.state.objects[aura as usize].attached_to = Some(aura_game.object_ref(host));
    aura_game.phase_out_batch(&[aura]);
    aura_game.move_object(host, Zone::Graveyard);
    aura_game.begin_turn();
    assert_eq!(aura_game.state.objects[aura as usize].attached_to, None);
    aura_game.state_based_actions();
    assert_eq!(aura_game.object_view(aura).zone, Zone::Graveyard);
}
