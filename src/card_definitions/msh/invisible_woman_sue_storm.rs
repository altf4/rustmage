// Direct typed Oracle behavior; the batch event follows Oracle's one-or-more recipients.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CounterBatchPlacedByYou {
            kind: CounterKind::PlusOne,
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Hero"),
                basic: false,
                other: true,
                cards_only: false,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::CreateTokens {
                token: Card(5390),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
