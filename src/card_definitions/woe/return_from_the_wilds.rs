// Direct typed behavior from the paired Oracle and pinned XMage sources.
const SEARCH: Instruction = Instruction::Search(SearchSpec {
    filter: SearchFilter::BasicLand,
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
});

const HUMAN: Instruction = Instruction::CreateTokens {
    token: Card::from_catalogue_id(5254),
    amount: 1,
    tapped: false,
};

const FOOD: Instruction = Instruction::CreateTokens {
    token: Card::from_catalogue_id(5246),
    amount: 1,
    tapped: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Put a basic land from your library onto the battlefield tapped, then shuffle. Create a 1/1 white Human creature token.",
            targets: TargetPlan::NONE,
            program: &[SEARCH, HUMAN],
        },
        SpellMode {
            text: "Put a basic land from your library onto the battlefield tapped, then shuffle. Create a Food token.",
            targets: TargetPlan::NONE,
            program: &[SEARCH, FOOD],
        },
        SpellMode {
            text: "Create a 1/1 white Human creature token, then create a Food token.",
            targets: TargetPlan::NONE,
            program: &[HUMAN, FOOD],
        },
    ]),
    mode_counts: Some(&[2; 3]),
    ..CardRules::EMPTY
};
