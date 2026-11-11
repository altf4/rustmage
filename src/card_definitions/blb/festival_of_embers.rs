// Direct typed behavior from complete pinned Oracle and XMage sources.
const GRAVEYARD: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::CastFromGraveyardWithLifeDuringOwnTurn {
            filter: CastFilter::InstantSorcery,
            life: 1,
        },
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GRAVEYARD,
    extras: &CardExtras {
        static_ability_groups: GRAVEYARD,
        zone_replacement: Some(ZoneReplacementRule::ExileControllerGraveyard),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SacrificeSource],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
