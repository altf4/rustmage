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
const TEMPORARY_ATTACHMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
            Instruction::MayAttachControlledEquipmentUntilNextEndStep {
                recipient: 0,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::UnattachEventObject],
    }],
    ..CardRules::EMPTY
};
const ENTRY_ATTACHMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[
            Instruction::MayAttachSource {
                recipient: 0,
                skip: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FIRST_STRIKE,
            },
            Instruction::RequireTargetsBlocked { group: 0 },
        ],
    }],
    ..CardRules::EMPTY
};
const DAMAGE_THEN_DESTROY_EQUIPMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Damage(6),
            Instruction::MayDestroyAttachedEquipment { recipient: 0 },
        ],
    ),
    ..CardRules::EMPTY
};

fn trigger(game: &mut Game, recipient: CardId) {
    game.begin_step(Step::BeginCombat);
    game.give_priority();
    choose_trigger_target(game, recipient);
}
fn choose_trigger_target(game: &mut Game, recipient: CardId) {
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
    panic!("targeted trigger did not reach the stack");
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

#[test]
fn the_next_end_step_unattaches_only_the_exact_selected_equipment_after_reassignment() {
    let _rules = fixture_rules::activate(&[(
        Card::from_name("Obliterating Bolt").unwrap(),
        &TEMPORARY_ATTACHMENT,
    )]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let selected = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let already = put(&mut game, player, "Gold Pan", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Lavaspur Boots",
        Zone::Battlefield,
    );
    let [
        creature_ref,
        other_ref,
        selected_ref,
        already_ref,
        foreign_ref,
    ] = [creature, other, selected, already, foreign].map(|id| game.object_ref(id));
    game.tap_objects(&[creature]);
    assert!(game.attach_equipment(already_ref, creature_ref));
    let spell = put(&mut game, player, "Obliterating Bolt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let view = game.observe(player.other()).equipment_attachment.unwrap();
    assert_eq!(view.candidates, vec![selected_ref]);
    assert_eq!(view.recipient.reference, creature_ref);
    assert_eq!(game.object_view(creature).controller, player);
    assert!(!game.object_view(creature).tapped);
    assert!(game.keywords(creature).contains(Keywords::HASTE));
    reject(&mut game, pick(already_ref));
    reject(&mut game, pick(foreign_ref));
    let mut declined = game.clone();
    act(&mut declined, FINISH);
    assert!(declined.observe(player).delayed_steps.is_none());

    act(&mut game, pick(selected_ref));
    assert_eq!(game.object_view(selected).attached_to, Some(creature_ref));
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    choose(&mut game, |action| {
        matches!(action, Action::Activate { card, ability: 0, target: Some(Target::Card(target)), .. }
            if *card == selected && *target == other)
    });
    settle(&mut game);
    assert_eq!(game.object_view(selected).attached_to, Some(other_ref));

    let mut blinked = game.clone();
    blinked.move_object(selected, Zone::Exile);
    blinked.move_object(selected, Zone::Battlefield);
    let returned = blinked.object_ref(selected);
    assert_ne!(returned, selected_ref);
    assert!(blinked.attach_equipment(returned, other_ref));
    for branch in [&mut game, &mut blinked] {
        branch.begin_step(Step::End);
        branch.give_priority();
        branch.prepare_triggers();
        assert_eq!(branch.state.stack.len(), 1);
        assert!(branch.state.stack[0].targets.is_empty());
        assert_eq!(
            branch.state.stack[0].event_object.as_ref().unwrap().object,
            selected_ref
        );
        settle(branch);
        assert_eq!(branch.object_view(already).attached_to, Some(creature_ref));
        assert!(branch.observe(player).delayed_steps.is_none());
    }
    assert_eq!(game.object_view(selected).attached_to, None);
    assert_eq!(blinked.object_view(selected).attached_to, Some(other_ref));
}

#[test]
fn the_same_callback_uses_the_opponents_next_end_step_if_cast_with_flash() {
    let _rules = fixture_rules::activate(&[(
        Card::from_name("Obliterating Bolt").unwrap(),
        &TEMPORARY_ATTACHMENT,
    )]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    put(&mut game, player, "High Fae Trickster", Zone::Battlefield);
    let creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let selected = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let selected_ref = game.object_ref(selected);
    let spell = put(&mut game, player, "Obliterating Bolt", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, spell, Some(Target::Card(creature)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(&mut game, pick(selected_ref));
    assert_eq!(
        game.object_view(selected).attached_to,
        Some(game.object_ref(creature))
    );
    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
    assert_eq!(game.state.active, opponent);
    assert_eq!(game.state.stack[0].controller, player);
    let saved = game.snapshot();
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let actor = game.actor().unwrap();
    indexed.apply_index(actor, game.revision(), index).unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(selected).attached_to, None);
}

fn enter_with_attachment_trigger(game: &mut Game, recipient: CardId) -> CardId {
    let equipment = put(game, Player::P0, "Short Bow", Zone::Hand);
    cast(game, equipment, None);
    choose_trigger_target(game, recipient);
    equipment
}

fn begin_blocks(game: &mut Game, attacker: CardId) {
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(game, Action::Pass);
    }
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::Blockers);
}

#[test]
fn the_successful_optional_attachment_grants_exact_first_strike_and_requires_a_legal_block() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &ENTRY_ATTACHMENT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let bystander = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let blocker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let equipment = enter_with_attachment_trigger(&mut game, attacker);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
    assert!(game.state.stack.is_empty());
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(equipment).attached_to, None);
    assert_eq!(declined.block_requirement_weight(attacker), 0);
    assert!(!declined.keywords(attacker).contains(Keywords::FIRST_STRIKE));

    let accept = Action::ChooseOptional { accept: true };
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == accept)
        .unwrap();
    let saved = game.snapshot();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, accept);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    let original = game.object_ref(attacker);
    assert_eq!(game.object_view(equipment).attached_to, Some(original));
    assert!(game.state.stack.is_empty());
    assert!(game.keywords(attacker).contains(Keywords::FIRST_STRIKE));
    assert_eq!(game.block_requirement_weight(attacker), 1);
    assert!(!game.keywords(bystander).contains(Keywords::FIRST_STRIKE));
    assert_eq!(game.block_requirement_weight(bystander), 0);

    game.move_object(equipment, Zone::Graveyard);
    let mut expired = game.clone();
    expired.begin_step(Step::End);
    expired.give_priority();
    act(&mut expired, Action::Pass);
    act(&mut expired, Action::Pass);
    assert!(!expired.keywords(attacker).contains(Keywords::FIRST_STRIKE));
    assert_eq!(expired.block_requirement_weight(attacker), 0);
    let mut blinked = game.clone();
    blinked.move_object(attacker, Zone::Exile);
    blinked.move_object(attacker, Zone::Battlefield);
    assert_ne!(blinked.object_ref(attacker), original);
    assert!(!blinked.keywords(attacker).contains(Keywords::FIRST_STRIKE));
    assert_eq!(blinked.block_requirement_weight(attacker), 0);

    let mut unavailable = game.clone();
    unavailable.tap_objects(&[blocker]);
    begin_blocks(&mut unavailable, attacker);
    act(&mut unavailable, Action::FinishBlockers);

    begin_blocks(&mut game, attacker);
    reject(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(game.object_view(attacker).damage, 0);
}

