//! Entry counters and subsequent counter events share replacements but retain
//! distinct event boundaries. Typed fixtures leave all TDM admission unchanged.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
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
const GRAVE_CAST: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SelfGraveyardCastIf(
            StatValue::Fixed(1),
        )],
    }],
    extras: &CardExtras {
        graveyard_cast_entry_counters: &[(CounterKind::Finality, 1)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROTECTED: &[(CounterKind, u64)] =
    &[(CounterKind::Hexproof, 1), (CounterKind::Indestructible, 1)];
const PRE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(GRAVE)),
        &[Instruction::ReturnToBattlefieldWithCounters(PROTECTED)],
    ),
    ..CardRules::EMPTY
};
const POST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: GRAVE,
            min: 0,
            max: 2,
        }),
        &[Instruction::ReturnToBattlefieldThenCounters(&[(
            CounterKind::PlusOne,
            1,
        )])],
    ),
    ..CardRules::EMPTY
};
const WITH_PLUS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(GRAVE)),
        &[Instruction::ReturnToBattlefieldWithCounters(&[(
            CounterKind::PlusOne,
            1,
        )])],
    ),
    ..CardRules::EMPTY
};
const ENTRY_PLUS_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::SelfEnters,
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::PlusOne),
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..CardRules::EMPTY
};
const WAIT_RETURN: CardRules = CardRules {
    entry: EntryTapped::PayLife(2),
    ..ENTRY_PLUS_OBSERVER
};
const ENTRY_HEX_OBSERVER: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::SelfEnters,
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::Hexproof),
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..ENTRY_PLUS_OBSERVER
};
const SUCCESSOR: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureCountersPlaced { first_only: true },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Entering,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const SIEGE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(
            &TriggerCondition::ControlledCreatureCountersPlaced { first_only: false },
        ),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NamedModes(&["Sultai", "Abzan"])),
        entry_mode_triggers: &[(0, 0)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MIXED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(PERMANENT)),
        &[Instruction::PutCountersMixed {
            subject: Subject::Targets,
            amounts: &[(CounterKind::Stun, 1), (CounterKind::Charge, 1)],
        }],
    ),
    ..CardRules::EMPTY
};
const ONE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(PERMANENT)),
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::Charge,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const STEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(PERMANENT)),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Controller,
        }],
    ),
    ..CardRules::EMPTY
};
const DOUBLE: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Permanent,
    kind: None,
    factor: 2,
    add: 0,
    effects_only: false,
};
const DOUBLER: CardRules = CardRules {
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
const BARRIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};
const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(PERMANENT)),
        &[
            Instruction::Destroy,
            Instruction::SearchTargetControllerWithEntryCounters {
                spec: SearchSpec {
                    filter: SearchFilter::BasicLand,
                    maximum: 1,
                    destination: SearchDestination::Battlefield,
                    tapped: true,
                    reveal: false,
                },
                amounts: &[(CounterKind::Stun, 1)],
            },
        ],
    ),
    ..CardRules::EMPTY
};
const FAILED_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(PERMANENT))),
    aura: Some(AuraSpec {
        kinds: 64,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn ready(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if predicate(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("bounded counter scenario did not reach decision");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.entry_batch.is_none()
            && g.state.pending == Pending::Priority
    });
}
fn give(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn spell(g: &mut Game, p: Player, name: &str, target: CardId) {
    give(g, p);
    let id = put(g, p, name, Zone::Hand);
    cast(g, id, Some(Target::Card(target)));
    drain(g);
}
fn return_two(g: &mut Game, p: Player, targets: &[CardId]) {
    give(g, p);
    let id = put(g, p, "Opt", Zone::Hand);
    act(g, Action::AnnounceCast { card: id });
    for &id in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn entry(g: &mut Game, moves: Vec<entries::EntryMove>) {
    g.stage_entries(moves, None);
    for _ in 0..16 {
        if g.state.entry_batch.is_none() {
            return;
        }
        if g.advance_entries() {
            choose(g, |a| {
                matches!(
                    a,
                    Action::ChooseReplacement { .. } | Action::ChooseEntryTimestamp { .. }
                )
            });
        }
    }
    panic!("entry budget");
}
fn projected(g: &Game, moves: &[entries::EntryMove], kind: CounterKind) -> Vec<Vec<u64>> {
    let world = FundingState::from_game(g, &goal(moves[0].controller));
    let mut out = Vec::new();
    world.visit_entry_moves(moves.to_vec(), true, None, &mut |next| {
        out.push(
            moves
                .iter()
                .map(|m| CharacteristicWorld::counter_count(next, m.before.id, kind))
                .collect(),
        );
        false
    });
    out.sort();
    out.dedup();
    out
}

#[test]
fn fixed_subtype_entry_bonus_is_one_per_provider_not_existing_dragons() {
    let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), &GLOBE)]);
    for p in [Player::P0, Player::P1] {
        for existing in [0, 2] {
            let mut g = scenario(p);
            put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
            for _ in 0..existing {
                put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
            }
            let id = put(&mut g, p, "Shivan Dragon", Zone::Hand);
            cast(&mut g, id, None);
            drain(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
            let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Graveyard);
            let m = g.entry_move(enemy, Zone::Battlefield, false);
            entry(&mut g, vec![m]);
            assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        }
    }
}
#[test]
fn simultaneous_provider_absence_and_prior_presence_match_projection() {
    let _scope = fixture_rules::activate(&[(card("Swiftfoot Boots"), &GLOBE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let provider = put(&mut g, p, "Swiftfoot Boots", Zone::Graveyard);
    let dragon = put(&mut g, p, "Shivan Dragon", Zone::Graveyard);
    let moves = vec![
        g.entry_move(provider, Zone::Battlefield, false),
        g.entry_move(dragon, Zone::Battlefield, false),
    ];
    assert_eq!(
        projected(&g, &moves, CounterKind::PlusOne),
        vec![vec![0, 0]]
    );
    entry(&mut g, moves);
    assert_eq!(g.counter_count(dragon, CounterKind::PlusOne), 0);
    let another = put(&mut g, p, "Shivan Dragon", Zone::Graveyard);
    let moves = vec![
        g.entry_move(provider, Zone::Graveyard, false),
        g.entry_move(another, Zone::Battlefield, false),
    ];
    assert_eq!(
        projected(&g, &moves, CounterKind::PlusOne),
        vec![vec![0, 1]]
    );
    entry(&mut g, moves);
    assert_eq!(g.counter_count(another, CounterKind::PlusOne), 1);
}
#[test]
fn subtype_choice_and_counter_replacements_are_precommit_in_both_worlds() {
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
    let _scope = fixture_rules::activate(&[
        (card("Swiftfoot Boots"), &GLOBE),
        (card("Bear Cub"), &CHOOSE),
        (card("Short Bow"), &DOUBLER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    let id = put(&mut g, p, "Bear Cub", Zone::Hand);
    cast(&mut g, id, None);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::EntryValue { .. })
    });
    act(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Dragon").unwrap()),
        },
    );
    drain(&mut g);
    assert_eq!(g.counter_count(id, CounterKind::PlusOne), 2);
    assert!(g.has_subtype(id, "Dragon"));
    let next = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let mut m = g.entry_move(next, Zone::Battlefield, false);
    m.choice = Some(EntryChoiceValue::CreatureType(
        CreatureType::from_name("Dragon").unwrap(),
    ));
    assert!(projected(&g, &[m], CounterKind::PlusOne).contains(&vec![2]));
}
#[test]
fn graveyard_origin_counters_require_an_actual_cast_and_keep_normal_costs() {
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &GRAVE_CAST)]);
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Hand, Zone::Graveyard] {
            let mut g = scenario(p);
            let id = put(&mut g, p, "Bear Cub", origin);
            let before = g.state.players[p.index()].mana;
            cast(&mut g, id, None);
            drain(&mut g);
            assert_ne!(g.state.players[p.index()].mana, before);
            assert_eq!(
                g.counter_count(id, CounterKind::Finality),
                u64::from(origin == Zone::Graveyard)
            );
        }
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let m = g.entry_move(id, Zone::Battlefield, false);
        assert_eq!(
            projected(&g, std::slice::from_ref(&m), CounterKind::Finality),
            vec![vec![0]]
        );
        entry(&mut g, vec![m]);
        assert_eq!(g.counter_count(id, CounterKind::Finality), 0);
        let id = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: id });
    }
}
#[test]
fn another_graveyard_permission_still_uses_graveyard_origin_and_countered_spell_can_retry() {
    const NO_SELF: CardRules = CardRules {
        static_groups: &[],
        ..GRAVE_CAST
    };
    let _scope = fixture_rules::activate(&[(card("Bear Cub"), &NO_SELF)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let id = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let object = g.object_ref(id);
    g.grant_graveyard_permission(p, object, GraveyardCastKind::Cast);
    cast(&mut g, id, None);
    give(&mut g, p.other());
    let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
    cast(&mut g, counter, Some(Target::Spell(id)));
    drain(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    assert_eq!(g.counter_count(id, CounterKind::Finality), 0);
    let object = g.object_ref(id);
    g.grant_graveyard_permission(p, object, GraveyardCastKind::Cast);
    give(&mut g, p);
    cast(&mut g, id, None);
    drain(&mut g);
    assert_eq!(g.counter_count(id, CounterKind::Finality), 1);
    // Finality persists even after the card's printed abilities are replaced.
    g.apply_temporary_copy(
        g.object_ref(id),
        CopyRecipe::new(card("Swiftfoot Boots")),
        CopyDeadline::EndOfTurn,
    );
    g.move_object(id, Zone::Graveyard);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
}
#[test]
fn mixed_return_counters_are_visible_to_entry_observers_and_keyword_rules() {
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &PRE), (card("Bear Cub"), &ENTRY_HEX_OBSERVER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let before = g.object_ref(id);
        let mut movement = g.entry_move(id, Zone::Battlefield, false);
        movement.effect_counters = PROTECTED;
        let saved = g.snapshot();
        assert_eq!(
            projected(&g, std::slice::from_ref(&movement), CounterKind::Hexproof),
            vec![vec![1]]
        );
        assert_eq!(
            projected(&g, &[movement], CounterKind::Indestructible),
            vec![vec![1]]
        );
        assert_eq!(g.snapshot(), saved);
        spell(&mut g, p, "Opt", id);
        assert_ne!(g.object_ref(id), before);
        assert_eq!(g.counter_count(id, CounterKind::Hexproof), 1);
        assert_eq!(g.counter_count(id, CounterKind::Indestructible), 1);
        assert_eq!(g.state.players[p.index()].life, 27);
        assert!(
            g.keywords(id)
                .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
        );
        give(&mut g, p.other());
        let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
}
#[test]
fn counters_after_returns_do_not_retroactively_enable_entry_observers() {
    for with_entry in [false, true] {
        let rules = if with_entry { &WITH_PLUS } else { &POST };
        let _scope = fixture_rules::activate(&[
            (card("Opt"), rules),
            (card("Bear Cub"), &ENTRY_PLUS_OBSERVER),
        ]);
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        spell(&mut g, p, "Opt", id);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[0].life, if with_entry { 27 } else { 20 });
    }
}
#[test]
fn post_return_batch_tracks_successes_through_entry_choices_and_failed_aura_return() {
    let _scope = fixture_rules::activate(&[
        (card("Opt"), &POST),
        (card("Pacifism"), &FAILED_AURA),
        (card("Bear Cub"), &WAIT_RETURN),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let good = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let bad = put(&mut g, p, "Pacifism", Zone::Graveyard);
    let old = g.object_ref(good);
    return_two(&mut g, p, &[good, bad]);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::EntryLife { .. })
    });
    act(&mut g, Action::ChooseEntryLife { pay: false });
    drain(&mut g);
    assert_eq!(g.object_view(bad).zone, Zone::Graveyard);
    assert_ne!(g.object_ref(good), old);
    assert_eq!(g.counter_count(good, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(bad, CounterKind::PlusOne), 0);
}
#[test]
fn post_return_replacement_results_match_projection_without_parent_mutation() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &POST), (card("Short Bow"), &DOUBLER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    let a = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let b = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
    let mut moves = vec![
        g.entry_move(a, Zone::Battlefield, false),
        g.entry_move(b, Zone::Battlefield, false),
    ];
    for m in &mut moves {
        m.post_entry_counters = Some((p, &[(CounterKind::PlusOne, 1)]));
    }
    let saved = g.snapshot();
    assert_eq!(
        projected(&g, &moves, CounterKind::PlusOne),
        vec![vec![2, 2]]
    );
    assert_eq!(g.snapshot(), saved);
    return_two(&mut g, p, &[a, b]);
    drain(&mut g);
    assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
    assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
}
#[test]
fn searched_land_enters_tapped_with_stun_even_if_target_survives() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &SEARCH)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let land_ref = g.object_ref(land);
        g.record_control(land_ref, p.other(), None);
        g.state.objects[land as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        give(&mut g, p);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(land)));
        ready(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        assert!(g.observe(p).search_choice.is_none());
        let view = g.observe(p.other()).search_choice.unwrap();
        let found = view.cards[0].id;
        act(&mut g, Action::ChooseSearch { card: found });
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.object_view(found).controller, p.other());
        assert!(g.object_view(found).tapped);
        assert_eq!(g.counter_count(found, CounterKind::Stun), 1);
    }
}
#[test]
fn illegal_destroy_target_causes_no_search_and_zero_selection_still_shuffles() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &SEARCH)]);
    for invalid in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let land_ref = g.object_ref(land);
        g.record_control(land_ref, p.other(), None);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(land)));
        if invalid {
            g.move_object(land, Zone::Exile);
            drain(&mut g);
            assert!(g.observe(p.other()).search_choice.is_none());
        } else {
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            let before = g.state.rng;
            act(&mut g, Action::FinishSearch);
            drain(&mut g);
            assert_ne!(g.state.rng, before);
        }
    }
}
#[test]
fn mode_gated_own_creature_counter_observer_ignores_placer_and_triggers_once() {
    let _scope = fixture_rules::activate(&[
        (card("Anthem of Champions"), &SIEGE),
        (card("Opt"), &MIXED),
        (card("Shock"), &ONE),
    ]);
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let siege = put(&mut g, p, "Anthem of Champions", Zone::Hand);
            cast(&mut g, siege, None);
            ready(&mut g, |g| {
                matches!(g.state.pending, Pending::EntryValue { .. })
            });
            act(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::NamedMode(mode),
                },
            );
            drain(&mut g);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let hand = g.hand_count(p);
            spell(&mut g, p.other(), "Opt", own);
            assert_eq!(g.hand_count(p), hand + usize::from(mode == 0));
            spell(&mut g, p.other(), "Shock", enemy);
            assert_eq!(g.hand_count(p), hand + usize::from(mode == 0));
            spell(&mut g, p.other(), "Shock", own);
            assert_eq!(g.hand_count(p), hand + usize::from(mode == 0));
        }
    }
}
#[test]
fn first_mixed_counter_event_is_deduplicated_and_frozen_before_response() {
    let _scope = fixture_rules::activate(&[
        (card("Short Bow"), &SUCCESSOR),
        (card("Opt"), &MIXED),
        (card("Shock"), &ONE),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    let recipient = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(recipient)));
    ready(&mut g, |g| {
        g.state.stack.len() == 1
            && matches!(g.state.stack[0].kind, StackKind::TriggeredAbility { .. })
    });
    assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 0);
    let extra = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, extra, Some(Target::Card(recipient)));
    drain(&mut g);
    assert_eq!(g.counter_count(recipient, CounterKind::Stun), 1);
    assert_eq!(g.counter_count(recipient, CounterKind::Charge), 2);
    assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 1);
}
#[test]
fn first_history_precedes_observer_arrival_and_survives_copy_control_and_turn_boundaries() {
    let _scope = fixture_rules::activate(&[
        (card("Short Bow"), &SUCCESSOR),
        (card("Opt"), &MIXED),
        (card("Shock"), &ONE),
        (card("Unsummon"), &STEAL),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let recipient = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    // First placement while it is a noncreature, before any observer exists.
    g.apply_temporary_copy(
        g.object_ref(recipient),
        CopyRecipe::new(card("Swiftfoot Boots")),
        CopyDeadline::EndOfTurn,
    );
    spell(&mut g, p, "Opt", recipient);
    g.expire_temporary_copies(true);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    spell(&mut g, p, "Unsummon", recipient);
    spell(&mut g, p, "Shock", recipient);
    assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 0);
    g.begin_turn();
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    g.state.priority = p;
    spell(&mut g, p, "Shock", recipient);
    assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 1);
    let old = g.object_ref(recipient);
    g.move_object(recipient, Zone::Exile);
    let mut m = g.entry_move(recipient, Zone::Battlefield, false);
    m.controller = p;
    entry(&mut g, vec![m]);
    assert_ne!(g.object_ref(recipient), old);
    spell(&mut g, p, "Shock", recipient);
    assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 1);
}
#[test]
fn self_and_entry_counters_are_first_events_and_prohibited_placements_do_not_consume_first() {
    const ENTERING: CardRules = CardRules {
        extras: &CardExtras {
            fixed_entry_counters: &[(CounterKind::Charge, 1), (CounterKind::Stun, 1)],
            ..CardExtras::EMPTY
        },
        ..SUCCESSOR
    };
    let _scope = fixture_rules::activate(&[
        (card("Bear Cub"), &ENTERING),
        (card("Short Bow"), &BARRIER),
        (card("Shock"), &ONE),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let id = put(&mut g, p, "Bear Cub", Zone::Hand);
    cast(&mut g, id, None);
    drain(&mut g);
    assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
    let barrier = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
    let fresh = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    spell(&mut g, p.other(), "Shock", fresh);
    assert_eq!(g.counter_count(fresh, CounterKind::Charge), 0);
    assert!(!counter_placement_history::any_player_placed(
        &g.state.counter_placement_history,
        g.object_ref(fresh)
    ));
    g.move_object(barrier, Zone::Graveyard);
    spell(&mut g, p.other(), "Shock", fresh);
    assert_eq!(g.counter_count(fresh, CounterKind::PlusOne), 1);
}
#[test]
fn private_search_counter_payload_is_owned_across_raw_index_restore_and_parent_disposal() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &SEARCH)]);
    let p = Player::P0;
    let mut parent = scenario(p);
    let target = put(&mut parent, p.other(), "Forest", Zone::Battlefield);
    let spell = put(&mut parent, p, "Opt", Zone::Hand);
    cast(&mut parent, spell, Some(Target::Card(target)));
    ready(&mut parent, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
    let actor = p.other();
    let found = parent.observe(actor).search_choice.unwrap().cards[0].id;
    let action = Action::ChooseSearch { card: found };
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p, action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert!(raw.observe(p).search_choice.is_none());
    for child in [&mut raw, &mut indexed] {
        act(child, Action::FinishSearch);
        drain(child);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(found, CounterKind::Stun), 1);
    act(&mut sibling, Action::FinishSearch);
    drain(&mut sibling);
    assert_eq!(sibling.counter_count(found, CounterKind::Stun), 0);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    act(&mut indexed, Action::FinishSearch);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn mixed_entry_counters_use_replacements_and_prohibitions_without_canceling_return() {
    for blocked in [false, true] {
        let provider = if blocked { &BARRIER } else { &DOUBLER };
        let _scope = fixture_rules::activate(&[
            (card("Opt"), &PRE),
            (card("Short Bow"), provider),
            (card("Bear Cub"), &ENTRY_HEX_OBSERVER),
        ]);
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let id = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        spell(&mut g, p, "Opt", id);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        for kind in [CounterKind::Hexproof, CounterKind::Indestructible] {
            assert_eq!(g.counter_count(id, kind), if blocked { 0 } else { 2 });
        }
        assert_eq!(g.state.players[0].life, if blocked { 20 } else { 27 });
    }
}
#[test]
fn stale_return_target_never_receives_post_entry_counters_on_its_later_incarnation() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &POST)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let b = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
    return_two(&mut g, p, &[a, b]);
    let old = g.object_ref(b);
    g.move_object(b, Zone::Exile);
    g.move_object(b, Zone::Graveyard);
    assert_ne!(g.object_ref(b), old);
    drain(&mut g);
    assert_eq!(g.counter_count(a, CounterKind::PlusOne), 1);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
}
#[test]
fn post_entry_counter_completion_is_owned_while_return_choices_suspend() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &POST), (card("Bear Cub"), &WAIT_RETURN)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let recipient = put(&mut parent, p, "Bear Cub", Zone::Graveyard);
    return_two(&mut parent, p, &[recipient]);
    ready(&mut parent, |g| {
        matches!(g.state.pending, Pending::EntryLife { .. })
    });
    let action = Action::ChooseEntryLife { pay: false };
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
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(recipient, CounterKind::PlusOne), 1);
    assert_eq!(raw.state.players[p.index()].life, 20);
    act(&mut sibling, Action::ChooseEntryLife { pay: true });
    drain(&mut sibling);
    assert_eq!(sibling.state.players[p.index()].life, 18);
    assert_eq!(sibling.counter_count(recipient, CounterKind::PlusOne), 1);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
