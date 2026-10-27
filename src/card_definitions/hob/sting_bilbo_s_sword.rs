// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 1,
            },
        ]),
        program: &[
            Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::Hone,
                amount: AmountValue::TargetPlayerCreatureCount,
            },
            Instruction::SelectGroup(1),
            Instruction::AttachSource,
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 3,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
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
