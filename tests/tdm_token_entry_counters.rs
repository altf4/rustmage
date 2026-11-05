//! E85: fixed token factories share the pre-entry event pipeline with cards and
//! recipes, while ordinary factories retain their direct no-provider path.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const DRAGON: Card = Card(5293);
const SPIRIT: Card = Card(5326);
const ROLE: Card = Card(5319);
const GLOBE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledSubtypeEntryCounters {
            subtype: "Dragon",
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const OTHER_SUBTYPE: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OtherSubtypeEntryCounters("Dragon")],
    }],
    ..GLOBE
};
const MANA_VALUE: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OtherControlledEntryCountersByManaValue {
                filter: ReplacementFilter::Permanent,
                kind: CounterKind::Charge,
                maximum: 0,
                at_most: 2,
                above: 1,
            },
        ],
    }],
    ..GLOBE
};
const DOUBLE: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Permanent,
    kind: None,
    factor: 2,
    add: 0,
    effects_only: false,
};
const ADD: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Permanent,
    kind: None,
    factor: 1,
    add: 1,
    effects_only: false,
};
const COUNTER_DOUBLE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COUNTER_ADD: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, ADD)],
    }],
    extras: &CardExtras {
        replacement_effects: &[ADD],
        ..CardExtras::EMPTY
    },
    ..COUNTER_DOUBLE
};
const TOKEN_DOUBLE: ReplacementSpec = ReplacementSpec::DoubleTokens {
    effects_only: false,
};
const TOKEN_DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, TOKEN_DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[TOKEN_DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const TOKEN_ADDITION: ReplacementSpec = ReplacementSpec::AddToken(DRAGON);
const TOKEN_ADDER: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, TOKEN_ADDITION)],
    }],
    extras: &CardExtras {
        replacement_effects: &[TOKEN_ADDITION],
        ..CardExtras::EMPTY
    },
    ..TOKEN_DOUBLER
};
const BARRIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};
const DRAGON_TYPES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ControlledCreatureSubtype {
            grant: CreatureSubtypeGrant::Named("Dragon"),
            required_subtype: None,
        }],
    }],
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::SelfEnters,
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::PlusOne),
                minimum: Some(1),
                maximum: Some(1),
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..CardRules::EMPTY
};
const CREATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: DRAGON,
                amount: 1,
                tapped: false,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
const EQUIPMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 1,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const REFLEXIVE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokensReflexiveWithSubjects {
                token: DRAGON,
                amount: 2,
                ability: 0,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 5)],
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn ready(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..160 {
        if predicate(g) {
            return;
        }
        choose(g, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("bounded token-entry scenario did not reach decision");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.entry_batch.is_none()
            && g.state.replacements.is_none()
            && g.state.pending == Pending::Priority
    });
}
fn advance(g: &mut Game) {
    if !g.advance_replacements(false) {
        g.advance_entries();
    }
}
fn create(g: &mut Game, p: Player, token: Card, amount: u128) {
    g.create_tokens(p, token, amount, false);
    advance(g);
    drain(g);
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.is_token() && object.zone == Zone::Battlefield && object.controller == p)
                .then_some(id as CardId)
        })
        .collect()
}
fn suppress(g: &mut Game, id: CardId) {
    let source = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .resolved_ability_modifications
        .push(ResolvedAbilityModificationView {
            recipient: source,
            source,
            keywords: Keywords::NONE,
            timestamp,
        });
}
fn choose_replacement(g: &mut Game, source: CardId) {
    choose(g, |action| {
        matches!(action, Action::ChooseReplacement { source: object, .. }
        if object.card == source)
    });
}
fn choice_root(p: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    let double = put(&mut g, p, "Short Bow", Zone::Battlefield);
    let add = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
    let source = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, source, None);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::Replacement { .. })
    });
    (g, double, add)
}
fn owned_completion(g: &Game) -> bool {
    g.state
        .entry_batch
        .as_ref()
        .and_then(|batch| batch.replacement_resume.as_ref())
        .is_some_and(|state| {
            state
                .work
                .iter()
                .any(|work| matches!(work, replacements::Work::CreatedTokens { .. }))
        })
}

