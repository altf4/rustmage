// Direct Oracle and pinned XMage behavior; gated pending integration tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters, targets: TargetPlan::NONE,
            program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(5)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive, targets: TargetPlan::NONE,
            program: &[Instruction::CopyCastSpell { prior_spells: false }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1, first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPlaneswalkerActivation(0)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
        cycling: false, once_per_turn: false, sorcery: true, targets: TargetPlan::NONE,
        cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
            mana_component: false, tap: false, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: Some(-3) },
        program: &[Instruction::CopyNextInstantSorcery { ability: 1 }],
    }],
    ..CardRules::EMPTY
};
