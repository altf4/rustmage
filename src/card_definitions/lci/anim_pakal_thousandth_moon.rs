const GNOME: Card = Card::from_catalogue_id(5267);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledSubtypeAttacks {
            subtype: "Gnome",
            exclude: true,
            one_or_more: true,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::CreateTokensAttackingScaled {
                token: GNOME,
                amount: AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
            },
        ],
    }],
    ..CardRules::EMPTY
};