#[test]
fn fixed_factories_without_entry_rules_keep_the_direct_no_provider_path() {
    let mut g = scenario(Player::P0);
    let groups = [TokenGroupView {
        token: DRAGON,
        amount: 2,
        tapped: false,
    }];
    assert!(!g.fixed_token_entries_required(&groups));
    g.create_tokens(Player::P0, DRAGON, 2, false);
    assert!(g.state.entry_batch.is_none() && g.state.replacements.is_none());
    assert_eq!(tokens(&g, Player::P0).len(), 2);
    for id in tokens(&g, Player::P0) {
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        assert_eq!(g.state.objects[id as usize].copy_kind, CopyKind::None);
        assert_eq!(stats(&g, id), (4, 4));
    }
}

#[test]
fn fixed_dragons_use_each_live_providers_current_controller_and_ability() {
    let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), &GLOBE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        let second = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
        create(&mut g, p, DRAGON, 2);
        for id in tokens(&g, p) {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
        }
        suppress(&mut g, first);
        create(&mut g, p, DRAGON, 1);
        let last = *tokens(&g, p).last().unwrap();
        assert_eq!(g.counter_count(last, CounterKind::PlusOne), 1);
        g.record_control(g.object_ref(second), p.other(), None);
        g.refresh_controllers();
        create(&mut g, p, DRAGON, 1);
        let last = *tokens(&g, p).last().unwrap();
        assert_eq!(g.counter_count(last, CounterKind::PlusOne), 0);
        create(&mut g, p.other(), DRAGON, 1);
        assert_eq!(
            g.counter_count(tokens(&g, p.other())[0], CounterKind::PlusOne),
            2
        );
    }
}

#[test]
fn all_three_pre_entry_provider_families_apply_to_fixed_factories() {
    for (provider, kind, expected) in [
        (&OTHER_SUBTYPE, CounterKind::PlusOne, 2),
        (&MANA_VALUE, CounterKind::Charge, 2),
    ] {
        let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), provider)]);
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        put(&mut g, Player::P0, "Shivan Dragon", Zone::Battlefield);
        put(&mut g, Player::P0, "Shivan Dragon", Zone::Battlefield);
        create(&mut g, Player::P0, DRAGON, 3);
        for id in tokens(&g, Player::P0) {
            assert_eq!(g.counter_count(id, kind), expected);
        }
    }
}

#[test]
fn effective_types_and_copy_type_exceptions_control_dragon_eligibility() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &DRAGON_TYPES),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let types = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    create(&mut g, Player::P0, SPIRIT, 1);
    let spirit = tokens(&g, Player::P0)[0];
    assert!(g.has_subtype(spirit, "Dragon") && g.has_subtype(spirit, "Spirit"));
    assert_eq!(g.counter_count(spirit, CounterKind::PlusOne), 1);
    suppress(&mut g, types);
    create(&mut g, Player::P0, SPIRIT, 1);
    let spirit = *tokens(&g, Player::P0).last().unwrap();
    // Pinned CR613.6: the layer-four type contribution has already applied
    // before the source loses its ability in layer six.
    assert!(g.has_subtype(spirit, "Dragon"));
    assert_eq!(g.counter_count(spirit, CounterKind::PlusOne), 1);
    g.move_object(types, Zone::Hand);
    create(&mut g, Player::P0, SPIRIT, 1);
    let spirit = *tokens(&g, Player::P0).last().unwrap();
    assert!(!g.has_subtype(spirit, "Dragon"));
    assert_eq!(g.counter_count(spirit, CounterKind::PlusOne), 0);
    let mut recipe = CopyRecipe::new(DRAGON);
    recipe.exceptions = CopyExceptions::GOLEM;
    g.stage_copy_creation(Player::P0, recipe, 1);
    advance(&mut g);
    drain(&mut g);
    let additive = *tokens(&g, Player::P0).last().unwrap();
    // GOLEM adds a copiable subtype; only the explicit replace exception
    // suppresses the copied Dragon subtype in GrantedSubtypes::apply_copy_types.
    assert!(g.has_subtype(additive, "Golem") && g.has_subtype(additive, "Dragon"));
    assert_eq!(g.counter_count(additive, CounterKind::PlusOne), 1);
    recipe.overrides = Some(CopyOverrides {
        types: Some(CopyTypes {
            replace: true,
            golem: true,
            ..CopyTypes::default()
        }),
        ..CopyOverrides::NONE
    });
    g.stage_copy_creation(Player::P0, recipe, 1);
    advance(&mut g);
    drain(&mut g);
    let replacement = *tokens(&g, Player::P0).last().unwrap();
    assert!(g.has_subtype(replacement, "Golem") && !g.has_subtype(replacement, "Dragon"));
    assert_eq!(g.counter_count(replacement, CounterKind::PlusOne), 0);
}

