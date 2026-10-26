pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::EnsureGoblinArmy,
            Instruction::AmassGoblinsThenAttach(1),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 4,
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
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
        }],
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 0,
            },
            keywords: Keywords::MENACE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
