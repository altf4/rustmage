// Direct typed behavior; complete pinned Oracle and XMage source reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::REACH.union(Keywords::FIRST_STRIKE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            min: 1,
            max: 1,
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        }),
        program: &[
            Instruction::PutCountersScaled {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: AmountValue::SourcePower,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
