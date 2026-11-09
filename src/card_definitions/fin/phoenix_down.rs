// Directly authored from the paired Oracle and pinned XMage sources.
const PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
            mana_component: true,
            tap: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Return your target creature card with mana value 4 or less tapped.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: 4,
                })),
                program: &[Instruction::ReturnToBattlefieldTapped],
            },
            SpellMode {
                text: "Exile target Skeleton, Spirit, or Zombie.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
                    TargetRule::Filtered {
                        base: &PERMANENT,
                        predicate: &TargetPredicate::Subtype("Skeleton"),
                    },
                    TargetRule::Filtered {
                        base: &PERMANENT,
                        predicate: &TargetPredicate::Subtype("Spirit"),
                    },
                    TargetRule::Filtered {
                        base: &PERMANENT,
                        predicate: &TargetPredicate::Subtype("Zombie"),
                    },
                ]))),
                program: &[Instruction::Exile],
            },
        ])],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
