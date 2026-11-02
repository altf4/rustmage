// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLYING
        .union(Keywords::FIRST_STRIKE)
        .union(Keywords::LIFELINK),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING
                .union(Keywords::FIRST_STRIKE)
                .union(Keywords::LIFELINK),
        )],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 3 }],
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
            program: &[
                Instruction::ActivationX,
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCycled,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card(5422),
            amount: AmountValue::ChosenX,
            tapped: false,
        }],
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 7,
            toughness: 7,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
