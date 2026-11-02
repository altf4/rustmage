//! Exert activation costs and their exact next-untap obligations (CR701.43).
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const EXERT_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExertCost,
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const COUNTER_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const UNTAP_RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Untap],
    ),
    ..CardRules::EMPTY
};
fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn exert(g: &mut Game, card: CardId) {
    act(g, activation(card));
    pay(g);
}
fn reset_priority(g: &mut Game, p: Player) {
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn untap_step(g: &mut Game, p: Player) {
    g.state.active = p;
    g.untap_active_player();
}

#[test]
fn exert_is_paid_before_resolution_and_survives_countering() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXERT_RULES),
        (Card::Counterspell, &COUNTER_RULES),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let life = g.state.players[p.index()].life;
        act(&mut g, activation(source));
        assert!(
            g.observe(p).exertions.is_none(),
            "announcing is not paying exert"
        );
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 0],
            },
        );
        assert!(g.observe(p).exertions.is_none());
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        assert_eq!(
            g.state.players[p.index()].life,
            life,
            "ability has not resolved"
        );
        assert_eq!(
            g.observe(p).exertions,
            Some(vec![ExertionView {
                object: g.object_ref(source),
                player: p
            }])
        );
        let ability = g.state.stack.last().unwrap().id;
        act(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Counterspell", Zone::Hand);
        cast(&mut g, counter, Some(Target::Stack(ability)));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        untap_step(&mut g, p);
        assert!(
            g.object_view(source).tapped,
            "countering the effect does not refund exert cost"
        );
        assert!(g.observe(p).exertions.is_none());
        untap_step(&mut g, p);
        assert!(
            !g.object_view(source).tapped,
            "only the next untap is skipped"
        );
    }
}

#[test]
fn overlapping_exertions_expire_together_and_external_untaps_work() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXERT_RULES),
        (Card::GiantGrowth, &UNTAP_RULES),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let life = g.state.players[0].life;
    exert(&mut g, source);
    settle(&mut g);
    let untap = put(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    cast(&mut g, untap, Some(Target::Card(source)));
    settle(&mut g);
    assert!(!g.object_view(source).tapped);
    assert!(g.observe(Player::P0).exertions.is_some());
    exert(&mut g, source);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, life + 2);
    assert_eq!(g.observe(Player::P0).exertions.as_ref().unwrap().len(), 1);
    untap_step(&mut g, Player::P0);
    assert!(g.object_view(source).tapped);
    assert!(g.observe(Player::P0).exertions.is_none());
    untap_step(&mut g, Player::P0);
    assert!(
        !g.object_view(source).tapped,
        "two exertions skip one common untap, not two"
    );
}

#[test]
fn exert_expiration_is_bound_to_exerting_player_not_current_controller() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXERT_RULES)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        exert(&mut g, source);
        settle(&mut g);
        g.record_control(g.object_ref(source), p.other(), None);
        untap_step(&mut g, p.other());
        assert!(
            !g.object_view(source).tapped,
            "new controller's untap is not prohibited"
        );
        assert!(
            g.observe(p).exertions.is_some(),
            "other player's untap cannot consume original obligation"
        );
        g.tap_objects(&[source]);
        untap_step(&mut g, p);
        assert!(
            g.object_view(source).tapped,
            "not controlled by active player, no untap attempted"
        );
        assert!(
            g.observe(p).exertions.is_none(),
            "original next untap expires obligation even while another player controls it"
        );
        g.record_control(g.object_ref(source), p, None);
        untap_step(&mut g, p);
        assert!(
            !g.object_view(source).tapped,
            "regaining control cannot revive an expired exertion"
        );
    }
}

#[test]
fn exertions_for_different_players_coexist_and_expire_independently() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXERT_RULES)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    exert(&mut g, source);
    settle(&mut g);
    g.record_control(g.object_ref(source), Player::P1, None);
    g.untap_objects(&[source], untap_rules::UntapCause::Effect);
    // A controller needs haste or continuous control since turn start for a tap
    // symbol cost. The fixture's next turn starts before the second activation.
    g.state.objects[source as usize].summoning_sick = false;
    reset_priority(&mut g, Player::P1);
    exert(&mut g, source);
    settle(&mut g);
    let rows = g.observe(Player::P0).exertions.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].player, Player::P0);
    assert_eq!(rows[1].player, Player::P1);
    untap_step(&mut g, Player::P1);
    assert!(g.object_view(source).tapped);
    assert_eq!(
        g.observe(Player::P0).exertions.unwrap()[0].player,
        Player::P0
    );
    untap_step(&mut g, Player::P0);
    assert!(g.observe(Player::P0).exertions.is_none());
}

