//! A paid late-output child must carry its exact completed activation use into
//! the virtual parent world, without paying costs again or banning repeatables.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const W: Mana = [1, 0, 0, 0, 0, 0];
const U: Mana = [0, 1, 0, 0, 0, 0];
const R: Mana = [0, 0, 0, 1, 0, 0];
const G: Mana = [0, 0, 0, 0, 1, 0];
const ZERO: Mana = [0; 6];
const BASE: ActivatedAbility = ActivatedAbility {
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
    program: &[Instruction::AddMana(ManaProduction::Choice {
        mask: 11,
        amount: 1,
    })],
    sorcery: false,
    once_per_turn: true,
};
const ONCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[BASE],
    ..CardRules::EMPTY
};
const EXHAUST: CardRules = CardRules {
    activated: &[ActivatedAbility {
        exhaust: true,
        once_per_turn: false,
        ..BASE
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..ONCE
};
const REPEAT: CardRules = CardRules {
    activated: &[ActivatedAbility {
        once_per_turn: false,
        ..BASE
    }],
    ..ONCE
};
fn direct(card: CardId, payment: Mana, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment,
        counter_payment: Counters::default(),
        production,
    }
}
fn announce(card: CardId, production: Mana) -> Action {
    Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production,
    }
}
fn choices(g: &Game) -> Vec<Mana> {
    let mut values = g
        .legal_actions()
        .into_iter()
        .filter_map(|a| match a {
            Action::ChooseMana { production } => Some(production),
            _ => None,
        })
        .collect::<Vec<_>>();
    values.sort();
    values
}
fn blue_parent(g: &mut Game, p: Player) -> CardId {
    let spell = put(g, p, "Unending Whisper", Zone::Hand);
    g.state.players[p.index()].mana = W;
    act(g, Action::AnnounceCast { card: spell });
    spell
}
fn late_root(p: Player, source_name: &str) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let source = put(&mut g, p, source_name, Zone::Battlefield);
    let spell = blue_parent(&mut g, p);
    act(&mut g, direct(source, W, ZERO));
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    assert_eq!(
        g.state.players[p.index()].mana,
        ZERO,
        "the child already paid W once"
    );
    (g, source, spell)
}
fn finish_blue(g: &mut Game, p: Player, spell: CardId) {
    pay(g);
    settle(g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn paid_choice_once_rejects_fictitious_second_conversion() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ONCE)]);
    for p in [Player::P0, Player::P1] {
        let (mut g, source, spell) = late_root(p, "Grizzly Bears");
        assert_eq!(
            choices(&g),
            vec![U],
            "paid once-only use must constrain the parent promise"
        );
        assert_eq!(g.observe(p).mana_choices, vec![U]);
        reject(&mut g, Action::ChooseMana { production: R });
        act(&mut g, Action::ChooseMana { production: U });
        assert_eq!(g.state.objects[source as usize].used_abilities, 1);
        finish_blue(&mut g, p, spell);
    }
}

#[test]
fn paid_choice_exhaust_rejects_fictitious_second_use_including_autarch_reuse() {
    const REUSE_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ReuseExhaustBeforeFirstActivationOwnTurn],
    }];
    const REUSE: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        static_groups: REUSE_GROUPS,
        extras: &CardExtras {
            static_ability_groups: REUSE_GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules =
        fixture_rules::activate(&[(Card::GrizzlyBears, &EXHAUST), (Card::GoblinPiker, &REUSE)]);
    for p in [Player::P0, Player::P1] {
        for old_use in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            if old_use {
                put(&mut g, p, "Goblin Piker", Zone::Battlefield);
                g.state.objects[source as usize].used_abilities = 1;
            }
            let spell = blue_parent(&mut g, p);
            act(&mut g, direct(source, W, ZERO));
            assert_eq!(choices(&g), vec![U]);
            reject(&mut g, Action::ChooseMana { production: R });
            act(&mut g, Action::ChooseMana { production: U });
            assert!(
                g.state.players[p.index()]
                    .turn_facts
                    .exhaust_activated_this_turn
            );
            finish_blue(&mut g, p, spell);
        }
    }
}

#[test]
fn paid_choice_actual_jeskai_devotee_preserves_parent_completion_and_commit_timing() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source, spell) = late_root(p, "Jeskai Devotee");
        assert_eq!(
            g.state.objects[source as usize].used_abilities, 0,
            "actual commit timing stays after the owned output choice"
        );
        assert_eq!(choices(&g), vec![U]);
        reject(&mut g, Action::ChooseMana { production: R });
        act(&mut g, Action::ChooseMana { production: U });
        assert_eq!(g.state.objects[source as usize].used_abilities, 1);
        reject(&mut g, direct(source, U, U));
        finish_blue(&mut g, p, spell);
        assert_eq!(g.state.players[p.index()].mana, ZERO);
    }
}

