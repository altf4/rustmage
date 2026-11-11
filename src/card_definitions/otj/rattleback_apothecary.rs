// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CommittedCrime,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::ChooseGrantKeywords {
            subject: Subject::Targets,
            choices: &[Keywords::MENACE, Keywords::LIFELINK],
        }],
    }],
    ..CardRules::EMPTY
};
