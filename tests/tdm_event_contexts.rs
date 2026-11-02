//! Frozen death, delayed departure, graveyard observer and cast-entry contexts.
//! Typed fixtures establish shared rules without changing TDM definitions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const ZOMBIE: Card = Card(5290); // @token/8f39a824d63deabffa5a0255
const HOST: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::PlusOne, 2)],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithCounters {
            relation: CastRelation::You,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::PutEventCountersOnTargets],
    }],
    ..CardRules::EMPTY
};
const KILL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 3,
        }),
        &[Instruction::Destroy],
    ),
    ..CardRules::EMPTY
};
const MIXED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::PutCountersMixed {
            subject: Subject::Targets,
            amounts: &[
                (CounterKind::PlusOne, 3),
                (CounterKind::MinusOne, 5),
                (CounterKind::Soul, 2),
            ],
        }],
    ),
    ..CardRules::EMPTY
};
const DESPERATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: -1,
            },
            Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::GraveyardUnderControllerOnce,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
const FOREBEAR: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::CreatureDies {
                relation: CastRelation::You,
                other: false,
                nontoken: false,
                subtype: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [1, 0, 0, 0, 0, 0],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::ReturnSourceToHand(TargetZone::Graveyard),
        ],
    }],
    ..CardRules::EMPTY
};
const CEREMONY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEntersYouIfCast,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DestroyEntering,
            Instruction::CreateTokens {
                token: ZOMBIE,
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const COUNTER_EVENTS: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfCountersAdded {
                kind: CounterKind::PlusOne,
                controller_only: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfCountersAdded {
                kind: CounterKind::MinusOne,
                controller_only: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
        },
    ],
    ..CardRules::EMPTY
};
const PLAIN: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    ..CardRules::EMPTY
};
const IMMORTAL: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::INDESTRUCTIBLE,
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
const COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn give(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn idle(g: &Game) -> bool {
    g.outcome().is_some()
        || (g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty())
}
fn ready(g: &mut Game, target: Option<Target>, pred: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if pred(g) {
            return;
        }
        if let Some(target) = target {
            let a = Action::ChooseTarget { target };
            if g.legal_actions().contains(&a) {
                act(g, a);
                continue;
            }
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
                    | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("event context action bound");
}
fn drain(g: &mut Game) {
    ready(g, None, idle);
}
fn spell(g: &mut Game, p: Player, name: &str, targets: &[CardId]) -> CardId {
    give(g, p);
    let id = put(g, p, name, Zone::Hand);
    announce(
        g,
        id,
        &targets
            .iter()
            .map(|&id| Target::Card(id))
            .collect::<Vec<_>>(),
    );
    id
}
fn host(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(id, CounterKind::PlusOne, 2);
    id
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.is_token() && o.card == ZOMBIE)
                .then_some(id as CardId)
        })
        .collect()
}
fn pay_return(g: &mut Game, p: Player, pay: bool) {
    ready(g, None, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    assert_eq!(g.actor(), Some(p));
    if pay {
        choose(g, |a| matches!(a, Action::PayResolutionMana { .. }));
    } else {
        act(g, Action::DeclineResolutionMana);
    }
}
#[test]
fn simultaneous_self_and_other_deaths_freeze_separate_full_counter_multisets() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &HOST), (card("Opt"), &KILL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = host(&mut g, p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let recipient = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    g.state.objects[recipient as usize].growth.toughness = 100;
    g.add_counter(a, CounterKind::Soul, 3);
    g.add_counter(a, CounterKind::Stun, 1);
    g.add_counter(b, CounterKind::Charge, 4);
    spell(&mut g, p, "Opt", &[source, a, b]);
    ready(&mut g, Some(Target::Card(recipient)), idle);
    for (kind, count) in [
        (CounterKind::PlusOne, 2),
        (CounterKind::Soul, 3),
        (CounterKind::Stun, 1),
        (CounterKind::Charge, 4),
    ] {
        assert_eq!(g.counter_count(recipient, kind), count);
    }
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.counter_count(a, CounterKind::Soul), 0);
}
#[test]
fn sba_death_snapshot_preserves_plus_minus_and_sparse_counters_before_annihilation() {
    let _scope = fixture_rules::activate(&[
        (card("Grizzly Bears"), &HOST),
        (card("Shock"), &MIXED),
        (card("Shivan Dragon"), &COUNTER_EVENTS),
    ]);
    let p = Player::P1;
    let mut g = scenario(p);
    host(&mut g, p);
    let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let recipient = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    spell(&mut g, p, "Shock", &[dead]);
    ready(&mut g, Some(Target::Card(recipient)), idle);
    assert_eq!(g.object_view(dead).zone, Zone::Graveyard);
    assert_eq!(g.counter_count(recipient, CounterKind::Soul), 2);
    assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 0);
    assert_eq!(g.counter_count(recipient, CounterKind::MinusOne), 2);
    // Both original kinds must reach the replacement pipeline, even though the
    // recipient's later SBA annihilates three pairs.
    assert_eq!(stats(&g, recipient), (3, 3));
    assert_eq!(
        g.state.players[p.index()].life,
        25,
        "both original counter kinds generate independent placement events before annihilation"
    );
}
#[test]
fn later_observer_and_new_dying_incarnation_cannot_rewrite_an_older_death_snapshot() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &HOST), (card("Opt"), &KILL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = host(&mut g, p);
    let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let target = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    g.add_counter(dead, CounterKind::Soul, 3);
    spell(&mut g, p, "Opt", &[dead]);
    ready(&mut g, Some(Target::Card(target)), |g| {
        g.object_view(dead).zone == Zone::Graveyard
            && g.state.pending == Pending::Priority
            && !g.state.stack.is_empty()
    });
    g.add_counter(source, CounterKind::Soul, 80);
    g.move_object(dead, Zone::Battlefield);
    g.add_counter(dead, CounterKind::Soul, 99);
    ready(&mut g, Some(Target::Card(target)), idle);
    assert_eq!(g.counter_count(target, CounterKind::Soul), 3);
    assert_eq!(g.counter_count(dead, CounterKind::Soul), 99);
}
#[test]
fn zero_counter_and_opposing_deaths_do_not_trigger_and_optional_recipient_can_be_omitted() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &HOST), (card("Opt"), &KILL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = host(&mut g, p);
    let blank = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
    g.add_counter(opposing, CounterKind::Soul, 4);
    spell(&mut g, p, "Opt", &[blank, opposing]);
    ready(&mut g, None, |g| {
        g.object_view(blank).zone == Zone::Graveyard
    });
    assert!(
        !g.state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .chain(&g.state.ordering_triggers)
            .chain(g.state.announcing.iter().map(|a| &a.item))
            .any(|item| item.source.is_some_and(|s| s.id == source))
    );
    drain(&mut g);
    let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.add_counter(own, CounterKind::Soul, 7);
    spell(&mut g, p, "Opt", &[own]);
    drain(&mut g);
    assert_eq!(g.state.pending_triggers.len(), 0);
    assert_eq!(g.counter_count(source, CounterKind::Soul), 0);
}
#[test]
fn death_counter_payload_and_target_choice_are_owned_across_raw_index_restore() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &HOST), (card("Opt"), &KILL)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    host(&mut parent, p);
    let dead = put(&mut parent, p, "Bear Cub", Zone::Battlefield);
    let a = put(&mut parent, p, "Shivan Dragon", Zone::Battlefield);
    let b = put(&mut parent, p, "Hill Giant", Zone::Battlefield);
    parent.add_counter(dead, CounterKind::Soul, 3);
    spell(&mut parent, p, "Opt", &[dead]);
    let action = Action::ChooseTarget {
        target: Target::Card(a),
    };
    ready(&mut parent, None, |g| {
        g.object_view(dead).zone == Zone::Graveyard && g.legal_actions().contains(&action)
    });
    let saved = parent.snapshot();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|x| *x == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.counter_count(a, CounterKind::Soul), 3);
    act(
        &mut sibling,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    drain(&mut sibling);
    assert_eq!(sibling.counter_count(b, CounterKind::Soul), 3);
    assert_eq!(sibling.counter_count(a, CounterKind::Soul), 0);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn desperate_sba_death_draws_for_original_controller() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &DESPERATE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        spell(&mut g, p, "Opt", &[creature]);
        drain(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
#[test]
fn delayed_departure_uses_controller_at_departure_even_after_losing_creature_type() {
    let _scope = fixture_rules::activate(&[(card("Opt"), &DESPERATE), (card("Shock"), &STEAL)]);
    for owns in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        spell(&mut g, p, "Opt", &[id]);
        drain(&mut g);
        if !owns {
            spell(&mut g, p.other(), "Shock", &[id]);
            drain(&mut g);
        }
        let r = g.object_ref(id);
        g.apply_temporary_copy(r, CopyRecipe::new(Card::Forest), CopyDeadline::EndOfTurn);
        assert!(!g.is_creature(id));
        g.move_object(id, Zone::Graveyard);
        g.prepare_triggers();
        drain(&mut g);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            if owns { 2 } else { 0 }
        );
    }
}
#[test]
fn delayed_departure_is_exact_expires_and_is_counterable() {
    let _scope =
        fixture_rules::activate(&[(card("Opt"), &DESPERATE), (card("Unsummon"), &COUNTER)]);
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        spell(&mut g, p, "Opt", &[id]);
        drain(&mut g);
        if mode == 0 {
            g.move_object(id, Zone::Hand);
            g.move_object(id, Zone::Battlefield);
        }
        if mode == 1 {
            g.state.turn += 1;
        }
        g.move_object(id, Zone::Graveyard);
        g.prepare_triggers();
        if mode == 2 {
            ready(&mut g, None, |g| !g.state.stack.is_empty());
            let stack = g.state.stack.last().unwrap().id;
            give(&mut g, p.other());
            let counter = put(&mut g, p.other(), "Unsummon", Zone::Hand);
            cast(&mut g, counter, Some(Target::Stack(stack)));
        }
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn graveyard_observer_returns_after_paid_cost_and_can_decline_or_be_unfunded() {
    let _scope =
        fixture_rules::activate(&[(card("Grizzly Bears"), &FOREBEAR), (card("Opt"), &KILL)]);
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            spell(&mut g, p, "Opt", &[dead]);
            pay_return(&mut g, p, pay);
            drain(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if pay { Zone::Hand } else { Zone::Graveyard }
            );
        }
    }
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    spell(&mut g, p, "Opt", &[dead]);
    g.state.players[p.index()].mana = [0; 6];
    ready(&mut g, None, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PayResolutionMana { .. }))
    );
    act(&mut g, Action::DeclineResolutionMana);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
}
#[test]
fn preexisting_graveyard_observer_sees_each_simultaneous_death_but_new_forebear_sees_none() {
    let _scope =
        fixture_rules::activate(&[(card("Grizzly Bears"), &FOREBEAR), (card("Opt"), &KILL)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let old = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let new = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    spell(&mut g, p, "Opt", &[new, dead]);
    ready(&mut g, None, |g| {
        g.object_view(dead).zone == Zone::Graveyard
    });
    let packets: Vec<_> = g
        .state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .filter(|x| x.source.is_some_and(|s| s.id == old || s.id == new))
        .collect();
    assert_eq!(packets.len(), 2);
    assert!(packets.iter().all(|x| x.source.unwrap().id == old));
    for _ in 0..2 {
        pay_return(&mut g, p, false);
    }
    drain(&mut g);
    assert_eq!(g.object_view(new).zone, Zone::Graveyard);
}
#[test]
fn graveyard_source_new_incarnation_never_rebinds_old_packet_and_leaving_unregisters() {
    let _scope =
        fixture_rules::activate(&[(card("Grizzly Bears"), &FOREBEAR), (card("Opt"), &KILL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    spell(&mut g, p, "Opt", &[dead]);
    ready(&mut g, None, |g| {
        g.object_view(dead).zone == Zone::Graveyard
    });
    g.move_object(source, Zone::Exile);
    g.move_object(source, Zone::Graveyard);
    pay_return(&mut g, p, true);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    g.move_object(source, Zone::Hand);
    assert!(!g.state.death_sources.contains(&source));
}
#[test]
fn graveyard_observer_owned_payment_restore_preserves_exact_source() {
    let _scope =
        fixture_rules::activate(&[(card("Grizzly Bears"), &FOREBEAR), (card("Opt"), &KILL)]);
    let p = Player::P1;
    let mut parent = scenario(p);
    let source = put(&mut parent, p, "Grizzly Bears", Zone::Graveyard);
    let dead = put(&mut parent, p, "Bear Cub", Zone::Battlefield);
    spell(&mut parent, p, "Opt", &[dead]);
    ready(&mut parent, None, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
    let action = parent
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayResolutionMana { .. }))
        .unwrap();
    let rev = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|x| *x == action)
        .unwrap();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut sibling = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(source).zone, Zone::Hand);
    act(&mut sibling, Action::DeclineResolutionMana);
    drain(&mut sibling);
    assert_eq!(sibling.object_view(source).zone, Zone::Graveyard);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn cast_creature_entry_observer_destroys_exact_entrant_and_tokens_do_not_retrigger() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &CEREMONY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = put(&mut g, p, "Short Bow", Zone::Hand);
        cast(&mut g, observer, None);
        drain(&mut g);
        let entrant = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, entrant, None);
        drain(&mut g);
        assert_eq!(g.object_view(entrant).zone, Zone::Graveyard);
        let made = tokens(&g, p);
        assert_eq!(made.len(), 1);
        assert_eq!(stats(&g, made[0]), (2, 2));
    }
}
#[test]
fn direct_and_other_caster_entries_do_not_qualify_cast_observer() {
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &CEREMONY)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    let direct = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    g.move_object(direct, Zone::Battlefield);
    drain(&mut g);
    assert_eq!(g.object_view(direct).zone, Zone::Battlefield);
    assert!(tokens(&g, p).is_empty());
    g.state.active = p.other();
    g.state.priority = p.other();
    let spell = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
    cast(&mut g, spell, None);
    let item = g.state.stack.pop().unwrap();
    assert_eq!(item.source.unwrap().id, spell);
    let mut movement = g.entry_move(spell, Zone::Battlefield, false);
    movement.controller = p;
    movement.cast_event = item.actual_cast_event().map(Box::new);
    movement.cast_origin = item.cast.map(|c| c.origin);
    g.stage_entries(vec![movement], None);
    assert!(!g.advance_entries());
    g.prepare_triggers();
    drain(&mut g);
    assert_eq!(g.object_view(spell).controller, p);
    assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    assert!(tokens(&g, p).is_empty());
}
#[test]
fn cast_observer_token_creation_is_independent_of_indestructibility_or_stale_entry() {
    for mode in 0..3 {
        let rules = if mode == 0 { &IMMORTAL } else { &PLAIN };
        let _scope =
            fixture_rules::activate(&[(card("Short Bow"), &CEREMONY), (card("Bear Cub"), rules)]);
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let entrant = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, entrant, None);
        ready(&mut g, None, |g| {
            g.object_view(entrant).zone == Zone::Battlefield
                && g.state.pending == Pending::Priority
                && !g.state.stack.is_empty()
        });
        if mode == 1 {
            g.move_object(entrant, Zone::Hand);
            g.move_object(entrant, Zone::Battlefield);
        }
        if mode == 2 {
            let r = g.object_ref(entrant);
            g.apply_temporary_copy(r, CopyRecipe::new(Card::Forest), CopyDeadline::EndOfTurn);
        }
        drain(&mut g);
        assert_eq!(tokens(&g, p).len(), 1);
        assert_eq!(
            g.object_view(entrant).zone,
            if mode == 2 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}

#[test]
fn copied_death_counters_use_normal_replacements_and_prohibition() {
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
    for blocked in [false, true] {
        let _scope = fixture_rules::activate(&[
            (card("Grizzly Bears"), &HOST),
            (card("Opt"), &KILL),
            (card("Short Bow"), if blocked { &BARRIER } else { &DOUBLER }),
        ]);
        let p = Player::P0;
        let mut g = scenario(p);
        host(&mut g, p);
        put(
            &mut g,
            if blocked { p.other() } else { p },
            "Short Bow",
            Zone::Battlefield,
        );
        let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let recipient = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        g.add_counter(dead, CounterKind::Soul, 3);
        g.add_counter(dead, CounterKind::Stun, 2);
        spell(&mut g, p, "Opt", &[dead]);
        ready(&mut g, Some(Target::Card(recipient)), idle);
        assert_eq!(
            g.counter_count(recipient, CounterKind::Soul),
            if blocked { 0 } else { 6 }
        );
        assert_eq!(
            g.counter_count(recipient, CounterKind::Stun),
            if blocked { 0 } else { 4 }
        );
    }
}
#[test]
fn token_death_snapshot_survives_ceasing_and_slot_reuse() {
    let _scope = fixture_rules::activate(&[(card("Grizzly Bears"), &HOST), (card("Opt"), &KILL)]);
    let p = Player::P1;
    let mut g = scenario(p);
    host(&mut g, p);
    let recipient = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
    g.stage_copy_creation(p, CopyRecipe::new(card("Bear Cub")), 1);
    assert!(!g.advance_entries());
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| (o.zone == Zone::Battlefield && o.is_token()).then_some(id as CardId))
        .unwrap();
    let before = g.object_ref(token);
    g.add_counter(token, CounterKind::Soul, 3);
    spell(&mut g, p, "Opt", &[token]);
    ready(&mut g, Some(Target::Card(recipient)), |g| {
        g.state.objects[token as usize].zone == Zone::Outside
            && g.state.pending == Pending::Priority
            && !g.state.stack.is_empty()
    });
    g.stage_copy_creation(p, CopyRecipe::new(card("Bear Cub")), 1);
    assert!(!g.advance_entries());
    assert_ne!(g.object_ref(token), before);
    assert_eq!(g.object_view(token).zone, Zone::Battlefield);
    g.add_counter(token, CounterKind::Soul, 99);
    drain(&mut g);
    assert_eq!(g.counter_count(recipient, CounterKind::Soul), 3);
    assert_eq!(g.counter_count(token, CounterKind::Soul), 99);
}

