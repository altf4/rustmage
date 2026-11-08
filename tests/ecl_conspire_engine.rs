//! Conspire instances share the ordinary ordered payment and spell-copy engines.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const GRANT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NoncreatureConspire],
    }],
    ..CardRules::EMPTY
};
fn setup(p: Player, instances: usize) -> (Game, CardId, Vec<ObjectRef>) {
    let mut g = scenario(p);
    for _ in 0..instances {
        put(&mut g, p, "Short Bow", Zone::Battlefield);
    }
    let mut donors = Vec::new();
    for _ in 0..instances * 2 {
        let donor = put(&mut g, p, "Fog Bank", Zone::Battlefield);
        g.state.objects[donor as usize].summoning_sick = true;
        donors.push(g.object_ref(donor));
    }
    let source = put(&mut g, p, "Divination", Zone::Hand);
    (g, source, donors)
}
fn finish_cast(g: &mut Game) {
    for _ in 0..24 {
        if g.state.announcing.is_none() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::PayCosts { .. }
                    | Action::BeginOrderedCosts
                    | Action::ChooseCastCost { .. }
                    | Action::PayConspire { .. }
            )
        });
    }
    panic!("conspire payment bound");
}
fn finish(g: &mut Game) {
    for _ in 0..80 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
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
    panic!("conspire resolution bound");
}
#[test]
fn each_conspire_instance_taps_two_distinct_matching_creatures_and_copies_once() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    for p in [Player::P0, Player::P1] {
        for instances in [1, 2] {
            let (mut g, source, donors) = setup(p, instances);
            act(&mut g, Action::AnnounceCast { card: source });
            for _ in 0..instances {
                act(&mut g, Action::ChooseConspire { pay: true });
            }
            finish_cast(&mut g);
            assert!(
                donors
                    .iter()
                    .all(|r| g.state.objects[r.card as usize].tapped)
            );
            finish(&mut g);
            assert_eq!(g.hand_count(p), 2 * (instances + 1));
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        }
    }
}
#[test]
fn conspire_rejects_duplicate_enemy_colorless_and_nonmatching_payment_creatures() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    let p = Player::P0;
    let (mut g, source, donors) = setup(p, 1);
    let wrong = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Fog Bank", Zone::Battlefield);
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseConspire { pay: true });
    act(&mut g, Action::BeginOrderedCosts);
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
        .position(|r| matches!(r.clause, CastCostClause::Conspire(_)))
        .unwrap() as u16;
    act(&mut g, Action::ChooseCastCost { clause });
    for second in [donors[0], g.object_ref(wrong), g.object_ref(enemy)] {
        reject(
            &mut g,
            Action::PayConspire {
                objects: [donors[0], second],
            },
        );
    }
    act(
        &mut g,
        Action::PayConspire {
            objects: [donors[0], donors[1]],
        },
    );
    finish_cast(&mut g);
    finish(&mut g);
    assert_eq!(g.hand_count(p), 4);
}
#[test]
fn insufficient_creatures_offer_decline_and_creature_spells_receive_no_conspire() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    let (mut g, source, donors) = setup(Player::P1, 1);
    g.state.objects[donors[0].card as usize].tapped = true;
    act(&mut g, Action::AnnounceCast { card: source });
    assert!(
        g.legal_actions()
            .contains(&Action::ChooseConspire { pay: false })
    );
    reject(&mut g, Action::ChooseConspire { pay: true });
    act(&mut g, Action::ChooseConspire { pay: false });
    finish_cast(&mut g);
    finish(&mut g);
    assert_eq!(g.hand_count(Player::P1), 2);
    let creature = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
    cast(&mut g, creature, None);
    finish(&mut g);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
}
#[test]
fn independent_conspire_declarations_survive_raw_index_and_restore() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    let (mut parent, source, _) = setup(Player::P0, 1);
    act(&mut parent, Action::AnnounceCast { card: source });
    let saved = parent.snapshot();
    let action = Action::ChooseConspire { pay: true };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(Player::P0, indexed.revision(), index)
        .unwrap();
    act(&mut raw, action);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    finish_cast(&mut raw);
    finish(&mut raw);
    finish_cast(&mut indexed);
    finish(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::ChooseConspire { pay: false });
    finish_cast(&mut indexed);
    finish(&mut indexed);
    assert_eq!(indexed.hand_count(Player::P0), 2);
    assert_eq!(raw.hand_count(Player::P0), 4);
}
#[test]
fn conspire_can_tap_a_creature_before_the_printed_sacrifice_cost_but_not_after() {
    use rustmage_core::cards::authored::CardExtras;
    const SAC: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::Draw(PlayerSet::Controller, 1)],
        ),
        extras: &CardExtras {
            additional_cost: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
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
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &GRANT),
        (Card::Divination, &SAC),
    ]);
    let (mut g, source, donors) = setup(Player::P0, 1);
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseConspire { pay: true });
    if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
        act(&mut g, Action::ChooseAdditionalCost { object: donors[0] });
    }
    act(&mut g, Action::BeginOrderedCosts);
    let clauses = &g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses;
    let conspire = clauses
        .iter()
        .position(|r| matches!(r.clause, CastCostClause::Conspire(_)))
        .unwrap() as u16;
    let additional = clauses
        .iter()
        .position(|r| matches!(r.clause, CastCostClause::Additional))
        .unwrap() as u16;
    reject(&mut g, Action::ChooseCastCost { clause: additional });
    act(&mut g, Action::ChooseCastCost { clause: conspire });
    act(
        &mut g,
        Action::PayConspire {
            objects: [donors[0], donors[1]],
        },
    );
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::ChooseCastCost { .. }
                    | Action::ChooseAdditionalCost { .. }
                    | Action::PayCosts { .. }
            )
        });
    }
    assert_eq!(
        g.state.objects[donors[0].card as usize].zone,
        Zone::Graveyard
    );
    assert!(g.state.objects[donors[1].card as usize].tapped);
    finish(&mut g);
    assert_eq!(g.hand_count(Player::P0), 2);
}
#[test]
fn conspire_copy_keeps_the_cast_spell_after_original_counter_and_provider_departure() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    let (mut g, source, _) = setup(Player::P0, 1);
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseConspire { pay: true });
    finish_cast(&mut g);
    let original = g
        .state
        .stack
        .iter()
        .find(|s| s.kind == StackKind::Spell)
        .unwrap()
        .id;
    assert!(g.counter_stack_item(original, false));
    let provider = g
        .state
        .objects
        .iter()
        .position(|o| o.card == Card::from_name("Short Bow").unwrap())
        .unwrap() as CardId;
    g.move_object(provider, Zone::Graveyard);
    finish(&mut g);
    assert_eq!(g.hand_count(Player::P0), 2);
}
#[test]
fn convoke_and_conspire_cannot_spend_the_same_matching_creatures() {
    use rustmage_core::cards::authored::CardExtras;
    const CONVOKE: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::Draw(PlayerSet::Controller, 1)],
        ),
        extras: &CardExtras {
            has_convoke: true,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &GRANT),
        (Card::Divination, &CONVOKE),
    ]);
    let (mut g, source, donors) = setup(Player::P0, 1);
    let a = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseConspire { pay: true });
    act(&mut g, Action::BeginConvoke);
    let mana = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == CastCostClause::Mana)
        .unwrap() as u16;
    act(&mut g, Action::ChooseCastCost { clause: mana });
    reject(
        &mut g,
        Action::ChooseConvoker {
            object: donors[0],
            contribution: convoke::ConvokeContribution::Generic,
        },
    );
    for id in [a, b] {
        let object = g.object_ref(id);
        act(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: convoke::ConvokeContribution::Generic,
            },
        );
    }
    finish_cast(&mut g);
    finish(&mut g);
    assert_eq!(g.hand_count(Player::P0), 2);
}

