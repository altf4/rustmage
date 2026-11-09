//! Live exact Equipment links used by continuous abilities and targets.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const HASTE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledEquippedCreatureKeywords(
            Keywords::HASTE,
        )],
    }],
    ..CardRules::EMPTY
};

const ATTACKING_EQUIPPED: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Filtered {
        base: &TargetRule::Creature,
        predicate: &TargetPredicate::Attacking,
    },
    predicate: &TargetPredicate::Equipped,
};

const SHOPKEEP: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::one(ATTACKING_EQUIPPED)),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::MENACE,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn equipped_haste_respects_current_control_and_projected_attachment_departure() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &HASTE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let equipment = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let aura = put(&mut game, player, "Pacifism", Zone::Battlefield);
    let elf_ref = game.object_ref(elf);
    let equipment_ref = game.object_ref(equipment);
    game.state.objects[elf as usize].summoning_sick = true;
    game.state.objects[aura as usize].attached_to = Some(elf_ref);
    game.state.players[player.index()].mana = [0; 6];
    let goal = FundingGoal {
        actor: player,
        required: [0, 0, 0, 0, 1, 0],
        generic: 0,
        reserve: None,
        source: None,
    };
    let tap = Action::TapMana { card: elf };
    assert!(!game.equipped(elf));
    assert!(!game.legal_actions().contains(&tap));
    assert!(!game.can_fund(goal.clone()));

    assert!(game.attach_equipment(equipment_ref, elf_ref));
    assert!(game.equipped(elf));
    assert!(game.legal_actions().contains(&tap));
    assert!(game.can_fund(goal.clone()));
    let before = game.snapshot();
    let projected = FundingState::from_game(&game, &goal);
    assert!(projected.activation_keywords(elf).contains(Keywords::HASTE));
    assert!(
        projected.cost_departures(&[equipment_ref], Zone::Graveyard, &mut |next| {
            assert!(!next.equipped(elf));
            assert!(!next.activation_keywords(elf).contains(Keywords::HASTE));
            true
        })
    );
    assert_eq!(game.snapshot(), before);

    let mut blanked = game.clone();
    let provider_ref = blanked.object_ref(provider);
    blanked.remove_abilities_exact(
        &[provider_ref],
        elf_ref,
        player,
        AnimationDuration::EndOfTurn,
    );
    assert!(blanked.equipped(elf));
    assert!(!blanked.object_view(elf).keywords.contains(Keywords::HASTE));
    assert!(!blanked.can_fund(goal.clone()));

    let mut transferred = game.clone();
    transferred.record_control(provider_ref, player.other(), None);
    assert!(transferred.equipped(elf));
    assert!(
        !transferred
            .object_view(elf)
            .keywords
            .contains(Keywords::HASTE)
    );
    assert!(!transferred.can_fund(goal));

    act(&mut game, tap);
    assert_eq!(game.state.players[player.index()].mana[4], 1);
}

#[test]
fn attacking_equipped_target_rechecks_the_last_attachment_after_binding() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &SHOPKEEP)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let shopkeep = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let equipped = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    assert!(game.attach_equipment(game.object_ref(equipment), game.object_ref(equipped)));
    game.begin_step(Step::BeginCombat);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Attack { card: equipped });
    act(&mut game, Action::Attack { card: plain });
    act(&mut game, Action::FinishAttackers);
    assert!(!game.state.attackers.iter().any(|row| row.card == shopkeep));

    let invalid = Action::ChooseTarget {
        target: Target::Card(plain),
    };
    reject(&mut game, invalid);
    let chosen = Action::ChooseTarget {
        target: Target::Card(equipped),
    };
    let before = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == chosen)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, chosen);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let mut lost = game.clone();
    lost.move_object(equipment, Zone::Exile);
    settle(&mut lost);
    assert!(
        !lost
            .object_view(equipped)
            .keywords
            .contains(Keywords::MENACE)
    );
    settle(&mut game);
    assert!(
        game.object_view(equipped)
            .keywords
            .contains(Keywords::MENACE)
    );

    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
}
