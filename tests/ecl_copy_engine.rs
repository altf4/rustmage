//! Typed engine contracts for the audited ECL copy and entry mechanics.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const COPY: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::CHANGELING,
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        entry_copy_keywords: Keywords::CHANGELING,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MIRROR: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::WithoutSubtype("Aura"),
        })),
        &[Instruction::CopyControlledNonlandsFromTarget],
    ),
    ..CardRules::EMPTY
};
fn wait_copy(g: &mut Game) {
    for _ in 0..8 {
        if matches!(g.state.pending, Pending::EntryCopy { .. }) {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("missing entry copy");
}
#[test]
fn copy_exception_is_copiable_owned_and_matches_funding_projection() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let original = put(&mut g, Player::P1, "Savannah Lions", Zone::Battlefield);
    let entrant = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let mut movement = g.entry_move(entrant, Zone::Battlefield, false);
    entry_copies::apply_blueprint(&mut movement, g.copiable_recipe(g.object_ref(original)));
    let goal = FundingGoal {
        actor: Player::P0,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&g, &goal);
    let mut projection = None;
    world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
        projection =
            Some(characteristic_query::CharacteristicQuery::in_world(next).object_view(entrant));
        false
    });
    g.stage_entries(vec![movement], None);
    g.advance_entries();
    assert_eq!(g.object_view(entrant), projection.unwrap());
    assert!(g.object_view(entrant).has_subtype("Eldrazi"));
    assert!(
        g.copiable_recipe(g.object_ref(entrant))
            .overrides
            .unwrap()
            .keywords
            .contains(Keywords::CHANGELING)
    );
    let second = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut g, second, None);
    wait_copy(&mut g);
    let action = Action::ChooseEntryCopy {
        object: Some(g.object_ref(entrant)),
    };
    let snapshot = g.snapshot();
    let mut branch = g.clone();
    let index = branch
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    branch
        .apply_index(branch.actor().unwrap(), branch.revision(), index)
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.snapshot(), branch.snapshot());
    settle(&mut g);
    assert!(g.object_view(second).has_subtype("Elf"));
    g.restore(&snapshot).unwrap();
    assert!(matches!(g.state.pending, Pending::EntryCopy { .. }));
}
#[test]
fn mirror_cohort_is_permanent_keeps_physical_identity_and_does_not_enter() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &MIRROR)]);
    let mut g = scenario(Player::P0);
    let recipient = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let artifact = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let land = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let donor = put(&mut g, Player::P1, "Island", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(donor)));
    settle(&mut g);
    for id in [recipient, artifact] {
        assert_eq!(g.state.objects[id as usize].card, Card::Island);
        assert_eq!(g.object_ref(id).generation, 0);
        assert!(
            g.temporary_copy_view(g.object_ref(id))
                .unwrap()
                .layers
                .iter()
                .all(|layer| matches!(layer.deadline, CopyDeadline::WhileSource { .. }))
        );
    }
    assert_eq!(g.state.objects[land as usize].card, Card::Forest);
    g.move_object(recipient, Zone::Graveyard);
    assert_eq!(
        g.state.objects[recipient as usize].card,
        Card::from_name("Savannah Lions").unwrap()
    );
}
#[test]
fn triggered_only_stack_rule_includes_delayed_triggers_and_excludes_activated_abilities() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let mut packet = g.activation_item(Player::P0, source, 0);
    let rule = TargetRule::TriggeredAbility { relation: 0 };
    assert!(!g.stack_target_rule_matches(rule, &packet, Player::P0));
    packet.kind = StackKind::DelayedReturn;
    assert!(g.stack_target_rule_matches(rule, &packet, Player::P0));
    assert!(!g.stack_target_rule_matches(rule, &packet, Player::P1));
}
#[test]
fn granted_persist_uses_predeath_counters_owner_and_exact_graveyard_incarnation() {
    const PERSIST: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            &[Instruction::GrantUntilEndTurn { ability: 0 }],
        ),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::ReturnDeath(
                ReturnDeathFilter::SourceWithoutCounter(CounterKind::MinusOne),
            )),
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnDeathCard(ReturnKind::Persist)],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &PERSIST)]);
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[target as usize].controller = Player::P0;
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    settle(&mut g);
    g.sacrifice_batch(&[(Player::P0, target)]);
    assert_eq!(g.state.pending_triggers.len(), 1);
    let packet = g.state.pending_triggers.pop().unwrap();
    assert_eq!(packet.controller, Player::P0);
    g.state.stack.push(packet);
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[target as usize].controller, Player::P1);
    assert_eq!(g.object_view(target).counters.get(CounterKind::MinusOne), 1);
    let again = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, again, Some(Target::Card(target)));
    settle(&mut g);
    g.sacrifice_batch(&[(Player::P1, target)]);
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn transformation_triggers_new_face_once_without_an_entry_event() {
    const TRANSFORMED: CardRules = CardRules {
        effect: Effect::Creature(4, 4),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrTransforms,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(
        Card::from_name("The Invincible Iron Man").unwrap(),
        &TRANSFORMED,
    )]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Tony Stark", Zone::Battlefield);
    let before = g.object_ref(source);
    g.transform_permanent(before);
    assert_eq!(g.object_ref(source), before);
    assert_eq!(g.state.pending_triggers.len(), 1);
    let packet = g.state.pending_triggers.pop().unwrap();
    g.state.stack.push(packet);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 23);
    g.transform_permanent(before);
    assert!(g.state.pending_triggers.is_empty());
}

