//! Bounded as-entry modes and their printed static, triggered and activated
//! providers. These fixtures exercise the engine without admitting TDM cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const MODES: &[&str] = &["Jeskai", "Temur"];
const OWN_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: OWN_CREATURES,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const GRANT: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: OWN_CREATURES,
    source_only: false,
    own_turn: false,
    keywords: Keywords::TRAMPLE.union(Keywords::HASTE),
};
const STATIC: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::EntryMode(1),
            StaticAbilityFragment::KeywordGrant(0, GRANT),
            StaticAbilityFragment::StaticModifier(0, BONUS),
        ],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceEntryModeActivation {
            mode: 0,
            ability: 0,
        }],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceEntryModeActivation {
            mode: 1,
            ability: 1,
        }],
    },
];
const SIEGE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: STATIC,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
    ],
    activated: &[
        ActivatedAbility {
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
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 1, 0,
            ]))],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NamedModes(MODES)),
        entry_mode_triggers: &[(0, 0), (1, 0)],
        static_ability_groups: STATIC,
        static_modifiers: &[BONUS],
        keyword_grants: &[GRANT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GRAVEYARD: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::EntryMode(1),
            StaticAbilityFragment::PlayLandFromGraveyard,
        ],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NamedModes(&["Temur", "Sultai"])),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const STEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Controller,
        }],
    ),
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 32,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: false,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const BLANK: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }))),
    aura: Some(AuraSpec {
        kinds: 0,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::RemoveAttachedAbilities],
    }],
    ..CardRules::EMPTY
};

fn siege() -> Card {
    Card::from_name("Anthem of Champions").unwrap()
}
fn choice(mode: u16) -> Action {
    Action::ChooseEntryValue {
        value: EntryChoiceValue::NamedMode(mode),
    }
}
fn goal(p: Player, required: Mana) -> FundingGoal {
    FundingGoal {
        actor: p,
        required,
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn ready(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if predicate(g) {
            return;
        }
        choose(g, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("bounded named-mode fixture did not reach its decision");
}
fn drain(g: &mut Game) {
    ready(g, |g| {
        g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
    });
}
fn enter_choice(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Anthem of Champions", Zone::Hand);
    cast(g, id, None);
    ready(g, |g| matches!(g.state.pending, Pending::EntryValue { .. }));
    id
}
fn enter(g: &mut Game, p: Player, mode: u16) -> CardId {
    let id = enter_choice(g, p);
    act(g, choice(mode));
    drain(g);
    id
}
fn chosen(g: &Game, id: CardId) -> Option<EntryChoiceValue> {
    CharacteristicWorld::entry_value(g, g.object_ref(id))
}

#[test]
fn entry_modes_are_mandatory_public_bounded_and_transactional() {
    let _scope = fixture_rules::activate(&[(siege(), &SIEGE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = enter_choice(&mut g, p);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Stack);
        assert_eq!(stats(&g, creature), (2, 2));
        assert_eq!(g.actor(), Some(p));
        for viewer in [p, p.other()] {
            let view = g.observe(viewer).entry_value.unwrap();
            assert_eq!(view.kind, EntryChoiceKind::NamedModes(MODES));
            assert!(view.opponent_hand.is_none());
            assert!(view.selected.is_empty());
        }
        reject(&mut g, choice(2));
        reject(&mut g, choice(u16::MAX));
        reject(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Red),
            },
        );
        reject(&mut g, Action::Pass);
        act(&mut g, choice(1));
        drain(&mut g);
        assert_eq!(chosen(&g, id), Some(EntryChoiceValue::NamedMode(1)));
        assert_eq!(stats(&g, creature), (3, 2));
        assert!(
            g.keywords(creature)
                .contains(Keywords::TRAMPLE.union(Keywords::HASTE))
        );
        for viewer in [p, p.other()] {
            let view = g.observe(viewer);
            assert_eq!(
                view.objects
                    .iter()
                    .find(|o| o.id == id)
                    .unwrap()
                    .entry_choice,
                Some(EntryChoiceValue::NamedMode(1))
            );
            assert!(view.entry_value.is_none());
        }
        let encoded = serde_json::to_string(&choice(1)).unwrap();
        assert_eq!(serde_json::from_str::<Action>(&encoded).unwrap(), choice(1));
    }
}

#[test]
fn metadata_rejects_empty_modes_and_out_of_bounds_ability_gates() {
    let empty = CardRules {
        effect: Effect::Permanent,
        extras: &CardExtras {
            entry_choice: Some(EntryChoiceKind::NamedModes(&[])),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    assert!(std::panic::catch_unwind(|| empty.checked_effect()).is_err());
    const BAD_TRIGGER: CardRules = CardRules {
        extras: &CardExtras {
            entry_mode_triggers: &[(0, 2)],
            ..*SIEGE.extras
        },
        ..SIEGE
    };
    assert!(std::panic::catch_unwind(|| BAD_TRIGGER.checked_effect()).is_err());
    const BAD_STATIC: CardRules = CardRules {
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::EntryMode(2)],
        }],
        ..SIEGE
    };
    const BAD_ACTIVATION: CardRules = CardRules {
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceEntryModeActivation {
                mode: 0,
                ability: 2,
            }],
        }],
        ..SIEGE
    };
    const EMPTY_LABEL: CardRules = CardRules {
        extras: &CardExtras {
            entry_choice: Some(EntryChoiceKind::NamedModes(&[""])),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    assert!(std::panic::catch_unwind(|| BAD_STATIC.checked_effect()).is_err());
    assert!(std::panic::catch_unwind(|| BAD_ACTIVATION.checked_effect()).is_err());
    assert!(std::panic::catch_unwind(|| EMPTY_LABEL.checked_effect()).is_err());
    const UNINTEGRATED: CardRules = CardRules {
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::EntryMode(0),
                StaticAbilityFragment::StartYourEngines,
            ],
        }],
        ..SIEGE
    };
    assert!(std::panic::catch_unwind(|| UNINTEGRATED.checked_effect()).is_err());
    assert!(matches!(SIEGE.checked_effect(), Effect::Permanent));
}

