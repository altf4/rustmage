//! Tap substitutions pay a locked activation cost without mana or a tap symbol.
mod support;
use rustmage::{Card, CardName};
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DRAW: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 4,
        mana_component: true,
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::TapCreaturesForActivationMana,
        Instruction::Draw(PlayerSet::Controller, 1),
    ],
    sorcery: true,
    once_per_turn: false,
};
const TAP_PAYMENT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[DRAW],
    ..CardRules::EMPTY
};
const DISCOUNTED: CardRules = CardRules {
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: StatValue::Fixed(3),
                multiplier: 1,
            },
        )],
        ..CardExtras::EMPTY
    },
    ..TAP_PAYMENT
};
const WATERBEND: CardRules = CardRules {
    activated: &[ActivatedAbility {
        program: &[
            Instruction::WaterbendCost(4),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
        ..DRAW
    }],
    ..TAP_PAYMENT
};
const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(2)],
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

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn tap_creature(g: &Game, card: CardId) -> Action {
    Action::ChooseCreatureTapPayment {
        object: g.object_ref(card),
    }
}

#[test]
fn creature_taps_reserve_the_source_and_cannot_reuse_a_mana_producer() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &TAP_PAYMENT)]);
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        g.state.players[actor.index()].mana = [0; 6];
        let source = put(&mut g, actor, "Goblin Piker", Zone::Battlefield);
        let elves = put(&mut g, actor, "Llanowar Elves", Zone::Battlefield);
        let sick = put(&mut g, actor, "Bear Cub", Zone::Battlefield);
        g.state.objects[sick as usize].summoning_sick = true;
        let artifact = put(&mut g, actor, "Short Bow", Zone::Battlefield);
        let enemy = put(&mut g, actor.other(), "Bear Cub", Zone::Battlefield);
        let artifact_creature = put(&mut g, actor, "Gingerbrute", Zone::Battlefield);
        reject(&mut g, activate(source));
        let fourth = put(&mut g, actor, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::BeginCombat;
        reject(&mut g, activate(source));
        g.state.step = Step::Main1;
        g.state.objects[source as usize].summoning_sick = true;
        reject(&mut g, activate(source));
        g.state.objects[source as usize].summoning_sick = false;

        act(&mut g, activate(source));
        choose(
            &mut g,
            |a| matches!(a, Action::ActivateMana { card, .. } | Action::TapMana { card } if *card == elves),
        );
        assert!(g.object_view(elves).tapped);
        act(&mut g, Action::BeginCreatureTapPayment);
        assert!(g.observe(actor.other()).creature_tap_payment.is_none());
        assert!(g.observe(actor).waterbend_choice.is_none());
        for ineligible in [source, elves, artifact, enemy] {
            let attempt = tap_creature(&g, ineligible);
            reject(&mut g, attempt);
        }
        let stale = Action::ChooseCreatureTapPayment {
            object: ObjectRef {
                generation: g.object_ref(sick).generation + 1,
                ..g.object_ref(sick)
            },
        };
        reject(&mut g, stale);
        let early_payment = Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        };
        reject(&mut g, early_payment.clone());

        for contributor in [sick, artifact_creature, fourth] {
            let action = tap_creature(&g, contributor);
            act(&mut g, action.clone());
            assert!(!g.object_view(contributor).tapped);
            reject(&mut g, action);
        }
        assert_eq!(g.creature_tap_payment().unwrap().remaining.generic, 1);
        act(&mut g, early_payment);
        assert_eq!(g.state.players[actor.index()].mana, [0; 6]);
        for tapped in [source, sick, artifact_creature, fourth] {
            assert!(g.object_view(tapped).tapped);
        }
        assert!(!g.object_view(artifact).tapped);
        settle(&mut g);
        assert_eq!(g.state.players[actor.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn creature_substitution_covers_the_tax_but_waterbend_keeps_its_printed_cap() {
    let tax_card = Card::from_name("Ghost Vacuum").unwrap();
    for (rules, waterbend) in [(&TAP_PAYMENT, false), (&WATERBEND, true)] {
        let _rules = fixture_rules::activate(&[(Card::GoblinPiker, rules), (tax_card, &TAX)]);
        let actor = Player::P0;
        let mut g = scenario(actor);
        g.state.players[actor.index()].mana = [0; 6];
        let tax = put(&mut g, actor.other(), "Ghost Vacuum", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(tax),
            EntryChoiceValue::CardName(CardName::from_name("Goblin Piker").unwrap()),
        );
        let source = put(&mut g, actor, "Goblin Piker", Zone::Battlefield);
        let artifact = put(&mut g, actor, "Short Bow", Zone::Battlefield);
        let mut creatures = Vec::new();
        for _ in 0..5 {
            creatures.push(put(&mut g, actor, "Bear Cub", Zone::Battlefield));
        }
        reject(&mut g, activate(source));
        if waterbend {
            g.state.players[actor.index()].mana[5] = 2;
            act(&mut g, activate(source));
            act(&mut g, Action::BeginWaterbend);
            assert!(g.observe(actor).creature_tap_payment.is_none());
            for card in [artifact, creatures[0], creatures[1], creatures[2]] {
                let object = g.object_ref(card);
                act(&mut g, Action::ChooseWaterbender { object });
            }
            let object = g.object_ref(creatures[3]);
            reject(&mut g, Action::ChooseWaterbender { object });
            assert_eq!(g.waterbend_choice().unwrap().remaining.generic, 2);
        } else {
            creatures.push(put(&mut g, actor, "Bear Cub", Zone::Battlefield));
            act(&mut g, activate(source));
            act(&mut g, Action::BeginCreatureTapPayment);
            assert_eq!(g.creature_tap_payment().unwrap().remaining.generic, 6);
            for card in creatures {
                let action = tap_creature(&g, card);
                act(&mut g, action);
            }
            assert_eq!(g.creature_tap_payment().unwrap().remaining.generic, 0);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[actor.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[actor.index()].mana, [0; 6]);
    }
}

#[test]
fn discounted_creature_tap_choice_survives_raw_indexed_and_restored_branches() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &DISCOUNTED)]);
    let actor = Player::P0;
    let mut parent = scenario(actor);
    parent.state.players[actor.index()].mana = [0; 6];
    let source = put(&mut parent, actor, "Goblin Piker", Zone::Battlefield);
    let first = put(&mut parent, actor, "Bear Cub", Zone::Battlefield);
    let second = put(&mut parent, actor, "Bear Cub", Zone::Battlefield);
    act(&mut parent, activate(source));
    act(&mut parent, Action::BeginCreatureTapPayment);
    assert_eq!(parent.creature_tap_payment().unwrap().remaining.generic, 1);
    let snapshot = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    let choose_first = tap_creature(&raw, first);
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == choose_first)
        .unwrap();
    act(&mut raw, choose_first);
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let extra = tap_creature(&raw, second);
    reject(&mut raw, extra);
    pay(&mut raw);
    pay(&mut indexed);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert!(raw.object_view(first).tapped);
    assert!(!raw.object_view(second).tapped);

    indexed.restore(&snapshot).unwrap();
    let choose_second = tap_creature(&indexed, second);
    act(&mut indexed, choose_second);
    pay(&mut indexed);
    settle(&mut indexed);
    assert!(!indexed.object_view(first).tapped);
    assert!(indexed.object_view(second).tapped);
    assert_eq!(
        indexed.state.players[actor.index()].cards_drawn_this_turn,
        1
    );
    assert!(raw.object_view(first).tapped);
}
