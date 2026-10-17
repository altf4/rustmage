// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            colored: [0, 1, 1, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: true,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 3,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::Any,
            optional: false,
            reveal: false,
            remainder: LibraryRemainder::BottomAny,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(73),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(73))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(73))],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
