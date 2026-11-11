// Direct typed behavior from the paired Oracle and pinned XMage sources.
// A kindred Rat can receive the power modifier before it becomes a creature.
const RATS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Rat"),
    basic: false,
    other: false,
    cards_only: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5323),
                amount: 2,
                tapped: false,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Count(RATS),
                    minimum: Some(5),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::GrowFilteredScaled {
                filter: RATS,
                amount: AmountValue::Fixed(1),
                power: 2,
                toughness: 0,
            },
        ],
    }],
    ..CardRules::EMPTY
};
