//! Rule fixtures for bending events, attack-trigger observers and creature mana.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 2,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const BEND_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BendsYou,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::ElementsBentThisTurn),
                minimum: 4,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const EARTH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[LAND, LAND]),
        &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::RegisterEarthbendReturn,
            Instruction::SelectGroup(1),
            Instruction::AnimateEarthbendTargets,
            Instruction::RegisterEarthbendReturn,
        ],
    ),
    ..CardRules::EMPTY
};
const AIR: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 2,
        }),
        &[Instruction::AirbendTargets],
    ),
    ..CardRules::EMPTY
};
const ASCENSION: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OwnAttackerAbilityTriggered,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::SourceCounterQuantity(CounterKind::Quest),
                minimum: 4,
                skip: 2,
            },
            Instruction::MayCopyRelatedTriggeredAbility { skip: 1 },
            Instruction::CopyRelatedTriggeredAbility,
        ],
    }],
    ..CardRules::EMPTY
};
const TARGETED_ATTACK: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::Damage(1)],
    }],
    ..CardRules::EMPTY
};
const EACH_ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledCreatureAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
const ENEMY_ATTACK: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreaturesAttackYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const COUNTER_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const ZERO_FIRE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCombatRedScaled(AmountValue::Fixed(0))],
    }],
    ..CardRules::EMPTY
};
const CADETS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfAttacks),
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCombatRed(2)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalAttackTrigger {
            condition: StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Lesson"),
                basic: false,
                other: false,
                cards_only: true,
            }),
            ability: 0,
        }],
    }],
    ..CardRules::EMPTY
};
const CUB: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledCreatureTapManaBonus([
            0, 0, 0, 0, 1, 0,
        ])],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn order_triggers(game: &mut Game) {
    for _ in 0..16 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    panic!("trigger ordering budget exhausted");
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn bend_history_counts_unique_elements_and_dispatches_each_action() {
    let _rules = fixture_rules::activate(&[
        (named("Impact Tremors"), &BEND_OBSERVER),
        (named("Lightning Strike"), &EARTH),
        (named("Opt"), &AIR),
        (named("Bear Cub"), &ZERO_FIRE),
    ]);
    let p = Player::P1;
    let mut game = scenario(p);

    let earth = put(&mut game, p, "Lightning Strike", Zone::Hand);
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let island = put(&mut game, p, "Island", Zone::Battlefield);
    announce(
        &mut game,
        earth,
        &[Target::Card(forest), Target::Card(island)],
    );
    settle(&mut game);
    assert_eq!(stats(&game, forest), (1, 1));
    assert_eq!(game.object_view(island).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].turn_facts.bent_elements, 2);
    let returns = game.earthbend_views();
    assert_eq!(
        returns.len(),
        1,
        "only Forest still waits for its departure"
    );
    assert_eq!(returns[0].subject, game.object_ref(forest));
    assert!(game.object_view(island).tapped);
    assert_eq!(game.object_view(island).kinds & 3, 2);

    put(&mut game, p, "Impact Tremors", Zone::Battlefield);
    put(&mut game, p.other(), "Impact Tremors", Zone::Battlefield);
    let empty = put(&mut game, p, "Opt", Zone::Hand);
    announce(&mut game, empty, &[]);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 0);

    let air = put(&mut game, p, "Opt", Zone::Hand);
    let a = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    announce(&mut game, air, &[Target::Card(a), Target::Card(b)]);
    settle(&mut game);
    assert_eq!(game.object_view(a).zone, Zone::Exile);
    assert_eq!(game.object_view(b).zone, Zone::Exile);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(
        game.state.players[p.other().index()].cards_drawn_this_turn,
        0
    );

    let attacker = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    attack(&mut game, &[attacker]);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].turn_facts.bent_elements, 7);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 0);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);

    // The waterbend payment owner exercises this hook from actual cost routes.
    game.record_bend(p.other(), bending::BendKind::Water);
    game.record_bend(p, bending::BendKind::Water);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 21);
    assert_eq!(game.state.players[p.other().index()].life, 20);
    assert_eq!(
        game.observe(p).turn_facts.unwrap()[p.index()].bent_elements,
        15
    );
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.players[p.index()].turn_facts.bent_elements, 0);
    assert_eq!(
        game.state.players[p.other().index()]
            .turn_facts
            .bent_elements,
        0
    );
}

