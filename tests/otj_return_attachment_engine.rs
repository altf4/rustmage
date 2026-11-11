//! Exact attachment returns after earlier spell modes, with no priority between modes.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const VEHICLE: TargetRule = TargetRule::Filtered {
    base: &GRAVE,
    predicate: &TargetPredicate::Subtype("Vehicle"),
};
const ATTACHMENT: TargetRule = TargetRule::Union(&[
    TargetRule::Filtered {
        base: &GRAVE,
        predicate: &TargetPredicate::Subtype("Aura"),
    },
    TargetRule::Filtered {
        base: &GRAVE,
        predicate: &TargetPredicate::Subtype("Equipment"),
    },
]);
const RETURN_THEN_ATTACH: &[Instruction] = &[
    Instruction::ReturnToBattlefield,
    Instruction::SelectGroup(1),
    Instruction::ReturnTargetAttachmentToControlledCreature,
];
const FEE: ManaCost = ManaCost {
    colored: [0; 6],
    generic: 1,
};
const SPELL: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Return an attachment",
            targets: TargetPlan::Single(TargetSpec::one(ATTACHMENT)),
            program: &[Instruction::ReturnTargetAttachmentToControlledCreature],
        },
        SpellMode {
            text: "Return a creature and an attachment",
            targets: TargetPlan::Groups(&[TargetSpec::one(CREATURE), TargetSpec::one(ATTACHMENT)]),
            program: RETURN_THEN_ATTACH,
        },
        SpellMode {
            text: "Return a Vehicle and an attachment",
            targets: TargetPlan::Groups(&[TargetSpec::one(VEHICLE), TargetSpec::one(ATTACHMENT)]),
            program: RETURN_THEN_ATTACH,
        },
    ]),
    mode_costs: Some(&[FEE, FEE, FEE]),
    ..CardRules::EMPTY
};

fn pay_mode(g: &mut Game, card: CardId, mode: u16, targets: &[CardId]) {
    act(g, Action::AnnounceCast { card });
    act(g, Action::ChooseMode { mode });
    for &card in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    pay(g);
}

#[test]
fn the_recipient_is_a_mandatory_nontargeted_choice_after_an_earlier_creature_return() {
    let definition = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &SPELL)]);
    let player = Player::P1;
    for attachment_name in ["Petrify", "Swiftfoot Boots"] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        let returned = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let attachment = put(&mut game, player, attachment_name, Zone::Graveyard);
        let protected = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[protected as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
        let foreign = put(&mut game, player.other(), "Goblin Piker", Zone::Battlefield);
        let old_recipient = game.object_ref(returned);
        let grave_attachment = game.object_ref(attachment);

        pay_mode(&mut game, spell, 1, &[returned, attachment]);
        let targeted: Vec<_> = game
            .state
            .stack
            .last()
            .unwrap()
            .targets
            .iter()
            .map(|b| b.target)
            .collect();
        assert_eq!(
            targeted,
            vec![Target::Card(returned), Target::Card(attachment)]
        );
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.state.pending, Pending::EffectChoice { player });
        assert_eq!(
            game.state.objects[returned as usize].zone,
            Zone::Battlefield
        );
        assert_eq!(game.object_ref(attachment), grave_attachment);
        assert!(!game.legal_actions().contains(&Action::Pass));
        reject(&mut game, Action::FinishEffectChoice);
        for object in [
            old_recipient,
            game.object_ref(protected),
            game.object_ref(foreign),
        ] {
            reject(&mut game, Action::ChooseEffectObject { object });
        }

        let recipient = game.object_ref(returned);
        assert_ne!(recipient, old_recipient);
        let select = Action::ChooseEffectObject { object: recipient };
        let checkpoint = game.snapshot();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        act(&mut raw, select.clone());
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == select)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        game.restore(&checkpoint).unwrap();
        act(&mut game, select);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), game.snapshot());
        assert_eq!(
            game.state.objects[attachment as usize].zone,
            Zone::Battlefield
        );
        assert_eq!(
            game.state.objects[attachment as usize].attached_to,
            Some(recipient)
        );
        assert_ne!(game.object_ref(attachment), grave_attachment);
    }
}

#[test]
fn a_returned_noncreature_vehicle_cannot_receive_the_attachment_during_resolution() {
    let definition = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &SPELL)]);
    let player = Player::P0;
    for (attachment_name, expected) in [
        ("Petrify", Zone::Graveyard),
        ("Swiftfoot Boots", Zone::Battlefield),
    ] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        let vehicle = put(&mut game, player, "Clamorous Ironclad", Zone::Graveyard);
        let attachment = put(&mut game, player, attachment_name, Zone::Graveyard);
        put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);

        pay_mode(&mut game, spell, 2, &[vehicle, attachment]);
        settle(&mut game);
        assert_eq!(game.state.objects[vehicle as usize].zone, Zone::Battlefield);
        assert!(!game.is_creature(vehicle));
        assert_eq!(game.state.objects[attachment as usize].zone, expected);
        assert_eq!(game.state.objects[attachment as usize].attached_to, None);
    }
}

#[test]
fn a_moved_graveyard_target_cannot_rebind_to_a_new_incarnation() {
    let definition = Card::from_name("Divination").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &SPELL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let aura = put(&mut game, player, "Petrify", Zone::Graveyard);
    let foreign_aura = put(&mut game, player.other(), "Petrify", Zone::Graveyard);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[recipient as usize].gained_keywords = Keywords::HEXPROOF;

    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 0 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign_aura),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    pay(&mut game);
    let original = game.object_ref(aura);
    game.move_object(aura, Zone::Exile);
    game.move_object(aura, Zone::Graveyard);
    assert_ne!(game.object_ref(aura), original);
    settle(&mut game);
    assert_eq!(game.state.objects[aura as usize].zone, Zone::Graveyard);
}
