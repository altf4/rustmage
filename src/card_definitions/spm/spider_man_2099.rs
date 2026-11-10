// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::DOUBLE_STRIKE.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::PlayedFromOutsideOwnHandThisTurn,
        },
        targets: one(TargetRule::Any),
        program: &[Instruction::DamageScaled(AmountValue::SourcePower)],
    }],
    extras: &CardExtras {
        cast_after_first_three_turns: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
