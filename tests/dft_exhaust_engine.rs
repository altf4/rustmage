//! Shared exhaust lifecycle contracts. These fixtures do not admit DFT cards.
mod support;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const fn ability(
    exhaust: bool,
    cost: ActivationCost,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const COUNTER: &[Instruction] = &[Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: 1,
}];
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        ability(
            true,
            ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            COUNTER,
        ),
        ability(false, ActivationCost::ZERO, COUNTER),
    ],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ExhaustAbilityActivated,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const GRAVE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::ExhaustAbilityActivated,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
const EXHAUST_MANA: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ability(
        true,
        ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 3, 0,
        ]))],
    )],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const REUSE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::ReuseExhaustBeforeFirstActivationOwnTurn],
}];
const REUSE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: REUSE_GROUP,
    extras: &CardExtras {
        static_ability_groups: REUSE_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const REDUCE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OtherControlledExhaustGenericReduction(2)],
}];
const REDUCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: REDUCE_GROUP,
    activated: &[ability(
        true,
        ActivationCost {
            generic: 4,
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        COUNTER,
    )],
    extras: &CardExtras {
        static_ability_groups: REDUCE_GROUP,
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COPIER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ability(
        false,
        ActivationCost::ZERO,
        &[Instruction::CopyNextExhaustAbility { ability: 0 }],
    )],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedExhaustAbilityActivated,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedActivatedAbility],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn direct(g: &mut Game, id: CardId, index: u16) {
    choose(
        g,
        |a| matches!(a, Action::Activate { card, ability, .. } if *card == id && *ability == index),
    );
}
fn mana(g: &mut Game, id: CardId) {
    choose(
        g,
        |a| matches!(a, Action::ActivateMana { card, ability: 0, .. } if *card == id),
    );
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("64-action shared exhaust resolution ceiling");
}
fn fact(g: &Game, p: Player) -> bool {
    g.state.players[p.index()]
        .turn_facts
        .exhaust_activated_this_turn
}
fn activation_available(g: &Game, id: CardId, index: u16) -> bool {
    g.legal_actions().iter().any(|a| matches!(a,
        Action::Activate { card, ability, .. } | Action::ActivateMana { card, ability, .. }
        | Action::AnnounceActivate { card, ability, .. } | Action::AnnounceMana { card, ability, .. }
        if *card == id && *ability == index))
}

#[test]
fn actual_exhaust_events_include_mana_and_graveyard_observers_but_not_other_activations() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SOURCE),
        (Card::HillGiant, &OBSERVER),
        (named("Goblin Piker"), &GRAVE_OBSERVER),
        (Card::LlanowarElves, &EXHAUST_MANA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p, "Hill Giant", Zone::Battlefield);
        put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        put(&mut g, p, "Goblin Piker", Zone::Graveyard);
        let mana_source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        direct(&mut g, source, 1);
        drain(&mut g);
        assert!(!fact(&g, p));
        assert_eq!(g.state.players[p.index()].life, life);
        direct(&mut g, source, 0);
        assert!(fact(&g, p));
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let previous_green = g.state.players[p.index()].mana[4];
        mana(&mut g, mana_source);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 6);
        assert_eq!(g.state.players[p.index()].mana[4], previous_green + 3);
        assert!(!activation_available(&g, source, 0));
        for viewer in [p, p.other()] {
            assert!(g.observe(viewer).turn_facts.unwrap()[p.index()].exhaust_activated_this_turn);
        }
    }
}

