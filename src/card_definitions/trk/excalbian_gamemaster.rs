const DIFFERENT_LANDS: AmountValue = AmountValue::Stat(StatValue::DistinctNames(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
}));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: Card::from_catalogue_id(6008),
                amount: DIFFERENT_LANDS,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfOrControlledPermanentDies(1),
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