#[test]
fn paid_choice_repeatable_converter_can_complete_a_real_second_conversion() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &REPEAT)]);
    for p in [Player::P0, Player::P1] {
        let (mut g, source, spell) = late_root(p, "Grizzly Bears");
        assert!(
            choices(&g).contains(&R),
            "completed repeatable child is not an active ancestor"
        );
        act(&mut g, Action::ChooseMana { production: R });
        act(&mut g, direct(source, R, U));
        finish_blue(&mut g, p, spell);
        assert_eq!(g.state.players[p.index()].mana, ZERO);
    }
}

#[test]
fn paid_choice_nested_ancestor_uses_distinct_resources_instead_of_reusing_the_child() {
    const CHILD: CardRules = CardRules {
        activated: &[ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 31,
                amount: 1,
            })],
            ..BASE
        }],
        ..ONCE
    };
    const OUTER: CardRules = CardRules {
        effect: Effect::Creature(2, 1),
        activated: &[ActivatedAbility {
            once_per_turn: false,
            cost: ActivationCost {
                colored: G,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(U))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules =
        fixture_rules::activate(&[(Card::GrizzlyBears, &CHILD), (Card::GoblinPiker, &OUTER)]);
    for p in [Player::P0, Player::P1] {
        for extra in [false, true] {
            let mut g = scenario(p);
            let child = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let outer = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
            let forest = extra.then(|| put(&mut g, p, "Forest", Zone::Battlefield));
            let spell = blue_parent(&mut g, p);
            act(&mut g, announce(outer, U));
            act(&mut g, direct(child, W, ZERO));
            assert_eq!(choices(&g).contains(&R), extra);
            if extra {
                act(&mut g, Action::ChooseMana { production: R });
                act(
                    &mut g,
                    Action::TapMana {
                        card: forest.unwrap(),
                    },
                );
            } else {
                assert_eq!(choices(&g), vec![G]);
                reject(&mut g, Action::ChooseMana { production: R });
                act(&mut g, Action::ChooseMana { production: G });
            }
            pay(&mut g);
            finish_blue(&mut g, p, spell);
            assert_eq!(
                g.state.players[p.index()].mana,
                if extra { R } else { ZERO }
            );
        }
    }
}

#[test]
fn paid_choice_borrowed_use_belongs_to_the_exact_borrowed_slot() {
    const DONOR: CardRules = CardRules {
        effect: Effect::Permanent,
        ..ONCE
    };
    let _rules = fixture_rules::activate(&[(Card::from_name("Gilded Lotus").unwrap(), &DONOR)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let donor = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let forge = put(&mut g, p, "Territory Forge", Zone::Hand);
        cast(&mut g, forge, None);
        for _ in 0..64 {
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
                act(&mut g, target);
            } else if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else {
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
        }
        assert_eq!(g.object_view(donor).zone, Zone::Exile);
        let spell = blue_parent(&mut g, p);
        let exiled = g.object_ref(donor);
        act(
            &mut g,
            Action::ActivateBorrowedMana {
                card: forge,
                exiled,
                ability: 0,
                counter_payment: Counters::default(),
                payment: W,
                production: ZERO,
            },
        );
        assert_eq!(choices(&g), vec![U]);
        reject(&mut g, Action::ChooseMana { production: R });
        act(&mut g, Action::ChooseMana { production: U });
        assert_eq!(
            g.state.objects[forge as usize].used_abilities, 0,
            "borrowed receipt must not consume an unrelated printed slot"
        );
        finish_blue(&mut g, p, spell);
    }
}

#[test]
fn paid_choice_self_grant_receipts_are_independent_per_acquisition() {
    const TEMPLATE: CardRules = CardRules {
        extras: &CardExtras {
            self_granted_activation_templates: &[0],
            ..CardExtras::EMPTY
        },
        ..ONCE
    };
    let _rules = fixture_rules::activate(&[(Card::LlanowarElves, &TEMPLATE)]);
    for p in [Player::P0, Player::P1] {
        for second in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let object = g.object_ref(source);
            let stamp = g.next_role_timestamp();
            g.grant_recipient_ability(
                object,
                Card::LlanowarElves,
                SelfGrantedAbilityKind::Activated { ability: 0 },
                stamp,
                None,
            );
            let other_stamp = second.then(|| {
                let t = g.next_role_timestamp();
                g.grant_recipient_ability(
                    object,
                    Card::LlanowarElves,
                    SelfGrantedAbilityKind::Activated { ability: 0 },
                    t,
                    None,
                );
                t
            });
            let spell = blue_parent(&mut g, p);
            let start = |timestamp, production| Action::AnnounceSelfGrantedMana {
                source: object,
                grant_timestamp: timestamp,
                counter_payment: Counters::default(),
                production,
            };
            act(&mut g, start(stamp, ZERO));
            pay(&mut g);
            assert_eq!(choices(&g).contains(&R), second);
            if let Some(other) = other_stamp {
                act(&mut g, Action::ChooseMana { production: R });
                reject(&mut g, start(stamp, U));
                act(&mut g, start(other, U));
                pay(&mut g);
            } else {
                assert_eq!(choices(&g), vec![U]);
                reject(&mut g, Action::ChooseMana { production: R });
                act(&mut g, Action::ChooseMana { production: U });
            }
            assert_eq!(g.state.objects[source as usize].used_abilities, 0);
            finish_blue(&mut g, p, spell);
        }
    }
}

#[test]
fn paid_choice_owned_root_shares_one_legal_domain_and_restores_private_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Jeskai Devotee", Zone::Battlefield);
        let island = put(&mut g, p, "Island", Zone::Battlefield);
        let secret = put(&mut g, p, "Cancel", Zone::Hand);
        let spell = blue_parent(&mut g, p);
        act(&mut g, direct(source, W, ZERO));
        let action = Action::ChooseMana { production: U };
        assert!(
            choices(&g).contains(&R),
            "a distinct Island can fund the parent after wrong-color output"
        );
        let mut observed = g.observe(p).mana_choices;
        observed.sort();
        assert_eq!(observed, choices(&g));
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut sibling = g.clone();
        drop(g);
        assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
        assert_eq!(raw.snapshot(), saved);
        reject(&mut raw, Action::ChooseMana { production: G });
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        act(&mut sibling, Action::ChooseMana { production: R });
        act(&mut sibling, Action::TapMana { card: island });
        for child in [&mut raw, &mut indexed, &mut sibling] {
            finish_blue(child, p, spell);
            assert_eq!(child.state.objects[source as usize].used_abilities, 1);
            assert!(child.observe(p).objects.iter().any(|o| o.id == secret));
            assert!(
                !child
                    .observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == secret)
            );
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert!(!raw.object_view(island).tapped);
        assert!(sibling.object_view(island).tapped);
        assert_eq!(raw.state.players[p.index()].mana, ZERO);
        assert_eq!(sibling.state.players[p.index()].mana, R);
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        finish_blue(&mut indexed, p, spell);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

#[test]
fn paid_choice_receipt_distinguishes_native_copy_origin_and_departed_incarnation() {
    use rustmage_core::engine::ability_world::CharacteristicWorld;
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXHAUST)]);
    for p in [Player::P0, Player::P1] {
        for changed in [false, true] {
            let mut g = scenario(p);
            let donor = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let recipe = g.copiable_recipe(g.object_ref(donor));
            g.apply_temporary_copy(g.object_ref(source), recipe, CopyDeadline::EndOfTurn);
            blue_parent(&mut g, p);
            act(&mut g, direct(source, W, ZERO));
            let receipt = g.state.announcing.as_ref().unwrap().clone();
            if changed {
                g.apply_temporary_copy(g.object_ref(source), recipe, CopyDeadline::EndOfTurn);
            }
            let mut world = g.paid_mana_parent_world(&receipt);
            world.project_paid_mana_use(&receipt.item);
            assert_eq!(
                world.object(source).used_abilities,
                u64::from(!changed),
                "only the original copied ability origin consumes its printed slot"
            );
            assert!(world.exhaust_activated_this_turn(p));
        }
        let (mut g, source, _) = late_root(p, "Grizzly Bears");
        let receipt = g.state.announcing.as_ref().unwrap().clone();
        g.move_object(source, Zone::Graveyard);
        let mut departed = g.paid_mana_parent_world(&receipt);
        departed.project_paid_mana_use(&receipt.item);
        assert!(
            departed.exhaust_activated_this_turn(p),
            "player history survives departure"
        );
        g.move_object(source, Zone::Battlefield);
        let mut returned = g.paid_mana_parent_world(&receipt);
        returned.project_paid_mana_use(&receipt.item);
        assert_eq!(
            returned.object(source).used_abilities,
            0,
            "new incarnation has no old use"
        );
        assert!(returned.exhaust_activated_this_turn(p));
    }
}