#[test]
fn refueler_permission_is_own_turn_before_first_event_and_never_clears_spent_history() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SOURCE),
        (named("Goblin Piker"), &REUSE),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let provider = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        direct(&mut g, source, 0);
        drain(&mut g);
        assert!(!activation_available(&g, source, 0));
        g.move_object(provider, Zone::Graveyard);
        g.move_object(provider, Zone::Battlefield);
        assert!(
            !activation_available(&g, source, 0),
            "new provider cannot reset player history"
        );
        g.finish_cleanup();
        assert!(!fact(&g, p));
        g.state.active = p.other();
        g.state.priority = p;
        g.state.pending = Pending::Priority;
        assert!(
            !activation_available(&g, source, 0),
            "opponent turn cannot waive exhaust"
        );
        g.state.active = p;
        g.state.priority = p;
        assert!(activation_available(&g, source, 0));
        direct(&mut g, source, 0);
        drain(&mut g);
        assert_eq!(g.object_view(source).counters.plus_one, 2);
        assert_ne!(g.object_view(source).exhausted_abilities & 1, 0);
        assert!(!activation_available(&g, source, 0));
        let before = g.snapshot();
        reject(
            &mut g,
            Action::Activate {
                card: source,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn projected_mana_consumes_one_reuse_but_preserves_enclosing_starting_authorization() {
    const OUTER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ability(
            true,
            ActivationCost {
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            COUNTER,
        )],
        extras: &CardExtras {
            exhaust_mask: 1,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &OUTER),
        (named("Goblin Piker"), &REUSE),
        (Card::LlanowarElves, &EXHAUST_MANA),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let outer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let second = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    g.state.objects[outer as usize].used_abilities = 1;
    g.state.objects[source as usize].used_abilities = 1;
    g.state.objects[second as usize].used_abilities = 1;
    g.state.players[p.index()].mana = [0; 6];
    assert!(
        activation_available(&g, outer, 0),
        "one waived exhaust mana activation can fund3"
    );
    act(
        &mut g,
        Action::AnnounceActivate {
            card: outer,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .exhaust_reuse_authorized
    );
    mana(&mut g, source);
    assert!(fact(&g, p));
    assert!(!activation_available(&g, second, 0));
    assert!(
        g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PayCosts { .. })),
        "a later mana event must not invalidate the already declared exhaust"
    );
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(outer).counters.plus_one, 1);
    // A fresh search cannot combine two spent exhaust sources under one override.
    let mut other = scenario(p);
    put(&mut other, p, "Goblin Piker", Zone::Battlefield);
    for _ in 0..2 {
        let id = put(&mut other, p, "Llanowar Elves", Zone::Battlefield);
        other.state.objects[id as usize].used_abilities = 1;
    }
    other.state.players[p.index()].mana = [0; 6];
    assert!(!other.can_fund(funding::FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 6,
        reserve: None,
        source: None
    }));
}

#[test]
fn exhaust_discounts_stack_for_other_controlled_sources_and_recheck_suppression() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &REDUCE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    assert_eq!(
        g.activation_generic_discount(first, named("Grizzly Bears"), 0),
        2
    );
    let third = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(
        g.activation_generic_discount(first, named("Grizzly Bears"), 0),
        4
    );
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    direct(&mut g, first, 0);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(first).counters.plus_one, 1);
    let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
    let second_ref = g.object_ref(second);
    g.state.objects[aura as usize].attached_to = Some(second_ref);
    assert_eq!(
        g.activation_generic_discount(third, named("Grizzly Bears"), 0),
        2,
        "suppressed reducers and opposing reducers contribute zero"
    );
    assert_eq!(g.object_view(enemy).controller, p.other());
}

#[test]
fn next_nonmana_exhaust_copy_is_independent_from_source_and_ignores_mana_and_spells() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SOURCE),
        (named("Goblin Piker"), &COPIER),
        (Card::LlanowarElves, &EXHAUST_MANA),
        (Card::HillGiant, &OBSERVER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let copier = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let mana_source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    direct(&mut g, copier, 0);
    drain(&mut g);
    g.move_object(copier, Zone::Graveyard);
    mana(&mut g, mana_source);
    drain(&mut g);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p.other())));
    drain(&mut g);
    let life = g.state.players[p.index()].life;
    direct(&mut g, source, 0);
    drain(&mut g);
    assert_eq!(
        g.object_view(source).counters.plus_one,
        2,
        "original and one copied effect"
    );
    assert_eq!(
        g.state.players[p.index()].life,
        life + 1,
        "copy is not an activation event"
    );
    assert!(!activation_available(&g, source, 0));
    let next = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    direct(&mut g, next, 0);
    drain(&mut g);
    assert_eq!(
        g.object_view(next).counters.plus_one,
        1,
        "delayed callback consumed once"
    );
}

