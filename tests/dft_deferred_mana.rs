//! Deferred indexed mana announcement preflight couples every enclosing payment.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BLUE: Mana = [0, 3, 0, 0, 0, 0];
const ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
    sorcery: false,
    once_per_turn: false,
};
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ABILITY],
    ..CardRules::EMPTY
};
const GREEN_SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 2, 0,
        ]))],
        ..ABILITY
    }],
    ..CardRules::EMPTY
};
const LOSS_SOURCE: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        ..ABILITY
    }],
    ..SOURCE
};
const UNPAYABLE_SOURCE: CardRules = CardRules {
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        ..ABILITY
    }],
    ..SOURCE
};
fn announce(card: CardId, production: Mana) -> Action {
    Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn nested_costed_mana_announcements_reject_all_active_ancestor_instances() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SOURCE),
        (Card::GoblinPiker, &GREEN_SOURCE),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: spell });
        let before = g.snapshot();
        let revision = g.revision();
        assert!(g.legal_actions().contains(&announce(a, BLUE)));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        act(&mut g, announce(a, BLUE));
        act(&mut g, announce(b, [0, 0, 0, 0, 2, 0]));
        assert_eq!(g.state.mana_announcement_stack.as_ref().unwrap().len(), 2);
        act(&mut g, Action::TapMana { card: forest });
        // Both live sources remain untapped and can afford their generic cost;
        // only their exact active instances prohibit recursive activation.
        assert!(!g.object_view(a).tapped);
        assert!(!g.object_view(b).tapped);
        reject(&mut g, announce(a, BLUE));
        reject(&mut g, announce(b, [0, 0, 0, 0, 2, 0]));
        reject(
            &mut g,
            Action::ActivateMana {
                card: a,
                ability: 0,
                counter_payment: Counters::default(),
                payment: [0, 0, 0, 0, 1, 0],
                production: BLUE,
            },
        );
        pay(&mut g); // B -> two green, restore A.
        pay(&mut g); // A -> three blue, restore the spell.
        assert!(g.state.mana_announcement_stack.is_none());
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 1);
    }
}

#[test]
fn fundable_child_is_rejected_when_its_completion_would_strand_the_parent() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LOSS_SOURCE)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 2, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, announce(source, [0, 0, 0, 0, 1, 0]));
    assert!(g.state.mana_announcement_stack.is_none());
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
}

#[test]
fn payable_parent_does_not_authorize_a_child_with_an_unpayable_color() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &UNPAYABLE_SOURCE)]);
    let mut g = scenario(Player::P1);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, Player::P1, "Divination", Zone::Hand);
    g.state.players[1].mana = BLUE;
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, announce(source, BLUE));
    assert!(!g.object_view(source).tapped);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 2);
}

#[test]
fn a_resolved_child_can_be_activated_again_while_funding_its_restored_parent() {
    const TWO_BLUE: CardRules = CardRules {
        activated: &[ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 2, 0, 0, 0, 0,
            ]))],
            ..ABILITY
        }],
        ..SOURCE
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TWO_BLUE)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let forest = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    act(&mut g, Action::AnnounceCast { card: spell });
    // Completing the first activation leaves only two mana. Its preflight must
    // permit a fresh activation after resolution to finish the enclosing spell.
    act(&mut g, announce(source, [0, 2, 0, 0, 0, 0]));
    act(&mut g, Action::TapMana { card: forest });
    pay(&mut g);
    assert!(g.state.mana_announcement_stack.is_none());
    assert_eq!(g.state.players[0].mana, [0, 2, 0, 0, 0, 0]);
    act(&mut g, announce(source, [0, 2, 0, 0, 0, 0]));
    pay(&mut g);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn nested_parent_tap_cost_keeps_its_exact_source_distinct_from_an_identical_mana_source() {
    const A: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[
            ActivatedAbility {
                cost: ActivationCost {
                    colored: [0, 1, 0, 0, 0, 0],
                    generic: 0,
                    mana_component: true,
                    tap: true,
                    ..ActivationCost::ZERO
                },
                program: &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
                ..ABILITY
            },
            ActivatedAbility {
                cost: ActivationCost {
                    tap: true,
                    ..ActivationCost::ZERO
                },
                program: &[Instruction::AddMana(ManaProduction::Fixed([
                    0, 0, 0, 0, 1, 0,
                ]))],
                ..ABILITY
            },
        ],
        ..CardRules::EMPTY
    };
    const B: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 1, 0, 0, 0, 0,
            ]))],
            ..ABILITY
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &A), (Card::GoblinPiker, &B)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let c = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: a,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, announce(b, [0, 1, 0, 0, 0, 0]));
        // Repeating an outer boolean query must not reuse visited successes from
        // another action/allocation candidate. Its immutable world stays unmodified.
        {
            use rustmage_core::engine::funding::FundingState;
            let goal = g.funding_goal(g.state.announcing.as_ref().unwrap());
            let world = FundingState::from_game(&g, &goal);
            assert!(world.continuation_memo.is_none());
            for _ in 0..2 {
                assert!(world.reachable(&goal, &mut std::collections::HashSet::new()));
            }
            assert!(world.continuation_memo.is_none());
            let mut projected = world.clone();
            projected.prepare_entry_projection();
            assert!(projected.scoped_continuation_query().is_none());
            assert!(projected.reachable(&goal, &mut std::collections::HashSet::new()));
        }
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
            let mana = |card| Action::ActivateMana {
                card,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            };
            reject(branch, mana(a));
            let action = mana(c);
            if i == 1 {
                let index = branch
                    .legal_actions()
                    .iter()
                    .position(|a| *a == action)
                    .unwrap();
                branch
                    .apply_index(branch.actor().unwrap(), branch.revision(), index)
                    .unwrap();
            } else {
                act(branch, action);
            }
            pay(branch);
            pay(branch);
            settle(branch);
            assert_eq!(branch.counter_count(a, CounterKind::PlusOne), 1);
            assert!(
                branch.object_view(a).tapped
                    && branch.object_view(b).tapped
                    && branch.object_view(c).tapped
            );
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}

#[test]
fn a_basic_land_with_an_acquired_mana_template_is_not_interchanged_with_a_plain_basic() {
    use rustmage_core::cards::authored::CardExtras;
    const TEMPLATE: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 1, 0, 0, 0, 0,
            ]))],
            ..ABILITY
        }],
        extras: &CardExtras {
            self_granted_activation_templates: &[0],
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    const CONVERTER: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 1, 0,
            ]))],
            ..ABILITY
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::LlanowarElves, &TEMPLATE),
        (Card::GoblinPiker, &CONVERTER),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Forest", Zone::Battlefield);
        let c = put(&mut g, p, "Forest", Zone::Battlefield);
        let b = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let stamp = g.next_role_timestamp();
        let recipient = g.object_ref(a);
        g.grant_recipient_ability(
            recipient,
            Card::LlanowarElves,
            SelfGrantedAbilityKind::Activated { ability: 0 },
            stamp,
            None,
        );
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, announce(b, [0, 0, 0, 0, 1, 0]));
        reject(&mut g, Action::TapMana { card: a });
        act(&mut g, Action::TapMana { card: c });
        pay(&mut g);
        act(
            &mut g,
            Action::AnnounceSelfGrantedMana {
                source: recipient,
                grant_timestamp: stamp,
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        pay(&mut g);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 1);
    }
}
