// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ManifestDread],
        },
        TriggeredAbility {
            condition: TriggerCondition::YouTurnPermanentFaceUp,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersOnEventPermanent {
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
