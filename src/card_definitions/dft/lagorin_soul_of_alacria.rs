// Saddled attacks may choose zero, one or two Mounts/Vehicles of either controller.
const PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWhileSaddled,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Union(&[
                TargetRule::Filtered {
                    base: &PERMANENT,
                    predicate: &TargetPredicate::Subtype("Mount"),
                },
                TargetRule::Filtered {
                    base: &PERMANENT,
                    predicate: &TargetPredicate::Subtype("Vehicle"),
                },
            ]),
            min: 0,
            max: 2,
        }),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 1 }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
