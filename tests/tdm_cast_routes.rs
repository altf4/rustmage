//! Shared casting rules fixtures. No set definition or admission is changed.
use rustmage as _;
mod support;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ANY: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const GRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 4 | 8,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::GrantGraveyardCast(
            GraveyardCastKind::Harmonize,
        )],
    }],
    ..CardRules::EMPTY
};
const GRANT_ANY: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::GrantGraveyardCast(
            GraveyardCastKind::Harmonize,
        )],
    }],
    ..GRANT
};
const DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};
const MIXED: CardRules = CardRules {
    extras: &CardExtras {
        has_convoke: true,
        has_improvise: true,
        ..CardExtras::EMPTY
    },
    ..DRAW
};
const COLORLESS: CardRules = CardRules {
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 0, 0, 1],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..DRAW
};
const BOUNCE_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::ReturnSpellToHand],
    ),
    ..CardRules::EMPTY
};
const EXILE_BLOCK_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
}];
const EXILE_BLOCK: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: EXILE_BLOCK_GROUP,
    extras: &CardExtras {
        static_ability_groups: EXILE_BLOCK_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const X_LIFE: CardRules = CardRules {
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
const X_BARGAIN: CardRules = CardRules {
    extras: &CardExtras {
        x_symbols: 1,
        bargain_reduction: Some(1),
        ..CardExtras::EMPTY
    },
    ..X_LIFE
};
const X_HARMONIZE: CardRules = CardRules {
    extras: &CardExtras {
        x_symbols: 1,
        harmonize_x_symbols: 1,
        harmonize_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..X_LIFE
};
const DELVE_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::SpellDelve(CastFilter::Any)],
}];
const DELVE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: DELVE_GROUPS,
    extras: &CardExtras {
        static_ability_groups: DELVE_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const fn mana_ability(cost: ActivationCost) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 1, 0, 0, 0, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }
}
const SAC_MANA: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[mana_ability(ActivationCost {
        tap: true,
        sacrifice: true,
        ..ActivationCost::ZERO
    })],
    ..CardRules::EMPTY
};
const SAC_DELVEPROVIDER: CardRules = CardRules {
    activated: SAC_MANA.activated,
    ..DELVE
};
const GRAVE_MANA: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[mana_ability(ActivationCost {
        generic: 1,
        mana_component: true,
        tap: true,
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::ExileGraveyard,
            filter: ANY,
        }),
        ..ActivationCost::ZERO
    })],
    ..CardRules::EMPTY
};
const GRAVE_ADDITIONAL: CardRules = CardRules {
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::ExileGraveyardCards(1),
            filter: ANY,
        }),
        ..CardExtras::EMPTY
    },
    ..DRAW
};
const SAC_ADDITIONAL: CardRules = CardRules {
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: ANY,
        }),
        ..CardExtras::EMPTY
    },
    ..DRAW
};
const TOP_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::SpellDelve(CastFilter::Any),
        StaticAbilityFragment::CastCreatureFromLibraryTop,
    ],
}];
const TOP_DELVE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: TOP_GROUPS,
    extras: &CardExtras {
        static_ability_groups: TOP_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentNoncreatureSpellTax(2)],
}];
const TAX: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: TAX_GROUPS,
    extras: &CardExtras {
        static_ability_groups: TAX_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const REDUCTION_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(
        0,
        SpellReduction {
            source_only: false,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(1),
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        },
    )],
}];
const REDUCER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: REDUCTION_GROUPS,
    extras: &CardExtras {
        static_ability_groups: REDUCTION_GROUPS,
        cost_reductions: &[SpellReduction {
            source_only: false,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(1),
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn granted(card: CardId) -> Action {
    Action::AnnounceGrantedHarmonize {
        card,
        face: CastFace::Normal,
    }
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
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
    panic!("casting scenario resolution ceiling");
}
fn grant(g: &mut Game, p: Player, target: CardId) -> CardId {
    let source = put(g, p, "Grizzly Bears", Zone::Hand);
    cast(g, source, None);
    for _ in 0..24 {
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        }) {
            act(
                g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            break;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    drain(g);
    assert!(
        g.state
            .graveyard_cast_permissions
            .as_ref()
            .unwrap()
            .iter()
            .any(|row| row.object == g.object_ref(target)
                && row.kind == GraveyardCastKind::Harmonize)
    );
    source
}
fn clause(g: &mut Game, wanted: CastCostClause) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == wanted)
        .unwrap();
    act(
        g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}
fn start_delve(g: &mut Game) {
    act(g, Action::BeginDelve);
    clause(g, CastCostClause::Mana);
    assert!(matches!(g.state.pending, Pending::Delve { .. }));
}
fn reject_delver(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    reject(g, Action::ChooseDelver { object });
}
fn additional(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseAdditionalCost { object });
}
fn delver(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseDelver { object });
}

#[test]
fn granted_harmonize_uses_printed_colored_cost_exact_card_and_independent_duration() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GRANT)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Quick Study", Zone::Graveyard);
        let other = put(&mut g, p, "Quick Study", Zone::Graveyard);
        let source = grant(&mut g, p, spell);
        g.move_object(source, Zone::Exile);
        let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        assert!(
            !g.legal_actions().contains(&granted(spell)),
            "generic cannot replace U"
        );
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        assert!(
            !g.legal_actions().contains(&granted(other)),
            "grant is exact"
        );
        act(&mut g, granted(spell));
        let helper_ref = g.object_ref(helper);
        act(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(helper_ref),
            },
        );
        pay(&mut g);
        clause(&mut g, CastCostClause::Harmonize(helper_ref));
        assert!(g.state.objects[helper as usize].tapped);
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 3);
        drain(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn harmonize_grant_expires_and_does_not_follow_a_new_graveyard_incarnation() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GRANT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Quick Study", Zone::Graveyard);
    grant(&mut g, p, spell);
    let mut expired = g.clone();
    expired.finish_cleanup();
    expired.state.pending = Pending::Priority;
    expired.state.active = p;
    expired.state.priority = p;
    expired.state.step = Step::Main1;
    assert!(!expired.legal_actions().contains(&granted(spell)));
    g.move_object(spell, Zone::Hand);
    g.move_object(spell, Zone::Graveyard);
    assert!(!g.legal_actions().contains(&granted(spell)));
    reject(&mut g, granted(spell));
}

