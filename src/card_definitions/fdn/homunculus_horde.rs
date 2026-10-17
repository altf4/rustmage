pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(2),
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::CreateCopyToken {
            subject: Subject::Source,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
