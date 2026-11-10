// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersWithWebSlinging,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainLife(PlayerSet::Controller, 3),
            Instruction::CreateTokens {
                token: Card(5544),
                amount: 2,
                tapped: false,
            },
        ],
    }],
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
