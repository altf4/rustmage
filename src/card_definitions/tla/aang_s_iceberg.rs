// Direct typed behavior from Oracle and the pinned XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: true,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ExileUntilSourceLeaves],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost { generic: 3, mana_component: true, ..ActivationCost::ZERO },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::WaterbendCost(3),
            Instruction::SacrificeSourceOrSkip { skip: 1 },
            Instruction::Scry(2),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    keywords: Keywords::FLASH,
    ..CardRules::EMPTY
};
