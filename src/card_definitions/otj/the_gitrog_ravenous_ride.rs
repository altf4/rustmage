const KEYWORDS: Keywords = Keywords::TRAMPLE.union(Keywords::HASTE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: KEYWORDS,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYWORDS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseSaddleRiders(SaddleRiderEffect::SacrificeOneForPower),
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::SacrificedPower),
            Instruction::PutHandLands {
                maximum: AmountValue::SacrificedPower,
                tapped: true,
            },
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 1 }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
