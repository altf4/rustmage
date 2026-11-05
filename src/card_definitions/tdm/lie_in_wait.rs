// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[
            Instruction::ReturnToHand,
            Instruction::SelectGroup(1),
            Instruction::DamageByTargetPower { power_group: 0 },
        ],
    ),

    ..CardRules::EMPTY
};
