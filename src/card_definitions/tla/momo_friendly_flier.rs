// Direct typed behavior from the pinned Oracle and XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::FirstQualifiedCreature(CreatureCastQualifier::FlyingNonLemur),
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Own,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherFilteredEntersYou(EntryPredicate::CreatureWithKeyword(
            Keywords::FLYING,
        )),
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 1,
            toughness: 1,
        }],
    }],
    ..CardRules::EMPTY
};