#[test]
fn same_batch_new_providers_do_not_apply_retroactively_to_fixed_siblings() {
    const PROVIDER_TOKEN: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        ..GLOBE
    };
    let _scope = fixture_rules::activate(&[(DRAGON, &PROVIDER_TOKEN)]);
    let mut g = scenario(Player::P0);
    create(&mut g, Player::P0, DRAGON, 2);
    for id in tokens(&g, Player::P0) {
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
    }
    create(&mut g, Player::P0, DRAGON, 2);
    for id in &tokens(&g, Player::P0)[2..] {
        assert_eq!(g.counter_count(*id, CounterKind::PlusOne), 2);
    }
}

#[test]
fn replacement_added_and_doubled_groups_share_pre_event_provider_snapshot() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &TOKEN_DOUBLER),
        (card("Goldvein Pick"), &TOKEN_ADDER),
    ]);
    for add_first in [false, true] {
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        let double = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        let add = put(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
        g.create_tokens(Player::P0, SPIRIT, 1, false);
        assert!(g.advance_replacements(false));
        choose_replacement(&mut g, if add_first { add } else { double });
        drain(&mut g);
        let created = tokens(&g, Player::P0);
        assert_eq!(created.len(), if add_first { 4 } else { 3 });
        let dragons = created
            .iter()
            .filter(|&&id| g.has_subtype(id, "Dragon"))
            .count();
        assert_eq!(dragons, if add_first { 2 } else { 1 });
        for id in created {
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                u64::from(g.has_subtype(id, "Dragon"))
            );
        }
    }
}

#[test]
fn counter_replacement_order_finishes_before_entry_observers_or_next_instruction() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &COUNTER_DOUBLE),
        (card("Goldvein Pick"), &COUNTER_ADD),
        (card("Opt"), &CREATE),
    ]);
    for add_first in [false, true] {
        let (mut g, double, add) = choice_root(Player::P0);
        assert!(owned_completion(&g));
        assert!(tokens(&g, Player::P0).is_empty());
        assert_eq!(g.state.players[0].life, 20);
        let pending = g.observe(Player::P0).replacement_choice.unwrap();
        assert_eq!(pending.kind, "Counters");
        assert_eq!(pending.counters.unwrap().get(CounterKind::PlusOne), 1);
        choose_replacement(&mut g, if add_first { add } else { double });
        drain(&mut g);
        let id = tokens(&g, Player::P0)[0];
        assert_eq!(
            g.counter_count(id, CounterKind::PlusOne),
            if add_first { 4 } else { 3 }
        );
        assert_eq!(g.state.players[0].life, 21);
    }
}

