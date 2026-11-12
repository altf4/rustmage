//! Each live prowess grant owns a separate committed-cast trigger.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CAST_NONCREATURE: TriggerCondition = TriggerCondition::SpellCast {
    relation: CastRelation::You,
    filter: CastFilter::Noncreature,
    ordinal: None,
};
const PUMP: &[Instruction] = &[Instruction::Grow {
    subject: Subject::Source,
    power: 1,
    toughness: 1,
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureTrigger {
            ability: 1,
            other: true,
            nontoken: false,
        }],
    }],
    triggers: &[
        TriggeredAbility {
            condition: CAST_NONCREATURE,
            targets: TargetPlan::NONE,
            program: PUMP,
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&CAST_NONCREATURE),
            targets: TargetPlan::NONE,
            program: PUMP,
        },
    ],
    ..CardRules::EMPTY
};

fn grantors(game: &Game, source: ObjectRef) -> Vec<Option<ObjectRef>> {
    game.state
        .stack
        .iter()
        .chain(&game.state.pending_triggers)
        .chain(&game.state.ordering_triggers)
        .filter(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item
                    .source()
                    .is_some_and(|stack_source| stack_source.object == source)
        })
        .map(|item| item.attachment_trigger.as_ref().map(|grant| grant.grantor))
        .collect()
}

fn resolve(game: &mut Game) {
    for _ in 0..48 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
        {
            assert_eq!(game.state.pending, Pending::Priority);
            return;
        }
        choose(game, |action| {
            matches!(action, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    panic!("prowess scenario exceeded the resolution bound");
}

#[test]
fn printed_and_granted_prowess_stack_per_creature_and_per_provider() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PROVIDER),
        (Card::GrizzlyBears, &PROVIDER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let printed = put(&mut game, player, "Shipwreck Dowser", Zone::Battlefield);
    let vanilla = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);

    let mut creature_cast = game.clone();
    cast(&mut creature_cast, creature, None);
    assert_eq!(creature_cast.state.stack.len(), 1);
    assert!(creature_cast.state.ordering_triggers.is_empty());
    assert!(creature_cast.state.pending_triggers.is_empty());
    resolve(&mut creature_cast);

    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let before = game.snapshot();
    let announcement = Action::AnnounceCast { card: spell };
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == announcement)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, announcement.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert!(game.state.ordering_triggers.is_empty());
    pay(&mut game);
    pay(&mut indexed);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let a = game.object_ref(first);
    let b = game.object_ref(second);
    let inherent = game.object_ref(printed);
    let plain = game.object_ref(vanilla);
    for (recipient, expected) in [
        (a, vec![None, Some(b)]),
        (b, vec![None, Some(a)]),
        (inherent, vec![None, Some(a), Some(b)]),
        (plain, vec![Some(a), Some(b)]),
    ] {
        let grants = grantors(&game, recipient);
        assert_eq!(grants.len(), expected.len());
        assert!(expected.iter().all(|grant| grants.contains(grant)));
    }
    for excluded in [opposing, equipment] {
        assert!(grantors(&game, game.object_ref(excluded)).is_empty());
    }

    // These packets have already acquired their exact source and survive the grantor.
    game.move_object(first, Zone::Exile);
    game.move_object(vanilla, Zone::Exile);
    game.move_object(vanilla, Zone::Battlefield);
    resolve(&mut game);
    assert_eq!(stats(&game, second), (5, 5));
    assert_eq!(stats(&game, printed), (6, 6));
    assert_eq!(stats(&game, vanilla), (2, 2));

    indexed.restore(&before).unwrap();
    assert!(indexed.legal_actions().contains(&announcement));
    assert!(indexed.state.ordering_triggers.is_empty());
}

#[test]
fn the_ability_layer_suppresses_only_grants_older_than_the_removal() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PROVIDER),
        (Card::GrizzlyBears, &PROVIDER),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let old = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let recipient = put(&mut game, player, "Shipwreck Dowser", Zone::Battlefield);
    let aura = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(recipient)));
    resolve(&mut game);

    let newer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    pay(&mut game);
    assert_eq!(
        grantors(&game, game.object_ref(recipient)),
        [Some(game.object_ref(newer))]
    );
    resolve(&mut game);

    let aura = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(old)));
    resolve(&mut game);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    pay(&mut game);
    assert_eq!(
        grantors(&game, game.object_ref(recipient)),
        [Some(game.object_ref(newer))]
    );
    assert_eq!(grantors(&game, game.object_ref(newer)), [None]);
}

#[test]
fn provider_sacrificed_as_a_cast_cost_grants_nothing_at_commit() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let recipient = put(&mut game, player, "Shipwreck Dowser", Zone::Battlefield);
    let spell = put(&mut game, player, "Corrupted Conviction", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let object = game.object_ref(provider);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert_eq!(game.object_view(provider).zone, Zone::Graveyard);
    assert_eq!(grantors(&game, game.object_ref(recipient)), [None]);
    resolve(&mut game);
    assert_eq!(stats(&game, recipient), (4, 4));
}
