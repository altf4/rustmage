// Teamwork expands the target restriction when the cost is declared.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 3,
        }),
        &[
            Instruction::Exile,
            Instruction::IfTeamwork { skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 2,
            paid_targets: Some(one(TargetRule::Creature)),
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
