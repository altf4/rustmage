// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::Single(TargetSpec::one(TargetRule::Creature)), &[Instruction::Mill(PlayerSet::Controller,MillAmount::Fixed(1)), Instruction::PutCounters { subject: Subject::Targets, kind: CounterKind::PlusOne, amount: 2 }, Instruction::GrantKeywords { subject: Subject::Targets, keywords: Keywords::TRAMPLE }]),
    extras: &CardExtras { prepared_identity: Some("@prepare/kirol_history_buff_pack_a_punch"), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
