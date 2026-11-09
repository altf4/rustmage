// Direct typed behavior from the pinned Oracle and XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::SourceChosenCreatureType,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const PROGRAM: &[Instruction] = &[Instruction::SelectLibrary(LibrarySelectionSpec {
    look: 1,
    count: LibrarySelectionCount::Fixed(1),
    filter: LibrarySelectionFilter::SourceChosenCreatureType,
    optional: true,
    reveal: true,
    remainder: LibraryRemainder::OptionalGraveyard,
})];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: PROGRAM,
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: PROGRAM,
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        cost_reductions: &[REDUCTION],
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
