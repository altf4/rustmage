// Direct typed behavior; complete Oracle and XMage obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::PrepareSource],
    }],
    extras: &CardExtras {
        prepared_spell: Some(Card(5362)),
        enters_prepared: false,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