#[test]
fn granted_and_intrinsic_harmonize_costs_remain_separate_and_countered_spell_is_exiled() {
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &GRANT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Wild Ride", Zone::Graveyard);
    grant(&mut g, p, spell);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceHarmonize { card: spell })
    );
    act(&mut g, granted(spell));
    act(&mut g, Action::ChooseHarmonizer { object: None });
    let target = g
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Battlefield)
        .unwrap() as CardId;
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut g);
    act(&mut g, Action::Pass);
    let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
    cast(&mut g, counter, Some(Target::Spell(spell)));
    drain(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert_eq!(stats(&g, target), (2, 2));
}

#[test]
fn granted_harmonize_preserves_hybrid_and_twobrid_symbols() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &GRANT),
        (named("Abandon Attachments"), &DRAW),
        (named("Defibrillating Current"), &DRAW),
    ]);
    for (name, pool, should_work) in [
        ("Abandon Attachments", [0, 1, 0, 0, 0, 1], true),
        ("Abandon Attachments", [0, 0, 0, 1, 0, 1], true),
        ("Abandon Attachments", [0, 0, 1, 0, 0, 1], false),
        ("Defibrillating Current", [0, 0, 0, 0, 0, 6], true),
        ("Defibrillating Current", [1, 0, 1, 1, 0, 0], true),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let spell = put(&mut g, p, name, Zone::Graveyard);
        let source = grant(&mut g, p, spell);
        g.move_object(source, Zone::Exile); // Keep this table independent of creature discounts.
        g.state.players[p.index()].mana = pool;
        assert_eq!(
            g.legal_actions().contains(&granted(spell)),
            should_work,
            "{name} {pool:?}"
        );
        if !should_work {
            continue;
        }
        act(&mut g, granted(spell));
        if matches!(g.state.pending, Pending::SpellCost { .. }) {
            choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
        }
        act(&mut g, Action::ChooseHarmonizer { object: None });
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn x_in_printed_or_harmonize_cost_is_chosen_paid_and_does_not_change_printed_value_rules() {
    for intrinsic in [false, true] {
        let _scope = fixture_rules::activate(&[
            (Card::GrizzlyBears, &GRANT),
            (
                named("Spectral Denial"),
                if intrinsic { &X_HARMONIZE } else { &X_LIFE },
            ),
        ]);
        let p = Player::P1;
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Spectral Denial", Zone::Graveyard);
        if !intrinsic {
            let source = grant(&mut g, p, spell);
            g.move_object(source, Zone::Exile);
        }
        let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        act(
            &mut g,
            if intrinsic {
                Action::AnnounceHarmonize { card: spell }
            } else {
                granted(spell)
            },
        );
        let expected_max = if intrinsic { 3 } else { 5 };
        assert_eq!(
            g.x_choice().unwrap().intervals.last().unwrap().maximum,
            expected_max
        );
        reject(
            &mut g,
            Action::ChooseX {
                value: expected_max + 1,
            },
        );
        act(
            &mut g,
            Action::ChooseX {
                value: expected_max,
            },
        );
        let reference = g.object_ref(helper);
        act(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(reference),
            },
        );
        pay(&mut g);
        clause(&mut g, CastCostClause::Harmonize(reference));
        assert_eq!(
            g.state.stack.last().unwrap().mana_value(),
            1 + u64::from(expected_max)
        );
        drain(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            20 + i128::from(expected_max)
        );
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn harmonize_keeps_power_wide_until_tax_and_x_costs_are_reduced() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &GRANT),
        (named("Spectral Denial"), &X_BARGAIN),
        (named("Fanatical Firebrand"), &TAX),
    ]);
    for p in [Player::P0, Player::P1] {
        for surplus in [2u64, 1, u64::MAX - u64::from(u32::MAX)] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Spectral Denial", Zone::Graveyard);
            let grantor = grant(&mut g, p, spell);
            g.move_object(grantor, Zone::Exile);
            put(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
            let helper = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let power = u64::from(u32::MAX) + surplus;
            let artifact =
                (power == u64::MAX).then(|| put(&mut g, p, "Short Bow", Zone::Battlefield));
            g.state.objects[helper as usize].growth.power = i128::from(power) - 2;
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            act(&mut g, granted(spell));
            let expected_x = if surplus == 1 { u32::MAX - 1 } else { u32::MAX };
            assert_eq!(
                g.x_choice().unwrap().intervals.last().unwrap().maximum,
                expected_x
            );
            if surplus == 1 {
                reject(&mut g, Action::ChooseX { value: u32::MAX });
            }
            act(&mut g, Action::ChooseX { value: expected_x });
            let sacrifice = artifact.map(|id| g.object_ref(id));
            act(&mut g, Action::ChooseBargain { sacrifice });
            reject(&mut g, Action::ChooseHarmonizer { object: None });
            let object = g.object_ref(helper);
            act(
                &mut g,
                Action::ChooseHarmonizer {
                    object: Some(object),
                },
            );
            let announcement = g.state.announcing.as_ref().unwrap();
            assert_eq!(announcement.harmonize_discount, power);
            assert_eq!(announcement.total_cost.unwrap().generic, 0);
            pay(&mut g);
            if artifact.is_some() {
                clause(&mut g, CastCostClause::Additional);
            }
            clause(&mut g, CastCostClause::Harmonize(object));
            if let Some(artifact) = artifact {
                assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            }
            assert!(g.object_view(helper).tapped);
            assert_eq!(
                g.state.stack.last().unwrap().mana_value(),
                u64::from(expected_x) + 1
            );
            drain(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].life, 20 + i128::from(expected_x));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn delve_pays_only_generic_and_commits_exact_graveyard_cards() {
    let _scope = fixture_rules::activate(&[(named("Bear Cub"), &DELVE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        let first = put(&mut g, p, "Forest", Zone::Graveyard);
        let second = put(&mut g, p, "Forest", Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        g.state.players[p.index()].mana = [0; 6];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        start_delve(&mut g);
        reject_delver(&mut g, enemy);
        reject_delver(&mut g, spell);
        delver(&mut g, first);
        assert_eq!(
            g.object_view(first).zone,
            Zone::Graveyard,
            "selection only reserves"
        );
        reject_delver(&mut g, first);
        assert!(g.observe(p).delve_choice.is_some());
        assert!(g.observe(p.other()).delve_choice.is_none());
        delver(&mut g, second);
        pay(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Exile);
        assert_eq!(g.object_view(second).zone, Zone::Exile);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn delve_respects_locked_taxes_reductions_and_harmonize_stack_origin() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &GRANT),
        (named("Bear Cub"), &DELVE),
        (named("Fanatical Firebrand"), &TAX),
        (named("Short Bow"), &REDUCER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Quick Study", Zone::Graveyard);
    let source = grant(&mut g, p, spell);
    g.move_object(source, Zone::Exile);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    put(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    let fuel: Vec<_> = (0..3)
        .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
        .collect();
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, granted(spell));
    act(&mut g, Action::ChooseHarmonizer { object: None });
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .generic,
        3
    );
    start_delve(&mut g);
    reject_delver(&mut g, spell);
    for id in fuel {
        delver(&mut g, id);
    }
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert_eq!(g.hand_count(p), 2);
}

#[test]
fn sacrificed_mana_producer_can_be_delved_but_a_departed_grant_cannot_supply_delve() {
    let _scope = fixture_rules::activate(&[
        (named("Bear Cub"), &DELVE),
        (named("Fanatical Firebrand"), &SAC_MANA),
        (Card::GrizzlyBears, &SAC_DELVEPROVIDER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let producer = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
    let fuel = put(&mut g, p, "Forest", Zone::Graveyard);
    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    act(&mut g, Action::AnnounceCast { card: spell });
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana { card, .. } if *card == producer),
    );
    assert_eq!(g.object_view(producer).zone, Zone::Graveyard);
    start_delve(&mut g);
    delver(&mut g, producer);
    delver(&mut g, fuel);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(producer).zone, Zone::Exile);
    assert_eq!(g.hand_count(p), 2);

    let mut g = scenario(p);
    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    for _ in 0..3 {
        put(&mut g, p, "Forest", Zone::Graveyard);
    }
    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell }),
        "sacrificing the only provider for U removes delve before payment"
    );
}

#[test]
fn additional_graveyard_cost_and_delve_reserve_distinct_cards_in_either_order() {
    let _scope = fixture_rules::activate(&[
        (named("Bear Cub"), &DELVE),
        (named("Quick Study"), &GRAVE_ADDITIONAL),
    ]);
    let p = Player::P0;
    for mana_first in [false, true] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let first = put(&mut g, p, "Forest", Zone::Graveyard);
        let second = put(&mut g, p, "Forest", Zone::Graveyard);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        let third = put(&mut g, p, "Forest", Zone::Graveyard);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::BeginOrderedCosts);
        if mana_first {
            clause(&mut g, CastCostClause::Mana);
            delver(&mut g, first);
            delver(&mut g, second);
            pay(&mut g);
            clause(&mut g, CastCostClause::Additional);
            let invalid = g.object_ref(first);
            reject(&mut g, Action::ChooseAdditionalCost { object: invalid });
            additional(&mut g, third);
        } else {
            clause(&mut g, CastCostClause::Additional);
            additional(&mut g, third);
            clause(&mut g, CastCostClause::Mana);
            reject_delver(&mut g, third);
            delver(&mut g, first);
            delver(&mut g, second);
            pay(&mut g);
        }
        drain(&mut g);
        for id in [first, second, third] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn delve_selection_is_owned_raw_index_restorable_and_private() {
    let _scope = fixture_rules::activate(&[(named("Bear Cub"), &DELVE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let fuel: Vec<_> = (0..3)
        .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
        .collect();
    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    start_delve(&mut g);
    delver(&mut g, fuel[0]);
    let action = Action::ChooseDelver {
        object: g.object_ref(fuel[1]),
    };
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
    let root = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut sibling = g.clone();
    drop(g);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for viewer in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(raw.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    assert!(raw.observe(p.other()).delve_choice.is_none());
    delver(&mut sibling, fuel[2]);
    pay(&mut raw);
    pay(&mut indexed);
    pay(&mut sibling);
    drain(&mut raw);
    drain(&mut indexed);
    drain(&mut sibling);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(fuel[1]).zone, Zone::Exile);
    assert_eq!(raw.object_view(fuel[2]).zone, Zone::Graveyard);
    assert_eq!(sibling.object_view(fuel[1]).zone, Zone::Graveyard);
    assert_eq!(sibling.object_view(fuel[2]).zone, Zone::Exile);
    indexed.restore(&root).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    pay(&mut indexed);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn delve_provider_can_leave_in_a_later_cost_after_mana_is_paid() {
    let _scope = fixture_rules::activate(&[
        (named("Bear Cub"), &DELVE),
        (named("Quick Study"), &SAC_ADDITIONAL),
    ]);
    let p = Player::P1;
    let mut g = scenario(p);
    let provider = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let first = put(&mut g, p, "Forest", Zone::Graveyard);
    let second = put(&mut g, p, "Forest", Zone::Graveyard);
    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::BeginOrderedCosts);
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    reject(&mut g, Action::ChooseCastCost { clause: index });
    clause(&mut g, CastCostClause::Mana);
    delver(&mut g, first);
    delver(&mut g, second);
    pay(&mut g);
    clause(&mut g, CastCostClause::Additional);
    additional(&mut g, provider);
    assert_eq!(g.object_view(provider).zone, Zone::Graveyard);
    assert_eq!(g.object_view(first).zone, Zone::Exile);
    drain(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.hand_count(p), 1);
}

#[test]
fn delve_uses_the_same_total_from_hand_exile_and_permitted_library_top() {
    let _scope = fixture_rules::activate(&[(named("Bear Cub"), &TOP_DELVE)]);
    let p = Player::P0;
    for origin in [Zone::Hand, Zone::Exile, Zone::Library] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let fuel = put(&mut g, p, "Forest", Zone::Graveyard);
        let spell = put(&mut g, p, "Grizzly Bears", origin);
        if origin == Zone::Exile {
            let object = g.object_ref(spell);
            g.grant_exile_play_permission(p, object, false);
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        act(
            &mut g,
            if origin == Zone::Exile {
                Action::AnnounceExile { card: spell }
            } else {
                Action::AnnounceCast { card: spell }
            },
        );
        start_delve(&mut g);
        delver(&mut g, fuel);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(fuel).zone, Zone::Exile, "{origin:?}");
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield, "{origin:?}");
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn mana_child_must_preserve_its_ancestors_distinct_delve_resources() {
    let _scope = fixture_rules::activate(&[
        (named("Bear Cub"), &DELVE),
        (named("Fanatical Firebrand"), &GRAVE_MANA),
    ]);
    let p = Player::P0;
    for extra_fuel in [false, true] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let producer = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
        let fuel: Vec<_> = (0..if extra_fuel { 3 } else { 2 })
            .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
            .collect();
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        let child = Action::AnnounceMana {
            card: producer,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        };
        if extra_fuel {
            act(&mut g, child);
            additional(&mut g, fuel[2]);
            pay(&mut g);
            assert_eq!(g.object_view(fuel[2]).zone, Zone::Exile);
            assert!(g.state.mana_announcement_stack.is_none());
        } else {
            reject(&mut g, child);
        }
        start_delve(&mut g);
        delver(&mut g, fuel[0]);
        delver(&mut g, fuel[1]);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn granted_harmonize_recomputes_the_cost_of_an_existing_selected_spell_face() {
    // The unrestricted granting fixture exercises the reusable route. A real
    // instant/sorcery-only grant cannot target this creature card in a graveyard.
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &GRANT_ANY),
        (named("Charring Bite"), &DRAW),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Twinmaw Stormbrood", Zone::Graveyard);
    let grantor = grant(&mut g, p, spell);
    g.move_object(grantor, Zone::Exile);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    assert!(
        !g.legal_actions().contains(&granted(spell)),
        "front face costs 5W"
    );
    act(
        &mut g,
        Action::AnnounceGrantedHarmonize {
            card: spell,
            face: CastFace::Omen,
        },
    );
    act(&mut g, Action::ChooseHarmonizer { object: None });
    pay(&mut g);
    assert_eq!(g.state.stack.last().unwrap().mana_value(), 2);
    drain(&mut g);
    assert_eq!(
        g.object_view(spell).zone,
        Zone::Exile,
        "Harmonize overrides Omen shuffle"
    );
    assert_eq!(g.object_view(spell).card, named("Twinmaw Stormbrood"));
    assert_eq!(g.hand_count(p), 1);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn harmonize_exiles_on_a_stack_departure_that_would_return_the_spell_to_hand() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &GRANT),
        (named("Cancel"), &BOUNCE_SPELL),
    ]);
    let p = Player::P1;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Quick Study", Zone::Graveyard);
    grant(&mut g, p, spell);
    act(&mut g, granted(spell));
    act(&mut g, Action::ChooseHarmonizer { object: None });
    pay(&mut g);
    act(&mut g, Action::Pass);
    let bounce = put(&mut g, p.other(), "Cancel", Zone::Hand);
    cast(&mut g, bounce, Some(Target::Spell(spell)));
    drain(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert_eq!(g.hand_count(p), 0);
}

#[test]
fn delve_shares_one_locked_payment_with_convoke_and_improvise() {
    let _scope =
        fixture_rules::activate(&[(named("Bear Cub"), &DELVE), (named("Quick Study"), &MIXED)]);
    let p = Player::P0;
    for artifact in [false, true] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let creature = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let equipment = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let fuel = put(&mut g, p, "Forest", Zone::Graveyard);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::BeginDelve);
        clause(&mut g, CastCostClause::Mana);
        let object = g.object_ref(if artifact { equipment } else { creature });
        act(
            &mut g,
            if artifact {
                Action::ChooseImproviser { object }
            } else {
                Action::ChooseConvoker {
                    object,
                    contribution: convoke::ConvokeContribution::Generic,
                }
            },
        );
        delver(&mut g, fuel);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(equipment).tapped, artifact);
        assert_eq!(g.object_view(creature).tapped, !artifact);
        assert_eq!(g.object_view(fuel).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn delve_cannot_pay_a_required_colorless_symbol() {
    let _scope = fixture_rules::activate(&[
        (named("Bear Cub"), &DELVE),
        (named("Quick Study"), &COLORLESS),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let fuel: Vec<_> = (0..3)
        .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
        .collect();
    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut g, Action::AnnounceCast { card: spell });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
    }
    assert_eq!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap()
            .colored[5],
        1
    );
    start_delve(&mut g);
    delver(&mut g, fuel[0]);
    delver(&mut g, fuel[1]);
    reject_delver(&mut g, fuel[2]);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(fuel[2]).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn delve_obeys_a_live_graveyard_exile_prohibition() {
    let _scope = fixture_rules::activate(&[
        (named("Bear Cub"), &DELVE),
        (named("Short Bow"), &EXILE_BLOCK),
    ]);
    let p = Player::P1;
    let mut g = scenario(p);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let blocker = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
    let first = put(&mut g, p, "Forest", Zone::Graveyard);
    let second = put(&mut g, p, "Forest", Zone::Graveyard);
    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    g.move_object(blocker, Zone::Exile);
    act(&mut g, Action::AnnounceCast { card: spell });
    start_delve(&mut g);
    delver(&mut g, first);
    delver(&mut g, second);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(first).zone, Zone::Exile);
    assert_eq!(g.object_view(second).zone, Zone::Exile);
    assert_eq!(g.hand_count(p), 2);
}

#[test]
fn a_proposed_graveyard_spell_cannot_pay_a_mana_childs_graveyard_cost() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &GRANT),
        (named("Bear Cub"), &DELVE),
        (named("Fanatical Firebrand"), &GRAVE_MANA),
    ]);
    let p = Player::P0;
    for delve in [false, true] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Quick Study", Zone::Graveyard);
        let grantor = grant(&mut g, p, spell);
        g.move_object(grantor, Zone::Exile);
        if delve {
            let provider = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[provider as usize].tapped = true;
        }
        let producer = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
        let fuel: Vec<_> = (0..if delve { 2 } else { 0 })
            .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
            .collect();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, if delve { 1 } else { 3 }];
        assert!(
            !g.legal_actions().contains(&granted(spell)),
            "the proposed spell has left the graveyard before a mana child's exile cost; delve={delve}"
        );
        let material = put(&mut g, p, "Forest", Zone::Graveyard);
        act(&mut g, granted(spell));
        act(&mut g, Action::ChooseHarmonizer { object: None });
        choose(
            &mut g,
            |a| matches!(a, Action::AnnounceMana {card, ..} if *card == producer),
        );
        additional(&mut g, material);
        pay(&mut g);
        if delve {
            start_delve(&mut g);
            for id in fuel {
                delver(&mut g, id);
            }
        }
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 2);
    }
}
