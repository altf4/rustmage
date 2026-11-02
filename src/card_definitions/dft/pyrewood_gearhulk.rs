// The frozen other-creature group gains temporary stats/keywords; all prevention is disabled.
const OTHERS: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::VIGILANCE.union(Keywords::MENACE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowCreatures {
                filter: OTHERS,
                power: 2,
                toughness: 2,
            },
            Instruction::GrantCreatures {
                filter: OTHERS,
                keywords: Keywords::VIGILANCE.union(Keywords::MENACE),
            },
            Instruction::DamageCannotBePreventedThisTurn,
        ],
    }],
    ..CardRules::EMPTY
};
