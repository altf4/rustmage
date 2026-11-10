const CREATURE_OR_SPACECRAFT: TargetRule = TargetRule::Union(&[
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 1,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: 5,
    },
    TargetRule::Filtered {
        base: &TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 5,
        },
        predicate: &TargetPredicate::Subtype("Spacecraft"),
    },
]);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(CREATURE_OR_SPACECRAFT)),
        &[
            Instruction::ReturnToBattlefield,
            Instruction::CreateTokens {
                token: Card(5238),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 17,
                nonland: false,
                other: false,
                tokens: None,
                subtype: None,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
