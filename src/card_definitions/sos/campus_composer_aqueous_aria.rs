// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::NONE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: generic(2),
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    extras: &CardExtras {
        prepared_spell: Some(Card(5354)),
        enters_prepared: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
