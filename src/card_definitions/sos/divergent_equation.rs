// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 4 | 8,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: usize::MAX,
        }),
        &[Instruction::ReturnToHand, Instruction::ExileResolvingSpell],
    ),
    extras: &CardExtras {
        x_symbols: 2,
        targets_up_to_x: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
