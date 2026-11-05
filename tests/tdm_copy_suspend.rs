//! Shared spell-copy policy and suspend lifecycle; no TDM card admissions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const fn copy_program(retarget: bool) -> Instruction {
    Instruction::CopyCapturedSpell {
        count: SpellCopyCount::One,
        retarget,
    }
}
const NEXT: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyNextSpell { ability: 0 }],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedSpellCastAny,
        targets: TargetPlan::NONE,
        program: &[copy_program(true)],
    }],
    ..CardRules::EMPTY
};
const NARROW: CardRules = CardRules {
    effect: Effect::Creature(10, 10),
    activated: &[ActivatedAbility {
        program: &[Instruction::CopyNextInstantSorcery { ability: 0 }],
        ..NEXT.activated[0]
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedSpellCastSourcePower,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    ..CardRules::EMPTY
};
const NO_RETARGET: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: Some(2),
        },
        targets: TargetPlan::NONE,
        program: &[copy_program(false)],
    }],
    ..CardRules::EMPTY
};
const STORM: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpell {
            count: SpellCopyCount::EarlierSpellsThisTurn,
            retarget: true,
        }],
    }],
    ..CardRules::EMPTY
};
const MODAL: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Divide three damage",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 1,
                max: 2,
            }),
            program: &[Instruction::DamageDivided { total: 3 }],
        },
        SpellMode {
            text: "Draw a card",
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ]),
    ..CardRules::EMPTY
};
fn source(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Ghost Vacuum", Zone::Battlefield)
}
fn activate(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    drain(g);
}
fn drain(g: &mut Game) {
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
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
    panic!("bounded copy/suspend resolution");
}
fn wait(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        assert_ne!(
            g.state.pending,
            Pending::Retarget { player: Player::P0 },
            "unexpected retarget prompt"
        );
        assert_ne!(
            g.state.pending,
            Pending::Retarget { player: Player::P1 },
            "unexpected retarget prompt"
        );
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded copy/suspend decision");
}
fn entrants(g: &Game, card: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.card == card).then_some(id as CardId)
        })
        .collect()
}
fn event(g: &Game) -> &CopyEvent {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find_map(|item| item.copy_event.as_deref())
        .expect("captured spell")
}
fn first_spell(g: &mut Game, p: Player) {
    let spell = put(g, p, "Divination", Zone::Hand);
    cast(g, spell, None);
    drain(g);
}
fn counter_original(g: &mut Game, owner: Player, original: CardId) {
    if g.actor() != Some(owner.other()) {
        act(g, Action::Pass);
    }
    let cancel = put(g, owner.other(), "Cancel", Zone::Hand);
    cast(g, cancel, Some(Target::Spell(original)));
    wait(g, |g| {
        g.state.objects[original as usize].zone == Zone::Graveyard
    });
}

#[test]
fn next_arbitrary_spell_copy_is_one_shot_source_independent_and_permanent_copies_are_not_created() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &NEXT)]);
    for p in [Player::P0, Player::P1] {
        for name in ["Grizzly Bears", "Short Bow"] {
            let mut g = scenario(p);
            let provider = source(&mut g, p);
            activate(&mut g, provider);
            let doubler = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
            g.add_counter(doubler, CounterKind::Loyalty, 5);
            g.move_object(provider, Zone::Graveyard);
            let spell = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, spell, None);
            assert_eq!(event(&g).amount, 1);
            drain(&mut g);
            let all = entrants(&g, Card::from_name(name).unwrap());
            assert_eq!(all.len(), 2);
            assert_eq!(
                all.iter()
                    .filter(|&&id| g.state.objects[id as usize].is_token())
                    .count(),
                1
            );
            let second = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, second, None);
            drain(&mut g);
            assert_eq!(entrants(&g, Card::from_name(name).unwrap()).len(), 3);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
        }
    }
}

