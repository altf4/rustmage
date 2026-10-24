// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        prepared_spell: Some(Card(5382)),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::TokensEnterYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::PrepareSource],
    }],
    ..CardRules::EMPTY
};
