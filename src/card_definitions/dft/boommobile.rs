// Complete Oracle/XMage: activated-cost-only colored mana; X belongs to the exhaust header.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Choice {
                mask: 31,
                amount: 4,
            },
            quality: &ManaQuality {
                restriction: ManaRestriction::AbilityActivation,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: true,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                colored: [0, 0, 0, 1, 0, 0],
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
            program: &[
                Instruction::ActivationX,
                Instruction::DamageScaled(AmountValue::ChosenX),
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 2 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 5,
            toughness: 5,
        }),
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
