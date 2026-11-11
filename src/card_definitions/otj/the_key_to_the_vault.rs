pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedDealtCombatDamageToPlayer,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SelectLibraryToExileRemember(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::ExcludingKinds(2),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::Amount(AmountValue::CreatureCombatDamageAmount),
                destination: LibraryPickDestination::Exile,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::CastRememberedExileFree,
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
