// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Controller,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::Paradigm { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CastParadigmCopy],
    }],
    ..CardRules::EMPTY
};
