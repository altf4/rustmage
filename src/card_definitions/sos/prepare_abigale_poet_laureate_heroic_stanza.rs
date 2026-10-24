// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::Single(TargetSpec::one(TargetRule::Creature)), &[Instruction::PutCounters { subject: Subject::Targets, kind: CounterKind::PlusOne, amount: 1 }]),
    extras: &CardExtras { prepared_identity: Some("@prepare/abigale_poet_laureate_heroic_stanza"), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
