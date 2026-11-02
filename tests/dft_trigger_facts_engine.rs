//! Cast participants, historical declarations and source-independent delayed clauses.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const fn ability(targets: TargetPlan, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const JOCKEY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::CasterOffTurn {
                filter: &CastFilter::Any,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::CastingPlayer, 4)],
    }],
    ..CardRules::EMPTY
};
const MARCH: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::CasterOffTurn {
                filter: &CastFilter::Any,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const LEGACY_TURN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DuringOpponentsTurn(&TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const FEARLESS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::DeclaredAttackSubtypesAll(&["Pirate", "Vehicle"]),
            event: &TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const VEHICLE_PIRATE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ability(
        TargetPlan::NONE,
        &[Instruction::AnimateSource(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Named(&["Pirate"]),
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    )],
    ..CardRules::EMPTY
};
const VEHICLE_ONLY: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ability(
        TargetPlan::NONE,
        &[Instruction::AnimateSource(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    )],
    ..CardRules::EMPTY
};
const ADD_PIRATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::AddCreatureTypes {
            subject: Subject::Targets,
            types: &["Pirate"],
        }],
    ),
    ..CardRules::EMPTY
};
const GRIM: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::DiesOnce,
                ability: 0,
            },
        ],
    )],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Surveil(1)],
    }],
    ..CardRules::EMPTY
};
const HOWLER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ability(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::DelayTargetEvent {
            event: DelayedTargetEvent::CombatDamageToPlayer,
            ability: 0,
        }],
    )],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const STEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GainControlUntilEndTurn],
    ),
    ..CardRules::EMPTY
};
const REMOVE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};
const COPIER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ability(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    )],
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn direct(g: &mut Game, id: CardId, target: Option<Target>) {
    choose(
        g,
        |a| matches!(a,Action::Activate {card,ability:0,target:t,..} if *card==id&&*t==target),
    );
}
fn idle(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending == Pending::Priority
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("64-action trigger drain ceiling");
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    for &card in ids {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
fn grant(g: &mut Game, source: CardId, target: CardId) {
    direct(g, source, Some(Target::Card(target)));
    drain(g);
    assert!(g.observe(g.state.active).delayed_target_events.is_some());
}
fn wait_library(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("32-action surveil ceiling");
}
fn combat_to_end(g: &mut Game) {
    for _ in 0..96 {
        if g.state.step == Step::EndCombat && idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::AssignDamage {
                        target: Target::Player(_),
                        ..
                    }
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("96-action unblocked combat ceiling");
}
fn gonti_rules() -> &'static CardRules {
    let program = Box::leak(Box::new([Instruction::CreateTokensFor {
        players: PlayerSet::CastingPlayer,
        token: named("@token/b60f6cd3b44b22bd289fe4e2"),
        amount: 1,
        tapped: false,
    }]));
    let triggers = Box::leak(Box::new([TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::NotOwnedByCaster,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program,
    }]));
    Box::leak(Box::new(CardRules {
        effect: Effect::Creature(2, 2),
        triggers,
        ..CardRules::EMPTY
    }))
}
fn treasures(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, "Treasure")
        })
        .count()
}

#[test]
fn cast_off_turn_uses_actual_caster_independently_of_observer_controller() {
    let _scope =
        fixture_rules::activate(&[(Card::GrizzlyBears, &JOCKEY), (Card::HillGiant, &MARCH)]);
    for active in [Player::P0, Player::P1] {
        for caster in [Player::P0, Player::P1] {
            for observer in [Player::P0, Player::P1] {
                let mut g = scenario(active);
                g.state.priority = caster;
                let jockey = put(&mut g, observer, "Grizzly Bears", Zone::Battlefield);
                let march = put(&mut g, observer, "Hill Giant", Zone::Battlefield);
                let spell = put(&mut g, caster, "Shock", Zone::Hand);
                cast(&mut g, spell, Some(Target::Player(caster.other())));
                g.move_object(jockey, Zone::Graveyard);
                g.move_object(march, Zone::Graveyard);
                drain(&mut g);
                let mut lives = [20, 20];
                lives[caster.other().index()] -= 2;
                if caster != active {
                    lives[caster.index()] -= 4;
                    if caster != observer {
                        lives[observer.index()] += 1;
                    }
                }
                assert_eq!(
                    g.state.players.each_ref().map(|p| p.life),
                    lives,
                    "active={active:?} caster={caster:?} observer={observer:?}"
                );
            }
        }
    }
}