fn wait_until(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if condition(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("missing expected engine decision");
}
fn push_trigger(g: &mut Game) {
    assert_eq!(g.state.pending_triggers.len(), 1);
    let packet = g.state.pending_triggers.pop().unwrap();
    g.state.stack.push(packet);
}
#[test]
fn unbounded_counter_removal_accepts_wide_named_raw_allocations_and_restores() {
    const REMOVE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            &[Instruction::RemoveAnyTargetCounters],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &REMOVE)]);
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.set_counters(
        target,
        Counters::of(CounterKind::Soul, u64::MAX).with(CounterKind::Charge, u64::MAX),
    );
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    wait_until(&mut g, |g| {
        matches!(g.state.pending, Pending::TargetCounterRemoval { .. })
    });
    assert!(g.legal_actions().len() < 40);
    let remove =
        Counters::of(CounterKind::Soul, u64::MAX - 7).with(CounterKind::Charge, u64::MAX - 11);
    assert!(
        g.legal_action_space()
            .counter_removal
            .unwrap()
            .contains(&remove)
    );
    let before = g.snapshot();
    reject(
        &mut g,
        Action::ChooseCounterRemoval {
            counters: Counters::of(CounterKind::Stun, 1),
        },
    );
    act(&mut g, Action::ChooseCounterRemoval { counters: remove });
    settle(&mut g);
    assert_eq!(g.counter_count(target, CounterKind::Soul), 7);
    assert_eq!(g.counter_count(target, CounterKind::Charge), 11);
    g.restore(&before).unwrap();
    let action = Action::ChooseCounterRemoval {
        counters: Counters::default(),
    };
    let mut branch = g.clone();
    let index = branch
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    branch
        .apply_index(Player::P0, branch.revision(), index)
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.snapshot(), branch.snapshot());
    assert_eq!(g.counter_count(target, CounterKind::Soul), u64::MAX);
}
const CHOSEN_LOOK: &[Instruction] = &[Instruction::SelectLibrary(LibrarySelectionSpec {
    look: 1,
    count: LibrarySelectionCount::Fixed(1),
    filter: LibrarySelectionFilter::SourceChosenCreatureType,
    optional: true,
    reveal: true,
    remainder: LibraryRemainder::OptionalGraveyard,
})];
const LOOK_SOURCE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: CHOSEN_LOOK,
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
#[test]
fn chosen_type_library_decision_is_private_and_survives_departure_or_copy_reset() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &LOOK_SOURCE)]);
    for reset_copy in [false, true] {
        let mut g = scenario(Player::P0);
        let top = put(&mut g, Player::P0, "Llanowar Elves", Zone::Library);
        let source = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        let source_ref = g.object_ref(source);
        g.set_entry_value(
            source_ref,
            EntryChoiceValue::CreatureType(CreatureType::from_name("Elf").unwrap()),
        );
        assert!(g.queue_trigger(&g.object_view(source), 0));
        if reset_copy {
            g.apply_temporary_copy(
                source_ref,
                CopyRecipe::new(Card::GrizzlyBears),
                CopyDeadline::WhileSource { source: source_ref },
            );
            assert_eq!(g.entry_value(source_ref), None);
        } else {
            g.move_object(source, Zone::Exile);
        }
        push_trigger(&mut g);
        wait_until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        assert!(g.observe(Player::P1).library_selection.is_none());
        let view = g.observe(Player::P0).library_selection.unwrap();
        assert_eq!(view.candidates, vec![g.object_ref(top)]);
        let before = g.snapshot();
        let mut decline = g.clone();
        act(&mut decline, Action::FinishLibrarySelection);
        assert!(
            decline
                .observe(Player::P0)
                .library_selection
                .unwrap()
                .ordering
        );
        assert!(decline.observe(Player::P1).library_selection.is_none());
        act(&mut decline, Action::OrderLibraryBottom { card: top });
        settle(&mut decline);
        assert_eq!(decline.object_view(top).zone, Zone::Graveyard);
        act(&mut g, Action::SelectLibrary { card: top });
        act(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        assert!(g.state.known_hand.contains(&g.object_ref(top)));
        g.restore(&before).unwrap();
        assert_eq!(g.object_view(top).zone, Zone::Library);
    }
}
const TWO_COLORS: AmountValue = AmountValue::Fixed(2);
const VIVID_SELECT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::ControlledColors,
                filter: LibrarySelectionFilter::Any,
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::Shuffle,
            },
            look: LibraryLook::UntilNonlandCount(TWO_COLORS),
            destination: LibraryPickDestination::ExileCastThisTurn,
            public: true,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
