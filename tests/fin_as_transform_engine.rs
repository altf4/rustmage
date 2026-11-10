mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FRONT: &str = "Brigid, Clachan's Heart";
const BACK: &str = "Brigid, Doun's Mind";
const EMBLEM: Card = Card::from_catalogue_id(5338);
const DRAIN: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::CreatureDies {
        relation: CastRelation::Any,
        other: true,
        nontoken: false,
        subtype: None,
    },
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
        opponent_only: true,
    })),
    program: &[
        Instruction::LoseLife(PlayerSet::Targeted, 1),
        Instruction::GainLife(PlayerSet::Controller, 1),
        Instruction::IfThisAbilityResolutionNumber { number: 4, skip: 1 },
        Instruction::TransformSource,
    ],
};
const FRONT_RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[DRAIN],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileBattlefieldSourceReturn {
            transformed: true,
            under_owner: true,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BACK_RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTransforms,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        as_transform: Some(AsTransformEffect::CreateEmblem(EMBLEM)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const EMBLEM_RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::Any,
            other: false,
            nontoken: false,
            subtype: None,
        },
        program: &[
            Instruction::LoseLife(PlayerSet::Targeted, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
        ..DRAIN
    }],
    ..CardRules::EMPTY
};
const COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const FORK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(FRONT).unwrap(), &FRONT_RULES),
        (Card::from_name(BACK).unwrap(), &BACK_RULES),
        (EMBLEM, &EMBLEM_RULES),
        (Card::from_name("Unsummon").unwrap(), &COUNTER),
        (Card::from_name("Opt").unwrap(), &FORK),
    ])
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn queue_drain(game: &mut Game, caster: Player, controller: Player) {
    let victim = put(game, caster.other(), "Bear Cub", Zone::Battlefield);
    let shock = put(game, caster, "Shock", Zone::Hand);
    cast(game, shock, Some(Target::Card(victim)));
    pass_priority(game);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(controller.other()),
        },
    );
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].controller, controller);
}

fn emblems(game: &Game) -> Vec<ObjectView> {
    game.observe(Player::P0)
        .objects
        .into_iter()
        .filter(|object| object.card == EMBLEM && object.zone == Zone::Command)
        .collect()
}

#[test]
fn the_fourth_resolution_creates_the_emblem_for_the_current_controller_before_triggers() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, FRONT, Zone::Battlefield);
    let reference = game.object_ref(source);

    queue_drain(&mut game, player, player);
    let counter = put(&mut game, player, "Unsummon", Zone::Hand);
    let trigger = game.state.stack[0].id;
    cast(&mut game, counter, Some(Target::Stack(trigger)));
    settle(&mut game);
    assert!(game.observe(player).ability_resolutions.is_empty());

    for count in 1..=2 {
        queue_drain(&mut game, player, player);
        settle(&mut game);
        assert_eq!(game.observe(player).ability_resolutions[0].count, count);
        assert!(emblems(&game).is_empty());
    }

    queue_drain(&mut game, player, player);
    let copy = put(&mut game, player, "Opt", Zone::Hand);
    let trigger = game.state.stack[0].id;
    cast(&mut game, copy, Some(Target::Stack(trigger)));
    pass_priority(&mut game);
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    pass_priority(&mut game);
    assert_eq!(game.observe(player).ability_resolutions[0].count, 3);
    assert_eq!(game.object_view(source).name(), FRONT);
    assert!(emblems(&game).is_empty());

    game.change_controller(reference, opponent);
    act(&mut game, Action::Pass);
    let saved = game.snapshot();
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    indexed
        .apply_index(opponent, game.revision(), index)
        .unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.observe(player).ability_resolutions[0].count, 4);
    assert_eq!(game.object_ref(source), reference);
    assert_eq!(game.object_view(source).name(), BACK);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].controller, opponent);
    assert_eq!(emblems(&game).len(), 1);
    assert_eq!(emblems(&game)[0].controller, opponent);
    assert_eq!(game.state.players[player.index()].life, 24);
    assert_eq!(game.state.players[opponent.index()].life, 16);

    settle(&mut game);
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    pass_priority(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(emblems(&game).len(), 1);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[opponent.index()].life, 17);
}

#[test]
fn an_entry_with_the_back_face_up_is_distinct_from_an_in_place_transform() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, FRONT, Zone::Battlefield);
    let reference = game.object_ref(source);
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == source),
    );
    settle(&mut game);
    let entered = game.object_ref(source);
    assert_ne!(entered, reference);
    assert_eq!(game.object_view(source).name(), BACK);
    assert!(emblems(&game).is_empty());

    game.transform_permanent(entered);
    assert_eq!(game.object_view(source).name(), FRONT);
    assert!(emblems(&game).is_empty());
    game.transform_permanent(entered);
    assert_eq!(game.object_ref(source), entered);
    assert_eq!(game.object_view(source).name(), BACK);
    assert_eq!(emblems(&game).len(), 1);
}

#[test]
fn a_face_with_its_printed_abilities_removed_does_not_supply_a_rider() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, FRONT, Zone::Battlefield);
    let aura = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(source)));
    settle(&mut game);
    let reference = game.object_ref(source);
    game.transform_permanent(reference);
    assert_eq!(
        game.object_view(source).card,
        Card::from_name(BACK).unwrap()
    );
    assert_eq!(game.object_ref(source), reference);
    assert!(emblems(&game).is_empty());
    assert!(game.state.pending_triggers.is_empty());
}
