// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Discarded {
                relation: CastRelation::You,
                one_or_more: true,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamageDiscardAmount(PlayerSet::Opponent)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfCycled,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 1)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: true,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: true,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
