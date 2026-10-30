// Directly authored from complete pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),

    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Surveil(1)],
    }],
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5445)),

        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
