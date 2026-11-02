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
    effect: Effect::Creature(1, 1),
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
    effect: Effect::Creature(1, 3),
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
fn rollback(p: Player, replacement: bool) -> (Game, CardId, CardId, CardId) {
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
    let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let external = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
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
    assert!(g.object_view(source).tapped);
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
    assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
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
                let (root, source, external, _) = rollback(p, replacement);
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
