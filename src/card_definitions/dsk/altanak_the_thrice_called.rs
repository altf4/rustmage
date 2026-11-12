// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(9, 9),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTargeted {
            relation: CastRelation::Opponent,
            spells_only: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ReturnToBattlefieldTapped],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
