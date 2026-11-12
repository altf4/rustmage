// Direct typed behavior from complete pinned Oracle and XMage sources.
const LANDS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureScaling {
            subtype: None,
            value: StatValue::AtLeast {
                value: &LANDS,
                minimum: 10,
            },
            stats: Stats {
                power: 2,
                toughness: 2,
            },
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SourceOr(
            &EntryPredicate::CreatureToughnessGreaterThanPower,
        )),
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 1,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(2),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::Hand,
            },
            look: LibraryLook::Fixed(1),
            destination: LibraryPickDestination::BattlefieldTapped,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