#[test]
fn legacy_observer_relative_opponents_turn_wrapper_keeps_its_meaning() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &LEGACY_TURN)]);
    for active in [Player::P0, Player::P1] {
        let mut g = scenario(active);
        put(&mut g, active.other(), "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, active, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(active)));
        drain(&mut g);
        assert_eq!(g.state.players[active.other().index()].life, 21);
        assert_eq!(g.state.players[active.index()].life, 18);
    }
}

#[test]
fn foreign_owned_spell_creates_treasure_for_caster_after_observer_leaves() {
    let rules = gonti_rules();
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, rules)]);
    for caster in [Player::P0, Player::P1] {
        for observer in [Player::P0, Player::P1] {
            for foreign in [false, true] {
                let mut g = scenario(caster);
                let source = put(&mut g, observer, "Grizzly Bears", Zone::Battlefield);
                let owner = if foreign { caster.other() } else { caster };
                let spell = put(&mut g, owner, "Shock", Zone::Exile);
                let object = g.object_ref(spell);
                g.grant_exile_play_permission(caster, object, false);
                cast(&mut g, spell, Some(Target::Player(caster.other())));
                g.move_object(source, Zone::Graveyard);
                drain(&mut g);
                assert_eq!(treasures(&g, caster), usize::from(foreign));
                assert_eq!(treasures(&g, caster.other()), 0);
            }
        }
    }
}

#[test]
fn copying_a_foreign_spell_does_not_create_an_extra_cast_event() {
    let rules = gonti_rules();
    let _scope =
        fixture_rules::activate(&[(Card::GrizzlyBears, rules), (Card::HillGiant, &COPIER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let copier = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, p.other(), "Shock", Zone::Exile);
    let object = g.object_ref(spell);
    g.grant_exile_play_permission(p, object, false);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    direct(&mut g, copier, Some(Target::Spell(spell)));
    drain(&mut g);
    assert_eq!(treasures(&g, p), 1);
    assert_eq!(g.state.players[p.other().index()].life, 16);
}

#[test]
fn one_declared_attacker_can_be_both_pirate_and_vehicle_and_history_survives_departures() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &FEARLESS),
        (named("Clamorous Ironclad"), &VEHICLE_PIRATE),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        direct(&mut g, vehicle, None);
        drain(&mut g);
        assert!(g.has_subtype(vehicle, "Pirate") && g.has_subtype(vehicle, "Vehicle"));
        attack(&mut g, &[vehicle]);
        let history = g.observe(p).declared_attack_history.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].subtypes, vec!["Pirate", "Vehicle"]);
        g.move_object(vehicle, Zone::Graveyard);
        g.move_object(observer, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        combat_to_end(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(g.observe(p).declared_attack_history.is_none());
    }
}

#[test]
fn separately_declared_pirate_and_vehicle_qualify_but_later_type_gain_cannot_rewrite_history() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &FEARLESS),
        (named("Clamorous Ironclad"), &VEHICLE_ONLY),
        (named("Opt"), &ADD_PIRATE),
    ]);
    for with_pirate in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        direct(&mut g, vehicle, None);
        drain(&mut g);
        let mut attackers = vec![vehicle];
        if with_pirate {
            attackers.push(put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield));
        }
        attack(&mut g, &attackers);
        drain(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            if with_pirate { 21 } else { 20 }
        );
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(vehicle)));
        drain(&mut g);
        assert!(g.has_subtype(vehicle, "Pirate"));
        assert_eq!(
            g.step_guard(
                StepGuard::DeclaredAttackSubtypesAll(&["Pirate", "Vehicle"]),
                p,
                g.object_ref(observer),
                false
            ),
            with_pirate
        );
    }
}

