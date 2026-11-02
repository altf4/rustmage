// Restricted power-scaled mana and one private simultaneous all-permanent deployment.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::VIGILANCE,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::TapSourcePowerChoice { mask: 31 },
                quality: &ManaQuality {
                    restriction: ManaRestriction::AbilityActivation,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: true,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 10,
                colored: [0, 0, 0, 1, 1, 0],
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DeployHandPermanents],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        exhaust_mask: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
