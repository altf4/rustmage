// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::VIGILANCE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::VIGILANCE,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::DrawThenDiscardIfDrew {
                amount: AmountValue::ColorsSpent,
                discard: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};
