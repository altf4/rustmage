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

#[test]
fn immutable_copy_profiles_preserve_exact_recipients_and_allow_only_scoped_reuse() {
    use rustmage_core::engine::funding::FundingState;
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &SOURCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let producer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let copied = put(&mut g, p, "Forest", Zone::Battlefield);
        let plain = put(&mut g, p, "Forest", Zone::Battlefield);
        let temporary = put(&mut g, p, "Mountain", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        let copied_ref = g.object_ref(copied);
        let temporary_ref = g.object_ref(temporary);
        let mut recipe = CopyRecipe::new(Card::Forest);
        recipe.overrides = Some(CopyOverrides {
            name: Some("Immutable copied Forest"),
            ..CopyOverrides::NONE
        });
        g.set_copy_recipe(copied_ref, recipe);
        g.apply_temporary_copy(
            temporary_ref,
            CopyRecipe::new(Card::Forest),
            CopyDeadline::EndOfTurn,
        );
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, announce(producer, BLUE));
        {
            let goal = g.funding_goal(g.state.announcing.as_ref().unwrap());
            let world = FundingState::from_game(&g, &goal);
            assert_eq!(world.spell_lot_demand, Some(4));
            let scoped = world.scoped_continuation_query().unwrap();
            let exact = &scoped.continuation_memo.as_ref().unwrap().exact;
            assert!(exact.contains(&copied));
            assert!(exact.contains(&temporary));
            assert!(!exact.contains(&plain));
            for _ in 0..2 {
                assert!(world.reachable(&goal, &mut std::collections::HashSet::new()));
            }
            assert!(world.continuation_memo.is_none());
            let mut changed = world.clone();
            changed.copy_recipes.as_mut().unwrap()[0].recipe.overrides = None;
            assert!(!changed.continuation_memo_safe());
            assert!(changed.scoped_continuation_query().is_none());
            assert!(changed.reachable(&goal, &mut std::collections::HashSet::new()));
            let mut projected = world.clone();
            projected.prepare_entry_projection();
            assert!(projected.scoped_continuation_query().is_none());
        }
        // Even an inactive source-bound layer excludes early reuse. A later
        // projection could expose it, so checking only the active layer fails.
        let mut source_bound = g.clone();
        let source = source_bound.object_ref(plain);
        source_bound.apply_temporary_copy(
            temporary_ref,
            CopyRecipe::new(Card::Forest),
            CopyDeadline::WhileSource { source },
        );
        source_bound.apply_temporary_copy(
            temporary_ref,
            CopyRecipe::new(Card::Forest),
            CopyDeadline::EndOfTurn,
        );
        let goal = source_bound.funding_goal(source_bound.state.announcing.as_ref().unwrap());
        assert!(
            FundingState::from_game(&source_bound, &goal)
                .scoped_continuation_query()
                .is_none()
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
            let action = Action::TapMana { card: plain };
            if i == 1 {
                let index = branch
                    .legal_actions()
                    .iter()
                    .position(|candidate| *candidate == action)
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
            assert_eq!(branch.state.players[p.index()].cards_drawn_this_turn, 2);
            assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
            assert!(branch.object_view(plain).tapped);
            assert!(!branch.object_view(copied).tapped);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}

const ARTIFACT_MANA_SOURCE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ABILITY],
    ..CardRules::EMPTY
};

fn nonmana_parent_token(g: &mut Game, p: Player, identity: &str) -> CardId {
    let token = Card::from_name(identity).unwrap();
    g.create_tokens(p, token, 1, false);
    g.state
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| {
            object.card == token && object.controller == p && object.zone == Zone::Battlefield
        })
        .unwrap()
        .0 as CardId
}

fn real_forge_borrowing(g: &mut Game, p: Player, donor: CardId) -> CardId {
    let forge = put(g, p, "Territory Forge", Zone::Hand);
    cast(g, forge, None);
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            break;
        }
        let target = Action::ChooseTarget {
            target: Target::Card(donor),
        };
        if g.legal_actions().contains(&target) {
            act(g, target);
        } else if g.legal_actions().contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        } else {
            act(g, Action::Pass);
        }
    }
    assert_eq!(g.object_view(donor).zone, Zone::Exile);
    forge
}

