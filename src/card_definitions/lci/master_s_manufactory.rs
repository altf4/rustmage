// An exact source entry and a general artifact entry each qualify independently.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: StatValue::Sum(&[
                StatValue::SourceEnteredUnderYourControlThisTurn,
                StatValue::ArtifactEnteredThisTurn,
            ]),
            minimum: 1,
        },
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5574),
            amount: 1,
            tapped: false,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
