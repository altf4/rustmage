// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::CreatureDied,
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::CreateTokens {
            token: Card(5305),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[],
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
        plot_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