#[test]
fn chosen_modes_select_static_and_triggered_providers_in_both_seats() {
    let _scope = fixture_rules::activate(&[(siege(), &SIEGE)]);
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = enter(&mut g, p, mode);
            assert_eq!(stats(&g, own), if mode == 1 { (3, 2) } else { (2, 2) });
            assert_eq!(stats(&g, enemy), (2, 2));
            assert_eq!(g.keywords(own).contains(Keywords::TRAMPLE), mode == 1);
            assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
            let hand = g.hand_count(p);
            g.begin_step(Step::End);
            if mode == 0 {
                assert!(!g.state.pending_triggers.is_empty());
                // The printed ability already triggered. Losing the selected
                // acquisition before it reaches/resolves on the stack cannot
                // turn its mode into an intervening-if condition.
                g.apply_temporary_copy(
                    g.object_ref(id),
                    CopyRecipe::new(Card::from_name("Short Bow").unwrap()),
                    CopyDeadline::EndOfTurn,
                );
                assert!(chosen(&g, id).is_none());
            }
            drain(&mut g);
            assert_eq!(g.hand_count(p), hand + usize::from(mode == 0));
        }
    }
}

#[test]
fn mode_owned_activated_templates_have_real_presence_and_pay_costs() {
    let _scope = fixture_rules::activate(&[(siege(), &SIEGE)]);
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let id = enter(&mut g, p, mode);
            let life = g.state.players[p.index()].life;
            let action = Action::AnnounceSourceGrant {
                source: g.object_ref(id),
                ability: 0,
            };
            if mode == 0 {
                act(&mut g, action);
                pay(&mut g);
                drain(&mut g);
                assert_eq!(g.state.players[p.index()].life, life + 3);
            } else {
                reject(&mut g, action);
            }
            g.state.players[p.index()].mana = [0; 6];
            let available = g.legal_actions().into_iter().find(|a| {
                matches!(a,
                Action::ActivateMana { card, ability: 1, .. } if *card == id)
            });
            assert_eq!(available.is_some(), mode == 1);
            assert_eq!(g.can_fund(goal(p, [0, 0, 0, 0, 1, 0])), mode == 1);
            if let Some(action) = available {
                act(&mut g, action);
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
                assert!(g.object_view(id).tapped);
            }
        }
    }
}

