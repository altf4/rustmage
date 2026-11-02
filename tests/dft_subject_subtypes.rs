//! Live permanent predicates are distinct from creature-only recipient filters.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const OWN_PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const FORM: AnimationSpec = AnimationSpec {
    artifact: true,
    creature_types: AnimationCreatureTypes::Unchanged,
    base: None,
    keywords: Keywords::NONE,
    duration: AnimationDuration::EndOfTurn,
};
const TARGET_BRANCHES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: OWN_PERMANENT,
            min: 0,
            max: 1,
        }),
        &[
            Instruction::IfSubjectSubtype {
                subject: Subject::Targets,
                subtype: "Mount",
                skip: 1,
            },
            Instruction::MarkSaddled(Subject::Targets),
            Instruction::IfSubjectSubtype {
                subject: Subject::Targets,
                subtype: "Vehicle",
                skip: 1,
            },
            Instruction::AnimateTargets(FORM),
        ],
    ),
    ..CardRules::EMPTY
};
const SOURCE_BRANCH: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSubjectSubtype {
                subject: Subject::Source,
                subtype: "Vehicle",
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ENTRY_BRANCH: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 0,
            nontoken: false,
            subtype: Some("Vehicle"),
            include_source: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSubjectSubtype {
                subject: Subject::Entering,
                subtype: "Vehicle",
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 7),
        ],
    }],
    ..CardRules::EMPTY
};
const AURA_SPEC: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 0,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};
const ATTACHED_BRANCH: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::CreatureOrVehicle { relation: -1 },
    ))),
    aura: Some(AURA_SPEC),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSubjectSubtype {
                subject: Subject::Attached,
                subtype: "Vehicle",
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 11),
        ],
    }],
    extras: &CardExtras {
        aura: Some(AURA_SPEC),
        aura_kind_or_subtype: Some("Vehicle"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn activate(g: &mut Game, id: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("subtype scenario exceeded 64 actions");
}
fn reach_trigger(g: &mut Game, source: CardId) {
    for _ in 0..32 {
        if g.state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .chain(&g.state.ordering_triggers)
            .any(|s| {
                matches!(s.kind, StackKind::TriggeredAbility { .. })
                    && s.source.is_some_and(|s| s.id == source)
            })
        {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("expected trigger from {source}");
}
fn make_both(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    g.change_object_form(
        object,
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Mount"]),
            duration: AnimationDuration::Permanent,
            ..FORM
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    assert!(g.has_subtype(id, "Vehicle") && g.has_subtype(id, "Mount"));
}

#[test]
fn target_subtype_branches_distinguish_mount_vehicle_and_both_with_owned_restore() {
    let _scope = fixture_rules::activate(&[(named("Opt"), &TARGET_BRANCHES)]);
    for p in [Player::P0, Player::P1] {
        for shape in 0..3 {
            let mut parent = scenario(p);
            let id = put(
                &mut parent,
                p,
                if shape == 0 {
                    "Trained Arynx"
                } else {
                    "Veloheart Bike"
                },
                Zone::Battlefield,
            );
            if shape == 2 {
                make_both(&mut parent, id);
            }
            if shape == 0 {
                let blank = put(&mut parent, p, "Stop Cold", Zone::Hand);
                cast(&mut parent, blank, Some(Target::Card(id)));
                drain(&mut parent);
                assert!(!parent.printed_abilities_available(id));
            }
            let spell = put(&mut parent, p, "Opt", Zone::Hand);
            cast(&mut parent, spell, Some(Target::Card(id)));
            let saved = parent.snapshot();
            let mut raw = parent.clone();
            let mut indexed = parent.clone();
            let action = Action::Pass;
            let index = indexed
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            let actor = indexed.actor().unwrap();
            let revision = indexed.revision();
            act(&mut raw, action);
            indexed.apply_index(actor, revision, index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            drop(parent);
            drain(&mut raw);
            drain(&mut indexed);
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.is_saddled(raw.object_ref(id)), shape != 1);
            assert!(raw.is_creature(id));
            assert_eq!(
                raw.current_kinds(id) & 16 != 0,
                shape != 0,
                "a Mount is not made an artifact"
            );
            indexed.restore(&saved).unwrap();
            drain(&mut indexed);
            assert_eq!(raw.snapshot(), indexed.snapshot());
            raw.finish_cleanup();
            assert!(!raw.is_saddled(raw.object_ref(id)));
            if shape == 1 {
                assert!(!raw.is_creature(id));
            }
        }
    }
}

#[test]
fn target_subtype_uses_live_type_and_never_a_departed_or_reincarnated_target() {
    let _scope = fixture_rules::activate(&[(named("Opt"), &TARGET_BRANCHES)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let id = put(&mut g, p, "Veloheart Bike", Zone::Battlefield);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(id)));
    let mut departed = g.clone();
    let old = departed.object_ref(id);
    departed.move_object(id, Zone::Exile);
    departed.move_object(id, Zone::Battlefield);
    assert_ne!(old, departed.object_ref(id));
    drain(&mut departed);
    assert!(!departed.is_creature(id));
    assert!(!departed.is_saddled(departed.object_ref(id)));
    // The target is still a legal controlled permanent after losing Vehicle.
    let object = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .push(characteristic_layers::food_form(object, timestamp));
    assert!(!g.has_subtype(id, "Vehicle"));
    drain(&mut g);
    assert!(!g.is_creature(id));
    assert!(!g.is_saddled(g.object_ref(id)));
    let none = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, none, None);
    drain(&mut g);
    assert!(!g.is_creature(id));
}

#[test]
fn noncreature_source_branch_requires_the_original_live_incarnation() {
    let _scope = fixture_rules::activate(&[(named("Veloheart Bike"), &SOURCE_BRANCH)]);
    let p = Player::P0;
    let mut parent = scenario(p);
    let id = put(&mut parent, p, "Veloheart Bike", Zone::Battlefield);
    assert!(!parent.is_creature(id));
    activate(&mut parent, id);
    let mut departed = parent.clone();
    let old = departed.object_ref(id);
    departed.move_object(id, Zone::Exile);
    departed.move_object(id, Zone::Battlefield);
    assert_ne!(old, departed.object_ref(id));
    drain(&mut departed);
    assert_eq!(departed.state.players[p.index()].life, 20);
    drain(&mut parent);
    assert_eq!(parent.state.players[p.index()].life, 23);
}

#[test]
fn entering_subtype_branch_survives_observer_departure_but_not_entrant_reincarnation() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &ENTRY_BRANCH)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let observer = put(&mut parent, p, "Grizzly Bears", Zone::Battlefield);
    let id = put(&mut parent, p, "Veloheart Bike", Zone::Hand);
    cast(&mut parent, id, None);
    reach_trigger(&mut parent, observer);
    assert!(!parent.is_creature(id));
    let mut departed = parent.clone();
    let old = departed.object_ref(id);
    departed.move_object(id, Zone::Exile);
    // Remove the observer before reentry: this branch tests only its already
    // queued old-entrant packet rather than adding a second valid entry event.
    departed.move_object(observer, Zone::Graveyard);
    departed.move_object(id, Zone::Battlefield);
    assert_ne!(old, departed.object_ref(id));
    drain(&mut departed);
    // Both Bike entries gain 2; the stale subtype packet must add nothing.
    assert_eq!(departed.state.players[p.index()].life, 24);
    parent.move_object(observer, Zone::Graveyard);
    drain(&mut parent);
    // The original Bike entry gains 2 and the live subtype packet gains 7.
    assert_eq!(parent.state.players[p.index()].life, 29);
}

#[test]
fn attached_subtype_branch_accepts_an_uncrewed_vehicle_and_uses_frozen_attachment() {
    let _scope = fixture_rules::activate(&[(named("Pacifism"), &ATTACHED_BRANCH)]);
    let p = Player::P0;
    let mut parent = scenario(p);
    let vehicle = put(&mut parent, p, "Veloheart Bike", Zone::Battlefield);
    let aura = put(&mut parent, p, "Pacifism", Zone::Hand);
    cast(&mut parent, aura, Some(Target::Card(vehicle)));
    reach_trigger(&mut parent, aura);
    assert!(!parent.is_creature(vehicle));
    let mut stale = parent.clone();
    stale.move_object(vehicle, Zone::Exile);
    stale.move_object(vehicle, Zone::Battlefield);
    drain(&mut stale);
    // The returning Bike gains 2; the old attachment packet must add nothing.
    assert_eq!(stale.state.players[p.index()].life, 22);
    parent.move_object(aura, Zone::Graveyard);
    drain(&mut parent);
    assert_eq!(parent.state.players[p.index()].life, 31);
}
