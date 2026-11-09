pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::IfKicked { skip: 2 },
            Instruction::ScaledPowerDamage {
                source: 0,
                recipient: 1,
                factor: 2,
            },
            Instruction::Skip(1),
            Instruction::CreaturePowerDamageAny {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost::ZERO),
        kicker_additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::ReturnToHand,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 2,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
