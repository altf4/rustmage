mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::MayAttachAnyControlledEquipment { recipient: 0 }],
    }],
    ..CardRules::EMPTY
};
const RECIPIENT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HEXPROOF.union(Keywords::PROTECTION_RED),
    ..CardRules::EMPTY
};

fn trigger(game: &mut Game, recipient: CardId) {
    game.begin_step(Step::BeginCombat);
    game.give_priority();
    let target = Target::Card(recipient);
    for _ in 0..8 {
        if game.state.stack.last().is_some_and(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item.targets.iter().any(|bound| bound.target == target)
        }) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
                || *action == Action::ChooseTarget { target }
        });
    }
    panic!("targeted combat trigger did not reach the stack");
}

fn pick(equipment: ObjectRef) -> Action {
    Action::ChooseEquipmentAttachment {
        equipment: Some(equipment),
    }
}
const FINISH: Action = Action::ChooseEquipmentAttachment { equipment: None };

#[test]
fn select_any_number_then_order_only_the_simultaneous_new_attachments() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &SOURCE),
        (Card::from_name("Hill Giant").unwrap(), &RECIPIENT),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let previous = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let second = put(&mut game, player, "Gold Pan", Zone::Battlefield);
    let already = put(&mut game, player, "Lavaspur Boots", Zone::Battlefield);
    let protected = put(&mut game, player, "Cori-Steel Cutter", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let [first, second, already, protected, foreign] =
        [first, second, already, protected, foreign].map(|id| game.object_ref(id));
    let recipient_ref = game.object_ref(recipient);
    let previous = game.object_ref(previous);
    assert!(game.attach_equipment(first, previous));
    assert!(game.attach_equipment(already, recipient_ref));

    trigger(&mut game, recipient);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let view = game.observe(player.other()).equipment_attachment.unwrap();
    assert_eq!(view.selected, Some(vec![]));
    assert_eq!(view.candidates, vec![first, second, already]);
    assert_eq!(view.recipient.reference, recipient_ref);
    reject(&mut game, pick(protected));
    reject(&mut game, pick(foreign));

    let mut declined = game.clone();
    act(&mut declined, FINISH);
    assert_eq!(declined.object_view(first.card).attached_to, Some(previous));
    assert_eq!(declined.object_view(second.card).attached_to, None);

    act(&mut game, pick(already));
    act(&mut game, pick(first));
    reject(&mut game, pick(first));
    let saved = game.snapshot();
    let rev = game.revision();
    let next = pick(second);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == next)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, next);
    indexed.apply_index(player, rev, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(
        game.legal_actions()
            .into_iter()
            .filter(|action| matches!(action, Action::ChooseEquipmentAttachment { .. }))
            .collect::<Vec<_>>(),
        vec![FINISH]
    );
    assert_eq!(game.object_view(first.card).attached_to, Some(previous));
    assert_eq!(game.object_view(second.card).attached_to, None);

    act(&mut game, FINISH);
    let view = game.observe(player).equipment_attachment.unwrap();
    assert_eq!(view.ordering, Some(true));
    assert_eq!(view.selected, None);
    assert_eq!(view.candidates, vec![first, second]);
    assert_eq!(game.object_view(first.card).attached_to, Some(previous));
    reject(&mut game, FINISH);
    act(&mut game, pick(second));
    assert_eq!(game.state.pending, Pending::Priority);
    for item in [first, second, already] {
        assert_eq!(game.object_view(item.card).attached_to, Some(recipient_ref));
    }
}

#[test]
fn missing_equipment_or_an_illegal_announced_target_skips_the_choice() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &SOURCE)]);
    let player = Player::P0;
    for bounce in [false, true] {
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let equipment = put(
            &mut game,
            if bounce { player } else { player.other() },
            "Short Bow",
            Zone::Battlefield,
        );
        trigger(&mut game, recipient);
        if bounce {
            let responder = game.actor().unwrap();
            let spell = put(&mut game, responder, "Unsummon", Zone::Hand);
            cast(&mut game, spell, Some(Target::Card(recipient)));
        }
        settle(&mut game);
        assert!(game.observe(player).equipment_attachment.is_none());
        assert_eq!(game.object_view(equipment).attached_to, None);
        assert_eq!(
            game.object_view(recipient).zone,
            if bounce {
                Zone::Hand
            } else {
                Zone::Battlefield
            }
        );
    }
}
