// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: one(object(2 | 16)),
        program: &[Instruction::ExileAndBorrowActivatedAbilities],
    }],
    extras: &CardExtras {
        borrows_exiled_activations: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
