// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::SourceChosenNumberMatchesSpell,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NumberRange {
            minimum: 0,
            maximum: 10,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
