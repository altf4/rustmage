// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6,6),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility { condition: TriggerCondition::SelfEnters, targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object { zone: TargetZone::Battlefield, kinds: 1, relation: 1, other: false, nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX })), program: &[Instruction::Tap, Instruction::PutCounters { subject: Subject::Targets, kind: CounterKind::Stun, amount: 1 }] }],
    ..CardRules::EMPTY
};
