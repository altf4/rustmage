//! Attacking-only equipment rules and causal death trigger multiplicity.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLADE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::AttachmentAttackingKeywords(Keywords::FIRST_STRIKE),
                StaticAbilityFragment::AttachedAttackerMustBeBlocked,
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::EquippedCreatureAndOwnedEmblemDeath,
                condition: StatValue::Fixed(1),
            }],
        },
    ],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CREATURE_DEATH: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::CreatureDies {
        relation: CastRelation::Any,
        other: false,
        nontoken: false,
        subtype: None,
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
};
const BEARER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        CREATURE_DEATH,
        TriggeredAbility {
            condition: TriggerCondition::Sacrificed {
                own_turn: false,
                filter: SacrificeFilter {
                    kinds: 1,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            },
            ..CREATURE_DEATH
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureCardsEnterYourGraveyard,
            ..CREATURE_DEATH
        },
    ],
    ..CardRules::EMPTY
};
const EMBLEM: Card = Card::from_catalogue_id(5338);
const EMBLEM_RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    triggers: &[CREATURE_DEATH],
    ..CardRules::EMPTY
};

fn packets(game: &mut Game, source: CardId, ability: u16) -> usize {
    game.expand_trigger_multiplicity();
    game.state
        .pending_triggers
        .iter()
        .chain(&game.state.ordering_triggers)
        .chain(&game.state.stack)
        .filter(|item| {
            item.kind == StackKind::TriggeredAbility { ability }
                && item.source.is_some_and(|origin| origin.id == source)
        })
        .count()
}

#[test]
fn predeath_bearer_and_owned_emblem_double_but_a_sacrifice_trigger_does_not() {
    let sword = Card::from_name("The Masamune").unwrap();
    let bear = Card::from_name("Grizzly Bears").unwrap();
    let _rules =
        fixture_rules::activate(&[(sword, &BLADE), (bear, &BEARER), (EMBLEM, &EMBLEM_RULES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let blade = spawn(&mut game, opponent, sword, Zone::Battlefield);
    let bearer = spawn(&mut game, player, bear, Zone::Battlefield);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let own_emblem = game.state.objects.len() as CardId;
    game.create_emblem(player, EMBLEM);
    let rival_emblem = game.state.objects.len() as CardId;
    game.create_emblem(opponent, EMBLEM);
    let blade_ref = game.object_ref(blade);
    let bearer_ref = game.object_ref(bearer);
    assert!(game.attach_equipment(blade_ref, bearer_ref));

    let mut simultaneous = game.clone();
    let mut suppressed = game.clone();
    suppressed.remove_abilities_exact(
        &[bearer_ref],
        blade_ref,
        player,
        AnimationDuration::EndOfTurn,
    );
    let mut not_a_death = game.clone();
    not_a_death.move_object(victim, Zone::Exile);
    assert_eq!(packets(&mut not_a_death, own_emblem, 0), 0);

    game.sacrifice_batch(&[(player, victim)]);
    assert_eq!(packets(&mut game, bearer, 0), 2);
    assert_eq!(packets(&mut game, bearer, 1), 1);
    assert_eq!(packets(&mut game, bearer, 2), 2);
    assert_eq!(packets(&mut game, own_emblem, 0), 2);
    assert_eq!(packets(&mut game, rival_emblem, 0), 1);

    suppressed.sacrifice_batch(&[(player, victim)]);
    assert_eq!(packets(&mut suppressed, bearer, 0), 0);
    assert_eq!(packets(&mut suppressed, own_emblem, 0), 1);

    simultaneous.sacrifice_batch(&[(opponent, blade), (player, bearer)]);
    assert_eq!(simultaneous.object_view(blade).zone, Zone::Graveyard);
    assert_eq!(simultaneous.object_view(bearer).zone, Zone::Graveyard);
    assert_eq!(packets(&mut simultaneous, bearer, 0), 2);
    assert_eq!(packets(&mut simultaneous, bearer, 2), 0);
    assert_eq!(packets(&mut simultaneous, own_emblem, 0), 2);
    assert_eq!(packets(&mut simultaneous, rival_emblem, 0), 1);
}

#[test]
fn the_attacking_bearer_gains_first_strike_and_requires_an_available_block() {
    let sword = Card::from_name("The Masamune").unwrap();
    let _rules = fixture_rules::activate(&[(sword, &BLADE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let blade = spawn(&mut game, player, sword, Zone::Battlefield);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let blocker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    assert!(game.attach_equipment(game.object_ref(blade), game.object_ref(attacker)));
    assert!(!game.keywords(attacker).contains(Keywords::FIRST_STRIKE));
    assert_eq!(game.block_requirement_weight(attacker), 0);
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    assert!(game.keywords(attacker).contains(Keywords::FIRST_STRIKE));
    assert_eq!(game.block_requirement_weight(attacker), 1);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    reject(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(game.object_view(attacker).damage, 0);
}
