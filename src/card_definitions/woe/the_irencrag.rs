// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::LegendaryCreature),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayIfSourceOnBattlefield { skip: 1 },
            Instruction::BecomeLegendaryEquipment {
                name: "Everflame, Heroes' Legacy",
                equip: 1,
                power: 3,
                toughness: 3,
            },
        ],
    }],
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
        },
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            once_per_turn: false,
            sorcery: true,
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AttachSource],
        },
    ],
    extras: &CardExtras {
        self_granted_activation_templates: &[1],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
