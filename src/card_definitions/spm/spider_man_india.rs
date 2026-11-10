// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Creature,
            ordinal: None,
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FLYING,
            },
        ],
    }],
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 1, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
