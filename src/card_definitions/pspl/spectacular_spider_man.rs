// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLASH,
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::FLYING,
            }],
        },
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantCreatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
            }],
        },
    ],
    ..CardRules::EMPTY
};