#[test]
fn unused_next_copy_expires_and_existing_narrow_callback_does_not_consume_a_permanent_cast() {
    for narrow in [false, true] {
        let provider_name = if narrow { "Hill Giant" } else { "Ghost Vacuum" };
        let _scope = fixture_rules::activate(&[(
            Card::from_name(provider_name).unwrap(),
            if narrow { &NARROW } else { &NEXT },
        )]);
        let mut g = scenario(Player::P0);
        let provider = put(&mut g, Player::P0, provider_name, Zone::Battlefield);
        activate(&mut g, provider);
        if !narrow {
            g.expire_spell_copies();
        }
        let creature = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        cast(&mut g, creature, None);
        drain(&mut g);
        assert_eq!(entrants(&g, Card::GrizzlyBears).len(), 1);
        let hand = g.hand_count(Player::P0);
        let draw = put(&mut g, Player::P0, "Divination", Zone::Hand);
        cast(&mut g, draw, None);
        drain(&mut g);
        assert_eq!(g.hand_count(Player::P0), hand + if narrow { 4 } else { 2 });
    }
}

#[test]
fn printed_storm_freezes_all_players_prior_actual_casts_including_countered_and_graveyard_spells() {
    let dragon = Card::from_name("Shivan Dragon").unwrap();
    let _scope = fixture_rules::activate(&[(dragon, &STORM)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let doubler = put(&mut g, p, "Elspeth, Storm Slayer", Zone::Battlefield);
        g.add_counter(doubler, CounterKind::Loyalty, 5);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, Some(Target::Card(bear)));
        counter_original(&mut g, p, growth);
        drain(&mut g);
        let flashback = put(&mut g, p, "Think Twice", Zone::Graveyard);
        act(&mut g, Action::AnnounceFlashback { card: flashback });
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.state.objects[flashback as usize].zone, Zone::Exile);
        let original = put(&mut g, p, "Shivan Dragon", Zone::Hand);
        cast(&mut g, original, None);
        assert_eq!(event(&g).amount, 3);
        counter_original(&mut g, p, original);
        assert_eq!(
            event(&g).amount,
            3,
            "later counterspell is excluded from frozen count"
        );
        drain(&mut g);
        let copies = entrants(&g, dragon);
        assert_eq!(copies.len(), 3);
        assert!(
            copies
                .iter()
                .all(|&id| g.state.objects[id as usize].is_token())
        );
        assert_eq!(
            g.state
                .players
                .iter()
                .map(|p| p.spells_cast_this_turn)
                .sum::<u64>(),
            5,
            "copies were not casts"
        );
    }
}

#[test]
fn zero_count_storm_and_countered_storm_ability_make_no_extra_permanents() {
    const ABILITY_COUNTER: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::Counter],
        ),
        ..CardRules::EMPTY
    };
    let dragon = Card::from_name("Shivan Dragon").unwrap();
    let _scope = fixture_rules::activate(&[
        (dragon, &STORM),
        (Card::from_name("Opt").unwrap(), &ABILITY_COUNTER),
    ]);
    for counter_trigger in [false, true] {
        let mut g = scenario(Player::P0);
        if counter_trigger {
            let plain = put(&mut g, Player::P0, "Divination", Zone::Hand);
            cast(&mut g, plain, None);
            drain(&mut g);
        }
        let original = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
        cast(&mut g, original, None);
        assert_eq!(event(&g).amount, u64::from(counter_trigger));
        if counter_trigger {
            let trigger = g
                .state
                .stack
                .iter()
                .find(|s| s.kind != StackKind::Spell)
                .unwrap()
                .id;
            let counter = put(&mut g, Player::P0, "Opt", Zone::Hand);
            cast(&mut g, counter, Some(Target::Stack(trigger)));
        }
        drain(&mut g);
        assert_eq!(entrants(&g, dragon), vec![original]);
    }
}

