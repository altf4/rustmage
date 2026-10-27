// Complete Oracle behavior; all reveal/entry continuation uses the shared library frame.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::CreatureDies {
            relation: CastRelation::You,
            other: false,
            nontoken: true,
            subtype: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(1),
                optional: false,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::UntilKinds(1),
            destination: LibraryPickDestination::BattlefieldIfManaValueAtMostElseHand(
                AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 2,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            ),
            public: true,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
