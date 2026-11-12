//! Exact graveyard Eerie from an enchantment entry or a fully unlocked Room.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FEAR: Card = Card::GrizzlyBears;
const ROOM: Card = Card::from_catalogue_id(176); // Anthem of Champions
const LEFT: Card = Card::from_catalogue_id(1024); // Deeproot Pilgrimage, {1}{U}
const RIGHT: Card = Card::from_catalogue_id(2145); // Impact Tremors, {1}{R}

const FEAR_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::Eerie,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayIfSourceInZone {
                zone: TargetZone::Graveyard,
                skip: 1,
            },
            Instruction::ReturnSourceToHand(TargetZone::Graveyard),
        ],
    }],
    ..CardRules::EMPTY
};

const ROOM_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [LEFT, RIGHT],
            triggers: &[],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const HALF_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(ROOM),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const MULTIPLIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledPermanent,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn unlock(game: &mut Game, object: ObjectRef, door: RoomDoor, colored: Mana) {
    act(
        game,
        Action::BeginRoomUnlock {
            object,
            door,
            cost: ManaCost {
                colored,
                generic: 1,
            },
        },
    );
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
}

fn assert_source(game: &Game, player: Player, fear: ObjectRef) {
    let view = game.observe(player);
    assert_eq!(view.stack.len(), 1);
    assert_eq!(view.stack[0].source().unwrap().object, fear);
    assert_eq!(view.stack[0].controller, player);
}

#[test]
fn only_controlled_enchantment_entries_trigger_the_graveyard_card_once() {
    let multiplier = Card::from_name("Fountainport Bell").unwrap();
    let _rules = fixture_rules::activate(&[(FEAR, &FEAR_RULES), (multiplier, &MULTIPLIER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let fear = spawn(&mut game, player, FEAR, Zone::Graveyard);
    let opposing = spawn(&mut game, player.other(), FEAR, Zone::Graveyard);
    let permanent = spawn(&mut game, player, FEAR, Zone::Battlefield);
    spawn(&mut game, player, multiplier, Zone::Battlefield);

    let ordinary = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, ordinary, None);
    pass_pair(&mut game);
    assert!(game.observe(player).stack.is_empty());

    let mut suppressed = game.clone();
    put(
        &mut suppressed,
        player.other(),
        "Torpor Orb",
        Zone::Battlefield,
    );
    let enchanted = put(&mut suppressed, player, "Enduring Curiosity", Zone::Hand);
    cast(&mut suppressed, enchanted, None);
    pass_pair(&mut suppressed);
    assert!(suppressed.observe(player).stack.is_empty());

    let entry = put(&mut game, player, "Impact Tremors", Zone::Hand);
    cast(&mut game, entry, None);
    pass_pair(&mut game);
    assert_source(&game, player, game.object_ref(fear));
    pass_pair(&mut game);
    let saved = game.snapshot();

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(fear).zone, Zone::Graveyard);

    let accept = Action::ChooseOptional { accept: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == accept)
        .unwrap();
    declined.restore(&saved).unwrap();
    declined
        .apply_index(player, declined.revision(), index)
        .unwrap();
    act(&mut game, accept);
    assert_eq!(declined.snapshot(), game.snapshot());
    assert_eq!(game.object_view(fear).zone, Zone::Hand);
    assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
    assert_eq!(game.object_view(permanent).zone, Zone::Battlefield);
}

#[test]
fn a_full_room_unlock_queues_one_exact_graveyard_source() {
    let _rules = fixture_rules::activate(&[
        (FEAR, &FEAR_RULES),
        (ROOM, &ROOM_RULES),
        (LEFT, &HALF_RULES),
        (RIGHT, &HALF_RULES),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let room = spawn(&mut game, player, ROOM, Zone::Battlefield);
    let room = game.object_ref(room);
    let fear = spawn(&mut game, player, FEAR, Zone::Graveyard);
    let opposing = spawn(&mut game, player.other(), FEAR, Zone::Graveyard);
    unlock(&mut game, room, RoomDoor::Left, [0, 1, 0, 0, 0, 0]);
    assert!(game.observe(player).stack.is_empty());

    unlock(&mut game, room, RoomDoor::Right, [0, 0, 0, 1, 0, 0]);
    let old = game.object_ref(fear);
    assert_source(&game, player, old);

    let mut stale = game.clone();
    stale.move_object(fear, Zone::Exile);
    stale.move_object(fear, Zone::Graveyard);
    let new = stale.object_ref(fear);
    assert_ne!(new, old);
    settle(&mut stale);
    assert_eq!(stale.object_view(fear).zone, Zone::Graveyard);
    assert_eq!(stale.object_ref(fear), new);

    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(fear).zone, Zone::Hand);
    assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
}
