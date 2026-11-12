const fn in_own_graveyard(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}

const fn optional(rule: TargetRule) -> TargetSpec {
    TargetSpec {
        rule,
        min: 0,
        max: 1,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            optional(in_own_graveyard(16)),
            optional(in_own_graveyard(2)),
            optional(TargetRule::Filtered {
                base: &in_own_graveyard(32),
                predicate: &TargetPredicate::WithoutSubtype("Aura"),
            }),
        ]),
        &[Instruction::ReturnGraveyardTargetsAnimated {
            animation: &AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
                base: Some((5, 5)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            },
            x_counters: None,
        }],
    ),
    ..CardRules::EMPTY
};
