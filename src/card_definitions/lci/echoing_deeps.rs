pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 1,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::GraveyardLand { tapped: true },
            exceptions: CopyExceptions::CAVE,
            ..EntryCopyOptions::DEFAULT
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
