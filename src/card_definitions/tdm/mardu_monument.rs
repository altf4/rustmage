// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[
                SearchFilter::BasicLand,
                SearchFilter::AnyOf(&[
                    SearchFilter::Subtype("Mountain"),
                    SearchFilter::Subtype("Plains"),
                    SearchFilter::Subtype("Swamp"),
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
            generic: 2,
            colored: [1, 0, 1, 1, 0, 0],
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensWithTemporaryKeywords {
            token: Card(5255),
            amount: AmountValue::Fixed(3),
            keywords: Keywords::MENACE.union(Keywords::HASTE),
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