#[test]
fn no_retarget_policy_keeps_modal_targets_and_divisions_after_original_departure() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &NO_RETARGET),
        (Card::from_name("Abrade").unwrap(), &MODAL),
    ]);
    let mut g = scenario(Player::P0);
    source(&mut g, Player::P0);
    first_spell(&mut g, Player::P0);
    let a = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    g.add_counter(a, CounterKind::PlusOne, 10);
    g.add_counter(b, CounterKind::PlusOne, 10);
    let original = put(&mut g, Player::P0, "Abrade", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: original });
    act(&mut g, Action::ChooseMode { mode: 0 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    act(&mut g, Action::AllocateTargetAmount { amount: 1 });
    act(&mut g, Action::AllocateTargetAmount { amount: 2 });
    pay(&mut g);
    let inherited = event(&g).spell.clone();
    counter_original(&mut g, Player::P0, original);
    wait(&mut g, |g| {
        g.state
            .stack
            .iter()
            .any(|s| s.kind == StackKind::Spell && s.is_copy)
    });
    let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
    assert_eq!(copy.targets, inherited.targets);
    assert_eq!(copy.target_allocations, inherited.target_allocations);
    assert_eq!(copy.mode, inherited.mode);
    reject(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Player(Player::P0),
        },
    );
    drain(&mut g);
    assert_eq!(g.state.objects[a as usize].damage, 1);
    assert_eq!(g.state.objects[b as usize].damage, 2);
}

#[test]
fn no_retarget_policy_preserves_cast_x_and_paid_cost_facts_for_a_permanent_copy() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &NO_RETARGET)]);
    let mut g = scenario(Player::P0);
    source(&mut g, Player::P0);
    first_spell(&mut g, Player::P0);
    let original = put(&mut g, Player::P0, "Pterafractyl", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: original });
    act(&mut g, Action::ChooseX { value: 3 });
    pay(&mut g);
    let inherited = event(&g).spell.clone();
    wait(&mut g, |g| g.state.stack.iter().any(|s| s.is_copy));
    let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
    assert_eq!(copy.x, Some(3));
    assert_eq!(copy.paid_additional_cost, inherited.paid_additional_cost);
    drain(&mut g);
    let all = entrants(&g, Card::from_name("Pterafractyl").unwrap());
    assert_eq!(all.len(), 2);
    assert!(
        all.iter()
            .all(|&id| g.counter_count(id, CounterKind::PlusOne) == 3)
    );
}

#[test]
fn optional_retarget_copy_policy_is_owned_transactional_and_restorable_after_parent_disposal() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &NEXT)]);
    let mut parent = scenario(Player::P0);
    let provider = source(&mut parent, Player::P0);
    activate(&mut parent, provider);
    let victim = put(&mut parent, Player::P1, "Hill Giant", Zone::Battlefield);
    let original = put(&mut parent, Player::P0, "Shock", Zone::Hand);
    cast(&mut parent, original, Some(Target::Card(victim)));
    for _ in 0..12 {
        if matches!(parent.state.pending, Pending::Retarget { .. }) {
            break;
        }
        act(&mut parent, Action::Pass);
    }
    assert!(matches!(parent.state.pending, Pending::Retarget { .. }));
    let action = Action::SetStackTarget {
        index: 0,
        target: Target::Player(Player::P1),
    };
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    reject(
        &mut raw,
        Action::SetStackTarget {
            index: 9,
            target: Target::Player(Player::P0),
        },
    );
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    act(&mut raw, Action::FinishStackTargets);
    act(&mut indexed, Action::FinishStackTargets);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[1].life, 18);
    act(&mut sibling, Action::FinishStackTargets);
    drain(&mut sibling);
    assert_eq!(sibling.state.players[1].life, 20);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    act(&mut indexed, Action::FinishStackTargets);
    drain(&mut indexed);
    assert_eq!(indexed.snapshot(), raw.snapshot());
}

