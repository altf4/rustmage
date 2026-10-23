// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayEnteringPower { skip: 1 },
            Instruction::PayEnteringPowerLifeAndCounters,
        ],
    }],
    ..CardRules::EMPTY
};
