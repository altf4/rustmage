//! Shared sampled copiable bases and exact untargeted Endure recipients.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const SPIRIT: Card = Card(5326);
const ENDURE: &[Instruction] = &[Instruction::EndureScaled {
    recipient: EndureRecipient::Entering,
    amount: AmountValue::SourceTotalCounters,
    token: SPIRIT,
}];
const WARDEN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 1,
            nontoken: true,
            subtype: None,
            include_source: false,
        },
        targets: TargetPlan::NONE,
        program: ENDURE,
    }],
    ..CardRules::EMPTY
};
const MONUMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [1, 0, 1, 0, 1, 0],
            generic: 1,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateSizedToken {
            token: SPIRIT,
            size: AmountValue::GreatestCreatureToughness,
        }],
        sorcery: true,
        once_per_turn: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
    }],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const FIXED_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::CreateCopyTokenWithExceptions {
            subject: Subject::Targets,
            amount: 1,
            exceptions: CopyExceptions::GOLEM,
        }],
    ),
    ..CardRules::EMPTY
};
const STEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Controller,
        }],
    ),
    ..CardRules::EMPTY
};
const BARRIER_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CounterProhibition(1)],
}];
const BARRIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: BARRIER_GROUP,
    extras: &CardExtras {
        static_ability_groups: BARRIER_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn wait(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if predicate(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("32-action dynamic token budget exhausted");
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Spirit"))
            .then_some(id as CardId)
        })
        .collect()
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
}
fn root(p: Player, hot: u64, sparse: u64) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(source, CounterKind::PlusOne, hot);
    g.add_counter(source, CounterKind::Page, sparse);
    let entrant = put(&mut g, p, "Bear Cub", Zone::Hand);
    cast(&mut g, entrant, None);
    wait(&mut g, |g| {
        g.object_view(entrant).zone == Zone::Battlefield && g.state.stack.len() == 1
    });
    (g, source, entrant)
}
fn decision(g: &mut Game) {
    wait(g, |g| matches!(g.state.pending, Pending::Endure { .. }));
}
fn resolve_copy(g: &mut Game, name: &str, target: CardId) {
    let spell = put(g, g.state.active, name, Zone::Hand);
    cast(g, spell, Some(Target::Card(target)));
    settle(g);
}

#[test]
fn sacrificed_activation_samples_live_greatest_toughness_as_a_copiable_large_base() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &MONUMENT),
        (Card::from_name("Opt").unwrap(), &COPY),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tall = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.add_counter(tall, CounterKind::PlusOne, 69_997);
        let opposing = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.add_counter(opposing, CounterKind::PlusOne, 99_997);
        let monument = put(&mut g, p, "Short Bow", Zone::Battlefield);
        activate(&mut g, monument);
        assert_eq!(g.object_view(monument).zone, Zone::Graveyard);
        g.pay_counters(tall, &Counters::of(CounterKind::PlusOne, 1));
        settle(&mut g);
        let token = tokens(&g, p)[0];
        assert_eq!(
            stats(&g, token),
            (69_999, 69_999),
            "sample on resolution after sacrifice/response"
        );
        assert_eq!(g.counter_count(token, CounterKind::PlusOne), 0);
        assert_eq!(g.object_view(token).colors, 1);
        assert_eq!(g.keywords(token), Keywords::NONE);
        g.add_counter(token, CounterKind::PlusOne, 7);
        resolve_copy(&mut g, "Opt", token);
        let copied = *tokens(&g, p).last().unwrap();
        assert_eq!(
            stats(&g, copied),
            (69_999, 69_999),
            "the sampled base copies but seven counters do not"
        );
        resolve_copy(&mut g, "Opt", copied);
        assert_eq!(stats(&g, *tokens(&g, p).last().unwrap()), (69_999, 69_999));
    }
}

