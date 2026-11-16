// Oracle and pinned XMage reviewed. The second resolution is counted per source ability.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DamagePlayers(PlayerSet::Opponent, 1),
            Instruction::IfThisAbilityResolutionNumber { number: 2, skip: 1 },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
