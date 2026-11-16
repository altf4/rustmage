// Pinned Oracle; no XMage class exists at the audited revision. The tapped
// trigger installs a source-independent counter replacement until cleanup.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesTapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::IncreaseControlledPermanentCountersThisTurn { amount: 1 }],
    }],
    ..CardRules::EMPTY
};