#[test]
fn delayed_exhaust_copy_allows_retarget_and_owned_raw_index_restore() {
    const DAMAGE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
            ..ability(true, ActivationCost::ZERO, &[Instruction::Damage(2)])
        }],
        extras: &CardExtras {
            exhaust_mask: 1,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &DAMAGE),
        (named("Goblin Piker"), &COPIER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let copier = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    direct(&mut g, copier, 0);
    drain(&mut g);
    choose(
        &mut g,
        |a| matches!(a,Action::Activate{card,ability:0,target:Some(Target::Player(q)),..}if *card==source&&*q==p.other()),
    );
    for _ in 0..32 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert!(matches!(g.state.pending, Pending::Retarget { .. }));
    let action = Action::SetStackTarget {
        index: 0,
        target: Target::Player(p),
    };
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let actions = g.legal_actions();
    let index = actions.iter().position(|a| a == &action).unwrap();
    let snapshot = g.snapshot();
    let mut indexed = g.clone();
    let mut raw = g.clone();
    drop(g);
    indexed.apply_index(actor, revision, index).unwrap();
    act(&mut raw, action.clone());
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for viewer in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(raw.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    act(&mut raw, Action::FinishStackTargets);
    drain(&mut raw);
    assert_eq!(raw.state.players[p.index()].life, 18);
    assert_eq!(raw.state.players[p.other().index()].life, 18);
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    act(&mut indexed, Action::FinishStackTargets);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn copied_or_countered_exhaust_packets_do_not_emit_extra_events_and_cleanup_expires_delay() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SOURCE),
        (named("Goblin Piker"), &COPIER),
        (Card::HillGiant, &OBSERVER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let copier = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    direct(&mut g, copier, 0);
    drain(&mut g);
    g.finish_cleanup();
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    direct(&mut g, source, 0);
    drain(&mut g);
    assert_eq!(
        g.object_view(source).counters.plus_one,
        1,
        "expired delayed callback cannot copy"
    );
    assert_eq!(g.state.players[p.index()].life, 21);
    assert!(fact(&g, p));
}

#[test]
fn external_exhaust_grants_keep_separate_use_keys_from_recipient_and_other_grantors() {
    const GRANT_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPlaneswalkerActivation(0)],
    }];
    const GRANT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: GRANT_GROUP,
        activated: &[ability(
            true,
            ActivationCost::ZERO,
            &[Instruction::GainLife(PlayerSet::Controller, 1)],
        )],
        extras: &CardExtras {
            static_ability_groups: GRANT_GROUP,
            exhaust_mask: 1,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GRANT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let walker = put(&mut g, p, "Chandra, Spark Hunter", Zone::Battlefield);
    g.state.objects[walker as usize].counters.loyalty = 4;
    // Its printed first ability index has been used independently of both acquired abilities.
    g.state.objects[walker as usize].used_abilities = 1;
    for giver in [first, second] {
        let source = g.object_ref(walker);
        let grantor = g.object_ref(giver);
        act(
            &mut g,
            Action::AnnouncePlaneswalkerGrant {
                source,
                grantor,
                ability: 0,
            },
        );
        pay(&mut g);
        drain(&mut g);
    }
    assert_eq!(g.state.players[p.index()].life, 22);
    assert_eq!(g.observe(p).granted_exhaust_uses.unwrap().len(), 2);
    let source = g.object_ref(walker);
    let grantor = g.object_ref(first);
    reject(
        &mut g,
        Action::AnnouncePlaneswalkerGrant {
            source,
            grantor,
            ability: 0,
        },
    );
    g.move_object(first, Zone::Graveyard);
    g.move_object(first, Zone::Battlefield);
    let grantor = g.object_ref(first);
    act(
        &mut g,
        Action::AnnouncePlaneswalkerGrant {
            source,
            grantor,
            ability: 0,
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert_eq!(
        g.state.players[p.index()].life,
        23,
        "new giver incarnation grants a new exhaust ability"
    );
}

#[test]
fn exhaust_event_waits_for_every_suspended_activation_cost_and_survives_countering() {
    const PLACEMENT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ability(
            true,
            ActivationCost::ZERO,
            &[Instruction::GainLife(PlayerSet::Controller, 5)],
        )],
        extras: &CardExtras {
            exhaust_mask: 1,
            activation_put_counters: &[(0, CounterKind::PlusOne, 1)],
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PLACEMENT),
        (Card::HillGiant, &OBSERVER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    put(&mut g, p, "Loading Zone", Zone::Battlefield);
    put(
        &mut g,
        p,
        "Doc Samson, Super Psychiatrist",
        Zone::Battlefield,
    );
    direct(&mut g, source, 0);
    assert!(
        g.state.activation_payment.is_some(),
        "replacement choice suspends payment"
    );
    assert!(!fact(&g, p));
    assert_eq!(g.object_view(source).used_abilities & 1, 0);
    for _ in 0..8 {
        if g.state.activation_payment.is_none() {
            break;
        }
        choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
    }
    assert!(g.state.activation_payment.is_none());
    assert!(fact(&g, p));
    let packet = g
        .state
        .stack
        .iter()
        .find(|s| matches!(s.kind, StackKind::ActivatedAbility { ability: 0 }))
        .unwrap()
        .id;
    assert!(g.counter_stack_item(packet, false));
    drain(&mut g);
    assert_eq!(
        g.state.players[p.index()].life,
        21,
        "observer event already committed; original gain5 countered"
    );
    assert!(matches!(g.object_view(source).counters.plus_one, 3 | 4));
    assert!(!activation_available(&g, source, 0));
}

#[test]
fn distinct_borrowed_exhaust_abilities_retain_donor_slot_history_and_reuse_independently() {
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::BorrowGraveyardActivations("Elf")],
    }];
    const BORROWER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: GROUPS,
        extras: &CardExtras {
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &BORROWER),
        (Card::LlanowarElves, &SOURCE),
        (named("Goblin Piker"), &REUSE),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
    let second = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
    g.refresh_graveyard_borrowing();
    for donor in [first, second] {
        let graveyard = g.object_ref(donor);
        act(
            &mut g,
            Action::AnnounceGraveyardBorrowedAbility {
                card: source,
                graveyard,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        drain(&mut g);
    }
    assert_eq!(g.object_view(source).counters.plus_one, 2);
    assert_eq!(g.state.borrowed_uses.as_ref().unwrap().len(), 2);
    assert_eq!(
        g.object_view(source).used_abilities,
        0,
        "borrowed slots do not consume recipient printed bits"
    );
    g.finish_cleanup();
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let graveyard = g.object_ref(first);
    act(
        &mut g,
        Action::AnnounceGraveyardBorrowedAbility {
            card: source,
            graveyard,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(source).counters.plus_one, 3);
    let graveyard = g.object_ref(second);
    reject(
        &mut g,
        Action::AnnounceGraveyardBorrowedAbility {
            card: source,
            graveyard,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn kept_mana_rollback_preserves_exhaust_event_fact_and_exact_original_observer_packet() {
    let _rules = fixture_rules::activate(&[
        (Card::LlanowarElves, &EXHAUST_MANA),
        (Card::HillGiant, &OBSERVER),
    ]);
    for keep in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        g.state.objects[silver as usize].growth.power = -1;
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        let enemy_ref = g.object_ref(enemy);
        g.state.objects[aura as usize].attached_to = Some(enemy_ref);
        let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
        cast(&mut g, prison, None);
        for _ in 0..32 {
            if g.state.pending != Pending::Priority {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        drain(&mut g);
        let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 1, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        let sacrifice = Some(g.object_ref(prison));
        act(&mut g, Action::ChooseBargain { sacrifice });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        mana(&mut g, source);
        assert!(fact(&g, p));
        pay(&mut g);
        let clause = g
            .observe(p)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Additional)
            .unwrap() as u16;
        act(&mut g, Action::ChooseCastCost { clause });
        let object = g.object_ref(silver);
        act(&mut g, Action::ChooseAuraAttachment { object });
        assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
        act(&mut g, Action::ChooseManaRollback { keep });
        assert_eq!(fact(&g, p), keep);
        assert_eq!(g.object_view(source).exhausted_abilities & 1 != 0, keep);
        assert_eq!(g.object_view(source).tapped, keep);
        let life = g.state.players[p.index()].life;
        drain(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            life + if keep { 1 } else { 0 },
            "retained event queues one original observer, reversal queues none"
        );
    }
}