const SUSPENDER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        program: &[
            copy_program(false),
            Instruction::SuspendCapturedSpell { time: 4 },
        ],
        ..NO_RETARGET.triggers[0]
    }],
    ..CardRules::EMPTY
};
const ABILITY_COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
fn suspended(g: &mut Game, p: Player, name: &str) -> CardId {
    source(g, p);
    first_spell(g, p);
    let card = put(g, p, name, Zone::Hand);
    if name == "Pterafractyl" {
        act(g, Action::AnnounceCast { card });
        act(g, Action::ChooseX { value: 3 });
        pay(g);
    } else {
        cast(g, card, None);
    }
    drain(g);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
    assert_eq!(g.counter_count(card, CounterKind::Time), 4);
    assert!(
        g.observe(p)
            .suspend_cards
            .unwrap()
            .contains(&g.object_ref(card))
    );
    card
}
fn last_counter(g: &mut Game, card: CardId) {
    let amount = g.counter_count(card, CounterKind::Time);
    g.pay_counters(card, &Counters::of(CounterKind::Time, amount));
    g.give_priority();
}
fn offer(g: &mut Game) {
    wait(g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
}
fn take_suspend_cast(g: &mut Game, card: CardId) {
    let object = g.object_ref(card);
    act(
        g,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    act(g, Action::FinishResolutionCasts);
}
fn counter_kind(g: &mut Game, kind: StackKind) {
    wait(g, |g| g.state.stack.iter().any(|s| s.kind == kind));
    let id = g.state.stack.iter().find(|s| s.kind == kind).unwrap().id;
    let caster = g.actor().unwrap();
    let card = put(g, caster, "Opt", Zone::Hand);
    cast(g, card, Some(Target::Stack(id)));
    drain(g);
}

#[test]
fn suspend_grants_exact_physical_exile_card_and_only_owner_upkeep_ticks_with_counterable_packet() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER),
        (Card::from_name("Opt").unwrap(), &ABILITY_COUNTER),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = suspended(&mut g, p, "Hill Giant");
        let exact = g.object_ref(card);
        let copies = entrants(&g, Card::from_name("Hill Giant").unwrap());
        assert_eq!(copies.len(), 1);
        assert_ne!(copies[0], card);
        assert!(g.state.objects[copies[0] as usize].is_token());
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p;
        g.begin_step(Step::Upkeep);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.state.pending_triggers[0].controller, p);
        assert_eq!(g.state.pending_triggers[0].source.unwrap().object, exact);
        g.give_priority();
        counter_kind(&mut g, StackKind::SuspendTick);
        assert_eq!(g.counter_count(card, CounterKind::Time), 4);
        g.begin_step(Step::Upkeep);
        g.give_priority();
        drain(&mut g);
        assert_eq!(g.counter_count(card, CounterKind::Time), 3);
    }
}

#[test]
fn suspend_last_counter_removal_any_time_has_separate_counterable_cast_trigger_and_no_delayed_permission()
 {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER),
        (Card::from_name("Opt").unwrap(), &ABILITY_COUNTER),
    ]);
    let mut g = scenario(Player::P0);
    let card = suspended(&mut g, Player::P0, "Hill Giant");
    last_counter(&mut g, card);
    counter_kind(&mut g, StackKind::SuspendCast);
    assert_eq!(g.counter_count(card, CounterKind::Time), 0);
    assert!(g.state.resolution_free_casts.is_none());
    assert!(
        g.observe(Player::P1)
            .suspend_cards
            .unwrap()
            .contains(&g.object_ref(card))
    );
    g.begin_step(Step::Upkeep);
    assert!(g.state.pending_triggers.is_empty());
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::CastExiledDuringResolution { .. }))
    );
    g.add_counter(card, CounterKind::Time, 1);
    last_counter(&mut g, card);
    // A counter added while the last-counter trigger is waiting does not make
    // its exile-only intervening-if false.
    g.add_counter(card, CounterKind::Time, 2);
    offer(&mut g);
    act(&mut g, Action::FinishResolutionCasts);
    drain(&mut g);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
    assert_eq!(g.counter_count(card, CounterKind::Time), 2);
    assert!(g.state.resolution_free_casts.is_none());
}

#[test]
fn suspend_upkeep_last_counter_triggers_after_tick_and_rechecks_exact_suspension() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER)]);
    for removed_before_tick in [false, true] {
        let mut g = scenario(Player::P0);
        let card = suspended(&mut g, Player::P0, "Hill Giant");
        g.pay_counters(card, &Counters::of(CounterKind::Time, 3));
        g.begin_step(Step::Upkeep);
        g.give_priority();
        if removed_before_tick {
            g.move_object(card, Zone::Hand);
            drain(&mut g);
            assert!(g.observe(Player::P0).suspend_cards.is_none());
            assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
        } else {
            offer(&mut g);
            assert_eq!(g.counter_count(card, CounterKind::Time), 0);
            assert!(
                g.state
                    .stack
                    .iter()
                    .all(|s| s.kind != StackKind::SuspendTick)
            );
            act(&mut g, Action::FinishResolutionCasts);
            drain(&mut g);
        }
    }
}