#[test]
fn prohibited_entry_counters_do_not_prevent_creation_or_fire_counter_guarded_entry() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &BARRIER),
        (DRAGON, &OBSERVER),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    create(&mut g, Player::P0, DRAGON, 1);
    assert_eq!(
        g.counter_count(tokens(&g, Player::P0)[0], CounterKind::PlusOne),
        0
    );
    assert_eq!(g.state.players[0].life, 20);
}

#[test]
fn entry_counters_are_visible_to_entry_observers_before_separate_completion_counters() {
    let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), &GLOBE), (DRAGON, &OBSERVER)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.stage_token_creation(
        Player::P0,
        DRAGON,
        1,
        false,
        replacements::TokenCompletion::Counters {
            kind: CounterKind::PlusOne,
            amount: 1,
        },
    );
    advance(&mut g);
    drain(&mut g);
    assert_eq!(
        g.counter_count(tokens(&g, Player::P0)[0], CounterKind::PlusOne),
        2
    );
    assert_eq!(
        g.state.players[0].life, 27,
        "the exact-one entry guard freezes before the later second counter"
    );
}

#[test]
fn completion_counter_set_waits_for_entry_counter_replacement_then_runs_once() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &COUNTER_DOUBLE),
        (card("Goldvein Pick"), &COUNTER_ADD),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let double = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    put(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
    g.stage_token_creation(
        Player::P0,
        DRAGON,
        1,
        false,
        replacements::TokenCompletion::CounterSet(Box::new(Counters::of(CounterKind::Charge, 1))),
    );
    assert!(g.advance_replacements(false));
    assert!(owned_completion(&g));
    assert!(tokens(&g, Player::P0).is_empty());
    choose_replacement(&mut g, double);
    ready(&mut g, |g| {
        !tokens(g, Player::P0).is_empty() && matches!(g.state.pending, Pending::Replacement { .. })
    });
    let token = tokens(&g, Player::P0)[0];
    assert_eq!(g.counter_count(token, CounterKind::PlusOne), 3);
    assert_eq!(g.counter_count(token, CounterKind::Charge), 0);
    choose_replacement(&mut g, double);
    drain(&mut g);
    assert_eq!(g.counter_count(token, CounterKind::PlusOne), 3);
    assert_eq!(g.counter_count(token, CounterKind::Charge), 3);
}

#[test]
fn optional_equipment_attachment_waits_for_entry_choices_and_uses_entered_references() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &COUNTER_DOUBLE),
        (card("Goldvein Pick"), &COUNTER_ADD),
        (card("Basilisk Collar"), &EQUIPMENT),
    ]);
    for accept in [false, true] {
        let mut g = scenario(Player::P0);
        put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        put(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
        let equipment = put(&mut g, Player::P0, "Basilisk Collar", Zone::Battlefield);
        let source = g.object_ref(equipment);
        g.stage_token_creation(
            Player::P0,
            DRAGON,
            2,
            false,
            replacements::TokenCompletion::OptionalAttachSource(source),
        );
        assert!(g.advance_replacements(false));
        assert!(g.observe(Player::P0).created_token_choice.is_none());
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::CreatedToken { .. })
        });
        let candidates = g
            .observe(Player::P0)
            .created_token_choice
            .unwrap()
            .candidates;
        assert_eq!(candidates.len(), 2);
        for reference in &candidates {
            assert_eq!(g.object_ref(reference.card), *reference);
            assert_eq!(g.object_view(reference.card).zone, Zone::Battlefield);
        }
        if accept {
            act(
                &mut g,
                Action::ChooseCreatedToken {
                    object: candidates[1],
                },
            );
        } else {
            act(&mut g, Action::ChooseOptional { accept: false });
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(equipment).attached_to,
            accept.then_some(candidates[1])
        );
    }
}

