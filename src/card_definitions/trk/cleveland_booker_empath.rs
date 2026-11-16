// Pinned Oracle and complete XMage class; Worm token identity is shared TRK 6014.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6014),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 8,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowPlayerCreatures {
            players: PlayerSet::Controller,
            power: 2,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
