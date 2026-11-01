// Authored from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::NonmanaAbilityActivated,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedActivatedAbility],
    }],
    extras: &CardExtras {
        borrows_exiled_activations: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
