// Complete printed obligations from full Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put a +1/+1 counter on target Wolf you control.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Wolf"),
                })),
                program: &[Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
            SpellMode {
                text: "Create a Treasure token.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
