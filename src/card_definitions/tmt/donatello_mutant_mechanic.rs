// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentLeaves(DepartureFilter {
            kinds: 16,
            relation: CastRelation::You,
            other: false,
            graveyard_only: true,
            counters: DepartureCounters::Any,
        }),
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 17,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::PutDepartureCounters {
            subject: Subject::Targets,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 1,
            max: 1,
        }),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
            Instruction::AnimateTargetsIfNoncreature(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::AddNamed(&["Robot"]),
                base: Some((0, 0)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            }),
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
