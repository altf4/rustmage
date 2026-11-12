// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::DEFENDER,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(2),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::RemoveKeywords {
                subject: Subject::Source,
                keywords: Keywords::DEFENDER,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::VIGILANCE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
