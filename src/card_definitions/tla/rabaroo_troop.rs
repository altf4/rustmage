// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::FLYING,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: true,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Subtype("Plains"),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    }],
    ..CardRules::EMPTY
};