#[test]
fn declaration_history_is_opt_in_and_never_synthesized_from_live_attackers() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &FEARLESS),
        (named("Clamorous Ironclad"), &VEHICLE_PIRATE),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    direct(&mut g, vehicle, None);
    drain(&mut g);
    attack(&mut g, &[vehicle]);
    assert!(g.observe(p).declared_attack_history.is_none());
    let observer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    assert!(!g.step_guard(
        StepGuard::DeclaredAttackSubtypesAll(&["Pirate", "Vehicle"]),
        p,
        g.object_ref(observer),
        false
    ));
}

#[test]
fn delayed_death_surveillance_keeps_original_controller_and_private_owned_choice() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GRIM), (named("Shock"), &STEAL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    grant(&mut g, source, target);
    assert_eq!(stats(&g, target), (4, 3));
    g.move_object(source, Zone::Graveyard);
    g.state.priority = p.other();
    let steal = put(&mut g, p.other(), "Shock", Zone::Hand);
    cast(&mut g, steal, Some(Target::Card(target)));
    drain(&mut g);
    assert_eq!(g.object_view(target).controller, p.other());
    g.move_object(target, Zone::Graveyard);
    wait_library(&mut g);
    assert_eq!(g.actor(), Some(p));
    assert!(g.observe(p).library_choice.is_some());
    assert!(g.observe(p.other()).library_choice.is_none());
    let card = *g
        .state
        .resolving
        .as_ref()
        .unwrap()
        .library
        .as_ref()
        .unwrap()
        .remaining
        .first()
        .unwrap();
    let action = Action::ArrangeLibrary {
        card,
        destination: LibraryDestination::Graveyard,
    };
    let snapshot = g.snapshot();
    let rev = g.revision();
    let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    drop(g);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), snapshot);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(card).zone, Zone::Graveyard);
    assert!(raw.observe(p).delayed_target_events.is_none());
}

#[test]
fn delayed_death_requires_exact_creature_death_and_expires_at_cleanup() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GRIM)]);
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        grant(&mut g, source, target);
        if mode == 0 {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Battlefield);
            g.move_object(target, Zone::Graveyard);
        }
        if mode == 1 {
            put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            g.move_object(target, Zone::Graveyard);
        }
        if mode == 2 {
            g.finish_cleanup();
            g.move_object(target, Zone::Graveyard);
        }
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.stack.is_empty());
        assert!(g.observe(p).delayed_target_events.is_none());
    }
}

#[test]
fn delayed_combat_draw_is_original_controllers_and_repeats_for_double_strike_and_stacked_clauses() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &HOWLER), (named("Opt"), &REMOVE)]);
    for p in [Player::P0, Player::P1] {
        let attacker = p.other();
        let mut g = scenario(attacker);
        g.state.priority = p;
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut g, attacker, "Hill Giant", Zone::Battlefield);
        g.state.objects[target as usize].gained_keywords = Keywords::DOUBLE_STRIKE;
        grant(&mut g, source, target);
        g.state.priority = p;
        grant(&mut g, source, target);
        g.move_object(source, Zone::Graveyard);
        attack(&mut g, &[target]);
        combat_to_end(&mut g);
        assert_eq!(g.hand_count(p), 4);
        assert_eq!(g.hand_count(attacker), 0);
        assert_eq!(g.state.players[p.index()].life, 14);
        g.finish_cleanup();
        assert!(g.observe(p).delayed_target_events.is_none());
    }
}

#[test]
fn delayed_combat_draw_persists_while_the_watched_creature_loses_all_abilities() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &HOWLER), (named("Opt"), &REMOVE)]);
    let p = Player::P0;
    let attacker = p.other();
    let mut g = scenario(attacker);
    g.state.priority = p;
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut g, attacker, "Hill Giant", Zone::Battlefield);
    grant(&mut g, source, target);
    g.move_object(source, Zone::Graveyard);
    g.state.priority = p;
    let remove = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, remove, Some(Target::Card(target)));
    drain(&mut g);
    assert!(!g.printed_abilities_available(target));
    attack(&mut g, &[target]);
    combat_to_end(&mut g);
    assert_eq!(g.hand_count(p), 1);
    assert_eq!(g.hand_count(attacker), 0);
    assert_eq!(g.state.players[p.index()].life, 17);
}
