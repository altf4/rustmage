// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 6),
    keywords: Keywords::REACH
        .union(Keywords::VIGILANCE)
        .union(Keywords::MENACE)
        .union(Keywords::TRAMPLE)
        .union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Permanent,
                },
                predicate: &TargetPredicate::Nonlegendary,
            }),
        ]),
        program: &[
            Instruction::SelectGroup(0),
            Instruction::Destroy,
            Instruction::SelectGroup(1),
            Instruction::ReturnToBattlefield,
        ],
    }],
    extras: &CardExtras {
        meld_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
