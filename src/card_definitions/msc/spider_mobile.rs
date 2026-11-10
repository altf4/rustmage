// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksOrBlocks,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowScaled {
            subject: Subject::Source,
            amount: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Spider"),
                basic: false,
                other: false,
                cards_only: false,
            })),
            power: 1,
            toughness: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 2 }],
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
