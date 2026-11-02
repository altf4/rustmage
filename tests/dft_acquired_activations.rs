//! Acquired nonmana instances retain immutable identity through funding previews.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const fn activation(exhaust: bool, once: bool, amount: u16) -> ActivatedAbility {
    ActivatedAbility {
        exhaust,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: if amount == 1 { ONE } else { TWO },
        sorcery: false,
        once_per_turn: once,
    }
}
const ONE: &[Instruction] = &[Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: 1,
}];
const TWO: &[Instruction] = &[Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: 2,
}];
const PRINTED_ONCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[activation(false, true, 1)],
    ..CardRules::EMPTY
};
const COPIED_EXHAUST: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[activation(true, false, 1)],
    ..CardRules::EMPTY
};
const GRANTED_ONCE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ..activation(false, true, 2)
    }],
    extras: &CardExtras {
        self_granted_activation_templates: &[0],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GRANTED_EXHAUST: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ..activation(true, false, 2)
    }],
    ..GRANTED_ONCE
};
fn acquire(g: &mut Game, source: CardId) -> Action {
    let timestamp = g.next_role_timestamp();
    let source = g.object_ref(source);
    g.grant_recipient_ability(
        source,
        Card::LlanowarElves,
        SelfGrantedAbilityKind::Activated { ability: 0 },
        timestamp,
        None,
    );
    Action::AnnounceSelfGrantedAbility {
        source,
        grant_timestamp: timestamp,
        counter_payment: Counters::default(),
    }
}
fn use_printed(g: &mut Game, card: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    settle(g);
}
fn use_acquired(g: &mut Game, action: Action) {
    act(g, action);
    pay(g);
    settle(g);
}

#[test]
fn once_per_turn_grants_fund_independently_of_used_printed_slot_and_each_other() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PRINTED_ONCE),
        (Card::LlanowarElves, &GRANTED_ONCE),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    use_printed(&mut g, source);
    assert_eq!(g.state.objects[source as usize].used_abilities, 1);
    let first = acquire(&mut g, source);
    let second = acquire(&mut g, source);
    let before = g.snapshot();
    assert!(g.legal_actions().contains(&first));
    assert!(g.legal_actions().contains(&second));
    assert_eq!(
        g.snapshot(),
        before,
        "funding does not alter real slot history"
    );
    use_acquired(&mut g, first.clone());
    reject(&mut g, first.clone());
    assert!(g.legal_actions().contains(&second));
    use_acquired(&mut g, second.clone());
    reject(&mut g, second);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 5);
    assert_eq!(g.state.objects[source as usize].used_abilities, 1);
    g.state.turn += 1;
    assert!(
        g.legal_actions().contains(&first),
        "a new turn resets each acquired once-per-turn use"
    );
}

#[test]
fn exhaust_grants_fund_independently_of_a_used_copy_slot_and_survive_owned_restore() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PRINTED_ONCE),
        (Card::GoblinPiker, &COPIED_EXHAUST),
        (Card::LlanowarElves, &GRANTED_EXHAUST),
    ]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Battlefield);
    use_printed(&mut parent, source);
    parent.apply_temporary_copy(
        parent.object_ref(source),
        CopyRecipe::new(Card::GoblinPiker),
        CopyDeadline::EndOfTurn,
    );
    use_printed(&mut parent, source);
    assert_eq!(parent.state.objects[source as usize].used_abilities, 1);
    let first = acquire(&mut parent, source);
    let second = acquire(&mut parent, source);
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == first)
        .expect("unused acquired instance remains legal");
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, first.clone());
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    pay(&mut raw);
    settle(&mut raw);
    pay(&mut indexed);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    reject(&mut raw, first.clone());
    use_acquired(&mut raw, second.clone());
    reject(&mut raw, second);
    assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 6);
    raw.state.turn += 1;
    reject(&mut raw, first.clone());
    indexed.restore(&saved).unwrap();
    assert!(indexed.legal_actions().contains(&first));
    assert_eq!(indexed.state.objects[source as usize].used_abilities, 1);
    assert_eq!(indexed.counter_count(source, CounterKind::PlusOne), 2);
}

#[test]
fn acquired_target_discount_and_additional_tap_cost_share_one_projected_payment_context() {
    const GRANTED_TARGET: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::TapPermanents(1),
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: true,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            ..activation(false, true, 2)
        }],
        extras: &CardExtras {
            self_granted_activation_templates: &[0],
            activation_target_counter_discounts: &[(0, CounterKind::PlusOne)],
            ..CardExtras::EMPTY
        },
        ..GRANTED_ONCE
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PRINTED_ONCE),
        (Card::LlanowarElves, &GRANTED_TARGET),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    use_printed(&mut g, source);
    let target = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    g.add_counter(target, CounterKind::PlusOne, 1);
    let fuel = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let action = acquire(&mut g, source);
    g.state.players[0].mana = [0; 6];
    reject(&mut g, action.clone());
    let mana = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    assert!(
        g.legal_actions().contains(&action),
        "reserve Caravan for the additional tap and use Forest for the discounted total"
    );
    act(&mut g, action.clone());
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    let fuel_reference = g.object_ref(fuel);
    act(
        &mut g,
        Action::ChooseAdditionalCost {
            object: fuel_reference,
        },
    );
    if g.legal_actions()
        .contains(&Action::FinishAdditionalCostGroup)
    {
        act(&mut g, Action::FinishAdditionalCostGroup);
    }
    act(&mut g, Action::TapMana { card: mana });
    pay(&mut g);
    settle(&mut g);
    assert!(g.state.objects[fuel as usize].tapped && g.state.objects[mana as usize].tapped);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 3);
    reject(&mut g, action);
}

