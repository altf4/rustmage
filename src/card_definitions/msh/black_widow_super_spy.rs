// Direct whole Oracle behavior; the damaged player and ability controller are captured.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileDamagedLibraryUntilNonland,
            Instruction::MayCounterSourceOtherwiseCast,
        ],
    }],
    ..CardRules::EMPTY
};
