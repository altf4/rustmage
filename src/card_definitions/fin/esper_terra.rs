// Direct typed behavior from the paired Oracle and pinned XMage sources.
const COPY_TARGET: TargetPlan = one(TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 32,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Nonlegendary,
});
const COPY_ENCHANTMENT: &[Instruction] = &[Instruction::CreateHastyCopyWithSagaLore { maximum: 3 }];

const fn copy_chapter(number: u16) -> TriggeredAbility {
    TriggeredAbility {
        condition: TriggerCondition::Chapter { number },
        targets: COPY_TARGET,
        program: COPY_ENCHANTMENT,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    triggers: &[
        copy_chapter(1),
        copy_chapter(2),
        copy_chapter(3),
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::AddMana(ManaProduction::Fixed([2, 2, 2, 2, 2, 0])),
                Instruction::ExileBattlefieldSourceReturn {
                    transformed: false,
                    under_owner: false,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
