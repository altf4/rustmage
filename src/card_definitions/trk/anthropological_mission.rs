// Pinned Oracle and complete XMage class: count the quest counter before discard.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::SourceCounters(CounterKind::Quest)),
                minimum: 3,
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
