//! A granted activation reserves the exact controlled attachment it sacrifices.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DECONSTRUCT: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 3,
        mana_component: true,
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 16 | 32,
        relation: 2,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    program: &[Instruction::Destroy],
    sorcery: false,
    once_per_turn: false,
};
const HAMMER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentActivation {
            ability: 0,
            tap_attachment: false,
        }],
    }],
    extras: &CardExtras {
        attachment_activation_sacrifice_costs: &[0],
        ..CardExtras::EMPTY
    },
    activated: &[DECONSTRUCT],
    ..CardRules::EMPTY
};
const MANA: Mana = [0, 0, 0, 0, 0, 3];
const FUNDING_HAMMER: CardRules = CardRules {
    activated: &[
        DECONSTRUCT,
        ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed(MANA))],
            ..DECONSTRUCT
        },
    ],
    ..HAMMER
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn announce(grantor: ObjectRef) -> Action {
    Action::AnnounceAttachedAbility {
        grantor,
        ability: 0,
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn fund(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 1,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: MANA,
    }
}

#[test]
fn the_bearer_pays_only_its_exact_controlled_grantor() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &HAMMER)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let own = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let foreign = put(&mut game, other, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let victim = put(&mut game, other, "Swiftfoot Boots", Zone::Battlefield);
        put(&mut game, other, "Rest in Peace", Zone::Battlefield);
        let grantor = game.object_ref(own);
        let other_grantor = game.object_ref(foreign);
        let bearer = game.object_ref(creature);
        assert!(game.attach_equipment(grantor, bearer));
        assert!(game.attach_equipment(other_grantor, bearer));
        reject(&mut game, announce(other_grantor));

        let mut stolen = game.clone();
        stolen.record_control(other_grantor, player, None);
        assert!(stolen.legal_actions().contains(&announce(other_grantor)));

        game.state.objects[own as usize].gained_keywords = Keywords::CANT_BE_SACRIFICED;
        reject(&mut game, announce(grantor));
        game.state.objects[own as usize].gained_keywords = Keywords::NONE;
        game.state.objects[creature as usize].summoning_sick = true;
        reject(&mut game, announce(grantor));
        game.state.objects[creature as usize].summoning_sick = false;

        act(&mut game, announce(grantor));
        reject(&mut game, target(creature));
        act(&mut game, target(victim));
        assert_eq!(game.observe(player).payment_request.unwrap().generic, 3);

        let saved = game.snapshot();
        let revision = game.revision();
        let payment = Action::PayCosts { payment: MANA };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == payment)
            .unwrap();
        let mut raw = game.clone();
        let mut indexed = game;
        act(&mut raw, payment);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for branch in [&mut raw, &mut indexed] {
            assert!(branch.object_view(creature).tapped);
            assert_eq!(branch.object_view(own).zone, Zone::Exile);
            assert_eq!(branch.object_view(foreign).attached_to, Some(bearer));
            let packet = branch.state.stack.last().unwrap();
            assert_eq!(packet.source.unwrap().object, bearer);
            assert_eq!(packet.attachment_trigger.as_ref().unwrap().grantor, grantor);
            branch.move_object(creature, Zone::Graveyard);
            settle(branch);
            assert_eq!(branch.object_view(victim).zone, Zone::Exile);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        assert_eq!(indexed.object_view(own).zone, Zone::Battlefield);
        assert!(!indexed.object_view(creature).tapped);
    }
}

#[test]
fn mana_funding_cannot_spend_the_reserved_grantor() {
    let _rules = fixture_rules::activate(&[(named("Short Bow"), &FUNDING_HAMMER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let gear = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let victim = put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        let grantor = game.object_ref(gear);
        assert!(game.attach_equipment(grantor, game.object_ref(creature)));
        reject(&mut game, announce(grantor));

        let fuel = put(&mut game, player, "Short Bow", Zone::Battlefield);
        act(&mut game, announce(grantor));
        act(&mut game, target(victim));
        reject(&mut game, fund(gear));
        act(&mut game, fund(fuel));
        assert_eq!(game.object_view(fuel).zone, Zone::Graveyard);
        assert_eq!(game.object_view(gear).zone, Zone::Battlefield);
        pay(&mut game);
        assert_eq!(game.object_view(gear).zone, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    }
}
