//! Outcome-based resolution contracts, derived before ordinary card admissions.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ANY_CREATURE_VEHICLE: &[CostMatch] = &[CostMatch::Kinds(1), CostMatch::Subtype("Vehicle")];
const RUMMAGE: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        exhaust: true,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 0, 0, 0, 1, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DiscardUpToDraw { maximum: 2 },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FALLBACK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::OpponentSacrificeIfAbleElseDiscard {
            any: ANY_CREATURE_VEHICLE,
        }],
    ),
    ..CardRules::EMPTY
};
const REWARD: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 2,
        }),
        program: &[Instruction::DestroyTargetsThenSourceCountersIfOwnCreature {
            kind: CounterKind::PlusOne,
            amount: 2,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const GRAVE_COUNT: AmountValue = AmountValue::Stat(StatValue::Count(CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
}));
const FEAST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::CreatureOrVehicle {
            relation: -1,
        })),
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
            Instruction::DestroyTargetsAtMostManaValue(GRAVE_COUNT),
        ],
    ),
    ..CardRules::EMPTY
};
const EXILE_REPLACEMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileAllCardsAndTokens),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const SACRIFICE_PROHIBITION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentsCannotBeSacrificed],
    }],
    ..CardRules::EMPTY
};

fn reach_choice(g: &mut Game) {
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("missing effect choice");
}
fn start_rummage(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    reach_choice(g);
}
fn effect_pick(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::Sacrifice { object });
}
fn activate_reward(g: &mut Game, source: CardId, targets: &[CardId]) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
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

#[test]
fn capped_rummage_zero_one_two_are_optional_and_draw_exact_counts() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &RUMMAGE)]);
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let cards: Vec<_> = (0..3)
                .map(|_| put(&mut g, p, "Mountain", Zone::Hand))
                .collect();
            start_rummage(&mut g, source);
            assert!(g.state.players[p.index()].mana[4] < 8, "red cost was paid");
            for &id in cards.iter().take(count) {
                effect_pick(&mut g, id);
            }
            if count < 2 {
                act(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                count as u64
            );
            assert_eq!(
                cards
                    .iter()
                    .filter(|&&id| g.object_view(id).zone == Zone::Graveyard)
                    .count(),
                count
            );
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card == source)),
                "exhaust remains used"
            );
        }
    }
}

#[test]
fn rummage_pending_choice_branches_raw_index_and_restore_without_private_leaks() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &RUMMAGE)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let hidden = put(&mut g, p, "Mountain", Zone::Hand);
    start_rummage(&mut g, source);
    assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == hidden));
    let before = g.snapshot();
    let action = Action::Sacrifice {
        object: g.object_ref(hidden),
    };
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let rev = indexed.revision();
    act(&mut raw, action.clone());
    indexed.apply_index(p, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    reject(&mut raw, action);
    drop(g);
    indexed.restore(&before).unwrap();
    act(&mut indexed, Action::DeclineEffectPayment);
    settle(&mut indexed);
    assert_eq!(indexed.object_view(hidden).zone, Zone::Hand);
    assert_eq!(indexed.state.players[p.index()].cards_drawn_this_turn, 0);
    act(&mut raw, Action::DeclineEffectPayment);
    settle(&mut raw);
    assert_eq!(raw.object_view(hidden).zone, Zone::Graveyard);
    assert_eq!(raw.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn rummage_draws_actual_discards_after_prevention_and_redirection() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &RUMMAGE),
        (Card::from_name("Short Bow").unwrap(), &EXILE_REPLACEMENT),
    ]);
    for prevented in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let card = put(&mut g, Player::P0, "Mountain", Zone::Hand);
        if prevented {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(g.object_ref(card))));
        } else {
            put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        }
        start_rummage(&mut g, source);
        effect_pick(&mut g, card);
        act(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            u64::from(!prevented)
        );
        assert_eq!(
            g.object_view(card).zone,
            if prevented { Zone::Hand } else { Zone::Exile }
        );
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
    }
}

