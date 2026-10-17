// Direct typed behavior; complete Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureAttacksYou { colors: 1 },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
