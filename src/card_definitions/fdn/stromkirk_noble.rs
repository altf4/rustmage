// Direct typed behavior; complete Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::NO_HUMAN_BLOCKERS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
