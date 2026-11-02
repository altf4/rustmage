// One deduplicated entry replacement counts creatures OR Vehicles using entry-preview values.
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OtherControlledEntryCountersByManaValue {
                filter: ReplacementFilter::CreatureOrVehicle,
                kind: CounterKind::PlusOne,
                maximum: 4,
                at_most: 1,
                above: 3,
            },
        ],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::TRAMPLE,
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 3 }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        vehicle_stats: Some(Stats {
            power: 5,
            toughness: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
