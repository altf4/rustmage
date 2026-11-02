//! Nonmana sacrifice-X declarations and exact random graveyard play permissions.
mod support;
use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const SAC: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::SacrificeManaValue { value: None },
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 1,
        nonland: false,
        other: true,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
};
const GY_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const SIDISI: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            additional: Some(SAC),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::ManaValueExactly {
            base: &GY_CREATURE,
            amount: AmountValue::Sum(&[AmountValue::ChosenX, AmountValue::Fixed(1)]),
            value: None,
        })),
        program: &[
            Instruction::ActivationNonmanaX,
            Instruction::ReturnToBattlefield,
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const GRAVE_COUNT: CountFilter = CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
};
const TERSA: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Expression {
                value: StatValue::Count(GRAVE_COUNT),
                minimum: Some(7),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileRandomGraveyardPlayThisTurn],
    }],
    ..CardRules::EMPTY
};
const RANDOM: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileRandomGraveyardPlayThisTurn],
    ),
    ..CardRules::EMPTY
};
const LIFE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};
fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn choose_target(g: &mut Game, card: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn row(g: &mut Game, clause: CastCostClause) {
    let i = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == clause && !r.paid)
        .unwrap();
    act(g, Action::ChooseCastCost { clause: i as u16 });
}
fn finish_sacrifice(g: &mut Game, material: CardId) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        if actions.contains(&Action::BeginOrderedCosts) {
            act(g, Action::BeginOrderedCosts);
        } else if matches!(g.state.pending, Pending::CastCosts { .. }) {
            let clause = g
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap()
                .clauses
                .iter()
                .find(|r| !r.paid)
                .unwrap()
                .clause;
            row(g, clause);
        } else if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
            let object = g.object_ref(material);
            act(g, Action::ChooseAdditionalCost { object });
        } else {
            pay(g);
        }
    }
    panic!("sacrifice completion ceiling");
}
fn sidisi(g: &mut Game, p: Player) -> (CardId, CardId, CardId) {
    let source = put(g, p, "Grizzly Bears", Zone::Battlefield);
    let material = put(g, p, "Llanowar Elves", Zone::Battlefield);
    let target = put(g, p, "Bear Cub", Zone::Graveyard);
    (source, material, target)
}
fn attack(g: &mut Game, source: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: source });
    act(g, Action::FinishAttackers);
}
fn grave(g: &mut Game, p: Player, name: &str, amount: usize) -> Vec<CardId> {
    (0..amount)
        .map(|_| put(g, p, name, Zone::Graveyard))
        .collect()
}
fn exiled(g: &Game, cards: &[CardId]) -> Vec<CardId> {
    cards
        .iter()
        .copied()
        .filter(|&id| g.state.objects[id as usize].zone == Zone::Exile)
        .collect()
}
fn wait_trigger(g: &mut Game) {
    for _ in 0..10 {
        if !g.state.stack.is_empty() {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("trigger wait ceiling");
}

#[test]
fn nonmana_x_is_discrete_can_omit_zero_and_pays_no_generic_x_for_both_seats() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &SIDISI)]);
    for p in [Player::P0, Player::P1] {
        for additional_first in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let material = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let target = put(&mut g, p, "Vampire Nighthawk", Zone::Graveyard);
            g.state.players[p.index()].mana = [0; 6];
            act(&mut g, activate(source));
            let domain = g.x_choice().unwrap();
            assert!(domain.contains(2));
            assert!(!domain.contains(0));
            assert!(!domain.contains(1));
            assert!(!domain.contains(3));
            reject(&mut g, Action::ChooseX { value: 0 });
            act(&mut g, Action::ChooseX { value: 2 });
            choose_target(&mut g, target);
            assert_eq!(g.state.objects[material as usize].zone, Zone::Battlefield);
            assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
            if additional_first {
                act(&mut g, Action::BeginOrderedCosts);
                row(&mut g, CastCostClause::Additional);
                let object = g.object_ref(material);
                act(&mut g, Action::ChooseAdditionalCost { object });
                assert_eq!(g.state.objects[material as usize].zone, Zone::Graveyard);
                assert!(!g.state.objects[source as usize].tapped);
            }
            finish_sacrifice(&mut g, material);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.state.objects[source as usize].tapped);
            assert_eq!(g.state.objects[material as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.stack.last().unwrap().x, Some(2));
            settle(&mut g);
            assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
        }
    }
}

