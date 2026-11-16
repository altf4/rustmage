// Pinned Oracle and XMage: live Officer-token power and noncreature cast trigger.
const OFFICER_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Tokens(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 1,
        exclude: 0,
        subtype: Some("Officer"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6008),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, OFFICER_BONUS)],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[OFFICER_BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
