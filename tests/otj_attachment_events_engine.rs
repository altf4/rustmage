//! Committed attachment events and exact attachment-duration control.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CONTROL: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledAuraAttachesToOpponentNonland,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainControlWhileEventAuraAttached],
    }],
    ..CardRules::EMPTY
};
const AURA_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 32,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const EQUIPMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesAttachedToCreature,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
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

fn attachment_packet(game: &mut Game, observer: CardId) -> SpellView {
    for _ in 0..12 {
        if let Some(item) = game.state.stack.last().filter(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item.source.is_some_and(|source| source.id == observer)
        }) {
            return item.clone();
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("attachment trigger did not reach the stack");
}

#[test]
fn a_committed_aura_attachment_steals_an_exact_recipient_for_its_duration() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CONTROL)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, player, "Pacifism", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(recipient)));
    let packet = attachment_packet(&mut game, observer);
    assert!(packet.targets.is_empty());
    let event = packet.attachment_trigger.unwrap();
    assert_eq!(event.grantor, game.object_ref(aura));
    assert_eq!(event.recipient, game.object_ref(recipient));
    assert_eq!(game.object_view(recipient).controller, other);

    let mut lost_attachment = game.clone();
    lost_attachment.move_object(aura, Zone::Graveyard);
    settle(&mut lost_attachment);
    assert_eq!(lost_attachment.object_view(recipient).controller, other);

    game.record_control(event.grantor, other, None);
    game.move_object(observer, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(aura).controller, other);
    assert_eq!(game.object_view(recipient).controller, player);
    let saved = game.snapshot();
    game.move_object(aura, Zone::Graveyard);
    assert_eq!(game.object_view(recipient).controller, other);
    game.restore(&saved).unwrap();
    assert_eq!(game.object_view(recipient).controller, player);
}

#[test]
fn aura_copy_entering_attached_can_steal_and_costlier_recipients_cannot() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &CONTROL),
        (Card::Divination, &AURA_COPY),
    ]);
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
    let costly = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    let original = put(&mut game, player, "Pacifism", Zone::Hand);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, original, Some(Target::Card(observer)));
    settle(&mut game);
    assert_eq!(game.object_view(observer).controller, player);
    let mut costly_branch = game.clone();

    for (game, target) in [(&mut game, recipient), (&mut costly_branch, costly)] {
        cast(game, spell, Some(Target::Card(original)));
        act(game, Action::Pass);
        act(game, Action::Pass);
        let object = game.object_ref(target);
        act(game, Action::ChooseAuraAttachment { object });
        if target == recipient {
            let packet = attachment_packet(game, observer);
            let event = packet.attachment_trigger.unwrap();
            assert!(game.object_view(event.grantor.card).is_token());
            assert_eq!(event.recipient, object);
        }
        settle(game);
    }
    assert_eq!(game.object_view(recipient).controller, player);
    assert_eq!(costly_branch.object_view(costly).controller, other);
}

#[test]
fn equipment_triggers_only_when_its_exact_attachment_changes() {
    let bow = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(bow, &EQUIPMENT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    for (target, triggers) in [(first, true), (first, false), (second, true)] {
        act(
            &mut game,
            Action::AnnounceActivate {
                card: equipment,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        if triggers {
            let packet = attachment_packet(&mut game, equipment);
            let event = packet.attachment_trigger.unwrap();
            assert_eq!(event.grantor, game.object_ref(equipment));
            assert_eq!(event.recipient, game.object_ref(target));
        }
        settle(&mut game);
    }
    assert_eq!(game.state.players[player.index()].life, 22);
}