#[test]
fn an_already_attached_or_departed_source_cannot_pay_for_the_follow_up() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &ENTRY_ATTACHMENT)]);
    let player = Player::P0;
    for depart in [false, true] {
        let mut game = scenario(player);
        let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let equipment = enter_with_attachment_trigger(&mut game, recipient);
        if depart {
            game.move_object(equipment, Zone::Graveyard);
        } else {
            assert!(game.attach_equipment(game.object_ref(equipment), game.object_ref(recipient)));
        }
        settle(&mut game);
        assert!(!game.keywords(recipient).contains(Keywords::FIRST_STRIKE));
        assert_eq!(game.block_requirement_weight(recipient), 0);
    }
}

#[test]
fn lethal_damage_leaves_time_to_destroy_one_attached_equipment_without_targeting_it() {
    let _rules = fixture_rules::activate(&[(
        Card::from_name("Lightning Strike").unwrap(),
        &DAMAGE_THEN_DESTROY_EQUIPMENT,
    )]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let shielded = put(&mut game, player, "Gold Pan", Zone::Battlefield);
    let loose = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let [creature_ref, foreign_ref, shielded_ref, loose_ref] =
        [creature, foreign, shielded, loose].map(|id| game.object_ref(id));
    for equipment in [foreign_ref, shielded_ref] {
        assert!(game.attach_equipment(equipment, creature_ref));
    }
    game.state.objects[foreign as usize].gained_keywords = Keywords::HEXPROOF;
    game.add_counter(shielded, CounterKind::Shield, 1);
    let spell = put(&mut game, player, "Lightning Strike", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    assert_eq!(game.state.stack[0].targets.len(), 1);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).damage, 6);
    let view = game.observe(player.other()).effect_choice.unwrap();
    assert!(view.optional);
    assert_eq!(view.candidates, vec![foreign_ref, shielded_ref]);
    reject(&mut game, Action::ChooseEffectObject { object: loose_ref });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(creature).zone, Zone::Graveyard);
    for equipment in [foreign, shielded] {
        assert_eq!(declined.object_view(equipment).zone, Zone::Battlefield);
        assert_eq!(declined.object_view(equipment).attached_to, None);
    }

    let mut prevented = game.clone();
    act(
        &mut prevented,
        Action::ChooseEffectObject {
            object: shielded_ref,
        },
    );
    assert_eq!(prevented.object_view(shielded).zone, Zone::Battlefield);
    assert_eq!(prevented.counter_count(shielded, CounterKind::Shield), 0);
    assert_eq!(prevented.object_view(foreign).zone, Zone::Battlefield);

    let action = Action::ChooseEffectObject {
        object: foreign_ref,
    };
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
    assert_eq!(game.object_view(shielded).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(shielded, CounterKind::Shield), 1);
}

