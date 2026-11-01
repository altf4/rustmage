//! Literal complete entry/mana contracts for ordinary land families.
use super::coverage_bindings::plain_extras;
use super::*;
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Entry {
    Never,
    Always,
    AtMostTwo,
    AtLeastTwo,
    LowLife,
    Planeswalker,
    PayTwoLife,
}
impl Entry {
    fn label(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Always => "always",
            Self::AtMostTwo => "at_most_two",
            Self::AtLeastTwo => "at_least_two",
            Self::LowLife => "low_life",
            Self::Planeswalker => "planeswalker",
            Self::PayTwoLife => "pay_two_life",
        }
    }
    fn rule(self) -> EntryTapped {
        match self {
            Self::Never => EntryTapped::Never,
            Self::Always => EntryTapped::Always,
            Self::AtMostTwo => EntryTapped::UnlessAtMostTwo,
            Self::AtLeastTwo => EntryTapped::UnlessAtLeastTwo,
            Self::LowLife => EntryTapped::UnlessLowLife,
            Self::Planeswalker => EntryTapped::UnlessPlaneswalker,
            Self::PayTwoLife => EntryTapped::PayLife(2),
        }
    }
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EntryTrigger {
    None,
    GainLife,
    Scry,
    Surveil,
    DamageOpponent,
}
impl EntryTrigger {
    fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::GainLife => "gain_life",
            Self::Scry => "scry",
            Self::Surveil => "surveil",
            Self::DamageOpponent => "damage_opponent",
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
    pub basic: bool,
    pub intrinsic: Option<usize>,
    pub legacy_mana: Option<u16>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Family {
    pub id: String,
    pub entry: Entry,
    pub trigger: EntryTrigger,
    pub amount: u16,
    pub mask: u8,
    pub representative: String,
    pub members: Vec<Member>,
}
pub(crate) fn families() -> Vec<Family> {
    crate::engine::testing::fixture_json("land_families.json")
}
fn mana_ability(actual: &ActivatedAbility, mask: u8) {
    let ActivatedAbility {
        exhaust,
        guard,
        origin,
        cycling,
        cost,
        targets,
        program,
        sorcery,
        once_per_turn,
    } = *actual;
    assert!(!exhaust && !cycling && !sorcery && !once_per_turn);
    assert_eq!(guard, ActivationGuard::Always);
    assert_eq!(origin, ActivationOrigin::Battlefield);
    assert_eq!(targets, TargetPlan::NONE);
    assert_eq!(
        cost,
        ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None
        }
    );
    let mut fixed = [0; 6];
    if mask.count_ones() == 1 {
        fixed[mask.trailing_zeros() as usize] = 1;
    }
    assert!(
        program
            == [Instruction::AddMana(ManaProduction::Choice {
                mask,
                amount: 1
            })]
            || mask.count_ones() == 1
                && program == [Instruction::AddMana(ManaProduction::Fixed(fixed))]
    );
}
fn bind(member: &Member, family: &Family) {
    let card = Card::from_name(&member.card).unwrap();
    assert_eq!(card.index(), usize::from(member.catalogue_id));
    assert!(card.is_supported());
    assert_eq!(card.info().oracle_id, Some(member.oracle_id.as_str()));
    assert_eq!(card.info().type_line, member.type_line);
    assert_eq!(card.info().mana_cost, "");
    assert_eq!(card.cost(), ([0; 6], 0));
    let definition = &generated::DEFINITIONS[card.index()];
    assert_eq!(definition.kind, 2);
    assert_eq!(definition.basic, member.basic);
    assert_eq!(definition.mana_color, member.intrinsic);
    let rules = authored::rules(card).unwrap();
    let authored::CardRules {
        deck_limit,
        extras,
        effect,
        triggers,
        activated,
        keywords,
        enters,
        entry,
        mode_costs,
        aura,
        static_groups,
    } = rules;
    assert_eq!(*deck_limit, if member.basic { None } else { Some(4) });
    assert_eq!(*effect, Effect::Land);
    assert_eq!(card.effect(), *effect);
    assert_eq!(*keywords, Keywords::NONE);
    assert!(static_groups.is_empty());
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
    assert_eq!(*entry, family.entry.rule());
    assert_eq!(definition.entry, *entry);
    assert!(mode_costs.is_none() && aura.is_none());
    assert!(
        plain_extras(extras, Keywords::NONE, member.legacy_mana),
        "{} has an unaccounted wrapper",
        member.card
    );
    assert_eq!(card.triggers(), *triggers);
    assert_eq!(card.activations(), *activated);
    assert!(card.replacement_effects().is_empty() && card.cost_reductions().is_empty());
    if let Some(color) = member.intrinsic {
        assert_eq!(family.mask, 1 << color);
        assert!(activated.is_empty());
    } else {
        assert_eq!(activated.len(), 1);
        mana_ability(&activated[0], family.mask);
    }
    if matches!(family.trigger, EntryTrigger::None) {
        assert!(triggers.is_empty());
    } else {
        assert_eq!(triggers.len(), 1);
        let trigger = triggers[0];
        assert_eq!(trigger.condition, TriggerCondition::SelfEnters);
        let targets = if matches!(family.trigger, EntryTrigger::DamageOpponent) {
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::Player {
                    opponent_only: true,
                },
                min: 1,
                max: 1,
            })
        } else {
            TargetPlan::NONE
        };
        assert_eq!(trigger.targets, targets);
        let instruction = match family.trigger {
            EntryTrigger::GainLife => Instruction::GainLife(PlayerSet::Controller, family.amount),
            EntryTrigger::Scry => Instruction::Scry(family.amount),
            EntryTrigger::Surveil => Instruction::Surveil(family.amount),
            EntryTrigger::DamageOpponent => Instruction::Damage(family.amount.into()),
            EntryTrigger::None => unreachable!(),
        };
        assert_eq!(trigger.program, [instruction]);
    }
}
#[test]
fn complete_land_configurations_bind_the_engine_programs() {
    let mut seen = std::collections::BTreeSet::new();
    for family in families() {
        assert!(
            family
                .members
                .iter()
                .any(|m| m.card == family.representative)
        );
        for member in &family.members {
            let signature = format!(
                "plain_land_v1/{}/{}.{}/mana.{}/{}",
                family.entry.label(),
                family.trigger.label(),
                family.amount,
                family.mask,
                if member.intrinsic.is_some() {
                    "intrinsic"
                } else {
                    "activation"
                }
            );
            assert_eq!(family.id, signature);
            assert!(seen.insert(member.catalogue_id));
            println!("land binding {}: {}", family.id, member.card);
            bind(member, &family);
            crate::engine::testing::covered(&format!(
                "card/{}/binding/{}",
                member.catalogue_id, family.id
            ));
        }
    }
}