#[test]
fn nonmana_x_domain_keeps_holes_and_requires_a_matching_target_and_cost() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &SIDISI)]);
    let mut g = scenario(Player::P0);
    let (source, _, _) = sidisi(&mut g, Player::P0);
    put(&mut g, Player::P0, "Vampire Nighthawk", Zone::Battlefield);
    put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    act(&mut g, activate(source));
    let d = g.x_choice().unwrap();
    assert!(d.contains(1));
    assert!(d.contains(3));
    assert!(!d.contains(0));
    assert!(!d.contains(2));
    reject(&mut g, Action::ChooseX { value: 2 });
    let before = g.snapshot();
    for _ in 0..5 {
        let _ = g.legal_actions();
        let _ = g.x_choice();
        let _ = g.observe(Player::P1);
    }
    assert_eq!(g.snapshot(), before);
}

#[test]
fn nonmana_x_zero_can_sacrifice_a_token_and_return_a_one_mana_creature() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &SIDISI)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let token = put(
        &mut g,
        Player::P0,
        "@token/b73d837afe1978c0c8507afb",
        Zone::Battlefield,
    );
    let target = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    act(&mut g, activate(source));
    act(&mut g, Action::ChooseX { value: 0 });
    choose_target(&mut g, target);
    finish_sacrifice(&mut g, token);
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
}

#[test]
fn nonmana_x_does_not_sacrifice_before_targets_or_target_a_future_graveyard_card() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &SIDISI)]);
    let mut g = scenario(Player::P0);
    let (source, material, target) = sidisi(&mut g, Player::P0);
    let wrong = put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let opponent = put(&mut g, Player::P1, "Bear Cub", Zone::Graveyard);
    act(&mut g, activate(source));
    act(&mut g, Action::ChooseX { value: 1 });
    for card in [material, wrong, opponent] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    let object = g.object_ref(material);
    reject(&mut g, Action::ChooseAdditionalCost { object });
    choose_target(&mut g, target);
    assert_eq!(g.state.objects[material as usize].zone, Zone::Battlefield);
    finish_sacrifice(&mut g, material);
    settle(&mut g);
}

#[test]
fn nonmana_x_cost_checks_exact_mana_value_other_source_control_and_incarnation() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &SIDISI)]);
    let mut g = scenario(Player::P0);
    let (source, material, target) = sidisi(&mut g, Player::P0);
    let opponent = put(&mut g, Player::P1, "Llanowar Elves", Zone::Battlefield);
    act(&mut g, activate(source));
    act(&mut g, Action::ChooseX { value: 1 });
    choose_target(&mut g, target);
    act(&mut g, Action::BeginOrderedCosts);
    row(&mut g, CastCostClause::Mana);
    pay(&mut g);
    row(&mut g, CastCostClause::Additional);
    for card in [source, opponent] {
        let object = g.object_ref(card);
        reject(&mut g, Action::ChooseAdditionalCost { object });
    }
    let old = g.object_ref(material);
    g.move_object(material, Zone::Hand);
    g.move_object(material, Zone::Battlefield);
    reject(&mut g, Action::ChooseAdditionalCost { object: old });
    let object = g.object_ref(material);
    act(&mut g, Action::ChooseAdditionalCost { object });
    settle(&mut g);
}

