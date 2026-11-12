// Direct typed behavior from the paired Oracle and pinned XMage sources.
const LAND: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 2,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Deal damage to target creature",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::DamageScaled(AmountValue::Stat(
                StatValue::CONTROLLER_LANDS,
            ))],
        },
        SpellMode {
            text: "Destroy target land creature or nonbasic land",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
                TargetRule::Filtered {
                    base: &LAND,
                    predicate: &TargetPredicate::Kinds(1),
                },
                TargetRule::Filtered {
                    base: &LAND,
                    predicate: &TargetPredicate::Nonbasic,
                },
            ]))),
            program: &[Instruction::Destroy],
        },
    ]),
    ..CardRules::EMPTY
};
