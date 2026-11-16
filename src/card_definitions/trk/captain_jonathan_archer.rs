// Oracle and pinned XMage reviewed. Federation excludes Borg creature types.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6013),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: AmountValue::Stat(StatValue::DistinctControlledCreatureTypesExcept("Borg")),
            power: 1,
            toughness: 1,
        }],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
