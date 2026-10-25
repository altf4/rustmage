// Direct typed behavior from complete frozen Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLYING.union(Keywords::FLASH),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::BlinkUntilEndStep { under_owner: true }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 1 }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
