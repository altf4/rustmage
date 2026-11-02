//! Completed external mana-cost receipts survive a genuinely failed foreign choice.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ARTIFACT: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 16,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const COST: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::Sacrifice,
    filter: ARTIFACT,
};
const FUEL: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            additional: Some(COST),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 3, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const FILTERED: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::FilteredCost {
                    action: CostAction::Sacrifice,
                    any: &[CostMatch::Kinds(16), CostMatch::Subtype("Goblin")],
                },
                filter: SacrificeFilter {
                    kinds: 0,
                    ..ARTIFACT
                },
            }),
            ..FUEL.activated[0].cost
        },
        ..FUEL.activated[0]
    }],
    ..FUEL
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Sacrificed {
            filter: ARTIFACT,
            own_turn: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("bounded trigger settlement")
}
fn rollback(p: Player, replacement: bool, dependent: bool) -> (Game, CardId, CardId, CardId) {
    let mut g = scenario(p);
    let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
    g.state.objects[silver as usize].growth.power = -1;
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
    let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
    cast(&mut g, prison, None);
    for _ in 0..32 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    drain(&mut g);
    let source = put(
        &mut g,
        p,
        "Llanowar Elves",
        if dependent {
            Zone::Exile
        } else {
            Zone::Battlefield
        },
    );
    let external = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    // Neither the mana producer nor the observer can pay casualty1. Only
    // Silverquill can, until the foreign Aura reduces its power to zero.
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let returner = dependent.then(|| {
        let returner = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
        g.state.exile_links.push(ExileLink {
            ability_origin: None,
            source: g.object_ref(returner),
            exiled: g.object_ref(source),
            return_zone: Zone::Battlefield,
        });
        returner
    });
    if replacement {
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
    }
    let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    let sacrifice = Some(g.object_ref(prison));
    act(&mut g, Action::ChooseBargain { sacrifice });
    act(&mut g, Action::ChooseCasualty { pay: true });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    if let Some(returner) = returner {
        act(
            &mut g,
            Action::ActivateMana {
                card: returner,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
    act(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 3, 0],
        },
    );
    let object = g.object_ref(external);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    assert_eq!(
        g.object_view(external).zone,
        if replacement {
            Zone::Exile
        } else {
            Zone::Graveyard
        }
    );
    assert_eq!(g.object_view(source).tapped, !dependent);
    pay(&mut g);
    let clause = g
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut g, Action::ChooseCastCost { clause });
    let object = g.object_ref(silver);
    act(&mut g, Action::ChooseAuraAttachment { object });
    assert!(
        matches!(g.state.pending, Pending::ManaRollback { .. }),
        "{p:?}/replacement={replacement}/dependent={dependent}: {:?}",
        g.state.pending
    );
    assert_eq!(g.object_view(spell).zone, Zone::Hand);
    (g, source, external, spell)
}

#[test]
fn retained_external_sacrifice_preserves_replacement_movement_and_one_original_trigger() {
    for rules in [&FUEL, &FILTERED] {
        let _scope =
            fixture_rules::activate(&[(Card::LlanowarElves, rules), (Card::HillGiant, &OBSERVER)]);
        for p in [Player::P0, Player::P1] {
            for replacement in [false, true] {
                let (root, source, external, _) = rollback(p, replacement, false);
                for keep in [false, true] {
                    let mut raw = root.clone();
                    let saved = raw.snapshot();
                    let mut indexed = raw.clone();
                    indexed.restore(&saved).unwrap();
                    let action = Action::ChooseManaRollback { keep };
                    let index = indexed.legal_actions().iter().position(|a| *a == action)
                        .expect("completed reversible external mana payment may be retained or reversed");
                    assert!(raw.observe(p).mana_rollback.unwrap().can_keep);
                    assert!(raw.observe(p.other()).mana_rollback.is_none());
                    let actor = indexed.actor().unwrap();
                    let revision = indexed.revision();
                    act(&mut raw, action);
                    indexed.apply_index(actor, revision, index).unwrap();
                    assert_eq!(raw.snapshot(), indexed.snapshot());
                    assert_eq!(raw.object_view(source).tapped, keep);
                    assert_eq!(
                        raw.object_view(external).zone,
                        if keep {
                            if replacement {
                                Zone::Exile
                            } else {
                                Zone::Graveyard
                            }
                        } else {
                            Zone::Battlefield
                        }
                    );
                    assert_eq!(
                        raw.state.players[p.index()].mana,
                        if keep {
                            [0, 0, 1, 0, 3, 0]
                        } else {
                            [0, 0, 1, 0, 0, 0]
                        }
                    );
                    let life = raw.state.players[p.index()].life;
                    drain(&mut raw);
                    assert_eq!(raw.state.players[p.index()].life, life + i128::from(keep));
                }
            }
        }
    }
}

#[test]
fn reversing_the_entry_event_makes_a_dependent_no_tap_exert_mana_receipt_unretainable() {
    const EXERT_FUEL: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: false,
                ..FUEL.activated[0].cost
            },
            program: &[
                Instruction::ExertCost,
                Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 0, 3, 0])),
            ],
            ..FUEL.activated[0]
        }],
        ..FUEL
    };
    const RETURNER: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 1, 0,
            ]))],
            ..FUEL.activated[0]
        }],
        ..FUEL
    };
    let _scope = fixture_rules::activate(&[
        (Card::LlanowarElves, &EXERT_FUEL),
        (Card::from_name("Druid of the Cowl").unwrap(), &RETURNER),
        (Card::HillGiant, &OBSERVER),
    ]);
    for p in [Player::P0, Player::P1] {
        let (root, source, external, _) = rollback(p, false, true);
        assert_eq!(root.state.cast_rollback.as_ref().unwrap().events.len(), 2);
        for keep_entry in [false, true] {
            let mut g = root.clone();
            act(&mut g, Action::ChooseManaRollback { keep: keep_entry });
            let saved = g.snapshot();
            let keep = Action::ChooseManaRollback { keep: true };
            assert_eq!(g.observe(p).mana_rollback.unwrap().can_keep, keep_entry);
            assert_eq!(g.legal_actions().contains(&keep), keep_entry);
            assert_eq!(g.snapshot(), saved, "retention query is transactional");
            if !keep_entry {
                reject(&mut g, keep.clone());
            }
            for keep_exert in [false, true]
                .into_iter()
                .filter(|keep| !*keep || keep_entry)
            {
                let mut raw = g.clone();
                let mut indexed = raw.clone();
                indexed.restore(&saved).unwrap();
                let action = Action::ChooseManaRollback { keep: keep_exert };
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
                assert_eq!(
                    raw.object_view(source).zone,
                    if keep_entry {
                        Zone::Battlefield
                    } else {
                        Zone::Exile
                    }
                );
                assert!(!raw.object_view(source).tapped, "exert is not a tap cost");
                assert_eq!(
                    raw.observe(p)
                        .exertions
                        .as_ref()
                        .is_some_and(|rows| rows.iter().any(|r| r.object.card == source)),
                    keep_exert
                );
                assert_eq!(
                    raw.object_view(external).zone,
                    if keep_exert {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}
