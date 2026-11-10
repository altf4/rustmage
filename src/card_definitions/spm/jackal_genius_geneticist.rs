pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::CreatureManaValueEqualsSourcePower,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CopyCapturedSpellModified {
                retarget: false,
                modification: CapturedSpellModification::Nonlegendary,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
