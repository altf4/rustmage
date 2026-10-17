// Gated core fixture: ordinary typed obligations, not an admission declaration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::SourceChosenColor,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