#[test]
fn reflexive_created_subjects_are_queued_only_after_all_entry_choices_commit() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &COUNTER_DOUBLE),
        (card("Goldvein Pick"), &COUNTER_ADD),
        (card("Opt"), &REFLEXIVE),
    ]);
    let (mut g, _, _) = choice_root(Player::P0);
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.state.players[0].life, 20);
    ready(&mut g, |g| {
        g.state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .chain(&g.state.ordering_triggers)
            .any(|packet| packet.reflexive_subject.is_some())
    });
    assert_eq!(tokens(&g, Player::P0).len(), 2);
    let packet = g
        .state
        .pending_triggers
        .iter()
        .chain(&g.state.stack)
        .chain(&g.state.ordering_triggers)
        .find(|packet| packet.reflexive_subject.is_some())
        .unwrap();
    let subject = packet.reflexive_subject.as_ref().unwrap();
    assert_eq!(subject.additional_objects.len(), 1);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 26);
}

#[test]
fn intrinsic_fixed_token_counters_use_entry_pipeline_without_external_provider() {
    const INTRINSIC: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        extras: &CardExtras {
            fixed_entry_counters: &[(CounterKind::Charge, 2)],
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(DRAGON, &INTRINSIC)]);
    let mut g = scenario(Player::P0);
    create(&mut g, Player::P0, DRAGON, 1);
    assert_eq!(
        g.counter_count(tokens(&g, Player::P0)[0], CounterKind::Charge),
        2
    );
}

#[test]
fn fixed_token_type_choices_are_precommit_and_then_enable_provider_bonus() {
    const CHOOSE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ChosenCreatureType],
        }],
        extras: &CardExtras {
            entry_choice: Some(EntryChoiceKind::CreatureType),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(SPIRIT, &CHOOSE), (card("Swiftfoot Boots"), &GLOBE)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.create_tokens(Player::P0, SPIRIT, 1, false);
    advance(&mut g);
    assert!(matches!(
        g.state.pending,
        Pending::EntryValue { player: Player::P0 }
    ));
    assert!(tokens(&g, Player::P0).is_empty());
    act(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Dragon").unwrap()),
        },
    );
    drain(&mut g);
    let id = tokens(&g, Player::P0)[0];
    assert!(g.has_subtype(id, "Dragon"));
    assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
}

#[test]
fn ordinary_copy_sized_and_simultaneous_copy_entries_keep_the_same_provider_rules() {
    let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), &GLOBE)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.stage_copy_creation(Player::P0, CopyRecipe::new(DRAGON), 1);
    advance(&mut g);
    drain(&mut g);
    let copied = tokens(&g, Player::P0)[0];
    assert_eq!(g.counter_count(copied, CounterKind::PlusOne), 1);
    g.stage_copy_creation(
        Player::P0,
        CopyRecipe::new(DRAGON).with_sampled_size(copies::SampledTokenSize::new(70_000)),
        1,
    );
    advance(&mut g);
    drain(&mut g);
    let sized = *tokens(&g, Player::P0).last().unwrap();
    assert_eq!(stats(&g, sized), (70_001, 70_001));
    g.stage_simultaneous_copy_creation(&[Player::P0, Player::P1], CopyRecipe::new(DRAGON));
    advance(&mut g);
    drain(&mut g);
    let own = *tokens(&g, Player::P0).last().unwrap();
    let enemy = tokens(&g, Player::P1)[0];
    assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
}

#[test]
fn reused_fixed_token_slots_receive_fresh_counters_and_exact_new_generations() {
    let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), &GLOBE)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    create(&mut g, Player::P0, DRAGON, 1);
    let first = tokens(&g, Player::P0)[0];
    let previous = g.object_ref(first);
    g.move_object(first, Zone::Graveyard);
    g.state_based_actions();
    create(&mut g, Player::P0, DRAGON, 1);
    let next = tokens(&g, Player::P0)[0];
    assert_eq!(first, next);
    assert_ne!(g.object_ref(next), previous);
    assert_eq!(g.counter_count(next, CounterKind::PlusOne), 1);
    assert_eq!(stats(&g, next), (5, 5));
}

