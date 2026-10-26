const MENACE_RECIPIENTS: &[BattlefieldFilter] = &[
    BattlefieldFilter {
        kinds: 0,
        relation: 0,
        nonland: false,
        tapped: None,
        without: Keywords::NONE,
        subtype: Some("Goblin"),
        max_mana: u32::MAX,
    },
    BattlefieldFilter {
        kinds: 0,
        relation: 0,
        nonland: false,
        tapped: None,
        without: Keywords::NONE,
        subtype: Some("Orc"),
        max_mana: u32::MAX,
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::ReturnToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureCardLeavesYourGraveyard,
            targets: TargetPlan::NONE,
            program: &[Instruction::EnsureGoblinArmy, Instruction::AmassGoblins(1)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
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
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantPermanentGroupKeywords {
            filters: MENACE_RECIPIENTS,
            keywords: Keywords::MENACE,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
