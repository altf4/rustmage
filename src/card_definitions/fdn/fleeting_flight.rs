pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::Single(TargetSpec::one(TargetRule::Creature)), &[
      Instruction::PutCounters { subject: Subject::Targets, kind: CounterKind::PlusOne, amount: 1 },
      Instruction::GrantKeywords { subject: Subject::Targets, keywords: Keywords::FLYING.union(Keywords::PREVENT_COMBAT_TO) }
    ]),
    ..CardRules::EMPTY
};
