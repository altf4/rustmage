mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const EQUIPMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::EndCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::AttachSourceToControlledCreature,
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
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

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn queue_equipment_trigger(game: &mut Game, equipment: CardId) {
    game.state.step = Step::CombatDamage;
    pass_priority(game);
    assert_eq!(game.state.step, Step::EndCombat);
    assert_eq!(game.state.stack.len(), 1);
    let trigger = &game.state.stack[0];
    assert_eq!(trigger.source.unwrap().object, game.object_ref(equipment));
    assert!(trigger.targets.is_empty());
}

#[test]
fn the_trigger_controller_must_choose_a_legal_recipient_even_for_opposing_equipment() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &EQUIPMENT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let hexproof = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let normal = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let protected = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let hexproof = game.object_ref(hexproof);
    let normal = game.object_ref(normal);
    let protected = game.object_ref(protected);
    let enemy = game.object_ref(enemy);
    game.grant_keyword_effects(&[hexproof.card], hexproof, Keywords::HEXPROOF);
    game.grant_keyword_effects(&[protected.card], protected, Keywords::PROTECTION_OPPONENTS);

    queue_equipment_trigger(&mut game, equipment);
    game.change_controller(game.object_ref(equipment), opponent);
    pass_priority(&mut game);
    assert_eq!(game.actor(), Some(player));
    let observation = game.observe(opponent);
    let view = observation.effect_choice.unwrap();
    assert_eq!(view.kind, EffectChoiceKind::EquipmentRecipient);
    assert_eq!(view.required, [1, 0]);
    assert_eq!(view.candidates, vec![hexproof, normal]);
    assert!(!view.optional);
    assert!(observation.resolving.unwrap().targets.is_empty());
    reject(&mut game, Action::DeclineEffectPayment);
    for recipient in [
        enemy,
        protected,
        ObjectRef {
            generation: normal.generation + 1,
            ..normal
        },
    ] {
        reject(&mut game, Action::ChooseEquipmentRecipient { recipient });
    }

    let first = Action::ChooseEquipmentRecipient {
        recipient: hexproof,
    };
    let serialized = serde_json::to_string(&first).unwrap();
    assert_eq!(serde_json::from_str::<Action>(&serialized).unwrap(), first);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == first)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut indexed = game.clone();
    assert_eq!(game.apply(opponent, first.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    act(&mut game, first);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(equipment).controller, opponent);
    assert_eq!(game.object_view(equipment).attached_to, Some(hexproof));
    assert_eq!(game.state.players[player.index()].life, 21);

    indexed.restore(&saved).unwrap();
    act(
        &mut indexed,
        Action::ChooseEquipmentRecipient { recipient: normal },
    );
    assert_eq!(indexed.object_view(equipment).attached_to, Some(normal));
    assert_eq!(indexed.state.players[player.index()].life, 21);
}

#[test]
fn an_empty_legal_recipient_set_continues_resolution_without_a_choice() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &EQUIPMENT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let protected = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.grant_keyword_effects(
        &[protected],
        game.object_ref(protected),
        Keywords::PROTECTION_EVERYTHING,
    );
    put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    queue_equipment_trigger(&mut game, equipment);
    settle(&mut game);
    assert_eq!(game.object_view(equipment).attached_to, None);
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn returning_the_physical_card_does_not_rebind_the_attachment_source() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &EQUIPMENT),
        (Card::from_name("Opt").unwrap(), &BLINK),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let original = game.object_ref(equipment);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let island = put(&mut game, opponent, "Island", Zone::Battlefield);
    queue_equipment_trigger(&mut game, equipment);
    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: island });
    let blink = put(&mut game, opponent, "Opt", Zone::Hand);
    cast(&mut game, blink, Some(Target::Card(equipment)));
    settle(&mut game);
    assert_ne!(game.object_ref(equipment), original);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
    assert_eq!(game.object_view(equipment).attached_to, None);
    assert_eq!(game.state.players[player.index()].life, 21);
}
