// Direct Oracle behavior; the cast ordinal includes spells cast before entry.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: true,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::OwnerLibrary],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::NonCreatureOrdinal(1),
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(1)],
        },
    ],
    ..CardRules::EMPTY
};
