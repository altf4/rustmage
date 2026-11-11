pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::SourceChosenNumberMatchesSpell,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::CastingPlayer, 2),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NumberRange {
            minimum: 1,
            maximum: 10,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
