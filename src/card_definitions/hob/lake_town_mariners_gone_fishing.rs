// Both printed faces authored directly from complete Oracle and pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: ManaCost {
                colored: [0; 6],
                generic: 2,
            },
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Lake-town Mariners // Gone Fishing"),
        adventure_spell: Some(Card(5415)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
