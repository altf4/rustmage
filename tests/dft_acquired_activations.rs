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
