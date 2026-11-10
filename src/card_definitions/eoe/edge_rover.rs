// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensFor {
            players: PlayerSet::Each,
            token: Card(5238),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
