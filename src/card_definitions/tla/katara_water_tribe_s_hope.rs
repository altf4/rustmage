pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5272),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::OwnTurn,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::WaterbendXCost { minimum: 1 },
            Instruction::SetPermanentGroupBaseStats {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exclude_source: false,
                power: Some(ResolvedStatValue::ChosenX),
                toughness: Some(ResolvedStatValue::ChosenX),
            },
        ],
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