#[test]
fn vivid_library_selection_uses_injective_color_matching_and_paid_cast_permissions() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Divination").unwrap(), &VIVID_SELECT)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let mono = put(&mut g, Player::P0, "Giant Growth", Zone::Library);
    let multicolor = put(&mut g, Player::P0, "Good-Fortune Unicorn", Zone::Library);
    let land = put(&mut g, Player::P0, "Forest", Zone::Library);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, spell, None);
    wait_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
    for viewer in [Player::P0, Player::P1] {
        let view = g.observe(viewer).library_selection.unwrap();
        assert_eq!(
            view.remaining.iter().map(|o| o.id).collect::<Vec<_>>(),
            vec![land, multicolor, mono]
        );
        assert_eq!(view.maximum, 2);
    }
    reject(&mut g, Action::SelectLibrary { card: land });
    act(&mut g, Action::SelectLibrary { card: multicolor });
    act(&mut g, Action::SelectLibrary { card: mono });
    act(&mut g, Action::FinishLibrarySelection);
    settle(&mut g);
    for id in [mono, multicolor] {
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert!(
            g.state
                .exile_permissions
                .iter()
                .any(|row| row.object == g.object_ref(id) && row.cast_only && !row.spend_any_mana)
        );
    }
    g.state.players[0].mana = [0; 6];
    assert!(!g.legal_actions().iter().any(
        |a| matches!(a, Action::Cast { card, .. } | Action::AnnounceCast { card } if *card == mono)
    ));
}
#[test]
fn shared_creature_type_anchors_are_all_chosen_by_caster_and_sacrifice_simultaneously() {
    const KEEP: CardRules = CardRules {
        effect: Effect::Spell(TargetPlan::NONE, &[Instruction::KeepSharedCreatureTypes]),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Divination").unwrap(), &KEEP)]);
    let mut g = scenario(Player::P0);
    let ours = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let dies_ours = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(&mut g, Player::P1, "Savannah Lions", Zone::Battlefield);
    let dies_theirs = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, spell, None);
    wait_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    let invalid = g.object_ref(theirs);
    reject(&mut g, Action::ChooseSharedCreature { object: invalid });
    let ours_ref = g.object_ref(ours);
    act(&mut g, Action::ChooseSharedCreature { object: ours_ref });
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(g.object_view(dies_ours).zone, Zone::Battlefield);
    let theirs_ref = g.object_ref(theirs);
    act(&mut g, Action::ChooseSharedCreature { object: theirs_ref });
    settle(&mut g);
    for id in [ours, theirs] {
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
    for id in [dies_ours, dies_theirs] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
const BATCH_COPY: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreaturesFromGraveyardEnterYou { other: true },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyEntryBatchCreature],
    }],
    ..CardRules::EMPTY
};
#[test]
fn graveyard_entry_batch_choice_keeps_last_copiable_values_after_departure_and_reentry() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Llanowar Elves").unwrap(), &BATCH_COPY)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let b = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let moves = vec![
        g.entry_move(a, Zone::Battlefield, false),
        g.entry_move(b, Zone::Battlefield, false),
    ];
    g.stage_entries(moves, None);
    g.advance_entries();
    assert_eq!(g.state.pending_triggers.len(), 1);
    let old = g.object_ref(a);
    g.apply_temporary_copy(
        old,
        CopyRecipe::new(Card::from_name("Hill Giant").unwrap()),
        CopyDeadline::WhileSource { source: old },
    );
    g.move_object(a, Zone::Exile);
    g.move_object(a, Zone::Battlefield);
    assert_ne!(g.object_ref(a), old);
    push_trigger(&mut g);
    wait_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EntryBatchCopy { .. })
    });
    let invalid = g.object_ref(a);
    reject(&mut g, Action::ChooseEntryBatchCopy { object: invalid });
    assert_eq!(g.observe(Player::P0).entry_batch_copy.len(), 2);
    let action = Action::ChooseEntryBatchCopy { object: old };
    let before = g.snapshot();
    let mut branch = g.clone();
    let index = branch
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    branch
        .apply_index(Player::P0, branch.revision(), index)
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.snapshot(), branch.snapshot());
    settle(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.copy_kind != CopyKind::None
                && o.card.name() == "Hill Giant")
            .count(),
        1
    );
    g.restore(&before).unwrap();
    assert!(matches!(g.state.pending, Pending::EntryBatchCopy { .. }));
}
const CROWN_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(
        0,
        ReplacementSpec::FirstTokensCopyAttached,
    )],
}];
const CROWN: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: CROWN_GROUPS,
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::FirstTokensCopyAttached],
        static_ability_groups: CROWN_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn equipped_crown(g: &mut Game, player: Player, recipient: CardId) -> CardId {
    let source = put(g, player, "Swiftfoot Boots", Zone::Battlefield);
    g.state.objects[source as usize].attached_to = Some(g.object_ref(recipient));
    source
}
fn copies(g: &Game, definition: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.copy_kind != CopyKind::None && o.card == definition
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn first_token_event_copy_replacement_preserves_keyword_completion_tapped_status_and_owned_choice()
{
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &CROWN)]);
    let mut g = scenario(Player::P0);
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let crown = equipped_crown(&mut g, Player::P0, bear);
    let token = Card::from_name("@token/spirit_1_1_white_flying").unwrap();
    g.stage_token_creation(
        Player::P0,
        token,
        2,
        true,
        replacements::TokenCompletion::Keywords {
            source: g.object_ref(crown),
            keywords: Keywords::FLYING,
        },
    );
    assert!(g.advance_replacements(false));
    let snapshot = g.snapshot();
    let mut branch = g.clone();
    let action = Action::ChooseOptional { accept: true };
    let index = branch
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap();
    branch
        .apply_index(Player::P0, branch.revision(), index)
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.snapshot(), branch.snapshot());
    let tokens = copies(&g, Card::GrizzlyBears);
    assert_eq!(tokens.len(), 2);
    assert!(
        tokens
            .iter()
            .all(|&id| g.object_view(id).tapped && g.keywords(id).contains(Keywords::FLYING))
    );
    assert!(g.state.players[0].token_creation_attempted_this_turn);
    g.stage_token_creation(
        Player::P0,
        token,
        1,
        false,
        replacements::TokenCompletion::None,
    );
    assert!(!g.advance_replacements(false));
    assert_eq!(copies(&g, Card::GrizzlyBears).len(), 2);
    g.restore(&snapshot).unwrap();
    act(&mut g, Action::ChooseOptional { accept: false });
    assert!(copies(&g, Card::GrizzlyBears).is_empty());
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card == token)
            .count(),
        2
    );
}
#[test]
fn token_copy_first_event_ignores_zero_and_consumes_unattached_attempt() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &CROWN)]);
    let mut g = scenario(Player::P0);
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let crown = equipped_crown(&mut g, Player::P0, bear);
    let token = Card::from_name("@token/spirit_1_1_white_flying").unwrap();
    g.stage_token_creation(
        Player::P0,
        token,
        0,
        false,
        replacements::TokenCompletion::None,
    );
    assert!(!g.state.players[0].token_creation_attempted_this_turn);
    g.state.objects[crown as usize].attached_to = None;
    g.stage_token_creation(
        Player::P0,
        token,
        1,
        false,
        replacements::TokenCompletion::None,
    );
    assert!(!g.advance_replacements(false));
    g.state.objects[crown as usize].attached_to = Some(g.object_ref(bear));
    g.stage_token_creation(
        Player::P0,
        token,
        1,
        false,
        replacements::TokenCompletion::None,
    );
    assert!(!g.advance_replacements(false));
    assert!(copies(&g, Card::GrizzlyBears).is_empty());
}

