const KEYWORDS: Keywords = Keywords::REACH.union(Keywords::VIGILANCE);
const OTHER_OWN: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: KEYWORDS,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYWORDS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCreatureGroupCounters {
                filters: &[
                    CreatureFilter {
                        tokens: Some(true),
                        ..OTHER_OWN
                    },
                    CreatureFilter {
                        subtype: Some("Rabbit"),
                        ..OTHER_OWN
                    },
                ],
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::If {
                guard: StepGuard::Threshold {
                    value: StepValue::TotalCreaturePower,
                    minimum: Some(10),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
