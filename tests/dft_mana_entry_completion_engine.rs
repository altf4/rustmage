//! Paid mana receipts participate in the same projected entry world as their parent.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BASE: ActivatedAbility = ActivatedAbility {
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
const COPY: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 17,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const LATE_POWER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::TapSourcePowerChoice { mask: 31 },
            quality: &ManaQuality {
                restriction: ManaRestriction::Unrestricted,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Turn,
            },
        })],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const COUNT: CardRules = CardRules {
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(ManaProduction::CountSubtype {
            color: 1,
            subtype: "Elf",
        })],
        ..LATE_POWER.activated[0]
    }],
    ..LATE_POWER
};
const THREE_PARENT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const FOUR_PARENT: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 4,
            ..THREE_PARENT.activated[0].cost
        },
        ..THREE_PARENT.activated[0]
    }],
    ..THREE_PARENT
};
fn direct(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production,
    }
}
fn index(g: &mut Game, action: Action) {
    let slot = g.legal_actions().iter().position(|a| *a == action).unwrap();
    g.apply_index(g.actor().unwrap(), g.revision(), slot)
        .unwrap();
}
fn returning_copy(g: &mut Game, p: Player, source: CardId) -> CardId {
    let returning = put(g, p, "Grizzly Bears", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(returning),
        return_zone: Zone::Battlefield,
    });
    returning
}
fn entry_choice(object: Option<ObjectRef>) -> Action {
    Action::ChooseEntryCopy { object }
}

#[test]
fn an_owned_entry_can_decline_copy_before_choosing_the_only_parent_funding_color() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &COPY),
        (Card::LlanowarElves, &LATE_POWER),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let druid = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
        let returning = returning_copy(&mut g, p, source);
        let target = put(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
        let spell = put(&mut g, p, "Pacifism", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        act(&mut g, direct(source, [0; 6]));
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(
            g.state.announcing.as_ref().unwrap().production,
            Some([0; 6])
        );
        assert!(g.legal_actions().contains(&entry_choice(None)));
        assert!(g.observe(p.other()).mana_choices.is_empty());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut restored = g.clone();
        restored.restore(&saved).unwrap();
        drop(g);
        for (i, branch) in [&mut raw, &mut indexed, &mut restored]
            .into_iter()
            .enumerate()
        {
            if i == 1 {
                index(branch, entry_choice(None));
            } else {
                act(branch, entry_choice(None));
            }
            assert!(matches!(branch.state.pending, Pending::EffectMana { .. }));
            // SBAs wait until the enclosing cast finishes, even for a 0/0.
            assert_eq!(branch.object_view(returning).zone, Zone::Battlefield);
            assert_eq!(branch.observe(p).mana_choices, vec![[1, 0, 0, 0, 0, 0]]);
            assert!(branch.observe(p.other()).mana_choices.is_empty());
            reject(
                branch,
                Action::ChooseMana {
                    production: [0, 0, 0, 0, 1, 0],
                },
            );
            let action = Action::ChooseMana {
                production: [1, 0, 0, 0, 0, 0],
            };
            if i == 1 {
                index(branch, action);
            } else {
                act(branch, action);
            }
            assert_eq!(branch.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
            let lot = &branch.state.mana_lots.as_ref().unwrap().pools[p.index()][0];
            assert_eq!(lot.amounts, [1, 0, 0, 0, 0, 0]);
            assert_eq!(lot.quality.retention, ManaRetention::Turn);
            act(branch, direct(druid, [0, 0, 0, 0, 1, 0]));
            pay(branch);
            settle(branch);
            assert_eq!(branch.object_view(spell).zone, Zone::Battlefield);
            assert_eq!(branch.object_view(returning).zone, Zone::Graveyard);
            assert_eq!(
                branch.object_view(spell).attached_to,
                Some(branch.object_ref(target))
            );
            assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}

#[test]
fn nonzero_previews_neither_admit_lost_quantity_nor_hide_new_quantity_during_entry() {
    // Before sacrificing the source there are two Elves. One return leaves the
    // count unchanged; two returns increase it. Both require every copier to
    // become an Elf, so declining or copying a Giant must be rejected.
    for (returns, parent_rules) in [(1, &THREE_PARENT), (2, &FOUR_PARENT)] {
        let _scope = fixture_rules::activate(&[
            (Card::GrizzlyBears, &COPY),
            (Card::LlanowarElves, &COUNT),
            (Card::HillGiant, parent_rules),
        ]);
        for p in [Player::P0, Player::P1] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let druid = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
            let parent = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let returning = (0..returns)
                .map(|_| returning_copy(&mut g, p, source))
                .collect::<Vec<_>>();
            g.state.players[p.index()].mana = [0; 6];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: parent,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(&mut g, direct(source, [0, 2, 0, 0, 0, 0]));
            assert_eq!(
                g.state.announcing.as_ref().unwrap().production,
                Some([0, 2, 0, 0, 0, 0])
            );
            let saved = g.snapshot();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            let mut restored = g.clone();
            restored.restore(&saved).unwrap();
            drop(g);
            for (i, branch) in [&mut raw, &mut indexed, &mut restored]
                .into_iter()
                .enumerate()
            {
                for _ in 0..returns {
                    reject(branch, entry_choice(None));
                    let wrong = entry_choice(Some(branch.object_ref(parent)));
                    reject(branch, wrong);
                    let action = entry_choice(Some(branch.object_ref(druid)));
                    if i == 1 {
                        index(branch, action);
                    } else {
                        act(branch, action);
                    }
                }
                assert_eq!(
                    branch.state.players[p.index()].mana,
                    [0, returns + 1, 0, 0, 0, 0]
                );
                for &object in &returning {
                    assert_eq!(branch.object_view(object).zone, Zone::Battlefield);
                    assert!(branch.has_subtype(object, "Elf"));
                    assert!(branch.object_view(object).summoning_sick);
                }
                act(branch, direct(druid, [0, 0, 0, 0, 1, 0]));
                pay(branch);
                settle(branch);
                assert_eq!(branch.counter_count(parent, CounterKind::PlusOne), 1);
                assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
            }
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.snapshot(), restored.snapshot());
        }
    }
}

#[test]
fn a_departed_animated_mountain_receipt_adds_its_frozen_tap_bonus_exactly_once() {
    const LAND: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: LATE_POWER.activated,
        ..CardRules::EMPTY
    };
    const PARENT: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                colored: [0, 0, 0, 1, 0, 0],
                ..THREE_PARENT.activated[0].cost
            },
            ..THREE_PARENT.activated[0]
        }],
        ..THREE_PARENT
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &COPY),
        (Card::Mountain, &LAND),
        (Card::HillGiant, &PARENT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Mountain", Zone::Battlefield);
        g.change_object_form(
            g.object_ref(source),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreatureAndLand,
        );
        assert_eq!(g.object_view(source).kinds & 3, 3);
        assert_eq!(stats(&g, source), (1, 1));
        let parent = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        returning_copy(&mut g, p, source);
        g.state.players[p.index()].mountain_mana_bonus = 1;
        g.state.players[p.index()].mana = [0; 6];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: parent,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, direct(source, [0; 6]));
        assert_eq!(
            g.state.announcing.as_ref().unwrap().item.mana_tap_bonus,
            Some(1)
        );
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut restored = g.clone();
        restored.restore(&saved).unwrap();
        drop(g);
        for (i, branch) in [&mut raw, &mut indexed, &mut restored]
            .into_iter()
            .enumerate()
        {
            let choice = entry_choice(None);
            if i == 1 {
                index(branch, choice);
            } else {
                act(branch, choice);
            }
            assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
            assert_eq!(branch.observe(p).mana_choices.len(), 5);
            reject(
                branch,
                Action::ChooseMana {
                    production: [2, 0, 0, 0, 0, 0],
                },
            );
            let choice = Action::ChooseMana {
                production: [1, 0, 0, 0, 0, 0],
            };
            if i == 1 {
                index(branch, choice);
            } else {
                act(branch, choice);
            }
            assert_eq!(branch.state.players[p.index()].mana, [1, 0, 0, 1, 0, 0]);
            let lots = &branch.state.mana_lots.as_ref().unwrap().pools[p.index()];
            assert_eq!(lots.len(), 1);
            assert_eq!(lots[0].amounts, [1, 0, 0, 0, 0, 0]);
            pay(branch);
            settle(branch);
            assert_eq!(branch.counter_count(parent, CounterKind::PlusOne), 1);
            assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}

