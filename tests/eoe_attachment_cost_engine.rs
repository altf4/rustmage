//! Cost payment binds the exact attachment that granted the ability.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::NONE,
};
const GRANTS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentStats(BONUS.stats),
        StaticAbilityFragment::AttachmentActivation {
            ability: 0,
            tap_attachment: false,
        },
    ],
}];
const CONTROL: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        generic: 15,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: one(TargetRule::Player {
        opponent_only: true,
    }),
    program: &[Instruction::ControlTargetPlayerNextTurn],
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    exhaust: false,
    cycling: false,
    sorcery: true,
    once_per_turn: false,
};
const BRACELET: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GRANTS,
    extras: &CardExtras {
        equipment: Some(BONUS),
        static_ability_groups: GRANTS,
        attachment_activation_exile_costs: &[0],
        activation_generic_reductions: &[(0, ActivationGenericReduction::SourcePower)],
        ..CardExtras::EMPTY
    },
    activated: &[CONTROL],
    ..CardRules::EMPTY
};
const MANA: Mana = [0, 0, 0, 0, 0, 12];
const SACRIFICE_MANA: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        sacrifice: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(MANA))],
    sorcery: false,
    ..CONTROL
};
const BRACELET_FUNDER: CardRules = CardRules {
    activated: &[CONTROL, SACRIFICE_MANA],
    ..BRACELET
};
const CREATURE_FUNDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[SACRIFICE_MANA],
    ..CardRules::EMPTY
};

fn equipment() -> Card {
    Card::from_name("Short Bow").unwrap()
}

fn setup(player: Player, equipment_owner: Player) -> (Game, CardId, ObjectRef) {
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, equipment_owner, "Short Bow", Zone::Battlefield);
    let grantor = game.object_ref(equipment);
    let recipient = game.object_ref(creature);
    assert!(game.attach_equipment(grantor, recipient));
    assert_eq!(stats(&game, creature), (3, 3));
    game.state.players[player.index()].mana = [0; 6];
    (game, creature, grantor)
}

fn activation(grantor: ObjectRef) -> Action {
    Action::AnnounceAttachedAbility {
        grantor,
        ability: 0,
    }
}

fn target(player: Player) -> Action {
    Action::ChooseTarget {
        target: Target::Player(player),
    }
}

fn apply_index(game: &mut Game, action: &Action) {
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| candidate == action)
        .unwrap();
    game.apply_index(game.actor().unwrap(), game.revision(), index)
        .unwrap();
}

fn mana_action(card: CardId, ability: u16) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: MANA,
    }
}

#[test]
fn exact_foreign_grantor_is_exiled_and_turn_control_survives_on_owned_branches() {
    let _rules = fixture_rules::activate(&[(equipment(), &BRACELET)]);
    let player = Player::P0;
    let opponent = player.other();
    let (mut game, creature, foreign) = setup(player, opponent);
    let twin = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let twin_ref = game.object_ref(twin);
    let recipient = game.object_ref(creature);
    assert!(game.attach_equipment(twin_ref, recipient));
    assert_eq!(stats(&game, creature), (4, 4));
    let secret = put(&mut game, opponent, "Island", Zone::Hand);
    game.state.players[player.index()].mana[5] = 11;
    game.state.step = Step::Upkeep;
    reject(&mut game, activation(foreign));
    game.state.step = Step::Main1;
    game.state.priority = opponent;
    reject(&mut game, activation(foreign));
    game.state.priority = player;
    let before = game.snapshot();

    let mut raw = game.clone();
    let mut indexed = game.clone();
    act(&mut raw, activation(foreign));
    apply_index(&mut indexed, &activation(foreign));
    reject(&mut raw, target(player));
    act(&mut raw, target(opponent));
    apply_index(&mut indexed, &target(opponent));
    assert_eq!(raw.observe(player).payment_request.unwrap().generic, 11);
    assert!(raw.observe(opponent).payment_request.is_none());
    let payment = Action::PayCosts {
        payment: [0, 0, 0, 0, 0, 11],
    };
    act(&mut raw, payment.clone());
    apply_index(&mut indexed, &payment);
    assert_eq!(indexed.snapshot(), raw.snapshot());
    assert_eq!(game.snapshot(), before);
    assert_eq!(raw.object_view(foreign.card).zone, Zone::Exile);
    assert_eq!(raw.object_view(twin).attached_to, Some(recipient));
    assert_eq!(stats(&raw, creature), (3, 3));
    let packet = raw.state.stack.last().unwrap();
    assert_eq!(packet.source().unwrap().object, recipient);
    assert_eq!(packet.attachment_trigger.as_ref().unwrap().grantor, foreign);

    indexed.restore(&before).unwrap();
    for action in [activation(twin_ref), target(opponent), payment] {
        apply_index(&mut indexed, &action);
    }
    assert_eq!(indexed.object_view(twin).zone, Zone::Exile);
    assert_eq!(
        indexed.object_view(foreign.card).attached_to,
        Some(recipient)
    );

    settle(&mut raw);
    let observation = raw.observe(player);
    assert_eq!(
        observation.player_control.unwrap().pending[opponent.index()],
        Some(player)
    );
    assert!(!observation.objects.iter().any(|object| object.id == secret));
    for _ in 0..32 {
        if raw.state.active == opponent {
            break;
        }
        choose(&mut raw, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    assert_eq!(raw.state.active, opponent);
    let observation = raw.observe(player);
    assert_eq!(
        observation.player_control.unwrap().current,
        Some(ControlledPlayer {
            player: opponent,
            controller: player,
        })
    );
    assert!(observation.objects.iter().any(|object| object.id == secret));
}

#[test]
fn funding_reserves_the_grantor_but_can_sacrifice_a_same_named_permanent() {
    let _rules = fixture_rules::activate(&[(equipment(), &BRACELET_FUNDER)]);
    let player = Player::P1;
    let (mut game, _, grantor) = setup(player, player);
    reject(&mut game, activation(grantor));
    let twin = put(&mut game, player, "Short Bow", Zone::Battlefield);
    act(&mut game, activation(grantor));
    act(&mut game, target(player.other()));
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 12);
    reject(&mut game, mana_action(grantor.card, 1));
    act(&mut game, mana_action(twin, 1));
    pay(&mut game);
    assert_eq!(game.object_view(twin).zone, Zone::Graveyard);
    assert_eq!(game.object_view(grantor.card).zone, Zone::Exile);
}

#[test]
fn the_frozen_grantor_is_payable_after_the_equipped_creature_funds_its_departure() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(equipment(), &BRACELET), (bear, &CREATURE_FUNDER)]);
    let player = Player::P0;
    let (mut game, creature, grantor) = setup(player, player);
    let recipient = game.object_ref(creature);
    act(&mut game, activation(grantor));
    act(&mut game, target(player.other()));
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 12);
    act(&mut game, mana_action(creature, 0));
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 12);
    pay(&mut game);
    assert_eq!(game.object_view(grantor.card).zone, Zone::Exile);
    assert_eq!(
        game.state.stack.last().unwrap().source().unwrap().object,
        recipient
    );
    settle(&mut game);
    assert_eq!(
        game.observe(player).player_control.unwrap().pending[player.other().index()],
        Some(player)
    );
}
