// Direct typed behavior from complete Oracle and pinned XMage sources.
const CONTROLLED: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const COST: ActivationCost = ActivationCost {
    generic: 2,
    mana_component: true,
    tap: true,
    ..ActivationCost::ZERO
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: COST,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &CONTROLLED,
                predicate: &TargetPredicate::Subtype("Mount"),
            })),
            program: &[Instruction::MarkSaddled(Subject::Targets)],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: COST,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &CONTROLLED,
                predicate: &TargetPredicate::Subtype("Vehicle"),
            })),
            program: &[Instruction::AnimateTargets(AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: None,
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            })],
            sorcery: false,
            once_per_turn: false,
        },
    ],
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
