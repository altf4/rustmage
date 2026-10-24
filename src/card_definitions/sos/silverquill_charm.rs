// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode { text: "Put two +1/+1 counters on target creature.", targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)), program: &[Instruction::PutCounters { subject: Subject::Targets, kind: CounterKind::PlusOne, amount: 2 }] },
        SpellMode { text: "Exile target creature with power 2 or less.", targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered { base: &TargetRule::Creature, predicate: &TargetPredicate::PowerAtMost(2) })), program: &[Instruction::Exile] },
        SpellMode { text: "Each opponent loses 3 life and you gain 3 life.", targets: TargetPlan::NONE, program: &[Instruction::LoseLife(PlayerSet::Opponent,3), Instruction::GainLife(PlayerSet::Controller,3)] },
    ]),
    ..CardRules::EMPTY
};
