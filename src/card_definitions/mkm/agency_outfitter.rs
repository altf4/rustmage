// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchGraveyardHandLibraryForNames([
            "Magnifying Glass",
            "Thinking Cap",
        ])],
    }],
    ..CardRules::EMPTY
};
