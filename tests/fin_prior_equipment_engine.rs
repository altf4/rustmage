mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::INDESTRUCTIBLE,
            },
            Instruction::PutSourceCountersOnTargets,
            Instruction::AttachPriorSourceEquipment { recipient: 0 },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};
const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &SOURCE),
        (Card::from_name("Opt").unwrap(), &BLINK),
        (Card::from_name("Unsummon").unwrap(), &RETURN),
    ])
}

fn activate(game: &mut Game, source: CardId, recipient: CardId, announced: bool) {
    let target = Target::Card(recipient);
    if announced {
        act(
            game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(game, Action::ChooseTarget { target });
        pay(game);
    } else {
        choose(game, |action| {
            matches!(
                action,
                Action::Activate { card, ability: 0, target: Some(candidate), .. }
                    if *card == source && *candidate == target
            )
        });
    }
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn direct_and_announced_sacrifice_freeze_counters_and_the_exact_equipment_choice() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    for announced in [false, true] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let local = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let opposing = put(&mut game, opponent, "Lavaspur Boots", Zone::Battlefield);
        let loose = put(&mut game, player, "Gold Pan", Zone::Battlefield);
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        let original = game.object_ref(source);
        let recipient_ref = game.object_ref(recipient);
        let local = game.object_ref(local);
        let opposing = game.object_ref(opposing);
        let loose = game.object_ref(loose);
        for equipment in [local, opposing] {
            assert!(game.attach_equipment(equipment, original));
        }
        game.add_counter(source, CounterKind::PlusOne, 2);
        game.add_counter(source, CounterKind::Charge, 3);
        activate(&mut game, source, recipient, announced);
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        assert_eq!(
            game.state.stack[0]
                .source_quantity_lki
                .as_ref()
                .unwrap()
                .attached_equipment
                .as_deref(),
            Some([local, opposing].as_slice())
        );

        let revive = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, revive, Some(Target::Card(source)));
        pass_priority(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Battlefield);
        assert_ne!(game.object_ref(source), original);
        assert_eq!(game.counter_count(source, CounterKind::Charge), 0);
        pass_priority(&mut game);
        assert_eq!(game.actor(), Some(player));
        assert!(game.keywords(recipient).contains(Keywords::INDESTRUCTIBLE));
        assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 4);
        assert_eq!(game.counter_count(recipient, CounterKind::Charge), 6);
        let view = game.observe(opponent).equipment_attachment.unwrap();
        assert_eq!(view.recipient.reference, recipient_ref);
        assert_eq!(view.candidates, vec![local, opposing]);
        assert!(view.required);
        reject(
            &mut game,
            Action::ChooseEquipmentAttachment { equipment: None },
        );
        reject(
            &mut game,
            Action::ChooseEquipmentAttachment {
                equipment: Some(loose),
            },
        );
        let action = Action::ChooseEquipmentAttachment {
            equipment: Some(opposing),
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let revision = game.revision();
        let saved = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(
            game.object_view(opposing.card).attached_to,
            Some(recipient_ref)
        );
        assert_eq!(game.object_view(local.card).attached_to, None);
        indexed.restore(&saved).unwrap();
        act(
            &mut indexed,
            Action::ChooseEquipmentAttachment {
                equipment: Some(local),
            },
        );
        assert_eq!(
            indexed.object_view(local.card).attached_to,
            Some(recipient_ref)
        );
    }
}

#[test]
fn a_blinked_equipment_cannot_be_selected_and_the_only_survivor_attaches() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let staying = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let blinking = put(&mut game, opponent, "Lavaspur Boots", Zone::Battlefield);
    let original = game.object_ref(source);
    for equipment in [staying, blinking] {
        assert!(game.attach_equipment(game.object_ref(equipment), original));
    }
    let blinked_ref = game.object_ref(blinking);
    activate(&mut game, source, recipient, false);
    act(&mut game, Action::Pass);
    let response = put(&mut game, opponent, "Opt", Zone::Hand);
    cast(&mut game, response, Some(Target::Card(blinking)));
    settle(&mut game);
    assert_ne!(game.object_ref(blinking), blinked_ref);
    assert_eq!(game.object_view(blinking).attached_to, None);
    assert_eq!(
        game.object_view(staying).attached_to,
        Some(game.object_ref(recipient))
    );
    assert!(game.observe(player).equipment_attachment.is_none());
}

#[test]
fn protection_can_leave_the_prior_equipment_set_empty_without_preventing_counters() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Lavaspur Boots", Zone::Battlefield);
    let recipient_ref = game.object_ref(recipient);
    assert!(game.attach_equipment(game.object_ref(opposing), game.object_ref(source)));
    game.grant_keyword_effects(&[recipient], recipient_ref, Keywords::PROTECTION_OPPONENTS);
    game.add_counter(source, CounterKind::PlusOne, 1);
    activate(&mut game, source, recipient, false);
    settle(&mut game);
    assert_eq!(game.object_view(opposing).attached_to, None);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    assert!(game.keywords(recipient).contains(Keywords::INDESTRUCTIBLE));
}
