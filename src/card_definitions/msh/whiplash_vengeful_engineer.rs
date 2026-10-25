// Direct typed implementation: both life changes use the same frozen X.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksIfEquipped,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseOpponentGainSame(
            AmountValue::SourceEquipmentCount,
        )],
    }],
    ..CardRules::EMPTY
};