#[test]
fn graveyard_observer_uses_its_owner_and_the_dying_creatures_controller() {
    let _scope =
        fixture_rules::activate(&[(card("Grizzly Bears"), &FOREBEAR), (card("Opt"), &KILL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[source as usize].controller = p.other();
    g.move_object(source, Zone::Graveyard);
    assert_eq!(g.object_view(source).controller, p);
    let borrowed = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    g.state.objects[borrowed as usize].controller = p;
    spell(&mut g, p, "Opt", &[borrowed]);
    pay_return(&mut g, p, true);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Hand);
    g.move_object(source, Zone::Graveyard);
    let given = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.state.objects[given as usize].controller = p.other();
    spell(&mut g, p, "Opt", &[given]);
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
}

#[test]
fn ceased_graveyard_token_or_card_copy_cannot_duplicate_a_reused_slots_death_observer() {
    let _scope = fixture_rules::activate(&[
        (card("Grizzly Bears"), &FOREBEAR),
        (card("Hill Giant"), &HOST),
        (card("Opt"), &KILL),
    ]);
    for card_copy in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let vacant = if card_copy {
            let copy = g.allocate_card_copy(card("Grizzly Bears"), p);
            g.move_object(copy.card, Zone::Graveyard);
            g.state_based_actions();
            copy.card
        } else {
            g.stage_copy_creation(p, CopyRecipe::new(card("Grizzly Bears")), 1);
            assert!(!g.advance_entries());
            let id = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, o)| {
                    (o.zone == Zone::Battlefield && o.is_token()).then_some(id as CardId)
                })
                .unwrap();
            spell(&mut g, p, "Opt", &[id]);
            drain(&mut g);
            id
        };
        assert_eq!(g.state.objects[vacant as usize].zone, Zone::Outside);
        assert!(!g.state.death_sources.contains(&vacant));
        g.stage_copy_creation(p, CopyRecipe::new(card("Hill Giant")), 1);
        assert!(!g.advance_entries());
        assert_eq!(g.object_view(vacant).zone, Zone::Battlefield);
        assert_eq!(
            g.state
                .death_sources
                .iter()
                .filter(|&&id| id == vacant)
                .count(),
            1
        );
        let dead = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let recipient = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        g.add_counter(dead, CounterKind::Soul, 3);
        spell(&mut g, p, "Opt", &[dead]);
        ready(&mut g, Some(Target::Card(recipient)), idle);
        assert_eq!(g.counter_count(recipient, CounterKind::Soul), 3);
    }
}
