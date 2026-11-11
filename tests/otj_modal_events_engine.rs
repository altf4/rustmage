//! Cast observers consume committed mode choices and actual caster history.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ONE: SpellMode = SpellMode {
    text: "First",
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
};
const TWO: SpellMode = SpellMode {
    text: "Two",
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
};
const THREE: SpellMode = SpellMode {
    text: "Three",
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
};
const ZERO: SpellMode = SpellMode {
    text: "None",
    targets: TargetPlan::NONE,
    program: &[],
};
const MULTI_MODE: CardRules = CardRules {
    effect: Effect::Modal(&[ZERO, ONE, TWO]),
    mode_counts: Some(&[0, 1, 2]),
    ..CardRules::EMPTY
};
const MODE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ModalSpellCastByYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalUpToX {
            modes: &[ZERO, ONE, TWO, THREE],
            counts: &[0, 1, 2, 3],
        }],
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

#[test]
fn an_actual_modal_cast_limits_its_trigger_to_the_number_of_printed_modes_selected() {
    let _rules = fixture_rules::activate(&[
        (card("Hill Giant"), &MODE_OBSERVER),
        (card("Opt"), &MULTI_MODE),
    ]);
    let p = Player::P0;
    for count in [0, 1, 2] {
        let mut g = scenario(p);
        put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, p, "Opt", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: count });
        pay(&mut g);
        for _ in 0..8 {
            if g.legal_actions().contains(&Action::ChooseMode { mode: 0 }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::PutTrigger { .. } | Action::Pass)
            });
        }
        let choices: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| match a {
                Action::ChooseMode { mode } => Some(mode),
                _ => None,
            })
            .collect();
        assert_eq!(choices, (0..=count).collect::<Vec<_>>());
        reject(&mut g, Action::ChooseMode { mode: count + 1 });
        act(&mut g, Action::ChooseMode { mode: count });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20 + i128::from(count) * 2);
    }
}

const OTHER_SPELL_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::DamageScaled(AmountValue::Sum(&[
            AmountValue::Fixed(2),
            AmountValue::OtherSpellsCastThisTurn,
        ]))],
    ),
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
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

#[test]
fn the_original_cast_subtracts_itself_but_its_copy_counts_the_original() {
    let _rules =
        fixture_rules::activate(&[(card("Shock"), &OTHER_SPELL_DAMAGE), (card("Opt"), &COPY)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Shock", Zone::Hand);
    let copy = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    let mut only_original = g.clone();
    settle(&mut only_original);
    assert_eq!(only_original.state.players[p.other().index()].life, 18);
    cast(&mut g, copy, Some(Target::Spell(spell)));
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, 13);
    assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
}

const NONCREATURE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayersScaled(
            PlayerSet::CastingPlayer,
            AmountValue::NoncreatureSpellsCastByCastingPlayer,
        )],
    }],
    ..CardRules::EMPTY
};

#[test]
fn a_casting_player_damage_trigger_uses_that_players_live_noncreature_total() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &NONCREATURE_OBSERVER)]);
    let observer = Player::P0;
    let caster = observer.other();
    let mut g = scenario(caster);
    put(&mut g, observer, "Hill Giant", Zone::Battlefield);
    let first = put(&mut g, caster, "Shock", Zone::Hand);
    let second = put(&mut g, caster, "Shock", Zone::Hand);
    cast(&mut g, first, Some(Target::Player(observer)));
    assert_eq!(g.state.stack.last().unwrap().controller, observer);
    cast(&mut g, second, Some(Target::Player(observer)));
    settle(&mut g);
    assert_eq!(g.state.players[observer.index()].life, 16);
    assert_eq!(g.state.players[caster.index()].life, 16);
}
