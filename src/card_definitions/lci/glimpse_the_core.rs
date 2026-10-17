// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Search your library for a basic Forest card, put that card onto the battlefield tapped, then shuffle.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::AllOf(&[
                    SearchFilter::BasicLand,
                    SearchFilter::Subtype("Forest"),
                ]),
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            })],
        },
        SpellMode {
            text: "Return target Cave card from your graveyard to the battlefield tapped.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Cave"),
            })),
            program: &[Instruction::ReturnToBattlefieldTapped],
        },
    ]),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
