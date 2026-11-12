// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutHandFiltered {
            filter: SearchFilter::Kinds(1 | 2 | 16 | 32 | 64 | 256),
            skip: 0,
        }],
    }],
    ..CardRules::EMPTY
};
