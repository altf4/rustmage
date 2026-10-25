// Direct typed behavior from complete pinned Oracle and XMage sources.
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::SourceEquipmentCount,
    stats: Stats {
        power: 2,
        toughness: 0,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::VIGILANCE.union(Keywords::MENACE),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            colored: [1, 0, 1, 0, 0, 0],
            generic: 3,
            alternatives: &[],
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
        program: &[Instruction::ReturnSourceWithCounterAndEquipment(
            CounterKind::Finality,
        )],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
