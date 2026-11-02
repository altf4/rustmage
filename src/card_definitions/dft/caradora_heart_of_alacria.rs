// This replacement adds one per counter-placement event, including noncreature Vehicles.
const REPLACEMENT: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::CreatureOrVehicle,
    kind: Some(CounterKind::PlusOne),
    factor: 1,
    add: 1,
    effects_only: false,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, REPLACEMENT)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
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
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        replacement_effects: &[REPLACEMENT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
