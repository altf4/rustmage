// Direct typed behavior from complete pinned Oracle and XMage sources.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::MustBeBlocked],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDamaged,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddRetainedManaScaled {
            color: 3,
            amount: AmountValue::DamageEventAmount,
            retention: ManaRetention::Turn,
        }],
    }],
    ..CardRules::EMPTY
};