#[test]
fn changing_control_keeps_the_choice_and_changes_its_live_recipients() {
    let _scope =
        fixture_rules::activate(&[(siege(), &SIEGE), (Card::from_name("Opt").unwrap(), &STEAL)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = enter(&mut g, p, 1);
        act(&mut g, Action::Pass);
        let steal = put(&mut g, p.other(), "Opt", Zone::Hand);
        cast(&mut g, steal, Some(Target::Card(id)));
        drain(&mut g);
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(chosen(&g, id), Some(EntryChoiceValue::NamedMode(1)));
        assert_eq!(stats(&g, own), (2, 2));
        assert_eq!(stats(&g, enemy), (3, 2));
        assert!(g.keywords(enemy).contains(Keywords::HASTE));
    }
}

#[test]
fn mode_permission_obeys_land_timing_allowance_and_source_departure() {
    let _scope = fixture_rules::activate(&[(siege(), &GRAVEYARD)]);
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Forest", Zone::Graveyard);
            let another = put(&mut g, p, "Island", Zone::Graveyard);
            let id = enter(&mut g, p, mode);
            let play = Action::PlayLand { card: land };
            assert_eq!(g.legal_actions().contains(&play), mode == 1);
            if mode == 1 {
                let mut upkeep = g.clone();
                upkeep.state.step = Step::Upkeep;
                reject(&mut upkeep, play.clone());
                let mut opposing = g.clone();
                opposing.state.active = p.other();
                reject(&mut opposing, play.clone());
                act(&mut g, play);
                assert_eq!(g.object_view(land).zone, Zone::Battlefield);
                reject(&mut g, Action::PlayLand { card: another });
            }
            g.move_object(id, Zone::Graveyard);
            assert!(chosen(&g, id).is_none());
            g.state.land_played = false;
            reject(&mut g, Action::PlayLand { card: another });
        }
    }
}

