// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            alternatives: &[ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 1,
            }],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords(32768),
        }],
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
    keywords: Keywords(0),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
        }],
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::OpponentLostLife,
            amount: EntryCounterAmount::Fixed(1),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
