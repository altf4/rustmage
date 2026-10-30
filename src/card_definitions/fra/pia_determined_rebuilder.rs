// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5266),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: one(TargetRule::Creature),
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 5,
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
        program: &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
            power: 1,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
