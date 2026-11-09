//! Damage observations and attached activation restrictions are independent.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const NONEQUIP: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 0,
        toughness: 0,
    },
    keywords: Keywords::NONE,
};
const TAP_ATTACHMENT: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::SelfEnters,
    targets: TargetPlan::NONE,
    program: &[Instruction::TapAttached],
};
const fn aura_targets(kinds: u8) -> TargetPlan {
    TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }))
}
const SLEEP: CardRules = CardRules {
    effect: Effect::Aura(aura_targets(1)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: NONEQUIP,
    }),
    triggers: &[
        TAP_ATTACHMENT,
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSource],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachedUntap(
            UntapRestriction::ControllerStep,
        )],
    }],
    ..CardRules::EMPTY
};
const SANCTUM: CardRules = CardRules {
    effect: Effect::Aura(aura_targets(1 | 16)),
    aura: Some(AuraSpec {
        kinds: 1 | 16,
        relation: -1,
        bonus: NONEQUIP,
    }),
    triggers: &[TAP_ATTACHMENT],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::AttachedUntap(UntapRestriction::ControllerStep),
            StaticAbilityFragment::AttachedActivationsProhibited,
        ],
    }],
    ..CardRules::EMPTY
};
const ARCHIVE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 2,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn shock(game: &mut Game, player: Player, target: CardId) {
    let damage = put(game, player, "Shock", Zone::Hand);
    cast(game, damage, Some(Target::Card(target)));
    settle(game);
}

#[test]
fn only_actual_damage_to_the_attached_creature_sacrifices_the_aura() {
    let sleep = Card::from_name("Sleep Magic").unwrap();
    let _rules = fixture_rules::activate(&[(sleep, &SLEEP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let unrelated = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let aura = spawn(&mut game, player, sleep, Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(victim))
    );
    assert!(game.object_view(victim).tapped);
    game.add_counter(victim, CounterKind::Shield, 1);

    shock(&mut game, player, unrelated);
    assert_eq!(game.object_view(aura).zone, Zone::Battlefield);
    shock(&mut game, player, victim);
    assert_eq!(game.counter_count(victim, CounterKind::Shield), 0);
    assert_eq!(game.object_view(victim).damage, 0);
    assert_eq!(game.object_view(aura).zone, Zone::Battlefield);
    shock(&mut game, player, victim);
    assert_eq!(game.object_view(victim).damage, 2);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
}

#[test]
fn the_activation_rule_applies_to_actual_and_projected_mana_without_blanking_triggers() {
    let sanctum = Card::from_name("Stuck in Summoner's Sanctum").unwrap();
    let archive = Card::from_name("Hedron Archive").unwrap();
    let _rules = fixture_rules::activate(&[(sanctum, &SANCTUM), (archive, &ARCHIVE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, archive, Zone::Battlefield);
    let aura = spawn(&mut game, player, sanctum, Zone::Hand);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(source)));
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    game.state.objects[source as usize].tapped = false;
    game.state.players[player.index()].mana = [0; 6];
    let produce = Action::ActivateMana {
        card: source,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 0, 0, 0, 0, 2],
    };
    let announce = Action::AnnounceCast { card: boots };
    reject(&mut game, produce.clone());
    reject(&mut game, announce.clone());

    let mut released = game.clone();
    released.move_object(aura, Zone::Graveyard);
    act(&mut released, announce);
    act(&mut released, produce);
    pay(&mut released);
    settle(&mut released);
    assert_eq!(released.object_view(boots).zone, Zone::Battlefield);

    game.state.step = Step::Main2;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    settle(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.object_view(aura).zone, Zone::Battlefield);
}