#[test]
fn blink_reprompts_and_in_place_copies_never_inherit_a_mode() {
    let _scope = fixture_rules::activate(&[(siege(), &SIEGE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let id = enter(&mut g, p, 1);
    let original = g.object_ref(id);
    let recipient = put(&mut g, p, "Short Bow", Zone::Battlefield);
    g.apply_temporary_copy(
        g.object_ref(recipient),
        CopyRecipe::new(siege()),
        CopyDeadline::EndOfTurn,
    );
    assert!(chosen(&g, recipient).is_none());
    assert!(!g.source_activation_template_available(recipient, 0));
    assert!(!g.source_activation_template_available(recipient, 1));
    let hand = g.hand_count(p);
    g.begin_step(Step::End);
    drain(&mut g);
    assert_eq!(g.hand_count(p), hand);
    assert_eq!(stats(&g, bear), (3, 2));
    g.apply_temporary_copy(
        original,
        CopyRecipe::new(Card::from_name("Short Bow").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    assert!(chosen(&g, id).is_none());
    assert_eq!(stats(&g, bear), (2, 2));
    g.expire_temporary_copies(true);
    assert_eq!(chosen(&g, id), Some(EntryChoiceValue::NamedMode(1)));
    assert_eq!(stats(&g, bear), (3, 2));
    g.move_object(id, Zone::Exile);
    let movement = g.entry_move(id, Zone::Battlefield, false);
    g.stage_entries(vec![movement], None);
    assert!(g.advance_entries());
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    assert!(chosen(&g, id).is_none());
    act(&mut g, choice(0));
    drain(&mut g);
    assert_ne!(g.object_ref(id), original);
    assert_eq!(chosen(&g, id), Some(EntryChoiceValue::NamedMode(0)));
    assert_eq!(stats(&g, bear), (2, 2));
}

#[test]
fn entering_copy_makes_its_own_choice_and_departure_uses_the_frozen_mode() {
    let _scope = fixture_rules::activate(&[
        (siege(), &SIEGE),
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = enter(&mut g, p, 1);
    let copy = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    cast(&mut g, copy, None);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::EntryCopy { .. })
    });
    let source_ref = g.object_ref(source);
    act(
        &mut g,
        Action::ChooseEntryCopy {
            object: Some(source_ref),
        },
    );
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    act(&mut g, choice(0));
    drain(&mut g);
    assert_eq!(chosen(&g, source), Some(EntryChoiceValue::NamedMode(1)));
    assert_eq!(chosen(&g, copy), Some(EntryChoiceValue::NamedMode(0)));
    let life = g.state.players[p.index()].life;
    // Both depart simultaneously. Only the copy's mode0 conditional SelfLeaves
    // exists in the frozen event, although its live choice is cleared already.
    let sources = g.death_trigger_sources();
    let events = g.move_objects_raw(&[source, copy], Zone::Exile);
    g.finish_zone_events(&events, &sources);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].life, life + 2);
}

#[test]
fn prospective_entry_modes_match_actual_funding_and_never_mutate_parent() {
    let _scope = fixture_rules::activate(&[(siege(), &SIEGE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let id = put(&mut g, p, "Anthem of Champions", Zone::Exile);
    g.state.players[p.index()].mana = [0; 6];
    let saved = g.snapshot();
    let movement = g.entry_move(id, Zone::Battlefield, false);
    let world = FundingState::from_game(&g, &goal(p, [0; 6]));
    let mut projected = Vec::new();
    world.visit_entry_moves(vec![movement], true, None, &mut |next| {
        let selected = next.entry_value(next.reference(id)).unwrap();
        let power = next.power(bear);
        let mana = next.source_activation_template_available(id, 1);
        projected.push((selected, power, mana));
        false
    });
    assert_eq!(projected.len(), 2);
    assert!(projected.contains(&(EntryChoiceValue::NamedMode(0), 2, false)));
    assert!(projected.contains(&(EntryChoiceValue::NamedMode(1), 3, true)));
    assert_eq!(g.snapshot(), saved);
    assert!(chosen(&g, id).is_none());
    assert!(!g.can_fund(goal(p, [0, 0, 0, 0, 1, 0])));
    let movement = g.entry_move(id, Zone::Battlefield, false);
    g.stage_entries(vec![movement], None);
    assert!(g.advance_entries());
    act(&mut g, choice(1));
    drain(&mut g);
    assert_eq!(stats(&g, bear), (3, 2));
    assert!(g.can_fund(goal(p, [0, 0, 0, 0, 1, 0])));
}

#[test]
fn pending_entry_mode_survives_raw_index_restore_and_parent_disposal() {
    let _scope = fixture_rules::activate(&[(siege(), &SIEGE)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let bear = put(&mut parent, p, "Bear Cub", Zone::Battlefield);
    let id = enter_choice(&mut parent, p);
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == choice(1))
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), choice(1)), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, choice(1)).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(stats(&raw, bear), (3, 2));
    act(&mut sibling, choice(0));
    drain(&mut sibling);
    assert_eq!(stats(&sibling, bear), (2, 2));
    assert_eq!(chosen(&sibling, id), Some(EntryChoiceValue::NamedMode(0)));
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, choice(1));
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn suppressing_mode_providers_keeps_the_choice_and_restores_only_its_abilities() {
    let _scope = fixture_rules::activate(&[
        (siege(), &SIEGE),
        (Card::from_name("Pacifism").unwrap(), &BLANK),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let id = enter(&mut g, p, 1);
    let aura = put(&mut g, p, "Pacifism", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(id)));
    drain(&mut g);
    assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(id)));
    assert_eq!(chosen(&g, id), Some(EntryChoiceValue::NamedMode(1)));
    assert_eq!(stats(&g, bear), (2, 2));
    assert!(!g.keywords(bear).contains(Keywords::TRAMPLE));
    g.state.players[p.index()].mana = [0; 6];
    assert!(!g.can_fund(goal(p, [0, 0, 0, 0, 1, 0])));
    g.move_object(aura, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(chosen(&g, id), Some(EntryChoiceValue::NamedMode(1)));
    assert_eq!(stats(&g, bear), (3, 2));
    assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
    assert!(g.can_fund(goal(p, [0, 0, 0, 0, 1, 0])));
}

#[test]
fn a_mode_gated_keyword_group_without_activated_templates_has_no_fast_path_leak() {
    const ONLY_KEYWORDS: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::EntryMode(1),
                StaticAbilityFragment::ControlledPermanentKeywords {
                    kinds: 1,
                    keywords: Keywords::VIGILANCE,
                },
            ],
        }],
        extras: &CardExtras {
            entry_choice: Some(EntryChoiceKind::NamedModes(MODES)),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(siege(), &ONLY_KEYWORDS)]);
    for mode in [0, 1] {
        let mut g = scenario(Player::P0);
        let creature = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        enter(&mut g, Player::P0, mode);
        assert_eq!(
            g.keywords(creature).contains(Keywords::VIGILANCE),
            mode == 1
        );
        let world = FundingState::from_game(&g, &goal(Player::P0, [0; 6]));
        assert_eq!(
            characteristic_query::CharacteristicQuery::in_world(&world)
                .keywords(creature)
                .contains(Keywords::VIGILANCE),
            mode == 1
        );
    }
}
