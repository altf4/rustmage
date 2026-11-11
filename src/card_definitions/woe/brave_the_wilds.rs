// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfBargained { skip: 1 },
            Instruction::AnimateTargets(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
                base: Some((3, 3)),
                keywords: Keywords::HASTE,
                duration: AnimationDuration::Permanent,
            }),
            Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        bargain_targets: Some(one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
