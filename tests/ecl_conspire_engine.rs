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
