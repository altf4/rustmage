// Direct typed behavior from the paired Oracle and pinned XMage sources.
const DAMAGE: ReplacementSpec =
    ReplacementSpec::ControlledDamageToOpponentPlusSourceCounters(CounterKind::Fire);
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DAMAGE)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    static_groups: GROUPS,
    extras: &CardExtras {
        replacement_effects: &[DAMAGE],
        x_symbols: 1,
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ChosenX(CounterKind::Fire),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
