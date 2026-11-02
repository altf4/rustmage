//! Conditional self permissions use the normal owned casting transaction.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SelfGraveyardCastIf(
            StatValue::AtLeast {
                value: &StatValue::ControllerSpeed,
                minimum: 4,
            },
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 1,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::VIGILANCE),
        ],
    },
];
const AURA_SPEC: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 1,
            toughness: 1,
        },
        keywords: Keywords::VIGILANCE,
    },
};
const AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::CreatureOrVehicle { relation: -1 },
    ))),
    aura: Some(AURA_SPEC),
    static_groups: GROUPS,
    extras: &CardExtras {
        aura: Some(AURA_SPEC),
        aura_kind_or_subtype: Some("Vehicle"),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn announce(id: CardId) -> Action {
    Action::AnnounceCast { card: id }
}
fn target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

#[test]
fn self_graveyard_cast_requires_owner_live_condition_timing_mana_and_target() {
    let _scope = fixture_rules::activate(&[(named("Pacifism"), &AURA)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Pacifism", Zone::Graveyard);
        // A graveyard controller is not casting authority or the predicate's player.
        g.state.objects[source as usize].controller = p.other();
        g.state.players[p.index()].speed = 3;
        g.state.players[p.other().index()].speed = 4;
        let vehicle = put(&mut g, p.other(), "Veloheart Bike", Zone::Battlefield);
        assert!(!g.has_graveyard_cast(p, source, GraveyardCastKind::Cast));
        reject(&mut g, announce(source));
        g.state.players[p.index()].speed = 4;
        assert!(g.has_graveyard_cast(p, source, GraveyardCastKind::Cast));
        assert!(!g.has_graveyard_cast(p, source, GraveyardCastKind::Play));
        assert!(!g.has_graveyard_cast(p, source, GraveyardCastKind::Flashback));
        assert!(g.legal_actions().contains(&announce(source)));
        g.state.active = p.other();
        g.state.priority = p.other();
        reject(&mut g, announce(source));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        reject(&mut g, announce(source));
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        reject(&mut g, announce(source));
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        let artifact = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        act(&mut g, announce(source));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        target(&mut g, vehicle);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(
            g.state.objects[source as usize].attached_to,
            Some(g.object_ref(vehicle))
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.keywords(vehicle).contains(Keywords::VIGILANCE));
        assert!(!g.is_creature(vehicle));
    }
}

#[test]
fn normal_graveyard_aura_payment_is_owned_and_does_not_install_flashback() {
    let _scope = fixture_rules::activate(&[(named("Pacifism"), &AURA)]);
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        parent.state.players[p.index()].speed = 4;
        parent.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        let source = put(&mut parent, p, "Pacifism", Zone::Graveyard);
        let old = parent.object_ref(source);
        let creature = put(&mut parent, p, "Grizzly Bears", Zone::Battlefield);
        act(&mut parent, announce(source));
        target(&mut parent, creature);
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let action = indexed
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let actor = indexed.actor().unwrap();
        let revision = indexed.revision();
        act(&mut raw, action);
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(
            raw.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Normal
        );
        assert!(!raw.state.objects[source as usize].flashback);
        drop(parent);
        settle(&mut raw);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(stats(&raw, creature), (3, 3));
        assert_ne!(raw.object_ref(source), old);
        raw.move_object(source, Zone::Graveyard);
        assert_eq!(raw.object_view(source).zone, Zone::Graveyard);
        raw.state.players[p.index()].speed = 3;
        reject(&mut raw, announce(source));
        raw.state.players[p.index()].speed = 4;
        assert!(raw.legal_actions().contains(&announce(source)));
        indexed.restore(&saved).unwrap();
        pay(&mut indexed);
        settle(&mut indexed);
        assert_eq!(indexed.object_view(source).zone, Zone::Battlefield);
        assert_eq!(stats(&indexed, creature), (3, 3));
    }
}

#[test]
fn conditional_permission_is_checked_at_start_and_never_leaks_to_other_zones_or_cards() {
    let _scope = fixture_rules::activate(&[(named("Pacifism"), &AURA)]);
    let p = Player::P0;
    let mut g = scenario(p);
    g.state.players[p.index()].speed = 4;
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
    let source = put(&mut g, p, "Pacifism", Zone::Graveyard);
    let other = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let target_id = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    reject(&mut g, announce(other));
    g.move_object(source, Zone::Exile);
    reject(&mut g, announce(source));
    g.move_object(source, Zone::Graveyard);
    act(&mut g, announce(source));
    target(&mut g, target_id);
    // Once the normal spell has been announced, loss of the origin predicate
    // does not rescind the already-used permission during its cost payment.
    g.state.players[p.index()].speed = 0;
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(stats(&g, target_id), (4, 4));
    let hand = put(&mut g, p, "Pacifism", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
    cast(&mut g, hand, Some(Target::Card(target_id)));
    settle(&mut g);
    assert_eq!(g.object_view(hand).zone, Zone::Battlefield);
    assert_eq!(stats(&g, target_id), (5, 5));
}
