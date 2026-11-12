pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::LoseHalfLifeRoundedUp(PlayerSet::EventPlayer)],
        },
        TriggeredAbility {
            condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::SourceWithoutCounters),
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnDeathCardWithCounters {
                under_controller: false,
                tapped: true,
                counter: CounterKind::Stun,
                amount: 2,
            }],
        },
    ],
    ..CardRules::EMPTY
};
