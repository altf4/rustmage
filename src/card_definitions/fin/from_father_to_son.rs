const VEHICLE: SearchSpec = SearchSpec {
    filter: SearchFilter::Subtype("Vehicle"),
    maximum: 1,
    destination: SearchDestination::Hand,
    tapped: false,
    reveal: true,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfCastFromGraveyard { skip: 2 },
            Instruction::Search(SearchSpec {
                destination: SearchDestination::Battlefield,
                ..VEHICLE
            }),
            Instruction::Skip(1),
            Instruction::Search(VEHICLE),
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [3, 0, 0, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
