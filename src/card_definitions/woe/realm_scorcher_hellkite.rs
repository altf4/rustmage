// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    keywords: Keywords::FLYING.union(Keywords::HASTE),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLYING.union(Keywords::HASTE),
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBargained,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Combination {
            mask: 31,
            amount: 4,
        })],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::Damage(1)],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
