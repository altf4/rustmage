// Storm is granted to the actual instant/sorcery spell, not an observer ability.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GrantInstantSorceryStorm { ability: 1 }],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(5)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Reflexive),
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyStormSpell],
        },
    ],
    ..CardRules::EMPTY
};
