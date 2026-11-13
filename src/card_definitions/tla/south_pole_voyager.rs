// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SourceOrSubtypeAny(&[
            "Ally",
        ])),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::IfThisAbilityResolutionNumber { number: 2, skip: 1 },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
