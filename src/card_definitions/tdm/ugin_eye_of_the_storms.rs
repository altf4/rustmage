// Complete Oracle and pinned XMage implementation.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
const COLORED_PERMANENT: TargetSpec = TargetSpec {
    rule: TargetRule::Filtered {
        base: &TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        predicate: &TargetPredicate::Colors(1 | 2 | 4 | 8 | 16),
    },
    min: 0,
    max: 1,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 7,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfCast,
            targets: TargetPlan::Single(COLORED_PERMANENT),
            program: &[Instruction::Exile],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Colorless,
                ordinal: None,
            },
            targets: TargetPlan::Single(COLORED_PERMANENT),
            program: &[Instruction::Exile],
        },
    ],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(2),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GainLife(PlayerSet::Controller, 3),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(0),
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 3,
            ]))],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-11),
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::AllOf(&[SearchFilter::Colorless, SearchFilter::Nonland]),
                maximum: u64::MAX,
                destination: SearchDestination::ExileFreeThisTurn,
                tapped: false,
                reveal: false,
            })],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