#[test]
fn suspend_free_cast_gains_noncopyable_haste_and_control_loss_ends_it_permanently() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER)]);
    for loss_on_stack in [false, true] {
        let mut g = scenario(Player::P0);
        let card = suspended(&mut g, Player::P0, "Hill Giant");
        last_counter(&mut g, card);
        offer(&mut g);
        g.state.players[0].mana = [0; 6];
        take_suspend_cast(&mut g, card);
        let spell = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell)
            .unwrap()
            .clone();
        assert_eq!(spell.x, None);
        assert_eq!(spell.cast.unwrap().payment, [0; 6]);
        assert!(g.keywords(card).contains(Keywords::HASTE));
        let copied = g.allocate_spell_copy(&spell, Player::P0);
        let copy = copied.source.unwrap().id;
        assert!(!g.keywords(copy).contains(Keywords::HASTE));
        g.state.stack.push(copied);
        if loss_on_stack {
            let r = g.object_ref(card);
            g.change_controller(r, Player::P1);
            g.change_controller(r, Player::P0);
            assert!(!g.keywords(card).contains(Keywords::HASTE));
        }
        drain(&mut g);
        assert_eq!(g.counter_count(card, CounterKind::PlusOne), 0);
        assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
        assert_eq!(g.keywords(card).contains(Keywords::HASTE), !loss_on_stack);
        g.state.turn += 1;
        assert_eq!(g.keywords(card).contains(Keywords::HASTE), !loss_on_stack);
        let r = g.object_ref(card);
        g.change_controller(r, Player::P1);
        g.change_controller(r, Player::P0);
        assert!(!g.keywords(card).contains(Keywords::HASTE));
    }
}

#[test]
fn suspend_free_cast_still_pays_mandatory_additional_cost_and_cannot_bypass_cast_prohibition() {
    const COSTLY: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        extras: &rustmage_core::cards::authored::CardExtras {
            additional_cost: Some(AdditionalCost {
                kind: EffectChoiceKind::SacrificeRange { minimum: 1 },
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..rustmage_core::cards::authored::CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER),
        (Card::from_name("Grizzly Bears").unwrap(), &COSTLY),
    ]);
    // Construct a paid first cast with its mandatory sacrifice, then suspend
    // that exact second spell. Its later free cast must pay again.
    for prohibited in [false, true] {
        let mut g = scenario(Player::P0);
        source(&mut g, Player::P0);
        first_spell(&mut g, Player::P0);
        let fodder = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        let card = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card });
        let object = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object });
        act(&mut g, Action::FinishAdditionalCostGroup);
        pay(&mut g);
        let original_paid = event(&g).spell.paid_additional_cost.clone();
        assert_eq!(original_paid.as_ref().unwrap().selected, 1);
        assert_eq!(original_paid.as_ref().unwrap().count, 1);
        wait(&mut g, |g| g.state.stack.iter().any(|s| s.is_copy));
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|s| s.is_copy)
                .unwrap()
                .paid_additional_cost,
            original_paid
        );
        drain(&mut g);
        assert_eq!(g.state.objects[fodder as usize].zone, Zone::Graveyard);
        assert_eq!(g.counter_count(card, CounterKind::Time), 4);
        // Remove the copied permanent so it cannot supply the second cost.
        for id in entrants(&g, Card::from_name("Grizzly Bears").unwrap()) {
            g.move_object(id, Zone::Graveyard);
        }
        last_counter(&mut g, card);
        offer(&mut g);
        g.state.players[0].mana = [0; 6];
        g.state.casting_prohibited = prohibited;
        let object = g.object_ref(card);
        let cast_action = Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        };
        assert!(!g.legal_actions().contains(&cast_action));
        reject(&mut g, cast_action.clone());
        let next_fodder = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        if prohibited {
            assert!(!g.legal_actions().contains(&cast_action));
            act(&mut g, Action::FinishResolutionCasts);
            drain(&mut g);
            assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
        } else {
            assert!(g.legal_actions().contains(&cast_action));
            act(&mut g, cast_action);
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            let stale = g.object_ref(fodder);
            reject(&mut g, Action::ChooseAdditionalCost { object: stale });
            let object = g.object_ref(next_fodder);
            act(&mut g, Action::ChooseAdditionalCost { object });
            act(&mut g, Action::FinishAdditionalCostGroup);
            pay(&mut g);
            let paid = g
                .state
                .stack
                .iter()
                .find(|s| s.source.is_some_and(|source| source.id == card))
                .unwrap()
                .paid_additional_cost
                .as_ref()
                .unwrap();
            assert_eq!((paid.selected, paid.count), (1, 1));
            act(&mut g, Action::FinishResolutionCasts);
            drain(&mut g);
            assert_eq!(g.state.objects[next_fodder as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
            assert!(g.keywords(card).contains(Keywords::HASTE));
        }
    }
}

