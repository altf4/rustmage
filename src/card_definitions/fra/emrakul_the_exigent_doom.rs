// Partial draft: ward payment and hand-exile permission remain unvalidated; gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(12, 12),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfCast,
            targets: TargetPlan::NONE,
            program: &[Instruction::UntapControlledLands],
        },
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessSacrifice(3)],
        },
    ],
    ..CardRules::EMPTY
};
