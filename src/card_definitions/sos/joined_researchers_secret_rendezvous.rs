// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FIRST_STRIKE,
    extras: &CardExtras {
        prepared_spell: Some(Card(5369)),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::OpponentHasMoreCardsInHand,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PrepareSource],
    }],
    ..CardRules::EMPTY
};
