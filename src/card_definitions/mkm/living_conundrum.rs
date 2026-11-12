// Direct typed behavior from the pinned Oracle and XMage sources.
const EMPTY_LIBRARY: StatValue = StatValue::ControllerLibraryEmpty;
const EMPTY_BASE: BaseStatSetter = BaseStatSetter {
    power: Some(10),
    toughness: Some(10),
    own_turn: false,
    condition: Some(EMPTY_LIBRARY),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    keywords: Keywords::HEXPROOF,
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::SkipEmptyLibraryDraws],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::SourceConditionalKeywords {
                    condition: EMPTY_LIBRARY,
                    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
                },
                StaticAbilityFragment::BaseStatSetter(0, EMPTY_BASE),
            ],
        },
    ],
    extras: &CardExtras {
        base_stat_setter: Some(EMPTY_BASE),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