const SPELL_COPY: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::TargetCount {
                filter: &CastFilter::InstantSorcery,
                count: 1,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::MayCopyCastSpellWithKeywords(Keywords::WITHER)],
    }],
    ..CardRules::EMPTY
};
#[test]
fn optional_spell_copy_grants_wither_to_exact_original_and_copy_but_not_copiable_values() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Llanowar Elves").unwrap(), &SPELL_COPY)]);
    for accept in [false, true] {
        let mut g = scenario(Player::P0);
        let observer = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        let target = put(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
        let other = put(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        wait_until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept });
        if accept {
            let copy = g.retarget_view().unwrap().item;
            assert!(g.keywords(spell).contains(Keywords::WITHER));
            assert!(
                g.keywords(copy.source.unwrap().id)
                    .contains(Keywords::WITHER)
            );
            assert!(!copy.source_keywords.contains(Keywords::WITHER));
            let independent = g.allocate_spell_copy(&copy, Player::P0);
            assert!(
                !g.keywords(independent.source.unwrap().id)
                    .contains(Keywords::WITHER)
            );
            g.move_object(independent.source.unwrap().id, Zone::Outside);
            let action = Action::SetStackTarget {
                index: 0,
                target: Target::Card(other),
            };
            let mut branch = g.clone();
            let index = branch
                .legal_actions()
                .iter()
                .position(|a| a == &action)
                .unwrap();
            branch
                .apply_index(Player::P0, branch.revision(), index)
                .unwrap();
            act(&mut g, action);
            assert_eq!(g.snapshot(), branch.snapshot());
            act(&mut g, Action::FinishStackTargets);
        }
        g.move_object(observer, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(
            g.counter_count(target, CounterKind::MinusOne),
            if accept { 2 } else { 0 }
        );
        assert_eq!(
            g.counter_count(other, CounterKind::MinusOne),
            if accept { 2 } else { 0 }
        );
        assert_eq!(g.object_view(target).damage, if accept { 0 } else { 2 });
        assert_eq!(g.state.players[0].instant_sorcery_cast_this_turn, 1);
    }
}
#[test]
fn optional_spell_copy_keeps_departed_spell_lki_and_capacity_failure_is_transactional() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Llanowar Elves").unwrap(), &SPELL_COPY)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Gigantosaurus", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    wait_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    let original = g
        .state
        .stack
        .iter()
        .position(|packet| packet.source.is_some_and(|source| source.id == spell))
        .unwrap();
    g.state.stack.remove(original);
    g.move_object(spell, Zone::Graveyard);
    let next = g.state.next_stack_id;
    g.state.next_stack_id = u64::MAX;
    let before = g.snapshot();
    assert!(matches!(
        g.apply(Player::P0, Action::ChooseOptional { accept: true }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    g.state.next_stack_id = next;
    act(&mut g, Action::ChooseOptional { accept: true });
    act(&mut g, Action::FinishStackTargets);
    settle(&mut g);
    assert_eq!(g.counter_count(target, CounterKind::MinusOne), 2);
    assert!(!g.keywords(spell).contains(Keywords::WITHER));
}
#[test]
fn target_count_counts_repeated_target_instances() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let bound = g.bind_target(Target::Card(target));
    let filter = CastFilter::TargetCount {
        filter: &CastFilter::InstantSorcery,
        count: 1,
    };
    let observer = g.object_view(source);
    for count in [0, 1, 2] {
        let targets = vec![bound; count];
        assert_eq!(
            g.cast_filter_matches_value(
                Some(Player::P0),
                filter,
                Card::from_name("Shock").unwrap(),
                1,
                Player::P0,
                &observer,
                &targets
            ),
            count == 1
        );
    }
}
const GRUB_ATTACK: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::MayBlightCopyAttacking],
    }],
    ..CardRules::EMPTY
};
#[test]
fn blight_copy_freezes_pre_cost_recipe_and_creates_intrinsic_sacrifice_without_haste() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &GRUB_ATTACK)]);
    for pay in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let victim = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        let foe = put(&mut g, Player::P1, "Llanowar Elves", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        wait_until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let invalid = g.object_ref(foe);
        reject(&mut g, Action::Blight { object: invalid });
        if pay {
            let object = g.object_ref(victim);
            let action = Action::Blight { object };
            let mut branch = g.clone();
            let index = branch
                .legal_actions()
                .iter()
                .position(|a| a == &action)
                .unwrap();
            branch
                .apply_index(Player::P0, branch.revision(), index)
                .unwrap();
            act(&mut g, action);
            assert_eq!(g.snapshot(), branch.snapshot());
        } else {
            act(&mut g, Action::DeclineEffectPayment);
        }
        settle(&mut g);
        let tokens = copies(&g, Card::from_name("Llanowar Elves").unwrap());
        assert_eq!(tokens.len(), usize::from(pay));
        if pay {
            let token = tokens[0];
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            assert_eq!(g.counter_count(token, CounterKind::MinusOne), 0);
            assert!(g.object_view(token).tapped);
            assert!(g.state.attackers.iter().any(|a| a.card == token));
            assert!(!g.keywords(token).contains(Keywords::HASTE));
            assert_eq!(
                g.copiable_recipe(g.object_ref(token))
                    .exceptions
                    .sacrifice_only_abilities,
                1
            );
            g.state.active = Player::P1;
            g.begin_step(Step::End);
            g.prepare_triggers();
            settle(&mut g);
            assert!(!g.on_battlefield(token));
        }
    }
}
#[test]
fn grouped_blink_returns_one_tapped_batch_and_damage_shield_lasts_to_next_actual_turn() {
    const GROUPS: &[TargetSpec] = &[TargetSpec {
        rule: TargetRule::Creature,
        min: 0,
        max: 2,
    }];
    const BLINK: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Groups(GROUPS),
            &[
                Instruction::BlinkGroupTappedUntilEndStep,
                Instruction::PreventPlayerDamageUntilNextTurn,
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &BLINK)]);
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, Player::P1, "Savannah Lions", Zone::Battlefield);
    g.state.objects[b as usize].controller = Player::P0;
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    announce(&mut g, spell, &[Target::Card(a), Target::Card(b)]);
    settle(&mut g);
    assert_eq!(g.object_view(a).zone, Zone::Exile);
    assert_eq!(g.object_view(b).zone, Zone::Exile);
    assert_eq!(g.state.delayed_returns.as_ref().unwrap().len(), 1);
    g.move_object(spell, Zone::Exile);
    let source = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    for preventable in [true, false] {
        let event = DamageEvent {
            preventable,
            source: g.damage_source(source),
            target: Target::Player(Player::P0),
            amount: 3,
            combat: false,
        };
        g.apply_damage_batch(&[event]);
        g.advance_replacements(false);
    }
    assert_eq!(g.state.players[0].life, 17);
    g.begin_step(Step::End);
    g.prepare_triggers();
    settle(&mut g);
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert_eq!(g.object_view(b).zone, Zone::Battlefield);
    assert!(g.object_view(a).tapped && g.object_view(b).tapped);
    assert_eq!(g.object_view(b).controller, Player::P1);
    g.state.active = Player::P1;
    g.begin_turn();
    assert_eq!(g.observe(Player::P0).player_damage_shields.len(), 1);
    g.state.active = Player::P0;
    g.begin_turn();
    assert!(g.observe(Player::P0).player_damage_shields.is_empty());
}