fn with_counter_payment(mut action: Action, counter_payment: Counters) -> Action {
    let Action::AnnounceSelfGrantedAbility {
        counter_payment: payment,
        ..
    } = &mut action
    else {
        unreachable!()
    };
    *payment = counter_payment;
    action
}

#[test]
fn acquired_exhaust_remove_one_counter_pays_before_putting_two_and_zero_cannot_start() {
    const REMOVE: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                remove: Some((Some(CounterKind::PlusOne), 1)),
                ..ActivationCost::ZERO
            },
            ..activation(true, false, 2)
        }],
        ..GRANTED_EXHAUST
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PRINTED_ONCE),
        (Card::LlanowarElves, &REMOVE),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let empty = acquire(&mut g, source);
    let action = with_counter_payment(empty.clone(), Counters::of(CounterKind::PlusOne, 1));
    reject(&mut g, action.clone());
    reject(&mut g, empty.clone());
    g.add_counter(source, CounterKind::PlusOne, 1);
    assert!(g.legal_actions().contains(&action));
    reject(&mut g, empty);
    reject(
        &mut g,
        with_counter_payment(action.clone(), Counters::of(CounterKind::Charge, 1)),
    );
    act(&mut g, action.clone());
    assert_eq!(
        g.state.announcing.as_ref().unwrap().counter_payment,
        Counters::of(CounterKind::PlusOne, 1)
    );
    assert_eq!(
        g.counter_count(source, CounterKind::PlusOne),
        1,
        "declaring a payment does not remove it early"
    );
    pay(&mut g);
    assert_eq!(
        g.counter_count(source, CounterKind::PlusOne),
        0,
        "the cost is paid before resolution"
    );
    settle(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
    reject(&mut g, action);
}

#[test]
fn acquired_wildcard_counter_payment_keeps_its_kind_in_owned_raw_index_branches() {
    const REMOVE_ANY: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                remove: Some((None, 1)),
                ..ActivationCost::ZERO
            },
            ..activation(false, false, 2)
        }],
        ..GRANTED_ONCE
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &PRINTED_ONCE),
        (Card::LlanowarElves, &REMOVE_ANY),
    ]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Battlefield);
    parent.add_counter(source, CounterKind::PlusOne, 1);
    parent.add_counter(source, CounterKind::Charge, 1);
    let base = acquire(&mut parent, source);
    let plus = with_counter_payment(base.clone(), Counters::of(CounterKind::PlusOne, 1));
    let charge = with_counter_payment(base, Counters::of(CounterKind::Charge, 1));
    assert!(parent.legal_actions().contains(&plus));
    assert!(parent.legal_actions().contains(&charge));
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut other = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == charge)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, charge);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(
        indexed.state.announcing.as_ref().unwrap().counter_payment,
        Counters::of(CounterKind::Charge, 1)
    );
    drop(parent);
    pay(&mut raw);
    pay(&mut indexed);
    assert_eq!(raw.counter_count(source, CounterKind::Charge), 0);
    assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 1);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 3);
    act(&mut other, plus);
    pay(&mut other);
    settle(&mut other);
    assert_eq!(other.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(other.counter_count(source, CounterKind::Charge), 1);
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.counter_count(source, CounterKind::Charge), 1);
    assert_eq!(indexed.counter_count(source, CounterKind::PlusOne), 1);
}

#[test]
fn empty_acquired_counter_receipts_preserve_inherited_action_wire_and_nonempty_round_trips() {
    let action = Action::AnnounceSelfGrantedAbility {
        source: ObjectRef {
            card: 17,
            generation: 2,
        },
        grant_timestamp: 31,
        counter_payment: Counters::default(),
    };
    let old = serde_json::json!({
        "kind":"AnnounceSelfGrantedAbility",
        "source":{"card":17,"generation":2},
        "grant_timestamp":31,
    });
    assert_eq!(serde_json::to_value(&action).unwrap(), old);
    assert_eq!(serde_json::from_value::<Action>(old).unwrap(), action);
    let action = with_counter_payment(action, Counters::of(CounterKind::Charge, 1));
    let json = serde_json::to_value(&action).unwrap();
    assert!(json.get("counter_payment").is_some());
    assert_eq!(serde_json::from_value::<Action>(json).unwrap(), action);
}