#[test]
fn suspend_never_rebinds_countered_original_reexiled_card_or_departed_last_counter_source() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER)]);
    let mut g = scenario(Player::P0);
    source(&mut g, Player::P0);
    first_spell(&mut g, Player::P0);
    let original = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
    cast(&mut g, original, None);
    counter_original(&mut g, Player::P0, original);
    drain(&mut g);
    assert_eq!(g.state.objects[original as usize].zone, Zone::Graveyard);
    assert!(g.observe(Player::P0).suspend_cards.is_none());
    assert_eq!(
        entrants(&g, Card::from_name("Hill Giant").unwrap()).len(),
        1
    );
    let mut g = scenario(Player::P0);
    let card = suspended(&mut g, Player::P0, "Hill Giant");
    let exact = g.object_ref(card);
    last_counter(&mut g, card);
    g.move_object(card, Zone::Exile);
    assert_ne!(g.object_ref(card), exact);
    g.add_counter(card, CounterKind::Time, 1);
    g.pay_counters(card, &Counters::of(CounterKind::Time, 1));
    drain(&mut g);
    assert!(g.observe(Player::P0).suspend_cards.is_none());
    assert!(g.state.resolution_free_casts.is_none());
}

#[test]
fn suspend_ephemeral_spell_copy_never_receives_persistent_grant_or_time_counters() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER)]);
    let mut g = scenario(Player::P0);
    let card = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
    cast(&mut g, card, None);
    let spell = g.state.stack.last().unwrap().clone();
    let copied = g.allocate_spell_copy(&spell, Player::P0);
    let copy = copied.source.unwrap().id;
    let mut packet = g.activation_item(Player::P0, card, 0);
    packet.copy_event = Some(Box::new(CopyEvent {
        spell: copied.clone(),
        amount: 1,
    }));
    g.state.stack.push(copied);
    g.suspend_captured_spell(&packet, 4);
    assert_eq!(g.state.objects[copy as usize].zone, Zone::Exile);
    assert_eq!(g.counter_count(copy, CounterKind::Time), 0);
    assert!(g.observe(Player::P0).suspend_cards.is_none());
    g.state_based_actions();
    g.give_priority();
    assert_eq!(g.state.objects[copy as usize].zone, Zone::Outside);
    let reused = g.allocate_spell_copy(&spell, Player::P0);
    assert_eq!(reused.source.unwrap().id, copy);
    assert_eq!(g.counter_count(copy, CounterKind::Time), 0);
    assert!(g.observe(Player::P0).suspend_cards.is_none());
}