#[test]
fn crown_copied_entry_choices_resume_keyword_completion_after_source_departures() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &CROWN)]);
    let mut g = scenario(Player::P0);
    let automaton = put(&mut g, Player::P0, "Adaptive Automaton", Zone::Battlefield);
    let crown = equipped_crown(&mut g, Player::P0, automaton);
    let source = g.object_ref(crown);
    g.stage_token_creation(
        Player::P0,
        Card::from_name("@token/spirit_1_1_white_flying").unwrap(),
        2,
        false,
        replacements::TokenCompletion::Keywords {
            source,
            keywords: Keywords::FLYING,
        },
    );
    assert!(g.advance_replacements(false));
    act(&mut g, Action::ChooseOptional { accept: true });
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    g.move_object(crown, Zone::Exile);
    g.move_object(automaton, Zone::Exile);
    for kind in ["Goblin", "Elemental"] {
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name(kind).unwrap()),
            },
        );
    }
    while matches!(g.state.pending, Pending::EntryTimestamp { .. }) {
        choose(&mut g, |a| matches!(a, Action::ChooseEntryTimestamp { .. }));
    }
    let tokens = copies(&g, Card::from_name("Adaptive Automaton").unwrap());
    assert_eq!(tokens.len(), 2);
    assert!(
        tokens
            .iter()
            .all(|&id| g.keywords(id).contains(Keywords::FLYING))
    );
    assert!(g.state.replacements.is_none());
    assert!(g.state.entry_batch.is_none());
}
#[test]
fn crown_copy_replacement_preserves_role_and_job_completion_semantics() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &CROWN)]);
    for role in [false, true] {
        let mut g = scenario(Player::P0);
        let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        equipped_crown(&mut g, Player::P0, bear);
        let spell = put(
            &mut g,
            Player::P0,
            if role {
                "Royal Treatment"
            } else {
                "Bard's Bow"
            },
            Zone::Hand,
        );
        cast(
            &mut g,
            spell,
            if role { Some(Target::Card(bear)) } else { None },
        );
        wait_until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        act(&mut g, Action::ChooseOptional { accept: true });
        if !role && matches!(g.state.pending, Pending::CreatedToken { .. }) {
            let token = g
                .observe(Player::P0)
                .created_token_choice
                .unwrap()
                .candidates[0];
            act(&mut g, Action::ChooseCreatedToken { object: token });
        }
        settle(&mut g);
        let tokens = copies(&g, Card::GrizzlyBears);
        assert_eq!(tokens.len(), 1);
        assert!(g.object_view(tokens[0]).attached_to.is_none());
        if role {
            assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
        } else {
            assert_eq!(
                g.object_view(spell).attached_to,
                Some(g.object_ref(tokens[0]))
            );
        }
    }
}
#[test]
fn reveal_until_permanent_count_stops_exactly_and_commits_selected_entries_as_one_batch() {
    const AURORA: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::All,
                    filter: LibrarySelectionFilter::Kinds(1 | 2 | 16 | 32 | 64 | 256),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::UntilCount {
                    kinds: 1 | 2 | 16 | 32 | 64 | 256,
                    amount: AmountValue::Fixed(2),
                },
                destination: LibraryPickDestination::Battlefield,
                public: true,
                maximum_mana_value: None,
            })],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Divination").unwrap(), &AURORA)]);
    let mut g = scenario(Player::P0);
    let unseen = put(&mut g, Player::P0, "Shock", Zone::Library);
    let land = put(&mut g, Player::P0, "Forest", Zone::Library);
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
    let instant = put(&mut g, Player::P0, "Opt", Zone::Library);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, spell, None);
    wait_until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
    assert_eq!(
        g.observe(Player::P1)
            .library_selection
            .unwrap()
            .remaining
            .iter()
            .map(|o| o.id)
            .collect::<Vec<_>>(),
        vec![instant, bear, land]
    );
    reject(&mut g, Action::SelectLibrary { card: instant });
    act(&mut g, Action::SelectLibrary { card: bear });
    act(&mut g, Action::SelectLibrary { card: land });
    act(&mut g, Action::FinishLibrarySelection);
    settle(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(g.object_view(land).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].library.last(), Some(&unseen));
    assert_eq!(g.state.players[0].library.first(), Some(&instant));
}

