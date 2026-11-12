const FORESTS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 2,
        exclude: 0,
        subtype: Some("Forest"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, FORESTS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[SearchFilter::Land, SearchFilter::Subtype("Forest")]),
            maximum: 2,
            destination: SearchDestination::BattlefieldThenHand { battlefield: 1 },
            tapped: true,
            reveal: true,
        })],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[FORESTS],
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 4,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
