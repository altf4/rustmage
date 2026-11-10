// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::NontokenArtifact),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5538),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
