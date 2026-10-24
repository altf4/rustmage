// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    extras: &CardExtras {
        prepared_spell: Some(Card(5368)),
        enters_prepared: true,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::Discard(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
