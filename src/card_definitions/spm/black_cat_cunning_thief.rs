pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::SelectTargetLibraryFaceDownPlay {
            look: 9,
            count: 2,
            spend_any_mana: true,
        }],
    }],
    ..CardRules::EMPTY
};
