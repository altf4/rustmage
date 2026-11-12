// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::CANT_BE_BLOCKED,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "Put two +1/+1 counters on this creature.",
                targets: TargetPlan::NONE,
                program: &[Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 2,
                }],
            },
            SpellMode {
                text: "Draw a card.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            },
            SpellMode {
                text: "This creature becomes a copy of another target creature you control.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: true,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                })),
                program: &[Instruction::CopyTargetForSourceIncarnation],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
