// Complete pinned Oracle; no XMage identity exists in the pinned inventory.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ManaSpent {
            minimum: 0,
            increment: false,
            event: &TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 2 },
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::ManaSpent),
            Instruction::Discard(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
