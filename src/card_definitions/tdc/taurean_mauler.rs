// Direct typed behavior; complete Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::CHANGELING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::Any,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
