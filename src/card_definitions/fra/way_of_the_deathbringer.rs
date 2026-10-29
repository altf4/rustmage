// Direct Oracle and pinned XMage behavior; gated pending complete interaction validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters, targets: TargetPlan::NONE,
        program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(5)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0, first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPlaneswalkerActivation(0)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
        cycling: false, once_per_turn: false, sorcery: true, targets: TargetPlan::NONE,
        cost: ActivationCost {
            colored: [0; 6], generic: 0, alternatives: &[], additional: None,
            mana_component: false, tap: false, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: Some(-2),
        },
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice, players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false,
                    other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None,
                },
                amount: 1, optional_skip: Some(1),
            },
            Instruction::CreateTokens { token: Card(5239), amount: 1, tapped: false },
        ],
    }],
    ..CardRules::EMPTY
};
