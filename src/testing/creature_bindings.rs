//! Complete, literal contracts for creatures with uncoupled common triggers.
use super::coverage_bindings::{Cost, plain_extras};
use super::*;
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Event {
    Enter,
    Die,
    EnterDie,
    Attack,
    LifeGained,
    OtherEnters,
    Draw,
    SecondDraw,
    NoncreatureCast,
    Tapped,
}
impl Event {
    fn label(self) -> &'static str {
        match self {
            Self::Enter => "enter",
            Self::Die => "die",
            Self::EnterDie => "enter_die",
            Self::Attack => "attack",
            Self::LifeGained => "life_gained",
            Self::OtherEnters => "other_enters",
            Self::Draw => "draw",
            Self::SecondDraw => "second_draw",
            Self::NoncreatureCast => "noncreature_cast",
            Self::Tapped => "tapped",
        }
    }
    fn condition(self) -> TriggerCondition {
        match self {
            Self::Enter => TriggerCondition::SelfEnters,
            Self::Die => TriggerCondition::SelfDies,
            Self::EnterDie => TriggerCondition::SelfEntersOrDies,
            Self::Attack => TriggerCondition::SelfAttacks,
            Self::LifeGained => TriggerCondition::LifeGained {
                first_only: false,
                own_turn: false,
            },
            Self::OtherEnters => TriggerCondition::OtherCreatureEntersYou,
            Self::Draw => TriggerCondition::CardDrawn {
                relation: CastRelation::You,
                ordinal: None,
            },
            Self::SecondDraw => TriggerCondition::CardDrawn {
                relation: CastRelation::You,
                ordinal: Some(2),
            },
            Self::NoncreatureCast => TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            Self::Tapped => TriggerCondition::SelfBecomesTapped,
        }
    }
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Payload {
    Draw,
    DrawDiscard,
    GainLife,
    Drain,
    DiscardOpponent,
    Surveil,
    Scry,
    Mill,
    Counters,
    Grow,
}
impl Payload {
    fn label(self) -> &'static str {
        match self {
            Self::Draw => "draw",
            Self::DrawDiscard => "draw_discard",
            Self::GainLife => "gain_life",
            Self::Drain => "drain",
            Self::DiscardOpponent => "discard_opponent",
            Self::Surveil => "surveil",
            Self::Scry => "scry",
            Self::Mill => "mill",
            Self::Counters => "counters",
            Self::Grow => "grow",
        }
    }
}
#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Ability {
    pub event: Event,
    pub effect: Payload,
    pub amount: u16,
    pub secondary: u16,
}
impl Ability {
    fn signature(self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.event.label(),
            self.effect.label(),
            self.amount,
            self.secondary
        )
    }
    fn program(self) -> Vec<Instruction> {
        let a = self.amount;
        let b = self.secondary;
        match self.effect {
            Payload::Draw => vec![Instruction::Draw(PlayerSet::Controller, a)],
            Payload::DrawDiscard => vec![
                Instruction::Draw(PlayerSet::Controller, a),
                Instruction::Discard(PlayerSet::Controller, b),
            ],
            Payload::GainLife => vec![Instruction::GainLife(PlayerSet::Controller, a)],
            Payload::Drain => vec![
                Instruction::LoseLife(PlayerSet::Opponent, a),
                Instruction::GainLife(PlayerSet::Controller, b),
            ],
            Payload::DiscardOpponent => vec![Instruction::Discard(PlayerSet::Opponent, a)],
            Payload::Surveil => vec![Instruction::Surveil(a)],
            Payload::Scry => vec![Instruction::Scry(a)],
            Payload::Mill => vec![Instruction::Mill(
                PlayerSet::Controller,
                MillAmount::Fixed(a),
            )],
            Payload::Counters => vec![Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: a.into(),
            }],
            Payload::Grow => vec![Instruction::Grow {
                subject: Subject::Source,
                power: a.into(),
                toughness: b.into(),
            }],
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Member {
    pub card: String,
    pub catalogue_id: u16,
    pub oracle_id: String,
    pub type_line: String,
    pub kinds: u16,
    pub cost: Cost,
    pub power: i32,
    pub toughness: i32,
    pub keywords: Vec<String>,
    pub paragraphs: Vec<u16>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Family {
    pub id: String,
    pub abilities: Vec<Ability>,
    pub representative: String,
    pub members: Vec<Member>,
}
pub(crate) fn families() -> Vec<Family> {
    crate::engine::testing::fixture_json("creature_families.json")
}
fn keywords(names: &[String]) -> Keywords {
    names.iter().fold(Keywords::NONE, |all, name| {
        all.union(match name.as_str() {
            "flying" => Keywords::FLYING,
            "first strike" => Keywords::FIRST_STRIKE,
            "double strike" => Keywords::DOUBLE_STRIKE,
            "deathtouch" => Keywords::DEATHTOUCH,
            "lifelink" => Keywords::LIFELINK,
            "vigilance" => Keywords::VIGILANCE,
            "reach" => Keywords::REACH,
            "trample" => Keywords::TRAMPLE,
            "haste" => Keywords::HASTE,
            "defender" => Keywords::DEFENDER,
            "hexproof" => Keywords::HEXPROOF,
            "indestructible" => Keywords::INDESTRUCTIBLE,
            "menace" => Keywords::MENACE,
            "flash" => Keywords::FLASH,
            other => panic!("unaccounted intrinsic keyword: {other}"),
        })
    })
}
fn bind(member: &Member, abilities: &[Ability]) {
    let card = Card::from_name(&member.card).unwrap();
    assert_eq!(card.index(), usize::from(member.catalogue_id));
    assert!(card.is_supported());
    assert_eq!(card.info().oracle_id, Some(member.oracle_id.as_str()));
    assert_eq!(card.info().type_line, member.type_line);
    assert_eq!(card.info().mana_cost, member.cost.printed);
    assert_eq!(card.cost(), (member.cost.colored, member.cost.generic));
    assert_eq!(generated::DEFINITIONS[card.index()].kind, member.kinds);
    let rules = authored::rules(card).unwrap();
    let authored::CardRules {
        deck_limit,
        extras,
        effect,
        triggers,
        activated,
        keywords: intrinsic,
        enters,
        entry,
        mode_costs,
        aura,
        static_groups,
    } = rules;
    assert_eq!(*deck_limit, Some(4));
    assert_eq!(*effect, Effect::Creature(member.power, member.toughness));
    assert_eq!(*intrinsic, keywords(&member.keywords));
    assert_eq!(
        *enters,
        InitialCounters {
            plus_one: 0,
            minus_one: 0,
            stun: 0,
            loyalty: 0,
            lore: 0
        }
    );
    assert_eq!(*entry, EntryTapped::Never);
    assert!(mode_costs.is_none() && aura.is_none());
    assert!(activated.is_empty());
    // Only intrinsic keyword bookkeeping is orthogonal to these triggers.
    assert!(super::coverage_bindings::keyword_groups(
        static_groups,
        *intrinsic
    ));
    assert!(
        plain_extras(extras, *intrinsic, None),
        "{} has an unaccounted wrapper",
        member.card
    );
    assert_eq!(card.effect(), *effect);
    assert_eq!(card.triggers(), *triggers);
    assert!(
        card.activations().is_empty()
            && card.replacement_effects().is_empty()
            && card.cost_reductions().is_empty()
    );
    assert_eq!(triggers.len(), abilities.len());
    assert_eq!(member.paragraphs.len(), abilities.len());
    for (actual, expected) in triggers.iter().zip(abilities) {
        assert_eq!(actual.condition, expected.event.condition());
        assert_eq!(actual.targets, TargetPlan::NONE);
        assert_eq!(actual.program, expected.program());
    }
}
#[test]
fn complete_creature_configurations_bind_the_engine_programs() {
    let mut seen = std::collections::BTreeSet::new();
    for family in families() {
        let signature = "plain_creature_v1/".to_string()
            + &family
                .abilities
                .iter()
                .map(|a| a.signature())
                .collect::<Vec<_>>()
                .join("+");
        assert_eq!(family.id, signature);
        assert!(
            family
                .members
                .iter()
                .any(|m| m.card == family.representative)
        );
        for member in family.members {
            println!("creature binding {}: {}", family.id, member.card);
            assert!(seen.insert(member.catalogue_id));
            bind(&member, &family.abilities);
            crate::engine::testing::covered(&format!(
                "card/{}/binding/{}",
                member.catalogue_id, family.id
            ));
        }
    }
}
