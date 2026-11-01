// Counter an independent ability, then blank its exact live source for this duration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Ability,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::CounterAbilityAndRemoveSourceAbilities],
    }],
    ..CardRules::EMPTY
};
