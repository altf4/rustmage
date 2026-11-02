// Search decline does not shuffle; the selected artifact's own mana value decides its destination.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::Artifact,
                maximum: 1,
                destination: SearchDestination::BattlefieldIfManaValueAtMostElseHand(2),
                tapped: false,
                reveal: true,
            }),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 2 }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
