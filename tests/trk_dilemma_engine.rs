//! A dilemma is the positive choice of modes, once per spell or ability.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EMPTY: SpellMode = SpellMode {
    text: "Choose no modes",
    targets: TargetPlan::NONE,
    program: &[],
};
const ONE: SpellMode = SpellMode {
    text: "Gain one life",
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
};
const TWO: SpellMode = SpellMode {
    text: "Gain two life",
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
};
const MODAL_SPELL: CardRules = CardRules {
    effect: Effect::Modal(&[EMPTY, ONE, TWO]),
    mode_counts: Some(&[0, 1, 2]),
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::FaceDilemma(CastRelation::You),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Modal(&[ONE, TWO])],
        },
    ],
    ..CardRules::EMPTY
};
const MODAL_ACTIVATION: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[ONE, TWO])],
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const COPY_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::YourSpell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};
const COUNT_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::Stat(StatValue::DilemmasFacedThisTurn),
        )],
    ),
    ..CardRules::EMPTY
};
const OPPONENT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FaceDilemma(CastRelation::Opponent),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
    }],
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &MODAL_SPELL),
        (Card::from_name("Hill Giant").unwrap(), &OBSERVER),
        (Card::from_name("Short Bow").unwrap(), &MODAL_ACTIVATION),
        (Card::from_name("Shock").unwrap(), &COUNT_SPELL),
        (Card::from_name("Bear Cub").unwrap(), &OPPONENT_OBSERVER),
    ])
}

#[test]
fn spell_modes_count_once_and_dilemma_trigger_resolves_above_spell() {
    let _rules = rules();
    for (mode, count, spell_life) in [(0, 0, 0), (1, 1, 1), (2, 1, 2)] {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        let opponent_spell = put(&mut g, p.other(), "Opt", Zone::Hand);
        reject(
            &mut g,
            Action::AnnounceCast {
                card: opponent_spell,
            },
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseMode { mode: 9 });
        let index = g
            .legal_actions()
            .iter()
            .position(|a| *a == Action::ChooseMode { mode })
            .unwrap();
        let mut indexed = g.clone();
        indexed.apply_index(p, g.revision(), index).unwrap();
        act(&mut g, Action::ChooseMode { mode });
        assert_eq!(g.snapshot(), indexed.snapshot());
        assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, count);
        pay(&mut g);
        if count > 0 {
            assert!(g.state.pending_triggers.len() > 0 || g.state.stack.len() > 1);
            // The observer's gain resolves while the originating spell remains.
            for _ in 0..12 {
                if g.state.players[p.index()].life > 20 {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::PutTrigger { .. } | Action::Pass)
                });
            }
            assert_eq!(g.state.players[p.index()].life, 23);
            assert!(!g.state.stack.is_empty());
        }
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            i128::from(20 + spell_life + count * 3)
        );
        let checkpoint = g.snapshot();
        let mut branched = g.clone();
        branched.restore(&checkpoint).unwrap();
        assert_eq!(branched.snapshot(), checkpoint);
    }
}

#[test]
fn activated_mode_faces_dilemma_and_failed_choice_rolls_back() {
    let _rules = rules();
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut g, Action::ChooseMode { mode: 7 });
    assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, 0);
    act(&mut g, Action::ChooseMode { mode: 1 });
    assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, 1);
    assert_eq!(g.observe(p).players[p.index()].dilemmas_faced_this_turn, 1);
    assert_eq!(
        g.observe(p.other()).players[p.index()].dilemmas_faced_this_turn,
        1
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 25);
    assert_eq!(g.state.players[p.other().index()].life, 24);
    let modal = put(&mut g, p, "Opt", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: modal });
    act(&mut g, Action::ChooseMode { mode: 1 });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, 2);
    assert_eq!(g.state.players[p.index()].life, 29);
    assert_eq!(g.state.players[p.other().index()].life, 28);
    let count_spell = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, count_spell, None);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 31);
}

#[test]
fn copied_modal_spell_keeps_its_modes_without_another_choice() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &MODAL_SPELL),
        (Card::from_name("Shock").unwrap(), &COPY_SPELL),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let modal = put(&mut g, p, "Opt", Zone::Hand);
    let copy = put(&mut g, p, "Shock", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: modal });
    act(&mut g, Action::ChooseMode { mode: 1 });
    pay(&mut g);
    assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, 1);
    cast(&mut g, copy, Some(Target::Spell(modal)));
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 22);
    assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, 1);
}

#[test]
fn triggered_ability_mode_faces_dilemma_and_new_turn_clears_count() {
    let _rules = rules();
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    g.begin_step(Step::Upkeep);
    g.give_priority();
    g.prepare_triggers();
    for _ in 0..8 {
        if g.legal_actions().contains(&Action::ChooseMode { mode: 1 }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    reject(&mut g, Action::ChooseMode { mode: 7 });
    act(&mut g, Action::ChooseMode { mode: 1 });
    assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, 1);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, 25);
    g.begin_turn();
    assert_eq!(g.state.players[p.index()].dilemmas_faced_this_turn, 0);
}
