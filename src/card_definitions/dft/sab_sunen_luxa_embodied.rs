// Counter parity includes all named/hot kinds, with zero even; draw checks after placement.
const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Source,
    attack: true,
    block: true,
    unless: Some(StepGuard::Expression {
        value: StatValue::Parity {
            value: &StatValue::SourceTotalCounters,
            odd: false,
        },
        minimum: Some(1),
        maximum: None,
    }),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::REACH
                .union(Keywords::TRAMPLE)
                .union(Keywords::INDESTRUCTIBLE),
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(0, RESTRICTION)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::REACH
        .union(Keywords::TRAMPLE)
        .union(Keywords::INDESTRUCTIBLE),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::IfAmountParity {
                value: AmountValue::SourceTotalCounters,
                odd: true,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
