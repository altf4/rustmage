const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Source,
    attack: true,
    block: false,
    unless: Some(StepGuard::Expression {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: Some("Wolf"),
            basic: false,
            other: true,
            cards_only: false,
        }),
        minimum: Some(2),
        maximum: None,
    }),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(0, RESTRICTION)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5412),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
