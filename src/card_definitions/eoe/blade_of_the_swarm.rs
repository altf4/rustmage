pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put two +1/+1 counters on this creature.",
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 2,
                }],
            },
            SpellMode {
                text: "Put target exiled card with warp on the bottom of its owner's library.",
                targets: one(TargetRule::ExileWithWarp),
                program: &[Instruction::BottomLibrary],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