#[test]
fn a_later_copy_base_exception_replaces_the_sample_and_cda_is_suppressed() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &MONUMENT),
        (Card::from_name("Opt").unwrap(), &FIXED_COPY),
    ]);
    let mut g = scenario(Player::P0);
    let tall = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    g.add_counter(tall, CounterKind::PlusOne, 7);
    let source = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    activate(&mut g, source);
    settle(&mut g);
    let token = tokens(&g, Player::P0)[0];
    resolve_copy(&mut g, "Opt", token);
    let copied = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.is_token() && o.zone == Zone::Battlefield && id as CardId != token)
                .then_some(id as CardId)
        })
        .unwrap();
    assert_eq!(stats(&g, copied), (3, 3));
    assert!(g.has_subtype(copied, "Golem"));
    assert_eq!(
        g.object_view(copied).copy_recipe.unwrap().sampled_size,
        None
    );
    let cda = Card::from_name("Sandman, Shifting Scoundrel").unwrap();
    let recipe = CopyRecipe::new(cda).with_sampled_size(copies::SampledTokenSize::new(70_000));
    g.apply_temporary_copy(g.object_ref(tall), recipe, CopyDeadline::EndOfTurn);
    assert_eq!(
        stats(&g, tall),
        (70_007, 70_007),
        "copiable base precedes counters"
    );
}

#[test]
fn zero_token_creation_and_zero_endure_have_distinct_results() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &MONUMENT),
        (Card::GrizzlyBears, &WARDEN),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let before = g.state.objects.len();
    activate(&mut g, source);
    settle(&mut g);
    assert_eq!(
        g.state.objects.len(),
        before + 1,
        "create one zero-toughness token, then SBA removes it"
    );
    assert!(tokens(&g, Player::P0).is_empty());
    let (mut g, _, entrant) = root(Player::P0, 0, 0);
    let before = g.state.objects.len();
    settle(&mut g);
    assert_eq!(
        g.state.objects.len(),
        before,
        "endure zero creates no token"
    );
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 0);
}

#[test]
fn scaled_endure_sums_all_kinds_at_resolution_and_doubles_only_the_selected_event() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &WARDEN)]);
    for put_counters in [false, true] {
        let (mut g, source, entrant) = root(Player::P0, 2, 3);
        g.add_counter(source, CounterKind::Soul, 4);
        let elspeth = put(
            &mut g,
            Player::P0,
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        g.add_counter(elspeth, CounterKind::Loyalty, 5);
        decision(&mut g);
        let view = g.observe(Player::P0).endure_choice.unwrap();
        assert_eq!(
            (view.source, view.amount, view.token_size),
            (g.object_ref(entrant), 9, Some(9))
        );
        act(&mut g, Action::ChooseEndure { put_counters });
        settle(&mut g);
        assert_eq!(
            g.counter_count(entrant, CounterKind::PlusOne),
            if put_counters { 9 } else { 0 }
        );
        let created = tokens(&g, Player::P0);
        assert_eq!(created.len(), if put_counters { 0 } else { 2 });
        for id in created {
            assert_eq!(stats(&g, id), (9, 9));
        }
    }
}

#[test]
fn departed_source_quantity_uses_final_lki_and_does_not_rebind_to_new_source() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &WARDEN)]);
    let (mut g, source, entrant) = root(Player::P0, 2, 3);
    g.add_counter(source, CounterKind::Soul, 4);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.add_counter(source, CounterKind::PlusOne, 100);
    decision(&mut g);
    assert_eq!(g.observe(Player::P0).endure_choice.unwrap().amount, 9);
    act(&mut g, Action::ChooseEndure { put_counters: true });
    settle(&mut g);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 9);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 100);
}

#[test]
fn entered_recipient_uses_its_current_controller_as_chooser_and_counter_placer() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &WARDEN),
        (Card::from_name("Opt").unwrap(), &STEAL),
    ]);
    for original in [Player::P0, Player::P1] {
        let (mut g, _, entrant) = root(original, 2, 3);
        let current = original.other();
        if g.actor() != Some(current) {
            act(&mut g, Action::Pass);
        }
        let steal = put(&mut g, current, "Opt", Zone::Hand);
        cast(&mut g, steal, Some(Target::Card(entrant)));
        wait(&mut g, |g| g.object_view(entrant).controller == current);
        put(
            &mut g,
            current,
            "Doc Samson, Super Psychiatrist",
            Zone::Battlefield,
        );
        decision(&mut g);
        assert_eq!(g.actor(), Some(current));
        assert!(g.observe(original).endure_choice.is_none());
        act(&mut g, Action::ChooseEndure { put_counters: true });
        settle(&mut g);
        assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 6);
    }
}

