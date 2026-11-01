// Typed behavior from the complete pinned Oracle and XMage sources.
const OWN_CREATURE_OR_ENCHANTMENT: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 33,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::OwnedByController,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target creature or enchantment spell.",
            targets: one(TargetRule::SpellKinds {
                kinds: 33,
                excluded: 0,
            }),
            program: &[Instruction::Counter],
        },
        SpellMode {
            text: "Return one or two target creatures and/or enchantments you own to your hand.",
            targets: TargetPlan::Single(TargetSpec {
                rule: OWN_CREATURE_OR_ENCHANTMENT,
                min: 1,
                max: 2,
            }),
            program: &[Instruction::ReturnToHand],
        },
    ]),
    ..CardRules::EMPTY
};
