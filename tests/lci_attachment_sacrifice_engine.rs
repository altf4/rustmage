//! An attack grant can sacrifice relative to its grantor and trigger from its bearer.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ANY_PERMANENT: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 2,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const TORCH: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentTrigger(0)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::GrantedAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::MaySacrificeForGrantedReflexive {
                filter: SacrificeFilter {
                    source_only: true,
                    ..ANY_PERMANENT
                },
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
            program: &[Instruction::Damage(2)],
        },
    ],
    ..CardRules::EMPTY
};
const BONUS: Stats = Stats {
    power: 3,
    toughness: 0,
};
const BLUNDERBUSS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentTrigger(0)],
        },
    ],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::GrantedAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::MaySacrificeForGrantedReflexive {
                filter: SacrificeFilter {
                    kinds: 16,
                    other: true,
                    ..ANY_PERMANENT
                },
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(CREATURE)),
            program: &[Instruction::DamageScaled(AmountValue::SourcePower)],
        },
    ],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: BONUS,
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const LIFELINK_ARTIFACT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK,
    ..CardRules::EMPTY
};
const LARGE_BLOCKER: CardRules = CardRules {
    effect: Effect::Creature(3, 12),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, bearer: ObjectRef, grantor: ObjectRef) {
    assert!(game.attach_equipment(grantor, bearer));
    pass_pair(game);
    pass_pair(game);
    act(game, Action::Attack { card: bearer.card });
    act(game, Action::FinishAttackers);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.kind, StackKind::TriggeredAbility { ability: 0 });
    assert_eq!(trigger.source.unwrap().object, bearer);
    assert_eq!(
        trigger.attachment_trigger.as_ref().unwrap().grantor,
        grantor
    );
    assert!(trigger.targets.is_empty());
}

fn choose_reflexive_target(game: &mut Game, bearer: ObjectRef, target: Target) {
    act(game, Action::ChooseTarget { target });
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.kind, StackKind::TriggeredAbility { ability: 1 });
    assert_eq!(trigger.source.unwrap().object, bearer);
    assert_eq!(trigger.targets[0].target, target);
}

#[test]
fn exact_grantor_is_optional_and_can_be_sacrificed_after_reattachment() {
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &TORCH),
        (named("Gingerbrute"), &LIFELINK_ARTIFACT),
    ]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut root = scenario(player);
        put(&mut root, other, "Rest in Peace", Zone::Battlefield);
        let gear = put(&mut root, player, "Short Bow", Zone::Battlefield);
        let second = put(&mut root, player, "Short Bow", Zone::Battlefield);
        let attacker = put(&mut root, player, "Gingerbrute", Zone::Battlefield);
        let new_bearer = put(&mut root, other, "Bear Cub", Zone::Battlefield);
        let grantor = root.object_ref(gear);
        let bearer = root.object_ref(attacker);
        attack(&mut root, bearer, grantor);

        let mut stolen = root.clone();
        stolen.record_control(grantor, other, None);
        pass_pair(&mut stolen);
        assert_eq!(stolen.state.pending, Pending::Priority);
        assert!(stolen.state.stack.is_empty());
        assert_eq!(stolen.object_view(gear).zone, Zone::Battlefield);

        let mut returned = root.clone();
        returned.move_object(gear, Zone::Graveyard);
        returned.move_object(gear, Zone::Battlefield);
        let new_grantor = returned.object_ref(gear);
        assert_ne!(new_grantor, grantor);
        assert!(returned.attach_equipment(new_grantor, bearer));
        pass_pair(&mut returned);
        assert_eq!(returned.state.pending, Pending::Priority);
        assert!(returned.state.stack.is_empty());

        assert!(root.attach_equipment(grantor, root.object_ref(new_bearer)));
        pass_pair(&mut root);
        assert_eq!(root.actor(), Some(player));
        assert_eq!(root.state.pending, Pending::EffectChoice { player });
        let action = Action::Sacrifice { object: grantor };
        let wrong = Action::Sacrifice {
            object: root.object_ref(second),
        };
        reject(&mut root, wrong);
        assert!(
            root.legal_actions()
                .iter()
                .all(|a| !matches!(a, Action::ChooseTarget { .. }))
        );

        let mut declined = root.clone();
        act(&mut declined, Action::DeclineEffectPayment);
        assert!(declined.state.stack.is_empty());
        assert_eq!(declined.object_view(gear).zone, Zone::Battlefield);

        let saved = root.snapshot();
        let revision = root.revision();
        let index = root
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut raw = root.clone();
        let mut indexed = root;
        act(&mut raw, action.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for branch in [&mut raw, &mut indexed] {
            assert_eq!(branch.object_view(gear).zone, Zone::Exile);
            assert_eq!(branch.object_view(second).zone, Zone::Battlefield);
            choose_reflexive_target(branch, bearer, Target::Player(other));
            settle(branch);
            assert_eq!(branch.state.players[other.index()].life, 18);
            assert_eq!(branch.state.players[player.index()].life, 22);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        assert!(indexed.legal_actions().contains(&action));
    }
}

#[test]
fn another_artifact_can_be_the_bearer_or_a_new_incarnation_of_the_grantor() {
    let _rules = fixture_rules::activate(&[
        (named("Short Bow"), &BLUNDERBUSS),
        (named("Gingerbrute"), &LIFELINK_ARTIFACT),
        (named("Grizzly Bears"), &LARGE_BLOCKER),
    ]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let gear = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let duplicate = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let enemy_gear = put(&mut game, other, "Short Bow", Zone::Battlefield);
        let attacker = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
        let nonartifact = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let target = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let grantor = game.object_ref(gear);
        let bearer = game.object_ref(attacker);
        attack(&mut game, bearer, grantor);

        let mut returned = game.clone();
        returned.move_object(gear, Zone::Graveyard);
        returned.move_object(gear, Zone::Battlefield);
        let current = returned.object_ref(gear);
        assert_ne!(grantor, current);
        pass_pair(&mut returned);
        act(&mut returned, Action::Sacrifice { object: current });
        choose_reflexive_target(&mut returned, bearer, Target::Card(target));
        returned.state.objects[attacker as usize].growth.power += 2;
        settle(&mut returned);
        assert_eq!(returned.object_view(target).damage, 4);

        game.state.objects[attacker as usize].growth.power += 2;
        pass_pair(&mut game);
        for excluded in [
            grantor,
            game.object_ref(nonartifact),
            game.object_ref(enemy_gear),
        ] {
            reject(&mut game, Action::Sacrifice { object: excluded });
        }
        assert!(game.legal_actions().contains(&Action::Sacrifice {
            object: game.object_ref(duplicate),
        }));
        act(&mut game, Action::Sacrifice { object: bearer });
        assert_eq!(game.object_view(attacker).zone, Zone::Graveyard);
        choose_reflexive_target(&mut game, bearer, Target::Card(target));
        settle(&mut game);
        assert_eq!(game.object_view(target).damage, 7);
        assert_eq!(game.state.players[player.index()].life, 27);
    }
}