#[test]
fn pending_fixed_entry_completion_is_owned_transactional_and_restorable_after_parent_drop() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &COUNTER_DOUBLE),
        (card("Goldvein Pick"), &COUNTER_ADD),
        (card("Opt"), &REFLEXIVE),
    ]);
    let (parent, double, add) = choice_root(Player::P0);
    assert!(owned_completion(&parent));
    let outer_bytes = parent
        .state
        .entry_batch
        .as_ref()
        .unwrap()
        .replacement_resume
        .as_ref()
        .unwrap()
        .bytes();
    let outer = parent
        .state
        .entry_batch
        .as_ref()
        .unwrap()
        .replacement_resume
        .as_ref()
        .unwrap();
    let replacements::Work::CreatedTokens {
        created,
        completion,
        ..
    } = &outer.work[0]
    else {
        panic!("fixed entry owns its completion");
    };
    let replacements::TokenCompletion::ReflexiveCreated { item, .. } = completion.as_ref() else {
        panic!("this root retains the nested reflexive spell payload");
    };
    assert_eq!(
        outer_bytes,
        std::mem::size_of::<replacements::ReplacementState>()
            + std::mem::size_of::<replacements::Work>()
            + created.len() * std::mem::size_of::<ObjectRef>()
            + std::mem::size_of::<replacements::TokenCompletion>()
            + std::mem::size_of::<SpellView>()
            + item.owned_bytes()
    );
    let action = parent
        .legal_actions()
        .into_iter()
        .find(|action| {
            matches!(action,
        Action::ChooseReplacement { source, .. } if source.card == double)
        })
        .unwrap();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = parent.snapshot();
    let observations =
        [Player::P0, Player::P1].map(|p| serde_json::to_value(parent.observe(p)).unwrap());
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    assert_eq!(raw.revision(), revision);
    reject(
        &mut raw,
        Action::ChooseReplacement {
            source: ObjectRef {
                card: double,
                generation: u64::MAX,
            },
            ability: 0,
        },
    );
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(raw.observe(p)).unwrap()),
        observations
    );
    raw.apply(Player::P0, action.clone()).unwrap();
    indexed.apply_index(Player::P0, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(raw.observe(p)).unwrap()),
        [Player::P0, Player::P1].map(|p| serde_json::to_value(indexed.observe(p)).unwrap())
    );
    assert_eq!(
        raw.counter_count(tokens(&raw, Player::P0)[0], CounterKind::PlusOne),
        3
    );
    choose_replacement(&mut sibling, add);
    drain(&mut sibling);
    assert_eq!(
        sibling.counter_count(tokens(&sibling, Player::P0)[0], CounterKind::PlusOne),
        4
    );
    assert_eq!(sibling.state.players[0].life, 26);
    indexed.restore(&saved).unwrap();
    assert!(owned_completion(&indexed));
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    indexed.apply(Player::P0, action).unwrap();
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let frames = |g: &Game| {
        [Player::P0, Player::P1].map(|p| {
            let mut frame = serde_json::to_value(g.observe(p)).unwrap();
            frame["revision"] = serde_json::json!(0);
            frame
        })
    };
    assert_eq!(
        frames(&raw),
        frames(&indexed),
        "only the restored child's top-level revision is normalized"
    );
}

