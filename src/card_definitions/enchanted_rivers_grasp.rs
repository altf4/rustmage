// One ETB program taps the exact enchanted permanent and removes every counter kind.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(1))),
    aura: Some(AuraSpec { kinds: 1, relation: -1,
        bonus: EquipmentSpec { stats: Stats { power: 0, toughness: 0 }, keywords: Keywords::NONE } }),
    triggers: &[TriggeredAbility { condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::TapAttached, Instruction::RemoveAllCountersFrom(Subject::Attached)] }],
    static_groups: &[StaticAbilityGroup { line: 2, first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachedUntap(UntapRestriction::ControllerStep)] }],
    ..CardRules::EMPTY
};
