// Direct typed behavior from the full frozen Oracle and pinned Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Targeted {
                filter: &CastFilter::InstantSorcery,
                target: CastTarget::PermanentKinds(16 | 2),
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CopyCastSpell {
                prior_spells: false,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};
