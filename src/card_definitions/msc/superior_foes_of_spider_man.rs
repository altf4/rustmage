// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::ManaValueMinimum {
                filter: &CastFilter::Any,
                minimum: 4,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayExileTop { skip: 1 },
            Instruction::ExileTopPlayUntilSourceExilesAnother,
        ],
    }],
    ..CardRules::EMPTY
};
