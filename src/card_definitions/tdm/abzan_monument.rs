// Directly authored from the pinned complete Oracle/XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[
                SearchFilter::BasicLand,
                SearchFilter::AnyOf(&[
                    SearchFilter::Subtype("Plains"),
                    SearchFilter::Subtype("Swamp"),
                    SearchFilter::Subtype("Forest"),
                ]),
            ]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 0, 1, 0, 1, 0],
            generic: 1,
            tap: true,
            sacrifice: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateSizedToken {
            token: Card(5326),
            size: AmountValue::GreatestCreatureToughness,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
