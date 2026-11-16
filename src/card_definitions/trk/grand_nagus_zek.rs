// Pinned XMage: live controlled Treasure count and one trigger each turn for
// any other creature's actual death.
const TREASURE_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 16,
        exclude: 0,
        subtype: Some("Treasure"),
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
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, TREASURE_BONUS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::CreatureDies {
            relation: CastRelation::Any,
            other: true,
            nontoken: false,
            subtype: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5302),
            amount: 1,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[TREASURE_BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
