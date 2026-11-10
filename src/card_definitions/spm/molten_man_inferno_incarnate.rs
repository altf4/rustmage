const MOUNTAIN_BOOST: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 2,
        exclude: 0,
        subtype: Some("Mountain"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};

const MOUNTAIN_ABILITY: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, MOUNTAIN_BOOST)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::AllOf(&[
                    SearchFilter::BasicLand,
                    SearchFilter::Subtype("Mountain"),
                ]),
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: true,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 2,
                    source_only: false,
                    other: false,
                    nonland: false,
                    tokens: None,
                    subtype: None,
                    keywords: Keywords::NONE,
                    artifact_enchantment_or_token: false,
                },
                amount: 1,
                optional_skip: None,
            }],
        },
    ],
    static_groups: MOUNTAIN_ABILITY,
    extras: &CardExtras {
        static_ability_groups: MOUNTAIN_ABILITY,
        scaling_modifiers: &[MOUNTAIN_BOOST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
