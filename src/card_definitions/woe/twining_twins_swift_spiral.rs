pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: generic(1),
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5562)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