#[test]
fn the_attack_source_must_cause_its_own_trigger_and_observers_order_last() {
    let _rules = fixture_rules::activate(&[
        (named("Impact Tremors"), &ASCENSION),
        (named("Grizzly Bears"), &EACH_ATTACKER),
        (named("Bear Cub"), &ENEMY_ATTACK),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let ascension = put(&mut game, p, "Impact Tremors", Zone::Battlefield);
    let attacker = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let companion = put(&mut game, p, "Llanowar Elves", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    attack(&mut game, &[attacker, companion]);
    let observers: Vec<_> = game
        .state
        .ordering_triggers
        .iter()
        .filter(|item| item.observes_trigger())
        .map(|item| item.id)
        .collect();
    assert_eq!(
        observers.len(),
        1,
        "only the event caused by the source itself"
    );
    reject(&mut game, Action::PutTrigger { id: observers[0] });
    order_triggers(&mut game);
    let stack = &game.state.stack;
    assert_eq!(
        stack.len(),
        5,
        "two attack events, a tap, a defender and an observer"
    );
    assert_eq!(stack[3].source().unwrap().object.card, enemy);
    assert_eq!(stack[4].source().unwrap().object.card, ascension);
    let cause = stack[4].related_stack.unwrap().id;
    assert!(stack[..3].iter().any(|item| {
        item.id == cause && item.event_object.as_ref().unwrap().object.card == attacker
    }));
    settle(&mut game);
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 1);
}

#[test]
fn a_related_attack_ability_can_be_copied_from_its_last_known_stack_packet() {
    let _rules = fixture_rules::activate(&[
        (named("Impact Tremors"), &ASCENSION),
        (named("Grizzly Bears"), &TARGETED_ATTACK),
        (named("Opt"), &COUNTER_ABILITY),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let ascension = put(&mut game, p, "Impact Tremors", Zone::Battlefield);
    game.add_counter(ascension, CounterKind::Quest, 3);
    let attacker = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let counter = put(&mut game, p, "Opt", Zone::Hand);
    attack(&mut game, &[attacker]);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    let cause = game.state.stack[0].id;
    assert_eq!(game.state.stack[1].related_stack.unwrap().id, cause);

    let mut removed = game.clone();
    cast(&mut removed, counter, Some(Target::Stack(cause)));
    pass_pair(&mut removed);
    assert!(removed.state.stack.iter().all(|item| item.id != cause));
    pass_pair(&mut removed);
    act(&mut removed, Action::ChooseOptional { accept: true });
    act(&mut removed, Action::FinishStackTargets);
    settle(&mut removed);
    assert_eq!(removed.counter_count(ascension, CounterKind::Quest), 4);
    assert_eq!(removed.state.players[p.other().index()].life, 19);

    pass_pair(&mut game);
    let checkpoint = game.snapshot();
    let accept = Action::ChooseOptional { accept: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == accept)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(p, game.revision(), index).unwrap();
    act(&mut game, accept);
    assert_eq!(indexed.snapshot(), game.snapshot());
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Player(p),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    assert!(game.state.stack.last().unwrap().is_copy);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 19);
    assert_eq!(game.state.players[p.other().index()].life, 19);
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 4);

    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, Action::ChooseOptional { accept: false });
    settle(&mut indexed);
    assert_eq!(indexed.state.players[p.index()].life, 20);
    assert_eq!(indexed.state.players[p.other().index()].life, 19);
}

#[test]
fn copied_firebending_is_a_second_bend_and_does_not_retrigger_the_ascension() {
    let _rules = fixture_rules::activate(&[
        (named("Impact Tremors"), &ASCENSION),
        (named("Authority of the Consuls"), &BEND_OBSERVER),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let ascension = put(&mut game, p, "Impact Tremors", Zone::Battlefield);
    game.add_counter(ascension, CounterKind::Quest, 3);
    put(&mut game, p, "Authority of the Consuls", Zone::Battlefield);
    let attacker = put(&mut game, p, "Fire Sages", Zone::Battlefield);
    attack(&mut game, &[attacker]);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 2);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.state.players[p.index()].turn_facts.bent_elements, 4);
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 4);
}

#[test]
fn conditional_firebending_is_owned_by_the_attacker_and_resolves_after_departure() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &CADETS)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let lesson = put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    let mut unqualified = game.clone();
    unqualified.move_object(lesson, Zone::Exile);
    attack(&mut unqualified, &[source]);
    assert!(unqualified.state.stack.is_empty());
    assert!(unqualified.state.pending_triggers.is_empty());

    attack(&mut game, &[source]);
    assert_eq!(game.state.stack.len(), 1);
    let packet = &game.state.stack[0];
    assert_eq!(packet.source().unwrap().object.card, source);
    assert_eq!(
        packet.attachment_trigger.as_ref().unwrap().recipient.card,
        source
    );
    game.move_object(source, Zone::Graveyard);
    game.move_object(lesson, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].combat_red_mana, 2);
    assert_eq!(game.state.players[p.index()].turn_facts.bent_elements, 4);
}

#[test]
fn creature_tap_mana_is_available_to_real_and_projected_payments() {
    let _rules = fixture_rules::activate(&[
        (named("Grizzly Bears"), &CUB),
        (named("Lightning Strike"), &EARTH),
    ]);
    let p = Player::P1;
    let mut game = scenario(p);
    let cub = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let island = put(&mut game, p, "Island", Zone::Battlefield);
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let spell = put(&mut game, p, "Giant Growth", Zone::Hand);
    let announce_growth = Action::AnnounceCast { card: spell };
    let mut ordinary = game.clone();
    ordinary.state.objects[forest as usize].tapped = true;
    ordinary.state.players[p.index()].mana = [0; 6];
    reject(&mut ordinary, announce_growth.clone());
    act(&mut ordinary, Action::TapMana { card: island });
    assert_eq!(ordinary.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);

    let earth = put(&mut game, p, "Lightning Strike", Zone::Hand);
    announce(
        &mut game,
        earth,
        &[Target::Card(island), Target::Card(forest)],
    );
    settle(&mut game);
    game.state.players[p.index()].mana = [0; 6];
    let mut attack_only = game.clone();
    attack(&mut attack_only, &[island]);
    assert_eq!(attack_only.state.players[p.index()].mana, [0; 6]);

    assert!(game.legal_actions().contains(&announce_growth));
    act(&mut game, announce_growth);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(cub),
        },
    );
    let tap = Action::TapMana { card: island };
    let index = game.legal_actions().iter().position(|a| *a == tap).unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(p, game.revision(), index).unwrap();
    act(&mut game, tap);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[p.index()].mana, [0, 1, 0, 0, 1, 0]);
    assert!(
        game.state.stack.is_empty(),
        "the bonus is a triggered mana ability"
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
    assert_eq!(stats(&game, cub), (5, 5));
}
