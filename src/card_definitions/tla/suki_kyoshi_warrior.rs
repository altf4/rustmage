const POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 1,
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

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensAttacking {
            token: Card(5272),
            amount: 1,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, POWER)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(POWER),
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 2,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
