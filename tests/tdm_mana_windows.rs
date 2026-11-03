//! Final locked mana totals, actual payment windows and speculative obligations.
mod support;
use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ZERO: ManaCost = ManaCost {
    colored: [0; 6],
    generic: 0,
};
const ONE: ManaCost = ManaCost {
    colored: [0; 6],
    generic: 1,
};
const TEMPLATE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const FREE_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::FreeSpellCast(CastFilter::Subtype(
        "Dragon",
    ))],
}];
const DRACOGENESIS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: FREE_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FREE_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentNoncreatureSpellTax(1)],
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
const ACT_TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(1)],
}];
const ACT_TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: ACT_TAX_GROUPS,
    extras: &CardExtras {
        static_ability_groups: ACT_TAX_GROUPS,
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
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 1,
        ]))],
        ..TEMPLATE
    }],
    ..CardRules::EMPTY
};
fn card(n: &str) -> Card {
    Card::from_name(n).unwrap()
}
fn free(id: CardId) -> Action {
    Action::AnnounceFree {
        card: id,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}
fn activate(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn produce(id: CardId) -> Action {
    Action::ActivateMana {
        card: id,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 0, 0, 0, 0, 1],
    }
}
fn empty(g: &mut Game, p: Player) {
    g.state.players[p.index()].mana = [0; 6];
}
fn no_mana(g: &mut Game, forest: CardId) {
    assert!(
        !g.legal_actions()
            .contains(&Action::TapMana { card: forest })
    );
    let saved = g.snapshot();
    let p = g.actor().unwrap();
    assert_eq!(
        g.apply(p, Action::TapMana { card: forest }),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
    assert!(!g.object_view(forest).tapped);
}
fn clause(g: &mut Game, kind: CastCostClause) {
    let i = g
        .observe(g.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == kind && !r.paid)
        .unwrap();
    act(g, Action::ChooseCastCost { clause: i as u16 });
}
fn finish(g: &mut Game, material: Option<CardId>) {
    for _ in 0..24 {
        if g.state.announcing.is_none() {
            return;
        }
        match g.state.pending {
            Pending::PayCosts { .. } => pay(g),
            Pending::CastCosts { .. } => {
                let i = g
                    .observe(g.actor().unwrap())
                    .cast_costs
                    .unwrap()
                    .clauses
                    .iter()
                    .position(|r| !r.paid)
                    .unwrap();
                act(g, Action::ChooseCastCost { clause: i as u16 });
            }
            Pending::AdditionalCost { .. } => {
                if g.legal_actions()
                    .contains(&Action::FinishAdditionalCostGroup)
                {
                    act(g, Action::FinishAdditionalCostGroup);
                } else {
                    let object = g.object_ref(material.unwrap());
                    act(g, Action::ChooseAdditionalCost { object });
                }
            }
            Pending::BeholdCost { .. } => {
                let object = g.object_ref(material.unwrap());
                act(g, Action::ChooseBeholdCost { object });
            }
            _ => panic!("unexpected payment phase {:?}", g.state.pending),
        }
    }
    panic!("bounded payment completion");
}
fn target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets)
    }
}
fn physical_provider(g: &mut Game, p: Player, name: &str) -> CardId {
    // Use the immutable requested identity with a scoped typed provider. No catalogue admission changes.
    let id = put(g, p, "Ghost Vacuum", Zone::Hand);
    g.state.objects[id as usize].card = card(name);
    g.move_object(id, Zone::Battlefield);
    id
}
#[test]
fn native_mox_jasper_zero_announcement_never_opens_a_mana_ability_window() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let f = put(&mut g, p, "Forest", Zone::Battlefield);
        let m = put(&mut g, p, "Mox Jasper", Zone::Hand);
        empty(&mut g, p);
        act(&mut g, Action::AnnounceCast { card: m });
        assert_eq!(g.observe(p).payment_request.unwrap().generic, 0);
        no_mana(&mut g, f);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(m).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn zero_payment_acknowledgment_is_owned_raw_index_restore_and_does_not_tap_a_land() {
    let p = Player::P1;
    let mut parent = scenario(p);
    let f = put(&mut parent, p, "Forest", Zone::Battlefield);
    let m = put(&mut parent, p, "Mox Jasper", Zone::Hand);
    empty(&mut parent, p);
    act(&mut parent, Action::AnnounceCast { card: m });
    let a = Action::PayCosts { payment: [0; 6] };
    let saved = parent.snapshot();
    let rev = parent.revision();
    let i = parent.legal_actions().iter().position(|x| *x == a).unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    no_mana(&mut raw, f);
    act(&mut raw, a.clone());
    indexed.apply_index(p, rev, i).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, a);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn priority_basic_intrinsic_granted_and_zero_cost_mana_abilities_remain_available() {
    const LAND: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LandMana {
            recipients: LandManaRecipients::YouControl,
            production: ManaProduction::Fixed([0, 0, 0, 0, 0, 1]),
        }],
    }];
    const GRANT: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: LAND,
        extras: &CardExtras {
            static_ability_groups: LAND,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const EXPLICIT_ZERO: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            ..TEMPLATE
        }],
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (card("Short Bow"), &GRANT),
        (card("Gilded Lotus"), &EXPLICIT_ZERO),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let f = put(&mut g, p, "Forest", Zone::Battlefield);
        let grantor = put(&mut g, p, "Short Bow", Zone::Battlefield);
        put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
        let m = put(&mut g, p, "Mox Jasper", Zone::Battlefield);
        let z = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        empty(&mut g, p);
        let a = g.legal_actions();
        assert!(a.contains(&Action::TapMana { card: f }));
        assert!(a.iter().any(|a|matches!(a,Action::TapGrantedMana {card,grantor:r,..}if *card==f&&*r==g.object_ref(grantor))));
        assert!(a.iter().any(
            |a| matches!(a,Action::ActivateMana {card,..}|Action::AnnounceMana {card,..}if *card==m)
        ));
        let action=a.into_iter().find(|a|matches!(a,Action::ActivateMana {card,..}|Action::AnnounceMana {card,..}if *card==z)).unwrap();
        act(&mut g, action);
        if g.state.announcing.is_some() {
            no_mana(&mut g, f);
            pay(&mut g);
        }
        assert_eq!(g.state.players[p.index()].mana[5], 1);
        assert!(g.legal_actions().contains(&Action::TapMana { card: f }));
    }
}
#[test]
fn dracogenesis_filtered_free_dragon_has_no_printed_mana_payment_window() {
    let _s = fixture_rules::activate(&[(card("Dracogenesis"), &DRACOGENESIS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        physical_provider(&mut g, p, "Dracogenesis");
        let f = put(&mut g, p, "Forest", Zone::Battlefield);
        let d = put(&mut g, p, "Shivan Dragon", Zone::Hand);
        empty(&mut g, p);
        act(&mut g, free(d));
        no_mana(&mut g, f);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(d).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn positive_tax_on_native_zero_spell_opens_a_window_and_locked_total_does_not_reopen_at_zero() {
    let _s = fixture_rules::activate(&[(Card::GoblinPiker, &TAX)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let f = put(&mut g, p, "Forest", Zone::Battlefield);
    let tax = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
    let m = put(&mut g, p, "Mox Jasper", Zone::Hand);
    empty(&mut g, p);
    act(&mut g, Action::AnnounceCast { card: m });
    assert_eq!(g.observe(p).payment_request.unwrap().generic, 1);
    assert!(g.legal_actions().contains(&Action::TapMana { card: f }));
    g.move_object(tax, Zone::Graveyard);
    assert_eq!(g.observe(p).payment_request.unwrap().generic, 1);
    act(&mut g, Action::TapMana { card: f });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn target_equip_discount_uses_final_zero_or_positive_total() {
    const GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::EquipTargetReduction(1)],
    }];
    const REDUCER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: GROUP,
        extras: &CardExtras {
            static_ability_groups: GROUP,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (Card::GrizzlyBears, &REDUCER),
        (Card::GoblinPiker, &ACT_TAX),
    ]);
    for taxed in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let f = put(&mut g, p, "Forest", Zone::Battlefield);
        let recipient = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let e = put(&mut g, p, "Short Bow", Zone::Battlefield);
        if taxed {
            let tax = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
            g.set_entry_value(
                g.object_ref(tax),
                EntryChoiceValue::CardName(CardName::from_name("Short Bow").unwrap()),
            );
        }
        empty(&mut g, p);
        act(&mut g, activate(e));
        target(&mut g, recipient);
        assert_eq!(
            g.observe(p).payment_request.unwrap().generic,
            u32::from(taxed)
        );
        if taxed {
            act(&mut g, Action::TapMana { card: f });
        } else {
            no_mana(&mut g, f);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(e).attached_to, Some(g.object_ref(recipient)));
    }
}
#[test]
fn explicit_zero_mana_component_and_pure_nonmana_activations_both_skip_the_window() {
    const A: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                mana_component: true,
                life: 1,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            ..TEMPLATE
        }],
        ..CardRules::EMPTY
    };
    const B: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                mana_component: false,
                ..A.activated[0].cost
            },
            ..A.activated[0]
        }],
        ..A
    };
    for rules in [&A, &B] {
        let _s = fixture_rules::activate(&[(card("Short Bow"), rules)]);
        let p = Player::P0;
        let mut g = scenario(p);
        let f = put(&mut g, p, "Forest", Zone::Battlefield);
        let s = put(&mut g, p, "Short Bow", Zone::Battlefield);
        empty(&mut g, p);
        act(&mut g, activate(s));
        no_mana(&mut g, f);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[0].life, 22);
    }
}
#[test]
fn positive_cost_payable_entirely_by_convoke_keeps_its_initial_mana_window() {
    const C: CardRules = CardRules {
        effect: Effect::Permanent,
        extras: &CardExtras {
            has_convoke: true,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[(card("Short Bow"), &C)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let f = put(&mut g, p, "Forest", Zone::Battlefield);
    let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let c2 = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let s = put(&mut g, p, "Short Bow", Zone::Hand);
    empty(&mut g, p);
    act(&mut g, Action::AnnounceCast { card: s });
    assert!(g.legal_actions().contains(&Action::TapMana { card: f }));
    act(&mut g, Action::BeginConvoke);
    let object = g.object_ref(c);
    act(
        &mut g,
        Action::ChooseConvoker {
            object,
            contribution: convoke::ConvokeContribution::Generic,
        },
    );
    let object = g.object_ref(c2);
    act(
        &mut g,
        Action::ChooseConvoker {
            object,
            contribution: convoke::ConvokeContribution::Generic,
        },
    );
    no_mana(&mut g, f);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(s).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}
#[test]
fn positive_cost_payable_entirely_by_delve_keeps_its_initial_mana_window() {
    const DELVE_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellDelve(CastFilter::Any)],
    }];
    const C: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: DELVE_GROUPS,
        extras: &CardExtras {
            static_ability_groups: DELVE_GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[(Card::GrizzlyBears, &C)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let f = put(&mut g, p, "Forest", Zone::Battlefield);
    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let fuel = put(&mut g, p, "Opt", Zone::Graveyard);
    let fuel2 = put(&mut g, p, "Opt", Zone::Graveyard);
    let s = put(&mut g, p, "Short Bow", Zone::Hand);
    empty(&mut g, p);
    act(&mut g, Action::AnnounceCast { card: s });
    assert!(g.legal_actions().contains(&Action::TapMana { card: f }));
    act(&mut g, Action::BeginDelve);
    clause(&mut g, CastCostClause::Mana);
    let object = g.object_ref(fuel);
    act(&mut g, Action::ChooseDelver { object });
    let object = g.object_ref(fuel2);
    act(&mut g, Action::ChooseDelver { object });
    no_mana(&mut g, f);
    pay(&mut g);
    finish(&mut g, None);
    settle(&mut g);
    assert_eq!(g.object_view(fuel).zone, Zone::Exile);
    assert_eq!(g.object_view(s).zone, Zone::Battlefield);
}
#[test]
fn resolution_zero_primary_with_positive_alternative_preserves_a_real_window() {
    const R: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[
                Instruction::MayPayManaAlternatives {
                    costs: &[ZERO, ONE],
                    skip: 1,
                },
                Instruction::GainLife(PlayerSet::Controller, 3),
            ],
        ),
        ..CardRules::EMPTY
    };
    const Z: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[
                Instruction::MayPayMana {
                    cost: ZERO,
                    skip: 1,
                },
                Instruction::GainLife(PlayerSet::Controller, 3),
            ],
        ),
        ..CardRules::EMPTY
    };
    for (rules, positive) in [(&R, true), (&Z, false)] {
        let _s = fixture_rules::activate(&[(card("Opt"), rules)]);
        let p = Player::P0;
        let mut g = scenario(p);
        let f = put(&mut g, p, "Forest", Zone::Battlefield);
        let s = put(&mut g, p, "Opt", Zone::Hand);
        announce(&mut g, s, &[]);
        for _ in 0..8 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        empty(&mut g, p);
        if positive {
            assert!(g.legal_actions().contains(&Action::TapMana { card: f }));
            act(&mut g, Action::TapMana { card: f });
        } else {
            no_mana(&mut g, f);
        }
        act(&mut g, Action::PayResolutionMana { payment: [0; 6] });
        settle(&mut g);
        assert_eq!(g.state.players[0].life, 23);
    }
}
#[test]
fn future_behold_witness_cannot_be_invented_by_a_free_zero_total_mana_child() {
    const D: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        extras: &CardExtras {
            behold_cast: Some(BeholdCastSpec {
                subtype: "Dragon",
                decline_generic: 1,
                grants_flash: false,
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (card("Dracogenesis"), &DRACOGENESIS),
        (card("Shivan Dragon"), &D),
        (card("Ghost Vacuum"), &PRODUCER),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    physical_provider(&mut g, p, "Dracogenesis");
    let prod = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
    let witness = put(&mut g, p, "Shivan Dragon", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(prod),
        exiled: g.object_ref(witness),
        return_zone: Zone::Battlefield,
        ability_origin: None,
    });
    let s = put(&mut g, p, "Shivan Dragon", Zone::Hand);
    empty(&mut g, p);
    act(&mut g, free(s));
    let saved = g.snapshot();
    assert!(
        !g.legal_actions()
            .contains(&Action::ChooseCastBehold { pay: true })
    );
    assert_eq!(
        g.apply(p, Action::ChooseCastBehold { pay: true }),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
    act(&mut g, Action::ChooseCastBehold { pay: false });
    assert!(g.legal_actions().contains(&produce(prod)));
    act(&mut g, produce(prod));
    assert_eq!(g.object_view(witness).zone, Zone::Battlefield);
    finish(&mut g, None);
    settle(&mut g);
    assert_eq!(g.object_view(s).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}
#[test]
fn full_proposal_cannot_restore_its_only_sacrifice_witness_without_a_positive_total() {
    const COST: AdditionalCost = AdditionalCost {
        kind: EffectChoiceKind::SacrificeRange { minimum: 1 },
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 1,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
    };
    const S: CardRules = CardRules {
        effect: Effect::Permanent,
        extras: &CardExtras {
            has_convoke: true,
            additional_cost: Some(COST),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _s = fixture_rules::activate(&[
        (card("Mox Jasper"), &S),
        (card("Ghost Vacuum"), &PRODUCER),
        (Card::GoblinPiker, &TAX),
    ]);
    for taxed in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let prod = put(&mut g, p, "Ghost Vacuum", Zone::Battlefield);
        let witness = put(&mut g, p, "Bear Cub", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(prod),
            exiled: g.object_ref(witness),
            return_zone: Zone::Battlefield,
            ability_origin: None,
        });
        if taxed {
            put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        }
        let s = put(&mut g, p, "Mox Jasper", Zone::Hand);
        empty(&mut g, p);
        let action = Action::AnnounceCast { card: s };
        if !taxed {
            let saved = g.snapshot();
            assert_eq!(g.apply(p, action), Err(Error::IllegalAction));
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.object_view(witness).zone, Zone::Exile);
            continue;
        }
        act(&mut g, action);
        act(&mut g, produce(prod));
        assert_eq!(g.object_view(witness).zone, Zone::Battlefield);
        finish(&mut g, Some(witness));
        settle(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Graveyard);
        assert_eq!(g.object_view(s).zone, Zone::Battlefield);
    }
}

#[test]
fn spell_reduction_to_zero_closes_window_but_a_positive_tax_still_opens_it() {
    const RED: SpellReduction = SpellReduction {
        source_only: false,
        filter: CastFilter::Any,
        amount: SpellReductionAmount::Fixed(5),
        turn: CostTurn::Any,
        guard: StepGuard::Always,
    };
    const GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, RED)],
    }];
    const REDUCER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: GROUP,
        extras: &CardExtras {
            static_ability_groups: GROUP,
            cost_reductions: &[RED],
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope =
        fixture_rules::activate(&[(Card::GrizzlyBears, &REDUCER), (Card::GoblinPiker, &TAX)]);
    for taxed in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let f = put(&mut g, p, "Forest", Zone::Battlefield);
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        if taxed {
            put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        }
        let spell = put(&mut g, p, "Gilded Lotus", Zone::Hand);
        empty(&mut g, p);
        act(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.observe(p).payment_request.unwrap().generic,
            u32::from(taxed)
        );
        if taxed {
            act(&mut g, Action::TapMana { card: f });
        } else {
            no_mana(&mut g, f);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn an_already_funded_positive_colored_total_still_allows_mana_abilities() {
    const SPELL: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::GainLife(PlayerSet::Controller, 1)],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Opt"), &SPELL)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let f = put(&mut g, p, "Forest", Zone::Battlefield);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    assert!(g.legal_actions().contains(&Action::TapMana { card: f }));
    act(&mut g, Action::TapMana { card: f });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
    assert_eq!(g.state.players[p.index()].life, 21);
}

const PAID_BEHOLD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ),
    extras: &CardExtras {
        behold_cast: Some(BeholdCastSpec {
            subtype: "Dragon",
            decline_generic: 0,
            grants_flash: false,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn beheld_before_mana(p: Player) -> (Game, CardId) {
    let mut g = scenario(p);
    let forest = put(&mut g, p, "Forest", Zone::Battlefield);
    for _ in 0..3 {
        put(&mut g, p, "Island", Zone::Battlefield);
    }
    let dragon = put(&mut g, p, "Shivan Dragon", Zone::Hand);
    let spell = put(&mut g, p, "Quick Study", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseCastBehold { pay: true });
    act(&mut g, Action::BeginOrderedCosts);
    clause(&mut g, CastCostClause::Behold("Dragon"));
    let object = g.object_ref(dragon);
    act(&mut g, Action::ChooseBeholdCost { object });
    clause(&mut g, CastCostClause::Mana);
    assert!(
        g.state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .mana_closed
    );
    (g, forest)
}
#[test]
fn nonmana_first_never_reopens_mana_abilities_but_funded_payment_is_owned_and_valid() {
    let _scope = fixture_rules::activate(&[(card("Quick Study"), &PAID_BEHOLD)]);
    for p in [Player::P0, Player::P1] {
        let (mut parent, f) = beheld_before_mana(p);
        no_mana(&mut parent, f);
        let a = Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 2],
        };
        let saved = parent.snapshot();
        let rev = parent.revision();
        let index = parent.legal_actions().iter().position(|x| *x == a).unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        drop(parent);
        act(&mut raw, a.clone());
        indexed.apply_index(p, rev, index).unwrap();
        settle(&mut raw);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[p.index()].life, 21);
        assert!(!raw.object_view(f).tapped);
        indexed.restore(&saved).unwrap();
        act(&mut indexed, a);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
#[test]
fn stranded_nonmana_first_proposal_cannot_invent_a_forbidden_late_mana_child() {
    let _scope = fixture_rules::activate(&[(card("Quick Study"), &PAID_BEHOLD)]);
    let p = Player::P0;
    let (mut g, f) = beheld_before_mana(p);
    // Query a stranded remaining-cost world: its unused Islands could fund it only
    // by activating mana abilities after the nonmana cost was already paid.
    empty(&mut g, p);
    let a = g.state.announcing.as_ref().unwrap();
    let goal = g.funding_goal(a);
    let purpose = g.purpose_for_item(&a.item);
    assert!(!g.can_fund_for(goal, purpose));
    no_mana(&mut g, f);
    let saved = g.snapshot();
    assert_eq!(
        g.apply(p, Action::PayCosts { payment: [0; 6] }),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
}

#[test]
fn symbol_group_replacement_context_keeps_the_already_closed_mana_window() {
    let _scope = fixture_rules::activate(&[(card("Quick Study"), &PAID_BEHOLD)]);
    let p = Player::P0;
    let (mut g, _) = beheld_before_mana(p);
    let material = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
    empty(&mut g, p);
    let a = g.state.announcing.as_ref().unwrap();
    let goal = g.funding_goal(a);
    let purpose = g.purpose_for_item(&a.item);
    let source = a.item.source.unwrap().object;
    let cost = AdditionalCost {
        kind: EffectChoiceKind::ExileGraveyardSymbols {
            color: ChoiceColor::Green,
            minimum: 1,
        },
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 0,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
    };
    assert!(!g.can_fund_symbol_cost_group(goal, purpose, cost, source, &[g.object_ref(material)]));
}
