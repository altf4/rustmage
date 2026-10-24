// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CardsLeaveGraveyard,
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
