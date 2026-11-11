const fn controlled_subtype(subtype: &'static str) -> CountFilter {
    CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some(subtype),
        basic: false,
        other: false,
        cards_only: false,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ControlledForeignOwnedCreatureSubtype("Mercenary")],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Targeted,
                amount: AmountValue::Stat(StatValue::CountUnion(&[
                    controlled_subtype("Assassin"),
                    controlled_subtype("Mercenary"),
                    controlled_subtype("Pirate"),
                    controlled_subtype("Rogue"),
                    controlled_subtype("Warlock"),
                ])),
            },
            Instruction::GrantExiledResultFlexibleCast,
        ],
    }],
    ..CardRules::EMPTY
};
