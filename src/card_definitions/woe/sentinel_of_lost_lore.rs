// Direct typed behavior from the pinned Oracle and XMage sources.
const OWN_ADVENTURE: TargetSpec = TargetSpec::one(TargetRule::Filtered {
    base: &TargetRule::FaceUpExiledCard {
        owner: CastRelation::You,
    },
    predicate: &TargetPredicate::HasAdventure,
});
const FOREIGN_ADVENTURE: TargetSpec = TargetSpec::one(TargetRule::Filtered {
    base: &TargetRule::FaceUpExiledCard {
        owner: CastRelation::Opponent,
    },
    predicate: &TargetPredicate::HasAdventure,
});
const GRAVEYARD_OWNER: TargetSpec = TargetSpec::one(TargetRule::Player {
    opponent_only: false,
});
const EXILE_GRAVEYARD: Instruction = Instruction::ExileGraveyard(PlayerSet::Targeted);

// Each entry is a nonempty subset of the three printed modes in printed order.
const MODES: &[SpellMode] = &[
    SpellMode {
        text: "Return an exiled Adventure card you own to your hand",
        targets: TargetPlan::Single(OWN_ADVENTURE),
        program: &[Instruction::ReturnToHand],
    },
    SpellMode {
        text: "Put an exiled Adventure card you don't own on the bottom of its owner's library",
        targets: TargetPlan::Single(FOREIGN_ADVENTURE),
        program: &[Instruction::BottomLibrary],
    },
    SpellMode {
        text: "Exile a player's graveyard",
        targets: TargetPlan::Single(GRAVEYARD_OWNER),
        program: &[EXILE_GRAVEYARD],
    },
    SpellMode {
        text: "Return your exiled Adventure card and put the other exiled Adventure card on the bottom",
        targets: TargetPlan::Groups(&[OWN_ADVENTURE, FOREIGN_ADVENTURE]),
        program: &[
            Instruction::ReturnToHand,
            Instruction::SelectGroup(1),
            Instruction::BottomLibrary,
        ],
    },
    SpellMode {
        text: "Return your exiled Adventure card and exile a player's graveyard",
        targets: TargetPlan::Groups(&[OWN_ADVENTURE, GRAVEYARD_OWNER]),
        program: &[
            Instruction::ReturnToHand,
            Instruction::SelectGroup(1),
            EXILE_GRAVEYARD,
        ],
    },
    SpellMode {
        text: "Put the other exiled Adventure card on the bottom and exile a player's graveyard",
        targets: TargetPlan::Groups(&[FOREIGN_ADVENTURE, GRAVEYARD_OWNER]),
        program: &[
            Instruction::BottomLibrary,
            Instruction::SelectGroup(1),
            EXILE_GRAVEYARD,
        ],
    },
    SpellMode {
        text: "Return your exiled Adventure card, put the other one on the bottom, and exile a player's graveyard",
        targets: TargetPlan::Groups(&[OWN_ADVENTURE, FOREIGN_ADVENTURE, GRAVEYARD_OWNER]),
        program: &[
            Instruction::ReturnToHand,
            Instruction::SelectGroup(1),
            Instruction::BottomLibrary,
            Instruction::SelectGroup(2),
            EXILE_GRAVEYARD,
        ],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(MODES)],
    }],
    ..CardRules::EMPTY
};
