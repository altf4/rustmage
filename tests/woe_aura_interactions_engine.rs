//! Grouped Aura targets, exact attached sacrifice payments and live attachment scaling.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OPPONENT_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const TAKEDOWN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::creature_you_control(),
                    predicate: &TargetPredicate::Enchanted,
                },
                min: 0,
                max: usize::MAX,
            },
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::creature_you_control(),
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                },
                min: 0,
                max: 1,
            },
            TargetSpec::one(OPPONENT_CREATURE),
        ]),
        &[Instruction::GroupsPowerDamage {
            sources: &[0, 1],
            recipient: 2,
        }],
    ),
    ..CardRules::EMPTY
};
const TROLL: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::SacrificeAttachedPermanent,
                filter: SacrificeFilter {
                    kinds: 32,
                    subtype: Some("Aura"),
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(OPPONENT_CREATURE)),
        program: &[Instruction::ExileIfDiesThisTurn, Instruction::FightSource],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const LEADER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupScaling {
            filters: &[CreatureFilter {
                relation: 0,
                other: true,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            }],
            value: StatValue::SourceAuraOrEquipmentCount,
            stats: Stats {
                power: 1,
                toughness: 0,
            },
        }],
    }],
    ..CardRules::EMPTY
};

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn cast_aura(g: &mut Game, player: Player, recipient: CardId) -> CardId {
    let aura = put(g, player, "Bestial Bloodline", Zone::Hand);
    cast(g, aura, Some(Target::Card(recipient)));
    settle(g);
    assert_eq!(
        g.object_view(aura).attached_to,
        Some(g.object_ref(recipient))
    );
    aura
}

#[test]
fn two_friendly_target_groups_share_one_damage_batch_and_are_revalidated() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &TAKEDOWN)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut g = scenario(player);
        let first = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let second = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let third = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let opponent = put(&mut g, other, "Hill Giant", Zone::Battlefield);
        g.state.objects[opponent as usize].growth.toughness = 15;
        let aura = cast_aura(&mut g, player, first);
        cast_aura(&mut g, player, second);
        let spell = put(&mut g, player, "Divination", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, target(third));
        act(&mut g, target(first));
        act(&mut g, target(second));
        act(&mut g, Action::FinishTargets);
        reject(&mut g, target(first));
        act(&mut g, target(third));
        act(&mut g, target(opponent));
        pay(&mut g);

        let mut partial = g.clone();
        act(&mut partial, Action::Pass);
        let disenchant = put(&mut partial, other, "Disenchant", Zone::Hand);
        cast(&mut partial, disenchant, Some(Target::Card(aura)));
        settle(&mut partial);
        assert_eq!(partial.object_view(first).power, 3);
        assert_eq!(partial.object_view(opponent).damage, 6);

        let mut missing = g.clone();
        act(&mut missing, Action::Pass);
        let bounce = put(&mut missing, other, "Unsummon", Zone::Hand);
        cast(&mut missing, bounce, Some(Target::Card(opponent)));
        settle(&mut missing);
        assert_eq!(missing.object_view(opponent).zone, Zone::Hand);
        assert_eq!(missing.object_view(first).damage, 0);

        settle(&mut g);
        assert_eq!(g.object_view(opponent).damage, 11);
    }
}

#[test]
fn an_attached_sacrifice_requires_the_payers_aura_on_the_exact_source() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &TROLL)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut g = scenario(player);
        let source = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let friend = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, other, "Bear Cub", Zone::Battlefield);
        let wrong_attachment = cast_aura(&mut g, player, friend);
        g.state.active = other;
        g.state.priority = other;
        let wrong_controller = cast_aura(&mut g, other, source);
        g.state.active = player;
        g.state.priority = player;
        reject(&mut g, activate(source));
        let payable = cast_aura(&mut g, player, source);
        assert_eq!(g.object_view(source).power, 8);
        act(&mut g, activate(source));
        act(&mut g, target(enemy));
        for object in [wrong_attachment, wrong_controller].map(|id| g.object_ref(id)) {
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(payable);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(payable).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).power, 6);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
        assert_eq!(g.object_view(source).damage, 2);
        assert_eq!(g.object_view(wrong_attachment).zone, Zone::Battlefield);
        assert_eq!(g.object_view(wrong_controller).zone, Zone::Battlefield);
    }
}

#[test]
fn aura_and_equipment_count_for_other_controlled_creatures_independent_of_attachment_owner() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &LEADER)]);
    let player = Player::P0;
    let mut g = scenario(player);
    let leader = put(&mut g, player, "Hill Giant", Zone::Battlefield);
    let friend = put(&mut g, player, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
    let bow = put(&mut g, player, "Short Bow", Zone::Battlefield);
    act(&mut g, activate(bow));
    act(&mut g, target(leader));
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(friend).power, 3);

    g.state.active = player.other();
    g.state.priority = player.other();
    let aura = cast_aura(&mut g, player.other(), leader);
    assert_eq!(g.object_view(leader).power, 5);
    assert_eq!(g.object_view(friend).power, 4);
    assert_eq!(g.object_view(friend).toughness, 2);
    assert_eq!(g.object_view(foe).power, 2);

    let removal = put(&mut g, player.other(), "Disenchant", Zone::Hand);
    cast(&mut g, removal, Some(Target::Card(aura)));
    settle(&mut g);
    assert_eq!(g.object_view(friend).power, 3);
}
