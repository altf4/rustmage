// Complete printed obligations from pinned Oracle and XMage sources.
const CHAPTER: &[Instruction] = &[
    Instruction::CreateTokens {
        token: Card(5302),
        amount: 1,
        tapped: false,
    },
    Instruction::IfAmountAtLeast {
        value: AmountValue::Stat(StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: Some("Treasure"),
            basic: false,
            other: false,
            cards_only: false,
        })),
        minimum: 4,
        skip: 2,
    },
    Instruction::SacrificeSourceOrSkip { skip: 1 },
    Instruction::CreateTokens {
        token: Card(5416),
        amount: 1,
        tapped: false,
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: CHAPTER,
        },
    ],
    ..CardRules::EMPTY
};