#[test]
fn a_nonmana_food_parent_keeps_its_source_through_native_and_borrowed_mana_payments() {
    use rustmage_core::engine::funding::FundingState;
    let _scope = fixture_rules::activate(&[(Card::GildedLotus, &ARTIFACT_MANA_SOURCE)]);
    for p in [Player::P0, Player::P1] {
        for borrowed in [false, true] {
            let mut g = scenario(p);
            let donor = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            let producer = if borrowed {
                real_forge_borrowing(&mut g, p, donor)
            } else {
                donor
            };
            let forest = put(&mut g, p, "Forest", Zone::Battlefield);
            let food = nonmana_parent_token(&mut g, p, "@token/283fd83a9aec83a51b736926");
            assert!(g.has_subtype(food, "Food"));
            let old_food = g.object_ref(food);
            let life = g.state.players[p.index()].life;
            let action = if borrowed {
                Action::AnnounceBorrowedMana {
                    card: producer,
                    exiled: g.object_ref(donor),
                    ability: 0,
                    counter_payment: Counters::default(),
                    production: BLUE,
                }
            } else {
                announce(producer, BLUE)
            };
            g.state.players[p.index()].mana = [0; 6];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: food,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let mut unpaid_source = g.clone();
            unpaid_source.state.objects[food as usize].tapped = true;
            reject(&mut unpaid_source, action.clone());
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
                    let index = branch
                        .legal_actions()
                        .iter()
                        .position(|candidate| *candidate == action)
                        .unwrap();
                    branch
                        .apply_index(branch.actor().unwrap(), branch.revision(), index)
                        .unwrap();
                } else {
                    act(branch, action.clone());
                }
                let goal = branch.funding_goal(branch.state.announcing.as_ref().unwrap());
                let world = FundingState::from_game(branch, &goal);
                assert!(world.resources.iter().any(|resource| {
                    resource.id == food && resource.object.generation == old_food.generation
                }));
                assert!(world.reachable(&goal, &mut std::collections::HashSet::new()));
                act(branch, Action::TapMana { card: forest });
                pay(branch);
                pay(branch);
                settle(branch);
                assert!(!branch.on_battlefield(food));
                assert_eq!(branch.state.players[p.index()].life, life + 3);
                assert_eq!(branch.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
            }
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.snapshot(), restored.snapshot());
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(raw.observe(viewer)).unwrap(),
                    serde_json::to_value(indexed.observe(viewer)).unwrap(),
                );
            }
        }
    }
}

#[test]
fn an_outer_discard_receipt_is_retained_without_overwriting_projected_departures() {
    use rustmage_core::engine::funding::FundingState;
    let _scope = fixture_rules::activate(&[(Card::GildedLotus, &ARTIFACT_MANA_SOURCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let producer = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        let discard = put(&mut g, p, "Divination", Zone::Hand);
        let discard_ref = g.object_ref(discard);
        let blood = nonmana_parent_token(&mut g, p, "@token/blood");
        g.state.players[p.index()].mana = [0; 6];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: blood,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseAdditionalCost {
                object: discard_ref,
            },
        );
        act(&mut g, announce(producer, BLUE));
        {
            let head = g.state.announcing.as_ref().unwrap();
            let frames = g.state.mana_announcement_stack.as_ref().unwrap();
            let goal = g.funding_goal(head);
            let mut world = FundingState::from_game(&g, &goal);
            assert!(world.resources.iter().any(|resource| resource.id == blood));
            let at = world
                .resources
                .iter()
                .position(|resource| resource.id == discard)
                .unwrap();
            assert_eq!(world.resources[at].object.zone, Zone::Hand);
            let appended = world
                .resources
                .iter()
                .map(|resource| resource.id)
                .collect::<Vec<_>>();
            // Costs may already have moved an exact object before another
            // parent transition. Installing the chain must keep that world,
            // including its new incarnation, controller and counters.
            world.resources[at].object.zone = Zone::Graveyard;
            world.resources[at].object.generation += 1;
            world.resources[at].object.controller = p.other();
            world.resources[at].object.counters.plus_one = 2;
            let projected = world.resources[at].object.clone();
            world.set_funding_continuation(head, frames, None);
            assert_eq!(world.resources[at].object, projected);
            assert_eq!(
                world
                    .resources
                    .iter()
                    .map(|resource| resource.id)
                    .collect::<Vec<_>>(),
                appended
            );
            assert!(!world.reachable(&goal, &mut std::collections::HashSet::new()));
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
            let action = Action::TapMana { card: forest };
            if i == 1 {
                let index = branch
                    .legal_actions()
                    .iter()
                    .position(|candidate| *candidate == action)
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
            assert_eq!(branch.object_view(discard).zone, Zone::Graveyard);
            assert!(!branch.on_battlefield(blood));
            assert_eq!(branch.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(branch.state.players[p.index()].mana, [0, 2, 0, 0, 0, 0]);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}
