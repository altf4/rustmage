// Direct typed behavior; complete pinned Oracle and XMage source reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    deck_limit: None,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::CreateTokensScaled {
            token: Card(5285),
            amount: AmountValue::Stat(StatValue::OtherNamedCreatures("Hare Apparent")),
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