#[test]
fn prevented_damage_still_offers_destruction_but_an_illegal_target_fizzles_the_spell() {
    let _rules = fixture_rules::activate(&[(
        Card::from_name("Lightning Strike").unwrap(),
        &DAMAGE_THEN_DESTROY_EQUIPMENT,
    )]);
    let player = Player::P0;
    for bounce in [false, true] {
        let mut game = scenario(player);
        let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let equipment = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
        let equipment_ref = game.object_ref(equipment);
        assert!(game.attach_equipment(equipment_ref, game.object_ref(creature)));
        game.add_counter(creature, CounterKind::Shield, 1);
        let spell = put(&mut game, player, "Lightning Strike", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        if bounce {
            let responder = game.actor().unwrap();
            let bounce = put(&mut game, responder, "Unsummon", Zone::Hand);
            cast(&mut game, bounce, Some(Target::Card(creature)));
            settle(&mut game);
            assert!(game.observe(player).effect_choice.is_none());
            assert_eq!(game.object_view(creature).zone, Zone::Hand);
            assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
        } else {
            act(&mut game, Action::Pass);
            act(&mut game, Action::Pass);
            assert_eq!(game.object_view(creature).damage, 0);
            assert_eq!(game.counter_count(creature, CounterKind::Shield), 0);
            act(
                &mut game,
                Action::ChooseEffectObject {
                    object: equipment_ref,
                },
            );
            assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
            assert_eq!(game.object_view(equipment).zone, Zone::Graveyard);
        }
    }
}