#[test]
fn graveyard_cast_entries_qualify_in_both_paths_but_permanent_spell_copies_do_not() {
    const COPIER: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Any,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCastSpell {
                prior_spells: false,
            }],
        }],
        ..CardRules::EMPTY
    };
    const ORDINARY: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        ..CardRules::EMPTY
    };
    const STAGED: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        ..COPY
    };
    for rules in [&ORDINARY, &STAGED] {
        let _scope = fixture_rules::activate(&[
            (Card::LlanowarElves, &BATCH_COPY),
            (Card::from_name("Ghost Vacuum").unwrap(), &COPIER),
            (Card::GrizzlyBears, rules),
        ]);
        let mut g = scenario(Player::P0);
        let observer = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
        let original = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        g.grant_graveyard_permission(Player::P0, g.object_ref(original), GraveyardCastKind::Cast);
        cast(&mut g, original, None);
        let mut saw_copy = false;
        for _ in 0..48 {
            if g.object_view(original).zone == Zone::Battlefield {
                break;
            }
            if !copies(&g, Card::GrizzlyBears).is_empty() {
                saw_copy = true;
                assert!(
                    g.state
                        .stack
                        .iter()
                        .chain(&g.state.pending_triggers)
                        .chain(&g.state.ordering_triggers)
                        .all(|item| item.source.is_none_or(|source| source.id != observer))
                );
            }
            choose(&mut g, |action| {
                matches!(
                    action,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishStackTargets
                        | Action::ChooseEntryCopy { object: None }
                )
            });
        }
        assert!(saw_copy, "the copied permanent entered before the original");
        assert_eq!(g.object_view(original).zone, Zone::Battlefield);
        wait_until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryBatchCopy { .. })
        });
        let rows = g.observe(Player::P0).entry_batch_copy;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].object, g.object_ref(original));
    }
}
