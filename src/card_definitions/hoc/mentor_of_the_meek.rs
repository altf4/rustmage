// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::CreaturePowerAtMost(2)),
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::MayPayMana {
                cost: generic(1),
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
