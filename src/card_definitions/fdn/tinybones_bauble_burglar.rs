// Direct typed Oracle behavior; full pinned XMage source and dependencies reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Discarded {
            relation: CastRelation::Opponent,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileDiscardedWithStash],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ForeignStashPlay],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        guard: ActivationGuard::Always,
        once_per_turn: false,
        exhaust: false,
        targets: TargetPlan::NONE,
        sorcery: true,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 3,
            alternatives: &[],
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            loyalty: None,
            remove: None,
            additional: None,
        },
        program: &[Instruction::Discard(PlayerSet::Opponent, 1)],
    }],
    ..CardRules::EMPTY
};
