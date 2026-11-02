//! Generic cast-copy consumers retain permanent spell packets, without inventing casts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CONSUMER: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::SpellCast {
        relation: CastRelation::You,
        filter: CastFilter::Any,
        ordinal: None,
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::CopyCastSpell {
        prior_spells: false,
    }],
};
const COPY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[CONSUMER],
    ..CardRules::EMPTY
};
const OPTIONAL: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::CopyCastSpell {
                prior_spells: false,
            },
        ],
        ..CONSUMER
    }],
    ..COPY
};
const COUNT_CASTS: CardRules = CardRules {
    triggers: &[
        CONSUMER,
        TriggeredAbility {
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            ..CONSUMER
        },
    ],
    ..COPY
};
const ETB: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
fn observer(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Ghost Vacuum", Zone::Battlefield)
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn advance(g: &mut Game) {
    choose(g, |a| {
        matches!(
            a,
            Action::Pass
                | Action::PutTrigger { .. }
                | Action::FinishTargets
                | Action::FinishStackTargets
        )
    });
}
fn drain(g: &mut Game) {
    for _ in 0..96 {
        if idle(g) {
            return;
        }
        advance(g);
    }
    panic!("bounded cast-copy resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..64 {
        if g.legal_actions().iter().any(&wanted) {
            return;
        }
        advance(g);
    }
    panic!("bounded cast-copy choice");
}
fn packet(g: &Game, source: CardId) -> &SpellView {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .find(|item| item.kind != StackKind::Spell && item.source.is_some_and(|s| s.id == source))
        .expect("observer packet")
}
fn entrants(g: &Game, name: &str) -> Vec<CardId> {
    let card = Card::from_name(name).unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.card == card).then_some(id as CardId)
        })
        .collect()
}
fn counter_original(g: &mut Game, caster: Player, original: CardId) {
    reach(g, |a| matches!(a, Action::Pass));
    if g.actor() != Some(caster.other()) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(caster.other()));
    let cancel = put(g, caster.other(), "Cancel", Zone::Hand);
    cast(g, cancel, Some(Target::Spell(original)));
    for _ in 0..16 {
        if g.object_view(original).zone == Zone::Graveyard {
            return;
        }
        advance(g);
    }
    panic!("counter must resolve before retained copy trigger");
}
#[test]
fn artifact_and_creature_casts_capture_the_actual_controller_spell_and_become_tokens() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &COPY)]);
    for p in [Player::P0, Player::P1] {
        for name in ["Swiftfoot Boots", "Grizzly Bears"] {
            let mut g = scenario(p);
            let source = observer(&mut g, p);
            let original = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, original, None);
            let event = packet(&g, source)
                .copy_event
                .as_ref()
                .expect("permanent cast must be captured");
            assert_eq!(event.amount, 1);
            assert_eq!(event.spell.source.unwrap().object, g.object_ref(original));
            assert_eq!(event.spell.controller, p);
            assert_eq!(event.spell.kind, StackKind::Spell);
            drain(&mut g);
            let copies = entrants(&g, name);
            assert_eq!(copies.len(), 2, "{name}");
            assert_eq!(
                copies
                    .iter()
                    .filter(|id| g.state.objects[**id as usize].is_token())
                    .count(),
                1
            );
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 0);
            assert!(g.state.stack_copy_data.is_none());
        }
    }
    let mut g = scenario(Player::P1);
    observer(&mut g, Player::P0);
    let foreign = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
    cast(&mut g, foreign, None);
    assert!(g.state.pending_triggers.is_empty() && g.state.ordering_triggers.is_empty());
    drain(&mut g);
    assert_eq!(entrants(&g, "Grizzly Bears").len(), 1);
}
#[test]
fn optional_payment_copies_only_when_paid_and_unfunded_forgery_is_transactional() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &OPTIONAL)]);
    for (accept, funded) in [(false, true), (true, true), (false, false)] {
        let mut g = scenario(Player::P0);
        observer(&mut g, Player::P0);
        let original = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Hand);
        cast(&mut g, original, None);
        g.state.players[0].mana = if funded { [0, 0, 0, 0, 0, 1] } else { [0; 6] };
        reach(&mut g, |a| matches!(a, Action::DeclineResolutionMana));
        if !funded {
            reject(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
        }
        if accept {
            choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
        } else {
            act(&mut g, Action::DeclineResolutionMana);
        }
        drain(&mut g);
        assert_eq!(
            entrants(&g, "Swiftfoot Boots").len(),
            if accept { 2 } else { 1 }
        );
        assert_eq!(g.state.players[0].mana[5], u32::from(funded && !accept));
    }
}
#[test]
fn countered_permanent_original_still_supplies_its_owned_blueprint_to_the_copy() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &COPY)]);
    for p in [Player::P0, Player::P1] {
        for name in ["Swiftfoot Boots", "Grizzly Bears"] {
            let mut g = scenario(p);
            let source = observer(&mut g, p);
            let original = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, original, None);
            let exact = g.object_ref(original);
            let observer_exact = g.object_ref(source);
            counter_original(&mut g, p, original);
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            assert_ne!(observer_exact, g.object_ref(source));
            assert_eq!(packet(&g, source).source.unwrap().object, observer_exact);
            assert_eq!(packet(&g, source).controller, p);
            assert_ne!(exact, g.object_ref(original));
            assert_eq!(
                packet(&g, source)
                    .copy_event
                    .as_ref()
                    .unwrap()
                    .spell
                    .source
                    .unwrap()
                    .object,
                exact
            );
            drain(&mut g);
            let copies = entrants(&g, name);
            assert_eq!(copies.len(), 1);
            assert!(g.state.objects[copies[0] as usize].is_token());
            assert_eq!(g.object_view(copies[0]).controller, p);
            assert_eq!(g.object_view(original).zone, Zone::Graveyard);
            if name == "Grizzly Bears" {
                assert_eq!(stats(&g, copies[0]), (2, 2));
            }
        }
    }
}
#[test]
fn resolving_permanent_copy_is_an_entry_not_a_cast_or_token_creation_event() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Ghost Vacuum").unwrap(), &COUNT_CASTS),
        (Card::GrizzlyBears, &ETB),
    ]);
    let mut g = scenario(Player::P0);
    observer(&mut g, Player::P0);
    put(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    let original = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut g, original, None);
    drain(&mut g);
    let bears = entrants(&g, "Grizzly Bears");
    assert_eq!(
        bears.len(),
        2,
        "token-creation doubling cannot multiply a resolving spell copy"
    );
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.players[0].creature_spells_cast_this_turn, 1);
    assert_eq!(g.state.players[0].life, 25, "two ETBs and one actual cast");
    assert_eq!(
        bears
            .iter()
            .filter(|id| g.state.objects[**id as usize].is_token())
            .count(),
        1
    );
}
#[test]
fn copied_permanent_spell_keeps_its_cast_x_even_after_the_original_is_countered() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let source = observer(&mut g, Player::P0);
    let original = put(&mut g, Player::P0, "Pterafractyl", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: original });
    act(&mut g, Action::ChooseX { value: 3 });
    pay(&mut g);
    assert_eq!(
        packet(&g, source).copy_event.as_ref().unwrap().spell.x,
        Some(3)
    );
    counter_original(&mut g, Player::P0, original);
    drain(&mut g);
    let copies = entrants(&g, "Pterafractyl");
    assert_eq!(copies.len(), 1);
    assert!(g.state.objects[copies[0] as usize].is_token());
    assert_eq!(g.counter_count(copies[0], CounterKind::PlusOne), 3);
    assert_eq!(stats(&g, copies[0]), (4, 3));
    assert_eq!(g.state.players[0].life, 22);
}
#[test]
fn aura_copy_retains_its_bound_target_and_can_choose_a_distinct_new_target() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let source = observer(&mut g, Player::P0);
    let first = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(first)));
    assert_eq!(
        packet(&g, source)
            .copy_event
            .as_ref()
            .unwrap()
            .spell
            .targets[0]
            .object,
        Some(g.object_ref(first))
    );
    reach(&mut g, |a| matches!(a, Action::FinishStackTargets));
    act(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(second),
        },
    );
    act(&mut g, Action::FinishStackTargets);
    drain(&mut g);
    let auras = entrants(&g, "Pacifism");
    assert_eq!(auras.len(), 2);
    let token = *auras
        .iter()
        .find(|id| g.state.objects[**id as usize].is_token())
        .unwrap();
    assert_eq!(
        g.state.objects[aura as usize].attached_to,
        Some(g.object_ref(first))
    );
    assert_eq!(
        g.state.objects[token as usize].attached_to,
        Some(g.object_ref(second))
    );
}
#[test]
fn inherited_instant_copy_preserves_mode_and_target_without_new_cast_events() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let source = observer(&mut g, Player::P0);
    let victim = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(victim, CounterKind::PlusOne, 5);
    let abrade = put(&mut g, Player::P0, "Abrade", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: abrade });
    act(&mut g, Action::ChooseMode { mode: 0 });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut g);
    assert_eq!(
        packet(&g, source).copy_event.as_ref().unwrap().spell.mode,
        Some(0)
    );
    drain(&mut g);
    assert_eq!(g.object_view(victim).damage, 6);
    assert_eq!(stats(&g, victim), (7, 7));
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.players[0].instant_sorcery_cast_this_turn, 1);
}
#[test]
fn unrelated_and_narrow_instant_only_programs_do_not_allocate_permanent_capture_payloads() {
    const UNRELATED: CardRules = CardRules {
        triggers: &[TriggeredAbility {
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            ..CONSUMER
        }],
        ..COPY
    };
    const PRIOR: CardRules = CardRules {
        triggers: &[TriggeredAbility {
            program: &[Instruction::CopyCastSpell { prior_spells: true }],
            ..CONSUMER
        }],
        ..COPY
    };
    const CHOSEN_X: CardRules = CardRules {
        triggers: &[TriggeredAbility {
            program: &[Instruction::CopyCastSpellChosenX],
            ..CONSUMER
        }],
        ..COPY
    };
    const PREPARED: CardRules = CardRules {
        triggers: &[TriggeredAbility {
            program: &[Instruction::CopyPreparedCastSpell],
            ..CONSUMER
        }],
        ..COPY
    };
    for rules in [&UNRELATED, &PRIOR, &CHOSEN_X, &PREPARED] {
        let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), rules)]);
        let mut g = scenario(Player::P0);
        let source = observer(&mut g, Player::P0);
        let original = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        cast(&mut g, original, None);
        assert!(packet(&g, source).copy_event.is_none());
        assert!(g.state.stack_copy_data.is_none());
        drain(&mut g);
        assert_eq!(entrants(&g, "Grizzly Bears").len(), 1);
        assert!(g.state.stack_copy_data.is_none());
    }
    let mut g = scenario(Player::P0);
    let original = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Hand);
    cast(&mut g, original, None);
    assert!(g.state.pending_triggers.is_empty() && g.state.ordering_triggers.is_empty());
    assert!(g.state.stack_copy_data.is_none());
    drain(&mut g);
}
#[test]
fn permanent_cast_does_not_consume_an_inherited_delayed_next_instant_permission() {
    let mut g = scenario(Player::P0);
    let loki = put(&mut g, Player::P0, "Loki Laufeyson", Zone::Battlefield);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: loki,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.observe(Player::P0).delayed_spell_copies.len(), 1);
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut g, bear, None);
    drain(&mut g);
    assert_eq!(entrants(&g, "Grizzly Bears").len(), 1);
    assert_eq!(g.observe(Player::P0).delayed_spell_copies.len(), 1);
    let shock = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    drain(&mut g);
    assert_eq!(g.observe(Player::P0).delayed_spell_copies.len(), 0);
    assert_eq!(g.state.players[1].life, 16);
}
#[test]
fn captured_countered_spell_and_optional_payment_survive_parent_disposal_and_owned_replay() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Ghost Vacuum").unwrap(), &OPTIONAL)]);
    for p in [Player::P0, Player::P1] {
        let mut parent = scenario(p);
        observer(&mut parent, p);
        let original = put(&mut parent, p, "Grizzly Bears", Zone::Hand);
        cast(&mut parent, original, None);
        counter_original(&mut parent, p, original);
        reach(&mut parent, |a| matches!(a, Action::DeclineResolutionMana));
        let payment = parent
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap();
        let saved = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let mut restored = parent.clone();
        let mut declined = parent.clone();
        restored.restore(&saved).unwrap();
        drop(parent);
        act(&mut declined, Action::DeclineResolutionMana);
        drain(&mut declined);
        assert!(entrants(&declined, "Grizzly Bears").is_empty());
        for (n, g) in [&mut raw, &mut indexed, &mut restored]
            .into_iter()
            .enumerate()
        {
            if n == 1 {
                let index = g
                    .legal_actions()
                    .iter()
                    .position(|a| *a == payment)
                    .unwrap();
                g.apply_index(p, g.revision(), index).unwrap();
            } else {
                act(g, payment.clone());
            }
            drain(g);
            let copies = entrants(g, "Grizzly Bears");
            assert_eq!(copies.len(), 1);
            assert!(g.state.objects[copies[0] as usize].is_token());
            assert_eq!(g.object_view(original).zone, Zone::Graveyard);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}