#[test]
fn suspend_optional_cast_is_public_owned_transactional_and_restorable_after_parent_disposal() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER)]);
    let mut parent = scenario(Player::P0);
    let card = suspended(&mut parent, Player::P0, "Hill Giant");
    last_counter(&mut parent, card);
    offer(&mut parent);
    assert_eq!(
        parent.observe(Player::P0).suspend_cards,
        parent.observe(Player::P1).suspend_cards
    );
    assert!(parent.observe(Player::P1).actions.is_empty());
    let action = Action::CastExiledDuringResolution {
        object: parent.object_ref(card),
        face: CastFace::Normal,
    };
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    reject(
        &mut raw,
        Action::CastExiledDuringResolution {
            object: ObjectRef {
                card,
                generation: 9999,
            },
            face: CastFace::Normal,
        },
    );
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    pay(&mut raw);
    pay(&mut indexed);
    act(&mut raw, Action::FinishResolutionCasts);
    act(&mut indexed, Action::FinishResolutionCasts);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert!(raw.keywords(card).contains(Keywords::HASTE));
    act(&mut sibling, Action::FinishResolutionCasts);
    drain(&mut sibling);
    assert_eq!(sibling.state.objects[card as usize].zone, Zone::Exile);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    pay(&mut indexed);
    act(&mut indexed, Action::FinishResolutionCasts);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn suspend_free_x_is_zero_and_a_zero_toughness_creature_still_dies_after_cast() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER)]);
    let mut g = scenario(Player::P0);
    let card = suspended(&mut g, Player::P0, "Pterafractyl");
    last_counter(&mut g, card);
    offer(&mut g);
    g.state.players[0].mana = [0; 6];
    take_suspend_cast(&mut g, card);
    let spell = g
        .state
        .stack
        .iter()
        .find(|s| s.kind == StackKind::Spell)
        .unwrap();
    assert_eq!(spell.x, Some(0));
    assert_eq!(spell.cast.unwrap().payment, [0; 6]);
    assert!(g.keywords(card).contains(Keywords::HASTE));
    drain(&mut g);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
    assert!(g.observe(Player::P0).suspend_cards.is_none());
    assert!(!g.keywords(card).contains(Keywords::HASTE));
}

#[test]
fn suspend_haste_obeys_layer_six_removal_later_grants_and_exact_battlefield_departure() {
    const REMOVE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &SUSPENDER),
        (Card::from_name("Shock").unwrap(), &REMOVE),
    ]);
    let mut g = scenario(Player::P0);
    let card = suspended(&mut g, Player::P0, "Hill Giant");
    last_counter(&mut g, card);
    offer(&mut g);
    take_suspend_cast(&mut g, card);
    drain(&mut g);
    assert!(g.keywords(card).contains(Keywords::HASTE));
    let remove = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, remove, Some(Target::Card(card)));
    drain(&mut g);
    assert!(!g.keywords(card).contains(Keywords::HASTE));
    let source = g.object_ref(card);
    g.grant_keyword_effects(&[card], source, Keywords::HASTE);
    assert!(g.keywords(card).contains(Keywords::HASTE));
    // The later generic grant is independent from suspend's control duration.
    g.change_controller(source, Player::P1);
    assert!(g.keywords(card).contains(Keywords::HASTE));
    g.move_object(card, Zone::Hand);
    g.move_object(card, Zone::Battlefield);
    assert!(!g.keywords(card).contains(Keywords::HASTE));
    assert!(g.observe(Player::P0).suspend_cards.is_none());
}

#[test]
fn copy_suspend_hot_layout_and_empty_observation_wire_are_measured() {
    let g = scenario(Player::P0);
    let json = serde_json::to_value(g.observe(Player::P0)).unwrap();
    assert!(json.get("suspend_cards").is_none());
    let sizes = format!(
        "Object={} ObjectView={} State={} DamageSource={}\n",
        std::mem::size_of::<Object>(),
        std::mem::size_of::<ObjectView>(),
        std::mem::size_of::<State>(),
        std::mem::size_of::<DamageSource>()
    );
    eprintln!("tdm-copy-suspend layout: {sizes}");
    if let Ok(path) = std::env::var("RUSTMAGE_TDM_COPY_SUSPEND_LAYOUT") {
        std::fs::write(path, sizes).unwrap();
    }
}