#[test]
fn departed_recipient_uses_controller_lki_and_never_rebinds_to_returned_object() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &WARDEN),
        (Card::from_name("Opt").unwrap(), &STEAL),
    ]);
    let (mut g, source, entrant) = root(Player::P0, 2, 3);
    act(&mut g, Action::Pass);
    let steal = put(&mut g, Player::P1, "Opt", Zone::Hand);
    cast(&mut g, steal, Some(Target::Card(entrant)));
    wait(&mut g, |g| g.object_view(entrant).controller == Player::P1);
    let old = g.object_ref(entrant);
    g.move_object(source, Zone::Hand);
    g.move_object(entrant, Zone::Hand);
    g.move_object(entrant, Zone::Battlefield);
    assert_ne!(g.object_ref(entrant), old);
    settle(&mut g);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 0);
    assert!(tokens(&g, Player::P0).is_empty());
    let created = tokens(&g, Player::P1);
    assert_eq!(created.len(), 1);
    assert_eq!(stats(&g, created[0]), (5, 5));
}

#[test]
fn scaled_counter_prohibition_forces_one_correctly_sized_spirit() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &WARDEN),
        (Card::from_name("Short Bow").unwrap(), &BARRIER),
    ]);
    let (mut g, _, entrant) = root(Player::P0, 20_000, 50_000);
    put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 0);
    assert!(g.observe(Player::P0).endure_choice.is_none());
    let created = tokens(&g, Player::P0);
    assert_eq!(created.len(), 1);
    assert_eq!(stats(&g, created[0]), (70_000, 70_000));
}

#[test]
fn scaled_pending_choice_survives_owned_raw_index_restore_and_capacity_failure() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &WARDEN)]);
    let (mut parent, _, entrant) = root(Player::P0, u64::MAX, 1);
    decision(&mut parent);
    assert_eq!(
        parent.observe(Player::P0).endure_choice.unwrap().amount,
        1u128 << 64
    );
    let saved = parent.snapshot();
    let revision = parent.revision();
    let action = Action::ChooseEndure {
        put_counters: false,
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(
        sibling.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert!(matches!(
        sibling.apply(Player::P0, Action::ChooseEndure { put_counters: true }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(sibling.snapshot(), saved);
    assert_eq!(sibling.revision(), revision);
    raw.apply(Player::P0, action.clone()).unwrap();
    indexed.apply_index(Player::P0, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(
        stats(&raw, tokens(&raw, Player::P0)[0]),
        (1i128 << 64, 1i128 << 64)
    );
    assert_eq!(raw.counter_count(entrant, CounterKind::PlusOne), 0);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn dynamic_source_endure_uses_the_sample_and_exact_source_for_each_outcome() {
    const SOURCE: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::EndureScaled {
                recipient: EndureRecipient::Source,
                amount: AmountValue::SourceTotalCounters,
                token: SPIRIT,
            }],
            sorcery: false,
            ..MONUMENT.activated[0]
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &SOURCE)]);
    for mode in 0..3 {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 2);
        g.add_counter(source, CounterKind::Page, 3);
        activate(&mut g, source);
        if mode == 2 {
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            settle(&mut g);
        } else {
            decision(&mut g);
            assert_eq!(g.observe(Player::P0).endure_choice.unwrap().amount, 5);
            act(
                &mut g,
                Action::ChooseEndure {
                    put_counters: mode == 0,
                },
            );
            settle(&mut g);
        }
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            [7, 2, 0][mode]
        );
        let created = tokens(&g, Player::P0);
        assert_eq!(created.len(), usize::from(mode != 0));
        for token in created {
            assert_eq!(stats(&g, token), (5, 5));
        }
    }
}
