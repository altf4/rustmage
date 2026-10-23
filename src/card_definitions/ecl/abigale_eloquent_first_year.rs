// Direct typed behavior from complete Oracle and pinned XMage sources.
const INTRINSIC: Keywords = Keywords::FLYING
    .union(Keywords::FIRST_STRIKE)
    .union(Keywords::LIFELINK);
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(INTRINSIC)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: INTRINSIC,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            min: 0,
            max: 1,
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        }),
        program: &[
            Instruction::RemoveAbilities {
                subject: Subject::Targets,
                duration: AnimationDuration::Permanent,
            },
            Instruction::PutCounterKinds {
                subject: Subject::Targets,
                counters: &[
                    (CounterKind::Flying, 1),
                    (CounterKind::FirstStrike, 1),
                    (CounterKind::Lifelink, 1),
                ],
            },
        ],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 0,
            },
        ],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
