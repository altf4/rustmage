// Direct typed behavior from the pinned Oracle and XMage sources.
const COLORS: AmountValue = AmountValue::Stat(StatValue::Colors(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
}));
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::All,
                filter: LibrarySelectionFilter::Kinds(1 | 2 | 16 | 32 | 64 | 256),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::UntilCount {
                kinds: 1 | 2 | 16 | 32 | 64 | 256,
                amount: COLORS,
            },
            destination: LibraryPickDestination::Battlefield,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    }],
    ..CardRules::EMPTY
};
