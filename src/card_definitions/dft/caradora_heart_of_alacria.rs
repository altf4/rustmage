// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::AnyOf(&[
                    SearchFilter::Subtype("Mount"),
                    SearchFilter::Subtype("Vehicle"),
                ]),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
        ],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            ReplacementSpec::Counters {
                filter: ReplacementFilter::CreatureOrVehicle,
                kind: Some(CounterKind::PlusOne),
                factor: 1,
                add: 1,
                effects_only: false,
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(
                0,
                ReplacementSpec::Counters {
                    filter: ReplacementFilter::CreatureOrVehicle,
                    kind: Some(CounterKind::PlusOne),
                    factor: 1,
                    add: 1,
                    effects_only: false,
                },
            )],
        }],
        replacement_effects: &[ReplacementSpec::Counters {
            filter: ReplacementFilter::CreatureOrVehicle,
            kind: Some(CounterKind::PlusOne),
            factor: 1,
            add: 1,
            effects_only: false,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
