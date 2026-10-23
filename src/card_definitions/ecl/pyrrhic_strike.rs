// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
const CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 3,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy artifact or enchantment",
            targets: one(object(16 | 32)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy creature with mana value 3 or greater",
            targets: one(CREATURE),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy artifact or enchantment, then destroy creature with mana value 3 or greater",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(object(16 | 32)),
                TargetSpec::one(CREATURE),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
    ]),
    extras: &CardExtras {
        mode_optional_additional_paid: &[false, false, true],
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::OptionalBlightCost(2),
            filter: SacrificeFilter {
                kinds: 1,
                keywords: Keywords::NONE,
                source_only: false,
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
