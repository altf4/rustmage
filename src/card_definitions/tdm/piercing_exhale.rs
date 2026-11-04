// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 65,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::CreaturePowerDamageAny {
                source: 0,
                recipient: 1,
            },
            Instruction::IfBeheld {
                beheld: true,
                skip: 1,
            },
            Instruction::Surveil(2),
        ],
    ),
    extras: &CardExtras {
        behold_cast: Some(BeholdCastSpec {
            subtype: "Dragon",
            decline_generic: 0,
            grants_flash: false,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
