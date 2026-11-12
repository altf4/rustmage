pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksUnlessDefenderControlsCreatureSubtype("Glimmer"),
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords::MENACE.union(Keywords::DEATHTOUCH),
        }],
    }],
    ..CardRules::EMPTY
};
