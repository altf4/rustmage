// Tezzeret's command-zone emblem, from pinned Oracle and XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    deck_limit: None,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
            Instruction::AnimateNoncreatureTargets(AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Named(&["Robot"]),
                base: Some((0, 0)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
