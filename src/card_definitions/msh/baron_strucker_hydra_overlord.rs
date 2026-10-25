// Whole Oracle behavior: the untargeted entrant connives for its current/last controller.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Subtype("Villain"),
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    }],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherSubtypeEntersYou("Villain"),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayOncePerTurn { skip: 2 },
            Instruction::RecordOncePerTurn,
            Instruction::Connive {
                subject: Subject::Entering,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
