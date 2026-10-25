// Direct typed behavior from the complete frozen Oracle and pinned Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::Damage(1)]),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpellChosenX],
    }],
    ..CardRules::EMPTY
};