#[test]
fn conspire_can_be_funded_by_mana_abilities_that_recolor_its_future_donors() {
    const WICKER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
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
            program: &[Instruction::AddMana(
                ManaProduction::AnyColorChangingSource(1),
            )],
            sorcery: false,
            once_per_turn: true,
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &GRANT),
        (Card::GrizzlyBears, &WICKER),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let first = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 3];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseConspire { pay: true });
    let action = |card, production| Action::ActivateMana {
        card,
        ability: 0,
        payment: [0, 0, 0, 0, 0, 1],
        production,
        counter_payment: Counters::default(),
    };
    reject(&mut g, action(first, [0, 0, 0, 1, 0, 0]));
    act(&mut g, action(first, [0, 1, 0, 0, 0, 0]));
    act(&mut g, action(second, [0, 1, 0, 0, 0, 0]));
    finish_cast(&mut g);
    assert!(g.state.objects[first as usize].tapped && g.state.objects[second as usize].tapped);
    finish(&mut g);
    assert_eq!(g.hand_count(Player::P0), 4);
}

#[test]
fn conspire_large_mixed_color_board_enumerates_exact_pairs_and_owned_payments() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    let mut eligible = Vec::new();
    for _ in 0..100 {
        let id = put(&mut g, p, "Fog Bank", Zone::Battlefield);
        g.state.objects[id as usize].copy_kind = CopyKind::Token;
        g.state.objects[id as usize].summoning_sick = true;
        eligible.push(g.object_ref(id));
    }
    let mut ineligible = Vec::new();
    for index in 0..100 {
        let (owner, name) = match index % 4 {
            0 => (p, "Bear Cub"),
            1 => (p, "Fanatical Firebrand"),
            2 => (p.other(), "Fog Bank"),
            _ => (p, "Fog Bank"),
        };
        let id = put(&mut g, owner, name, Zone::Battlefield);
        let object = &mut g.state.objects[id as usize];
        object.copy_kind = CopyKind::Token;
        object.tapped = index % 4 == 3;
        ineligible.push(g.object_ref(id));
    }
    let spell = put(&mut g, p, "Divination", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseConspire { pay: true });
    act(&mut g, Action::BeginOrderedCosts);
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
        .position(|r| matches!(r.clause, CastCostClause::Conspire(_)))
        .unwrap() as u16;
    act(&mut g, Action::ChooseCastCost { clause });
    let started = std::time::Instant::now();
    let actions = g.legal_actions();
    eprintln!(
        "200-token Conspire action enumeration: {:?}",
        started.elapsed()
    );
    let pairs: Vec<_> = actions
        .iter()
        .filter_map(|a| match a {
            Action::PayConspire { objects } => Some(*objects),
            _ => None,
        })
        .collect();
    assert_eq!(pairs.len(), 100 * 99 / 2);
    assert!(pairs.iter().all(|pair| pair[0].card < pair[1].card
        && eligible.contains(&pair[0])
        && eligible.contains(&pair[1])));
    let unique: std::collections::HashSet<_> = pairs.iter().copied().collect();
    assert_eq!(unique.len(), pairs.len());
    for invalid in ineligible.iter().step_by(25) {
        reject(
            &mut g,
            Action::PayConspire {
                objects: [eligible[0], *invalid],
            },
        );
    }
    reject(
        &mut g,
        Action::PayConspire {
            objects: [eligible[0], eligible[0]],
        },
    );
    reject(
        &mut g,
        Action::PayConspire {
            objects: [eligible[1], eligible[0]],
        },
    );
    let stale = ObjectRef {
        generation: eligible[1].generation + 1,
        ..eligible[1]
    };
    reject(
        &mut g,
        Action::PayConspire {
            objects: [eligible[0], stale],
        },
    );
    let action = Action::PayConspire {
        objects: [eligible[98], eligible[99]],
    };
    let index = actions.iter().position(|a| *a == action).unwrap();
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    drop(g);
    act(&mut raw, action.clone());
    indexed.apply_index(p, indexed.revision(), index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for (i, object) in eligible.iter().enumerate() {
        assert_eq!(raw.state.objects[object.card as usize].tapped, i >= 98);
    }
    finish_cast(&mut raw);
    finish(&mut raw);
    assert_eq!(raw.hand_count(p), 4);
    indexed.restore(&saved).unwrap();
    assert!(
        eligible
            .iter()
            .all(|object| !indexed.state.objects[object.card as usize].tapped)
    );
    act(&mut indexed, action);
    finish_cast(&mut indexed);
    finish(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn conspire_exact_pair_does_not_depend_on_mana_resource_insertion_order() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &GRANT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    // The plain creature is earlier in arena order; the mana creature is retained
    // first in the funding world. Both exact refs must form the same legal pair.
    let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseConspire { pay: true });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    act(&mut g, Action::BeginOrderedCosts);
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
        .position(|r| matches!(r.clause, CastCostClause::Conspire(_)))
        .unwrap() as u16;
    act(&mut g, Action::ChooseCastCost { clause });
    let objects = [g.object_ref(first), g.object_ref(second)];
    let pairs: Vec<_> = g
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::PayConspire { .. }))
        .collect();
    assert_eq!(pairs, vec![Action::PayConspire { objects }]);
    act(&mut g, Action::PayConspire { objects });
    finish_cast(&mut g);
    finish(&mut g);
    assert!(g.state.objects[first as usize].tapped && g.state.objects[second as usize].tapped);
    assert_eq!(g.stats(first).power, 8);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn conspire_intent_preserves_the_selected_hybrid_cost_when_a_donor_can_make_other_mana() {
    const PRODUCER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 2, 0, 0,
            ]))],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &GRANT),
        (Card::from_name("Fog Bank").unwrap(), &PRODUCER),
    ]);
    let p = Player::P0;
    let mut root = scenario(p);
    put(&mut root, p, "Short Bow", Zone::Battlefield);
    let first = put(&mut root, p, "Aegis Turtle", Zone::Battlefield);
    let producer = put(&mut root, p, "Fog Bank", Zone::Battlefield);
    let spell = put(&mut root, p, "Teach by Example", Zone::Hand);
    root.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
    act(&mut root, Action::AnnounceCast { card: spell });
    for pay_red in [false, true] {
        let mut g = root.clone();
        let payment = if pay_red {
            [0, 0, 0, 2, 0, 0]
        } else {
            [0, 2, 0, 0, 0, 0]
        };
        let cost = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cost_options()
            .position(|cost| cost.colored == payment && cost.generic == 0)
            .unwrap() as u16;
        act(&mut g, Action::ChooseSpellCost { cost });
        assert!(matches!(g.state.pending, Pending::Conspire { .. }));
        if pay_red {
            reject(&mut g, Action::ChooseConspire { pay: true });
            act(&mut g, Action::ChooseConspire { pay: false });
            act(
                &mut g,
                Action::ActivateMana {
                    card: producer,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: payment,
                },
            );
            act(&mut g, Action::PayCosts { payment });
            assert!(!g.object_view(first).tapped && g.object_view(producer).tapped);
        } else {
            act(&mut g, Action::ChooseConspire { pay: true });
            finish_cast(&mut g);
            assert!(g.object_view(first).tapped && g.object_view(producer).tapped);
        }
        let item = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell)
            .unwrap();
        assert_eq!(item.cast.unwrap().payment, payment);
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}