#[test]
fn sacrifice_fallback_cannot_discard_while_creature_or_uncrewed_vehicle_is_available() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &FALLBACK)]);
    for name in ["Grizzly Bears", "Broadcast Rambler"] {
        let mut g = scenario(Player::P0);
        let victim = put(&mut g, Player::P1, name, Zone::Battlefield);
        let hidden = put(&mut g, Player::P1, "Mountain", Zone::Hand);
        let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
        cast(&mut g, spell, None);
        reach_choice(&mut g);
        let hidden_ref = g.object_ref(hidden);
        reject(&mut g, Action::Sacrifice { object: hidden_ref });
        reject(&mut g, Action::DeclineEffectPayment);
        effect_pick(&mut g, victim);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.object_view(hidden).zone, Zone::Hand);
    }
}

#[test]
fn sacrifice_fallback_discards_when_matching_permanents_absent_or_prohibited() {
    let _scope = fixture_rules::activate(&[
        (Card::Divination, &FALLBACK),
        (
            Card::from_name("Short Bow").unwrap(),
            &SACRIFICE_PROHIBITION,
        ),
    ]);
    for p in [Player::P0, Player::P1] {
        for prohibit in [false, true] {
            let mut g = scenario(p);
            let creature =
                prohibit.then(|| put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield));
            if prohibit {
                put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            }
            let hidden = put(&mut g, p.other(), "Mountain", Zone::Hand);
            let spell = put(&mut g, p, "Divination", Zone::Hand);
            cast(&mut g, spell, None);
            reach_choice(&mut g);
            effect_pick(&mut g, hidden);
            settle(&mut g);
            assert_eq!(g.object_view(hidden).zone, Zone::Graveyard);
            if let Some(id) = creature {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn post_mill_threshold_uses_current_graveyard_and_does_not_restrict_announced_target() {
    let _scope = fixture_rules::activate(&[(Card::Divination, &FEAST)]);
    for existing in [1, 2] {
        let mut g = scenario(Player::P0);
        for _ in 0..existing {
            put(&mut g, Player::P0, "Mountain", Zone::Graveyard);
        }
        let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        settle(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if existing == 2 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(g.state.players[0].library.len(), 6);
    }
}

#[test]
fn post_mill_threshold_uses_replaced_mill_and_stale_target_fizzles_before_mill() {
    let _scope = fixture_rules::activate(&[
        (Card::Divination, &FEAST),
        (Card::from_name("Short Bow").unwrap(), &EXILE_REPLACEMENT),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    settle(&mut g);
    assert_eq!(
        g.object_view(target).zone,
        Zone::Battlefield,
        "exiled mill does not enlarge graveyard"
    );
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    let library = g.state.players[0].library.clone();
    settle(&mut g);
    assert_eq!(g.state.players[0].library, library);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
}

#[test]
fn destruction_reward_happens_once_only_for_actual_own_creature_destruction() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &REWARD)]);
    for mode in 0..5 {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let own = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        if mode == 1 {
            g.grant_keyword_effects(&[own], g.object_ref(source), Keywords::INDESTRUCTIBLE);
        }
        if mode == 2 {
            g.add_counter(own, CounterKind::Shield, 1);
        }
        let targets = if mode == 3 {
            vec![enemy]
        } else {
            vec![own, enemy]
        };
        activate_reward(&mut g, source, &targets);
        if mode == 4 {
            g.move_object(own, Zone::Hand);
            g.move_object(own, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            if mode == 0 { 2 } else { 0 }
        );
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        if mode == 2 {
            assert_eq!(g.counter_count(own, CounterKind::Shield), 0);
        }
    }
}

#[test]
fn destruction_reward_counts_exile_replacement_but_never_rebinds_source() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &REWARD),
        (Card::from_name("Short Bow").unwrap(), &EXILE_REPLACEMENT),
    ]);
    for reenter in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let own = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        activate_reward(&mut g, source, &[own]);
        if reenter {
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Exile);
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            if reenter { 0 } else { 2 }
        );
    }
}