#[test]
fn nonmana_x_preserves_sorcery_sickness_tap_control_and_sacrifice_prohibition() {
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledPermanentsCannotBeSacrificed],
    }];
    const BLOCK: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: GROUPS,
        extras: &CardExtras {
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SIDISI),
        (Card::from_name("Ghost Vacuum").unwrap(), &BLOCK),
    ]);
    let mut g = scenario(Player::P0);
    let (source, _, _) = sidisi(&mut g, Player::P0);
    for case in 0..4 {
        let mut bad = g.clone();
        match case {
            0 => bad.state.objects[source as usize].tapped = true,
            1 => bad.state.objects[source as usize].summoning_sick = true,
            2 => bad.state.active = Player::P1,
            _ => bad.state.objects[source as usize].controller = Player::P1,
        };
        reject(&mut bad, activate(source));
    }
    let block = put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    reject(&mut g, activate(source));
    g.move_object(block, Zone::Graveyard);
    assert!(g.legal_actions().contains(&activate(source)));
}

#[test]
fn nonmana_x_owned_declaration_target_and_cost_restore_after_parent_disposal() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &SIDISI)]);
    let mut parent = scenario(Player::P0);
    let (source, first, target) = sidisi(&mut parent, Player::P0);
    let second = put(&mut parent, Player::P0, "Llanowar Elves", Zone::Battlefield);
    act(&mut parent, activate(source));
    assert!(parent.observe(Player::P0).x_choice.is_some());
    assert!(parent.observe(Player::P1).x_choice.is_none());
    let action = Action::ChooseX { value: 1 };
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
    assert_eq!(
        raw.apply(Player::P1, action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    for g in [&mut raw, &mut indexed] {
        choose_target(g, target);
        finish_sacrifice(g, first);
        settle(g);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    act(&mut sibling, action.clone());
    choose_target(&mut sibling, target);
    finish_sacrifice(&mut sibling, second);
    settle(&mut sibling);
    assert_ne!(raw.snapshot(), sibling.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    choose_target(&mut indexed, target);
    finish_sacrifice(&mut indexed, first);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn nonmana_x_ability_uses_its_captured_value_after_source_departure_and_target_reentry_fizzles() {
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &SIDISI)]);
    for stale in [false, true] {
        let mut g = scenario(Player::P0);
        let (source, material, target) = sidisi(&mut g, Player::P0);
        act(&mut g, activate(source));
        act(&mut g, Action::ChooseX { value: 1 });
        choose_target(&mut g, target);
        finish_sacrifice(&mut g, material);
        g.move_object(source, Zone::Exile);
        if stale {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Graveyard);
        }
        settle(&mut g);
        assert_eq!(
            g.state.objects[target as usize].zone,
            if stale {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}

#[test]
fn random_graveyard_attack_checks_threshold_at_trigger_and_resolution_without_reroll() {
    let _s = fixture_rules::activate(&[(Card::GoblinPiker, &TERSA)]);
    for p in [Player::P0, Player::P1] {
        for amount in [6, 7] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
            let cards = grave(&mut g, p, "Forest", amount);
            let rng = g.state.rng.0;
            attack(&mut g, source);
            settle(&mut g);
            assert_eq!(exiled(&g, &cards).len(), usize::from(amount == 7));
            if amount == 6 {
                assert_eq!(g.state.rng.0, rng);
            }
        }
    }
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let cards = grave(&mut g, Player::P0, "Forest", 7);
    attack(&mut g, source);
    wait_trigger(&mut g);
    g.move_object(cards[0], Zone::Hand);
    let rng = g.state.rng.0;
    settle(&mut g);
    assert!(exiled(&g, &cards).is_empty());
    assert_eq!(g.state.rng.0, rng);
    assert!(g.state.exile_permissions.is_empty());
}

#[test]
fn random_graveyard_selection_is_owned_deterministic_and_queries_never_advance_rng() {
    let _s = fixture_rules::activate(&[(Card::GoblinPiker, &TERSA)]);
    let mut parent = scenario(Player::P0);
    let source = put(&mut parent, Player::P0, "Goblin Piker", Zone::Battlefield);
    let cards = grave(&mut parent, Player::P0, "Forest", 7);
    attack(&mut parent, source);
    wait_trigger(&mut parent);
    let saved = parent.snapshot();
    let revision = parent.revision();
    let actor = parent.actor().unwrap();
    let action = Action::Pass;
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    for _ in 0..7 {
        let _ = parent.legal_actions();
        let _ = parent.observe(Player::P1);
        let _ = parent.snapshot();
    }
    assert_eq!(parent.snapshot(), saved);
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, action.clone());
    indexed.apply_index(actor, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(exiled(&raw, &cards).len(), 1);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn random_graveyard_empty_and_singleton_avoid_rng_and_multiple_seeds_reach_each_card() {
    let _s = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RANDOM)]);
    for amount in [0, 1] {
        let mut g = scenario(Player::P0);
        let cards = grave(&mut g, Player::P0, "Forest", amount);
        let source = put(&mut g, Player::P0, "Opt", Zone::Hand);
        let rng = g.state.rng.0;
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(exiled(&g, &cards).len(), amount);
        assert_eq!(g.state.rng.0, rng);
    }
    let mut outcomes = std::collections::BTreeSet::new();
    for seed in 0..128 {
        let mut g = scenario(Player::P0);
        let cards = grave(&mut g, Player::P0, "Forest", 7);
        g.state.rng.0 = seed;
        let source = put(&mut g, Player::P0, "Opt", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        let made = exiled(&g, &cards);
        assert_eq!(made.len(), 1);
        outcomes.insert(made[0]);
    }
    assert_eq!(outcomes.len(), 7);
}

#[test]
fn random_graveyard_permission_keeps_controller_after_source_departure_and_control_change() {
    let _s = fixture_rules::activate(&[(Card::GoblinPiker, &TERSA), (Card::Divination, &LIFE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let cards = grave(&mut g, p, "Divination", 7);
        attack(&mut g, source);
        wait_trigger(&mut g);
        g.state.objects[source as usize].controller = p.other();
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        let made = exiled(&g, &cards);
        assert_eq!(made.len(), 1);
        let permission = g
            .state
            .exile_permissions
            .iter()
            .find(|r| r.object == g.object_ref(made[0]))
            .unwrap();
        assert_eq!(permission.player, p);
        assert!(!permission.without_paying_mana_cost);
        assert!(!permission.spend_any_mana);
        g.state.step = Step::Main2;
        g.state.priority = p;
        g.state.passes = 0;
        act(&mut g, Action::AnnounceExile { card: made[0] });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.objects[made[0] as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn random_graveyard_land_uses_ordinary_timing_allowance_expiry_and_exact_incarnation() {
    let _s = fixture_rules::activate(&[(Card::GoblinPiker, &TERSA)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let cards = grave(&mut g, Player::P0, "Forest", 7);
    attack(&mut g, source);
    settle(&mut g);
    let card = exiled(&g, &cards)[0];
    reject(&mut g, Action::PlayLand { card });
    g.state.step = Step::Main2;
    g.state.priority = Player::P0;
    g.state.passes = 0;
    let mut used = g.clone();
    used.state.land_played = true;
    reject(&mut used, Action::PlayLand { card });
    let mut expired = g.clone();
    expired.state.turn += 1;
    reject(&mut expired, Action::PlayLand { card });
    let mut stale = g.clone();
    stale.move_object(card, Zone::Graveyard);
    stale.move_object(card, Zone::Exile);
    reject(&mut stale, Action::PlayLand { card });
    act(&mut g, Action::PlayLand { card });
    assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
    assert!(g.state.land_played);
}

#[test]
fn random_graveyard_normal_spell_cost_allows_nonzero_printed_x_and_rejects_unfunded_cast() {
    const X: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::ChosenX,
            )],
        ),
        extras: &CardExtras {
            x_symbols: 1,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::GoblinPiker, &TERSA),
        (Card::from_name("Spectral Denial").unwrap(), &X),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let cards = grave(&mut g, Player::P0, "Spectral Denial", 7);
    attack(&mut g, source);
    settle(&mut g);
    let card = exiled(&g, &cards)[0];
    g.state.players[0].mana = [0; 6];
    reject(&mut g, Action::AnnounceExile { card });
    g.state.players[0].mana = [0, 1, 0, 0, 0, 4];
    act(&mut g, Action::AnnounceExile { card });
    act(&mut g, Action::ChooseX { value: 4 });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn random_graveyard_prevented_exile_grants_no_permission_or_unrelated_reentry_right() {
    struct Clear;
    impl Drop for Clear {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
        }
    }
    let _s = fixture_rules::activate(&[(Card::GoblinPiker, &TERSA)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let cards = grave(&mut g, Player::P0, "Forest", 7);
    attack(&mut g, source);
    wait_trigger(&mut g);
    let mut probe = g.clone();
    settle(&mut probe);
    let selected = exiled(&probe, &cards)[0];
    let _clear = Clear;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(Some(g.object_ref(selected))));
    settle(&mut g);
    assert!(exiled(&g, &cards).is_empty());
    assert!(g.state.exile_permissions.is_empty());
    assert_eq!(g.state.rng.0, probe.state.rng.0);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|r| r.set(None));
    g.move_object(selected, Zone::Exile);
    assert!(!g.can_play_exiled(Player::P0, selected));
}

#[test]
fn nonmana_x_cost_uses_copiable_face_down_and_off_stack_printed_x_mana_values() {
    const HYDRA: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SIDISI),
        (Card::from_name("Guiding Hydra").unwrap(), &HYDRA),
    ]);
    for shape in 0..3 {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let (material, x) = if shape == 0 {
            let id = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            g.apply_temporary_copy(
                g.object_ref(id),
                CopyRecipe::new(Card::LlanowarElves),
                CopyDeadline::EndOfTurn,
            );
            (id, 1)
        } else if shape == 1 {
            let id = put(&mut g, Player::P0, "Shivan Dragon", Zone::Library);
            let m = g.manifest_entry(g.object_ref(id), Player::P0).unwrap();
            g.stage_entries(vec![m], None);
            assert!(!g.advance_entries());
            (id, 0)
        } else {
            (
                put(&mut g, Player::P0, "Guiding Hydra", Zone::Battlefield),
                1,
            )
        };
        let target = put(
            &mut g,
            Player::P0,
            if x == 0 { "Savannah Lions" } else { "Bear Cub" },
            Zone::Graveyard,
        );
        assert_eq!(g.object_mana_value(material), u64::from(x));
        act(&mut g, activate(source));
        act(&mut g, Action::ChooseX { value: x });
        choose_target(&mut g, target);
        finish_sacrifice(&mut g, material);
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    }
}

#[test]
fn nonmana_x_acquired_ability_keeps_the_recipient_as_exact_cost_source() {
    const GRANT: CardRules = CardRules {
        extras: &CardExtras {
            self_granted_activation_templates: &[0],
            ..CardExtras::EMPTY
        },
        ..SIDISI
    };
    let _s = fixture_rules::activate(&[(Card::LlanowarElves, &GRANT)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let material = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let target = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let timestamp = g.next_role_timestamp();
    let object = g.object_ref(source);
    g.grant_recipient_ability(
        object,
        Card::LlanowarElves,
        SelfGrantedAbilityKind::Activated { ability: 0 },
        timestamp,
        None,
    );
    act(
        &mut g,
        Action::AnnounceSelfGrantedAbility {
            source: object,
            grant_timestamp: timestamp,
            counter_payment: Counters::default(),
        },
    );
    act(&mut g, Action::ChooseX { value: 1 });
    choose_target(&mut g, target);
    finish_sacrifice(&mut g, material);
    settle(&mut g);
    assert!(g.state.objects[source as usize].tapped);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    assert_eq!(
        g.object_view(source).card,
        Card::from_name("Bear Cub").unwrap()
    );
}

#[test]
fn nonmana_x_full_proposal_can_gain_its_only_sacrifice_witness_from_a_taxed_mana_child() {
    const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(1)],
    }];
    const TAX: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: TAX_GROUPS,
        extras: &CardExtras {
            static_ability_groups: TAX_GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const PRODUCER: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            sorcery: false,
            ..SIDISI.activated[0]
        }],
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SIDISI),
        (Card::from_name("Ghost Vacuum").unwrap(), &PRODUCER),
        (Card::GoblinPiker, &TAX),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let producer = put(&mut g, Player::P0, "Ghost Vacuum", Zone::Battlefield);
    let tax = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(tax),
        EntryChoiceValue::CardName(CardName::from_name("Grizzly Bears").unwrap()),
    );
    let target = put(&mut g, Player::P0, "Vampire Nighthawk", Zone::Graveyard);
    g.state.players[0].mana = [0; 6];
    reject(&mut g, activate(source));
    let material = put(&mut g, Player::P0, "Bear Cub", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(producer),
        exiled: g.object_ref(material),
        return_zone: Zone::Battlefield,
        ability_origin: None,
    });
    let mut no_window = g.clone();
    no_window.move_object(tax, Zone::Graveyard);
    reject(&mut no_window, activate(source));
    // The same mana ability remains legal before the nonmana announcement.
    act(
        &mut no_window,
        Action::ActivateMana {
            card: producer,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    act(&mut no_window, activate(source));
    act(&mut no_window, Action::ChooseX { value: 2 });
    choose_target(&mut no_window, target);
    finish_sacrifice(&mut no_window, material);
    settle(&mut no_window);
    assert_eq!(
        no_window.state.objects[target as usize].zone,
        Zone::Battlefield
    );
    act(&mut g, activate(source));
    assert!(g.x_choice().unwrap().contains(2));
    act(&mut g, Action::ChooseX { value: 2 });
    choose_target(&mut g, target);
    assert_eq!(g.state.objects[material as usize].zone, Zone::Exile);
    act(
        &mut g,
        Action::ActivateMana {
            card: producer,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(g.state.objects[material as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[producer as usize].zone, Zone::Graveyard);
    finish_sacrifice(&mut g, material);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
}

#[test]
fn nonmana_x_copy_retargets_using_the_captured_value_without_paying_a_second_sacrifice() {
    const COPY: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::CopyTargetAbility { group: 0 }],
        ),
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SIDISI),
        (Card::from_name("Opt").unwrap(), &COPY),
    ]);
    let mut g = scenario(Player::P0);
    let (source, material, first) = sidisi(&mut g, Player::P0);
    let second = put(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let wrong = put(&mut g, Player::P0, "Vampire Nighthawk", Zone::Graveyard);
    act(&mut g, activate(source));
    act(&mut g, Action::ChooseX { value: 1 });
    choose_target(&mut g, first);
    finish_sacrifice(&mut g, material);
    let id = g.state.stack.last().unwrap().id;
    g.move_object(source, Zone::Exile);
    let copy = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, copy, Some(Target::Stack(id)));
    for _ in 0..8 {
        if matches!(g.state.pending, Pending::Retarget { .. }) {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert_eq!(
        g.observe(Player::P0).retarget.as_ref().unwrap().item.x,
        Some(1)
    );
    reject(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(wrong),
        },
    );
    act(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(second),
        },
    );
    act(&mut g, Action::FinishStackTargets);
    settle(&mut g);
    assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[second as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[material as usize].zone, Zone::Graveyard);
}

#[test]
fn random_graveyard_cleanup_removes_permission_and_preserves_unrelated_hidden_cards() {
    let _s = fixture_rules::activate(&[(Card::GoblinPiker, &TERSA)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let hidden = put(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    let cards = grave(&mut g, Player::P0, "Forest", 7);
    attack(&mut g, source);
    settle(&mut g);
    let chosen = exiled(&g, &cards)[0];
    assert!(!g.observe(Player::P1).objects.iter().any(|o| o.id == hidden));
    assert!(g.observe(Player::P1).objects.iter().any(|o| o.id == chosen));
    assert!(g.can_play_exiled(Player::P0, chosen));
    g.finish_cleanup();
    assert!(!g.can_play_exiled(Player::P0, chosen));
    assert!(g.state.exile_permissions.is_empty());
}
