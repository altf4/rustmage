// Complete Oracle and pinned XMage implementation.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
const fn ability(amount: i32, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: loyalty(amount),
        targets: TargetPlan::NONE,
        program,
        sorcery: true,
        once_per_turn: false,
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 4,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Loyalty,
            amount: 1,
        }],
    }],
    activated: &[
        ability(
            1,
            &[Instruction::CreateTokens {
                token: Card(5467),
                amount: 1,
                tapped: false,
            }],
        ),
        ability(
            -3,
            &[
                Instruction::Draw(PlayerSet::Controller, 3),
                Instruction::Discard(PlayerSet::Controller, 2),
            ],
        ),
        ability(
            -10,
            &[
                Instruction::Draw(PlayerSet::Controller, 3),
                Instruction::CreateEmblem(Card(5468)),
            ],
        ),
    ],
    ..CardRules::EMPTY
};
