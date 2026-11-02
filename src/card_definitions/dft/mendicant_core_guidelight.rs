// Both live power and max-speed artifact-copy permission use the current controller.
const DYNAMIC: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 16,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, DYNAMIC)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Artifact,
                ordinal: None,
            },
            guard: StepGuard::Expression {
                value: StatValue::ControllerSpeed,
                minimum: Some(4),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::CopyCastSpell {
                prior_spells: false,
            },
        ],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        dynamic_stats: Some(DYNAMIC),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
