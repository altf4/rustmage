// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTransforms,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExtraTurn {
            power_up_prohibited: false,
        }],
    }],
    ..CardRules::EMPTY
};
