//! Evoke is an alternative-cost receipt; mana-spent ETBs use only actual payments.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const EVOKE: ManaCost = ManaCost {
    colored: [0, 0, 0, 0, 1, 0],
    generic: 0,
};
const NORMAL: ManaCost = ManaCost {
    colored: [0, 0, 0, 0, 1, 0],
    generic: 1,
};
const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersManaSpent {
                color: 4,
                minimum: 1,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEvokedEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSourceByController],
        },
    ],
    extras: &CardExtras {
        cost_alternatives: &[EVOKE],
        evoke_costs: &[EVOKE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn finish(g: &mut Game) {
    for _ in 0..80 {
        g.prepare_triggers();
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("evoke resolution bound");
}
fn route(g: &mut Game, source: CardId, cost: ManaCost) {
    act(g, Action::AnnounceCast { card: source });
    let choice = g
        .observe(g.actor().unwrap())
        .cost_choices
        .into_iter()
        .find(|c| c.cost == cost)
        .expect("offered alternative");
    act(g, Action::ChooseSpellCost { cost: choice.index });
    pay(g);
}
#[test]
fn alternative_receipt_and_actual_mana_payment_are_separate_for_both_seats() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    for p in [Player::P0, Player::P1] {
        for (cost, expected) in [(NORMAL, Zone::Battlefield), (EVOKE, Zone::Graveyard)] {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
            let source = put(&mut g, p, "Grizzly Bears", Zone::Hand);
            route(&mut g, source, cost);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().evoked,
                cost == EVOKE
            );
            finish(&mut g);
            assert_eq!(g.state.objects[source as usize].zone, expected);
            assert_eq!(g.state.players[p.index()].life, 22);
        }
    }
}
#[test]
fn linked_counter_cost_preserves_evoke_and_records_only_paid_mana() {
    const LINK: CardRules = CardRules {
        effect: Effect::Creature(2, 3),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LinkedExileCreatureCounterCasting(3)],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &RULES),
        (Card::from_name("Bear Cub").unwrap(), &LINK),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
        let provider = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let donor = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.add_counter(donor, CounterKind::Charge, 3);
        let spell = put(&mut g, p, "Grizzly Bears", Zone::Exile);
        let source = g.object_ref(provider);
        let exiled = g.object_ref(spell);
        g.state
            .linked_choices
            .get_or_insert_default()
            .links
            .push(LinkedExileView {
                source,
                exiled,
                ability_origin: None,
                exiled_turn: g.state.turn,
            });
        act(
            &mut g,
            Action::AnnounceLinkedExileCounterCost {
                card: spell,
                source,
                flash_surcharge: false,
            },
        );
        let choice = g
            .observe(p)
            .cost_choices
            .into_iter()
            .find(|c| c.cost == EVOKE)
            .expect("Evoke route remains available from linked exile");
        act(&mut g, Action::ChooseSpellCost { cost: choice.index });
        for _ in 0..12 {
            match g.state.pending {
                Pending::CastCosts { .. } => {
                    choose(&mut g, |action| matches!(action, Action::ChooseCastCost { .. }));
                }
                Pending::AdditionalCost { .. } => {
                    choose(&mut g, |action| {
                        matches!(action, Action::ChooseCreatureCounterCost { object, .. } if object.card == donor)
                    });
                }
                Pending::PayCosts { .. } => {
                    if g.legal_actions().contains(&Action::BeginOrderedCosts) {
                        act(&mut g, Action::BeginOrderedCosts);
                    } else {
                        pay(&mut g);
                    }
                }
                Pending::Priority => break,
                _ => panic!("unexpected linked Evoke payment {:?}", g.state.pending),
            }
        }
        let cast = g.state.stack.last().unwrap().cast.unwrap();
        assert!(cast.evoked);
        assert_eq!(cast.payment, [0, 0, 0, 0, 1, 0]);
        assert_eq!(g.counter_count(donor, CounterKind::Charge), 0);
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
#[test]
fn permanent_spell_copy_retains_evoke_without_claiming_an_actual_payment() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    route(&mut g, source, EVOKE);
    let original = g.state.stack.last().unwrap().clone();
    let copy = g.allocate_spell_copy(&original, Player::P1);
    let copied = copy.source.unwrap().object;
    assert!(copy.cast.unwrap().evoked);
    assert_eq!(copy.cast.unwrap().payment, [0; 6]);
    g.state.stack.push(copy);
    finish(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
    assert!(
        g.object_ref(copied.card) != copied
            || g.state.objects[copied.card as usize].zone != Zone::Battlefield
    );
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.state.players[1].life, 20);
}
#[test]
fn free_cast_cannot_combine_another_alternative_cost_or_fabricate_mana_spent() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Omniscience", Zone::Battlefield);
    g.state.players[0].mana = [0; 6];
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    act(
        &mut g,
        Action::AnnounceFree {
            card: source,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    pay(&mut g);
    assert!(!g.state.stack.last().unwrap().cast.unwrap().evoked);
    finish(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].life, 20);
}
#[test]
fn evoke_sacrifice_uses_current_controller_and_exact_entry_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    for blink in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
        route(&mut g, source, EVOKE);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
        let entered = g.object_ref(source);
        if blink {
            g.move_object(source, Zone::Exile);
            g.move_object(source, Zone::Battlefield);
        } else {
            g.change_controller(entered, Player::P1);
        }
        finish(&mut g);
        assert_eq!(
            g.state.objects[source as usize].zone,
            if blink {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}
#[test]
fn evoke_route_choice_and_receipt_are_owned_by_raw_index_and_restored_branches() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RULES)]);
    let mut parent = scenario(Player::P1);
    let source = put(&mut parent, Player::P1, "Grizzly Bears", Zone::Hand);
    act(&mut parent, Action::AnnounceCast { card: source });
    let saved = parent.snapshot();
    reject(&mut parent, Action::ChooseSpellCost { cost: u16::MAX });
    let choice = parent
        .observe(Player::P1)
        .cost_choices
        .into_iter()
        .find(|c| c.cost == EVOKE)
        .unwrap();
    let action = Action::ChooseSpellCost { cost: choice.index };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(Player::P1, indexed.revision(), index)
        .unwrap();
    act(&mut raw, action);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    pay(&mut raw);
    finish(&mut raw);
    pay(&mut indexed);
    finish(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    let choice = indexed
        .observe(Player::P1)
        .cost_choices
        .into_iter()
        .find(|c| c.cost == NORMAL)
        .unwrap();
    act(&mut indexed, Action::ChooseSpellCost { cost: choice.index });
    pay(&mut indexed);
    finish(&mut indexed);
    assert_eq!(
        indexed.state.objects[source as usize].zone,
        Zone::Battlefield
    );
    assert_eq!(raw.state.objects[source as usize].zone, Zone::Graveyard);
}
