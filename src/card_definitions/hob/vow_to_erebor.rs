// Complete printed obligations from the pinned Oracle and full XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::Untap,
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            },
            Instruction::MayAttachControlledEquipment {
                recipient: 0,
                required_subtype: Some("Dwarf"),
            },
        ],
    ),
    ..CardRules::EMPTY
};
