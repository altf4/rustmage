// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksDefendingPlayer,
        targets: one(TargetRule::DefendingCreature),
        program: &[
            Instruction::TargetStatsUntilNextTurn {
                power: -2,
                toughness: 0,
            },
            Instruction::RemoveKeywordsUntilNextTurn {
                subject: Subject::Targets,
                keywords: Keywords::FLYING,
            },
        ],
    }],
    ..CardRules::EMPTY
};
