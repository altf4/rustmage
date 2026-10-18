// Complete Oracle body authored from the paired pinned XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::TRAMPLE,
    extras: &CardExtras {
        graveyard_counter_cost: Some(6),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureCombatDamageOwnTurn,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCombatDealerCounters],
    }],
    ..CardRules::EMPTY
};
