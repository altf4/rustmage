//! Live static type effects and shared Aura eligibility use one characteristic world.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const VEHICLE_AURA_SPEC: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 1,
            toughness: 2,
        },
        keywords: Keywords::REACH,
    },
};
const VEHICLE_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::CreatureOrVehicle { relation: -1 },
    ))),
    aura: Some(VEHICLE_AURA_SPEC),
    keywords: Keywords::FLASH,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 2,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::REACH),
        ],
    }],
    extras: &CardExtras {
        aura: Some(VEHICLE_AURA_SPEC),
        aura_kind_or_subtype: Some("Vehicle"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const VEHICLE_ANIMATION: AnimationSpec = AnimationSpec {
    artifact: true,
    creature_types: AnimationCreatureTypes::Unchanged,
    base: None,
    keywords: Keywords::NONE,
    duration: AnimationDuration::EndOfTurn,
};
fn animate(g: &mut Game, id: CardId) {
    g.change_object_form(
        g.object_ref(id),
        VEHICLE_ANIMATION,
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
}
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}

#[test]
fn aura_can_enchant_uncrewed_vehicle_and_its_keyword_survives_animation_cleanup() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    assert!(!g.is_creature(vehicle));
    assert!(g.aura_can_enchant(
        Card::from_name("Pacifism").unwrap(),
        Player::P0,
        g.object_ref(vehicle)
    ));
    cast(&mut g, aura, Some(Target::Card(vehicle)));
    settle(&mut g);
    assert_eq!(
        g.state.objects[aura as usize].attached_to,
        Some(g.object_ref(vehicle))
    );
    assert!(g.keywords(vehicle).contains(Keywords::REACH));
    animate(&mut g, vehicle);
    assert_eq!(stats(&g, vehicle), (5, 4));
    assert!(g.expire_animations());
    g.state_based_actions();
    assert!(!g.is_creature(vehicle));
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Battlefield);
    assert!(g.keywords(vehicle).contains(Keywords::REACH));
}

#[test]
fn aura_resolution_rechecks_the_live_union_and_sba_rejects_a_later_ineligible_form() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P1, "Veloheart Bike", Zone::Battlefield);
    animate(&mut g, vehicle);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(vehicle)));
    g.expire_animations();
    settle(&mut g);
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Battlefield);
    assert!(g.keywords(vehicle).contains(Keywords::REACH));
    let object = g.object_ref(vehicle);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .push(characteristic_layers::food_form(object, timestamp));
    assert!(!g.aura_can_enchant(Card::from_name("Pacifism").unwrap(), Player::P0, object));
    g.state_based_actions();
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
}

#[test]
fn aura_union_rejects_an_ordinary_noncreature_artifact_transactionally() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let artifact = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: aura });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(
        g.state.objects[aura as usize].attached_to,
        Some(g.object_ref(vehicle))
    );
}

#[test]
fn projected_aura_entry_accepts_the_same_noncreature_vehicle_as_actual_entry() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Graveyard);
    let saved = g.snapshot();
    let movement = g.entry_move(aura, Zone::Battlefield, false);
    let world = FundingState::from_game(&g, &goal(Player::P0));
    let mut projected = None;
    world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
        if next.object(aura).zone == Zone::Battlefield {
            assert_eq!(next.object(aura).attached_to, Some(next.reference(vehicle)));
            assert!(
                characteristic_query::CharacteristicQuery::in_world(next)
                    .keywords(vehicle)
                    .contains(Keywords::REACH)
            );
            projected = Some(next.object(aura).clone());
        }
        true
    });
    assert!(projected.is_some());
    assert_eq!(g.snapshot(), saved);
    let mut actual = g.clone();
    actual.stage_entries(vec![movement], None);
    assert!(actual.advance_entries());
    choose(
        &mut actual,
        |action| matches!(action, Action::ChooseAuraAttachment { object } if object.card == vehicle),
    );
    assert_eq!(actual.state.objects[aura as usize], projected.unwrap());
    assert!(actual.keywords(vehicle).contains(Keywords::REACH));
}
