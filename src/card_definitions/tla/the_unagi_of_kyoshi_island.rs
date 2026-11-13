// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessWaterbend(4)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CardDrawn {
                relation: CastRelation::Opponent,
                ordinal: Some(2),
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 2)],
        },
    ],
    ..CardRules::EMPTY
};
