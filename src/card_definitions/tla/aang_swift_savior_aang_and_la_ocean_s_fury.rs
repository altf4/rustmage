// Direct typed behavior from Oracle and the pinned XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING.union(Keywords::FLASH),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Union(&[
                TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: -1,
                    other: true,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                TargetRule::Spell,
            ]),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::AirbendTargets],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost { generic: 8, mana_component: true, ..ActivationCost::ZERO },
        targets: TargetPlan::NONE,
        program: &[Instruction::WaterbendCost(8), Instruction::TransformSource],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras { transforming_double_face: true, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