#[test]
fn a_fixed_prefix_inside_a_root_plot_mana_child_cannot_exile_the_plotted_hand_subject() {
    const RR: Mana = [0, 0, 0, 2, 0, 0];
    const G: Mana = [0, 0, 0, 0, 1, 0];
    const CHILD: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(RR))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const PLOT_SUBJECT: CardRules = CardRules {
        effect: Effect::Creature(1, 2),
        extras: &CardExtras {
            plot_cost: Some(ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
            }),
            ..CardExtras::EMPTY
        },
        activated: &[ActivatedAbility {
            origin: ActivationOrigin::Hand,
            cost: ActivationCost {
                exile_self: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(G))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &CHILD),
        (
            Card::from_name("Slickshot Show-Off").unwrap(),
            &PLOT_SUBJECT,
        ),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let child = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let plotted = put(&mut g, p, "Slickshot Show-Off", Zone::Hand);
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let bad_prefix = direct(plotted, G);
        let mut free = g.clone();
        act(&mut free, bad_prefix.clone());
        assert_eq!(free.object_view(plotted).zone, Zone::Exile);
        assert_eq!(free.state.players[p.index()].mana, G);
        act(&mut g, Action::AnnouncePlot { card: plotted });
        act(
            &mut g,
            Action::AnnounceMana {
                card: child,
                ability: 0,
                counter_payment: Counters::default(),
                production: RR,
            },
        );
        assert!(g.state.mana_announcement_stack.is_none());
        assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
        assert!(matches!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .mana_output
                .as_ref()
                .unwrap()
                .resume
                .as_deref(),
            Some(Pending::PlotPayment { .. })
        ));
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut restored = g.clone();
        restored.restore(&saved).unwrap();
        drop(g);
        for (i, branch) in [&mut raw, &mut indexed, &mut restored]
            .into_iter()
            .enumerate()
        {
            reject(branch, bad_prefix.clone());
            assert_eq!(branch.object_view(plotted).zone, Zone::Hand);
            let good = Action::TapMana { card: forest };
            if i == 1 {
                index(branch, good);
            } else {
                act(branch, good);
            }
            pay(branch);
            assert!(matches!(branch.state.pending, Pending::PlotPayment { .. }));
            assert_eq!(branch.state.players[p.index()].mana, RR);
            let payment = Action::PayPlot { payment: RR };
            if i == 1 {
                index(branch, payment);
            } else {
                act(branch, payment);
            }
            assert_eq!(branch.object_view(plotted).zone, Zone::Exile);
            assert!(branch.object_view(child).tapped);
            assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
            assert_eq!(branch.state.pending, Pending::Priority);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}
