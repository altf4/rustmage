// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ExertCost,
                Instruction::CreateTokens {
                    token: Card(5252),
                    amount: 1,
                    tapped: false,
                },
            ],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Hand,
            cycling: true,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 2,
                mana_component: true,
                discard: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCycled,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantPermanentGroupKeywords {
            filters: &[BattlefieldFilter {
                kinds: 0,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: Some("Cat"),
                max_mana: u32::MAX,
            }],
            keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
        }],
    }],
    ..CardRules::EMPTY
};
