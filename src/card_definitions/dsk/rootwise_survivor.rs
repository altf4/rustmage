// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::HASTE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
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
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
            Instruction::AnimateTargets(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
                base: Some((0, 0)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            }),
            Instruction::GrantKeywordsUntilNextTurn {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
