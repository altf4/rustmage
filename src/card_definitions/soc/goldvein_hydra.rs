const TREASURE: Card = Card::from_catalogue_id(5302);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::VIGILANCE
        .union(Keywords::TRAMPLE)
        .union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: TREASURE,
            amount: AmountValue::SourcePower,
            tapped: true,
        }],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        x_entry_counters: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
