// Direct typed behavior from the pinned Oracle and XMage sources.
const AUDIT: &[Instruction] = &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
    spec: LibrarySelectionSpec {
        look: 6,
        count: LibrarySelectionCount::Fixed(1),
        filter: LibrarySelectionFilter::Kinds(32),
        optional: true,
        reveal: true,
        remainder: LibraryRemainder::BottomRandom,
    },
    look: LibraryLook::Fixed(6),
    destination: LibraryPickDestination::Hand,
    public: false,
    maximum_mana_value: None,
})];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: AUDIT,
        },
        TriggeredAbility {
            condition: TriggerCondition::CaseSolved(CastRelation::You),
            targets: TargetPlan::NONE,
            program: AUDIT,
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpendAnyColorForSpellSubtype("Case")],
    }],
    ..CardRules::EMPTY
};
