// Authored from pinned Oracle and complete XMage source; awaiting rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEntersYou,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::EnteredBattlefieldThisTurn,
        })),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
