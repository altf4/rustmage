pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCombatRed(2)],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::OwnTurnGraveyardFlashbackExceptSubtype("Lesson")],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::OwnTurnGraveyardSubtypeFlashback {
                subtype: "Lesson",
                cost: generic(1),
            }],
        },
    ],
    ..CardRules::EMPTY
};
