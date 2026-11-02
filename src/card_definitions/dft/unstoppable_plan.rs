// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapPermanentGroup(&[BattlefieldFilter {
            kinds: 0,
            relation: 0,
            nonland: true,
            tapped: None,
            without: Keywords::NONE,
            subtype: None,
            max_mana: u32::MAX,
        }])],
    }],
    ..CardRules::EMPTY
};
