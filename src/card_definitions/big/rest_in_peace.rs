// Direct typed behavior from the full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileGraveyard(PlayerSet::Each)],
    }],
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileAllCardsAndTokens),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
