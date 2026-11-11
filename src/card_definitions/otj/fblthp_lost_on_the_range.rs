pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: generic(2),
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::PlotFromLibraryTop],
        },
    ],
    ..CardRules::EMPTY
};