#[test]
fn unrelated_provider_preserves_doubled_roles_shared_timestamp_and_survivor_choice() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &TOKEN_DOUBLER),
    ]);
    for globe in [false, true] {
        for keep_last in [false, true] {
            let mut g = scenario(Player::P0);
            if globe {
                put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
            }
            put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
            let creature = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            let recipient = g.object_ref(creature);
            g.stage_token_creation(
                Player::P0,
                ROLE,
                1,
                false,
                replacements::TokenCompletion::RoleAttach(recipient),
            );
            advance(&mut g);
            g.state_based_actions();
            assert!(matches!(
                g.state.pending,
                Pending::Role { player: Player::P0 }
            ));
            let actions = g
                .legal_actions()
                .into_iter()
                .filter_map(|action| match action {
                    Action::KeepRole { object } => Some(object),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(actions.len(), 2);
            let timestamps = &g.state.roles.as_ref().unwrap().attached;
            assert_eq!(timestamps.len(), 2);
            assert_eq!(timestamps[0].timestamp, timestamps[1].timestamp);
            let keep = actions[usize::from(keep_last)];
            act(&mut g, Action::KeepRole { object: keep });
            drain(&mut g);
            assert_eq!(tokens(&g, Player::P0), vec![keep.card]);
            assert_eq!(g.object_view(keep.card).attached_to, Some(recipient));
            assert_eq!(stats(&g, creature), (3, 3));
        }
    }
}

#[test]
fn simultaneous_role_groups_keep_exact_original_recipients_with_unbound_additions() {
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &TOKEN_ADDER),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let first = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let second = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let recipients = vec![g.object_ref(first), g.object_ref(second)];
    g.stage_role_creation(Player::P0, ROLE, recipients.clone());
    advance(&mut g);
    drain(&mut g);
    let roles = tokens(&g, Player::P0)
        .into_iter()
        .filter(|&id| g.object_view(id).card == ROLE)
        .collect::<Vec<_>>();
    assert_eq!(roles.len(), 2);
    assert_eq!(g.object_view(roles[0]).attached_to, Some(recipients[0]));
    assert_eq!(g.object_view(roles[1]).attached_to, Some(recipients[1]));
    let dragon = tokens(&g, Player::P0)
        .into_iter()
        .find(|&id| g.object_view(id).card == DRAGON)
        .unwrap();
    assert_eq!(g.object_view(dragon).attached_to, None);
    assert_eq!(g.counter_count(dragon, CounterKind::PlusOne), 1);
}

#[test]
fn mobilize_defender_choice_and_immediate_defenders_wait_for_entry_then_keep_exact_delayed_refs() {
    const MOBILIZER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mobilize {
                    token: DRAGON,
                    amount: AmountValue::Fixed(2),
                },
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Short Bow"), &COUNTER_DOUBLE),
        (card("Goldvein Pick"), &COUNTER_ADD),
        (card("Bear Cub"), &MOBILIZER),
    ]);
    for choose_defenders in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let defender = choose_defenders.then(|| {
            let id = put(
                &mut g,
                p.other(),
                "Elspeth, Storm Slayer",
                Zone::Battlefield,
            );
            g.add_counter(id, CounterKind::Loyalty, 5);
            CombatDefender::Planeswalker(g.object_ref(id))
        });
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        g.move_object(source, Zone::Exile);
        if let Some(defender) = defender {
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::Mobilize { .. })
            });
            act(&mut g, Action::ChooseMobilizeDefender { defender });
            act(
                &mut g,
                Action::ChooseMobilizeDefender {
                    defender: CombatDefender::Player(p.other()),
                },
            );
        }
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert!(tokens(&g, p).is_empty());
        assert_eq!(g.state.players[p.index()].life, 20);
        drain(&mut g);
        let created = tokens(&g, p);
        assert_eq!(created.len(), 2);
        let delayed = &g.state.delayed_sacrifices.as_ref().unwrap()[0].view;
        assert_eq!(
            delayed.tokens,
            created
                .iter()
                .map(|&id| g.object_ref(id))
                .collect::<Vec<_>>()
        );
        for (index, id) in created.iter().copied().enumerate() {
            assert!(g.object_view(id).tapped && g.object_view(id).attacking);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 3);
            let expected = if index == 0 {
                defender.unwrap_or(CombatDefender::Player(p.other()))
            } else {
                CombatDefender::Player(p.other())
            };
            assert_eq!(
                g.state
                    .attackers
                    .iter()
                    .find(|a| a.card == id)
                    .unwrap()
                    .defender,
                Some(expected)
            );
        }
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        g.begin_step(Step::End);
        g.give_priority();
        drain(&mut g);
        assert!(tokens(&g, p).is_empty());
    }
}

