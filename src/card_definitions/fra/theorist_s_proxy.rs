// Direct typed rules from pinned Oracle and XMage; gated pending integration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3), keywords: Keywords::FLASH,
    static_groups: &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Ability, fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)] }],
    triggers: &[TriggeredAbility { condition: TriggerCondition::SelfEnters, targets: TargetPlan::NONE, program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(3)] }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield, cycling: false,
        cost: ActivationCost { colored: [0, 1, 0, 0, 0, 0], generic: 0, alternatives: &[], additional: None, mana_component: true, tap: false, sacrifice: true, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
        targets: TargetPlan::NONE, program: &[Instruction::GrantNextSpellUncounterable], sorcery: false, once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