#[test]
fn exert_prohibition_precedes_stun_and_expires_when_already_untapped() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXERT_RULES)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    exert(&mut g, source);
    settle(&mut g);
    g.state.objects[source as usize].counters.stun = 1;
    untap_step(&mut g, Player::P0);
    assert!(g.object_view(source).tapped);
    assert_eq!(
        g.counter_count(source, CounterKind::Stun),
        1,
        "no untap event reaches stun while exert prohibits it"
    );
    untap_step(&mut g, Player::P0);
    assert!(g.object_view(source).tapped);
    assert_eq!(g.counter_count(source, CounterKind::Stun), 0);
    untap_step(&mut g, Player::P0);
    assert!(!g.object_view(source).tapped);
    reset_priority(&mut g, Player::P0);
    exert(&mut g, source);
    settle(&mut g);
    g.untap_objects(&[source], untap_rules::UntapCause::Effect);
    untap_step(&mut g, Player::P0);
    assert!(
        g.observe(Player::P0).exertions.is_none(),
        "skip expires even when no tapped object is waiting"
    );
    g.tap_objects(&[source]);
    untap_step(&mut g, Player::P0);
    assert!(!g.object_view(source).tapped);
}

#[test]
fn source_departure_clears_exertion_and_ability_suppression_does_not() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXERT_RULES)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    exert(&mut g, source);
    settle(&mut g);
    let blank = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    cast(&mut g, blank, Some(Target::Card(source)));
    settle(&mut g);
    assert!(!g.object_view(source).printed_abilities_available());
    assert!(
        g.observe(Player::P1).exertions.is_some(),
        "exert is an external paid obligation"
    );
    g.move_object(blank, Zone::Graveyard);
    let before = g.object_ref(source);
    g.move_object(source, Zone::Exile);
    assert!(g.observe(Player::P0).exertions.is_none());
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), before);
    g.tap_objects(&[source]);
    untap_step(&mut g, Player::P0);
    assert!(
        !g.object_view(source).tapped,
        "new incarnation is not exerted"
    );
}

#[test]
fn exert_payment_and_public_obligation_are_owned_across_raw_index_restore() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXERT_RULES)]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Battlefield);
    act(&mut parent, activation(source));
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    drop(parent);
    pay(&mut raw);
    let choices = indexed.legal_actions();
    let position = choices
        .iter()
        .position(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    indexed
        .apply_index(indexed.actor().unwrap(), indexed.revision(), position)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(
        raw.observe(Player::P0).exertions,
        raw.observe(Player::P1).exertions
    );
    assert!(restored.observe(Player::P0).exertions.is_none());
    restored.restore(&saved).unwrap();
    pay(&mut restored);
    assert_eq!(raw.snapshot(), restored.snapshot());
    settle(&mut raw);
    settle(&mut indexed);
    settle(&mut restored);
    let obligation = raw.snapshot();
    untap_step(&mut raw, Player::P0);
    assert!(raw.observe(Player::P0).exertions.is_none());
    assert!(indexed.observe(Player::P0).exertions.is_some());
    restored.restore(&obligation).unwrap();
    untap_step(&mut restored, Player::P0);
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn exert_source_must_survive_its_mana_funding_even_without_a_tap_symbol() {
    const NO_TAP: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[
            ActivatedAbility {
                cost: ActivationCost {
                    generic: 1,
                    mana_component: true,
                    ..ActivationCost::ZERO
                },
                ..EXERT_RULES.activated[0]
            },
            ActivatedAbility {
                exhaust: false,
                guard: ActivationGuard::Always,
                origin: ActivationOrigin::Battlefield,
                cycling: false,
                cost: ActivationCost {
                    sacrifice: true,
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
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &NO_TAP)]);
    let mut g = scenario(Player::P0);
    g.state.players[0].mana = [0; 6];
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert!(
        !g.legal_actions().contains(&activation(source)),
        "sacrificing source to fund its own exert cost cannot work"
    );
    reject(&mut g, activation(source));
    let other = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert!(
        g.legal_actions().contains(&activation(source)),
        "another mana source can fund exert"
    );
    act(&mut g, activation(source));
    choose(
        &mut g,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == other),
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(other).zone, Zone::Graveyard);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert!(
        !g.object_view(source).tapped,
        "exert itself does not tap a permanent"
    );
    assert_eq!(
        g.observe(Player::P0).exertions.unwrap()[0].object,
        g.object_ref(source)
    );
}

#[test]
fn exert_header_preserves_mana_ability_classification_and_cost_commitment() {
    const EXERT_MANA: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ExertCost,
                Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 0, 1, 0])),
            ],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXERT_MANA)]);
    let mut g = scenario(Player::P0);
    g.state.players[0].mana = [0; 6];
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    choose(
        &mut g,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == source),
    );
    assert!(
        g.state.stack.is_empty(),
        "Exert cost header does not put a mana ability on the stack"
    );
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 0, 1, 0]);
    assert!(g.object_view(source).tapped);
    assert_eq!(
        g.observe(Player::P0).exertions.unwrap()[0].object,
        g.object_ref(source)
    );
}