#[test]
fn fixed_role_cohort_is_newer_than_existing_role_after_mixed_keyword_counter_timestamp_finalization()
 {
    const DRAGON_FLYING: CardRules = CardRules {
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ControlledSubtypeEntryCounters {
                subtype: "Dragon",
                kind: CounterKind::Flying,
                amount: 1,
            }],
        }],
        ..GLOBE
    };
    const ALL_FLYING: CardRules = CardRules {
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::OtherControlledEntryCountersByManaValue {
                    filter: ReplacementFilter::Permanent,
                    kind: CounterKind::Flying,
                    maximum: 0,
                    at_most: 1,
                    above: 1,
                },
            ],
        }],
        ..GLOBE
    };
    for role_gets_keyword in [false, true] {
        let provider = if role_gets_keyword {
            &ALL_FLYING
        } else {
            &DRAGON_FLYING
        };
        let _scope = fixture_rules::activate(&[
            (card("Swiftfoot Boots"), provider),
            (card("Short Bow"), &TOKEN_DOUBLER),
            (card("Goldvein Pick"), &TOKEN_ADDER),
        ]);
        for doubled in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let recipient = g.object_ref(creature);
            g.stage_token_creation(
                p,
                ROLE,
                1,
                false,
                replacements::TokenCompletion::RoleAttach(recipient),
            );
            advance(&mut g);
            drain(&mut g);
            let old = tokens(&g, p)[0];
            let old_ref = g.object_ref(old);
            let old_time = g
                .state
                .roles
                .as_ref()
                .unwrap()
                .attached
                .iter()
                .find(|row| row.aura == old_ref)
                .unwrap()
                .timestamp;
            assert!(old_time > 0);
            put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
            put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Short Bow", Zone::Battlefield);
            }
            g.stage_token_creation(
                p,
                ROLE,
                1,
                false,
                replacements::TokenCompletion::RoleAttach(recipient),
            );
            advance(&mut g);
            ready(&mut g, |g| {
                g.state.entry_batch.is_none() && g.state.replacements.is_none()
            });
            // Apply the ordinary Role uniqueness state-based action after the
            // complete simultaneous event, never during a pending entry choice.
            if g.state.pending == Pending::Priority {
                g.state_based_actions();
            }
            let roles = g
                .state
                .roles
                .as_ref()
                .unwrap()
                .attached
                .iter()
                .filter(|row| row.aura != old_ref && row.recipient == recipient)
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(roles.len(), if doubled { 2 } else { 1 });
            let cohort_time = roles[0].timestamp;
            assert!(cohort_time > old_time);
            for row in &roles {
                assert_eq!(
                    row.timestamp, cohort_time,
                    "the fixed Role cohort retains its shared survivor choice"
                );
                assert_eq!(
                    g.counter_count(row.aura.card, CounterKind::Flying),
                    u64::from(role_gets_keyword)
                );
            }
            for id in tokens(&g, p)
                .into_iter()
                .filter(|&id| g.object_view(id).card == DRAGON)
            {
                assert_eq!(g.counter_count(id, CounterKind::Flying), 1);
            }
            if doubled {
                let candidates = g
                    .legal_actions()
                    .into_iter()
                    .filter_map(|action| match action {
                        Action::KeepRole { object } => Some(object),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(candidates.len(), 2);
                assert!(!candidates.contains(&old_ref));
                act(
                    &mut g,
                    Action::KeepRole {
                        object: candidates[1],
                    },
                );
            }
            assert_ne!(
                g.object_view(old).zone,
                Zone::Battlefield,
                "a preexisting Role must not outrank its newly attached successor's deferred timestamp"
            );
            drain(&mut g);
        }
    }
}
