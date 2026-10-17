// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWhileSaddled,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords(8192),
            },
            Instruction::UntapCreatures(CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords(0),
                attacking: false,
                tokens: None,
                artifact: false,
            }),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::BecomeSaddled { minimum: 4 }],
        sorcery: true,
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
