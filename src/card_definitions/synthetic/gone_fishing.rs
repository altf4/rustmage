// The instant Adventure on Lake-town Mariners, using the established batch blink.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 3,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 2,
            max: 2,
        }),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(2439)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
