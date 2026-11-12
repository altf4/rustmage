mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SORCERY: &str = "Day of Judgment";
const INSTANT: &str = "Opt";
const FORETELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ),
    extras: &CardExtras {
        foretell_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(SORCERY).unwrap(), &FORETELL),
        (Card::from_name(INSTANT).unwrap(), &FORETELL),
    ])
}

fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("foretell timing checkpoint");
}

fn visible(game: &Game, viewer: Player, card: CardId) -> bool {
    game.observe(viewer)
        .objects
        .iter()
        .any(|object| object.id == card)
}

fn foretell(game: &mut Game, card: CardId) -> ObjectRef {
    let object = game.object_ref(card);
    act(game, Action::AnnounceForetell { object });
    choose(game, |action| matches!(action, Action::PayForetell { .. }));
    game.object_ref(card)
}

fn foretold_cast(object: ObjectRef) -> Action {
    Action::AnnounceForetold {
        object,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}

#[test]
fn foretell_is_a_private_priority_special_action_with_owned_payment() {
    let _rules = fixture();
    let player = Player::P1;
    let mut game = scenario(player);
    let card = put(&mut game, player, SORCERY, Zone::Hand);
    let spell = put(&mut game, player, INSTANT, Zone::Hand);
    let object = game.object_ref(card);
    game.remember_public_hand(object);
    let known_land = put(&mut game, player, "Forest", Zone::Hand);
    game.remember_public_hand(game.object_ref(known_land));
    cast(&mut game, spell, None);
    assert_eq!(game.state.stack.len(), 1);

    game.state.players[player.index()].mana = [0, 0, 0, 2, 0, 0];
    let action = Action::AnnounceForetell { object };
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert!(game.observe(player).plot_payment.unwrap().foretell);
    assert!(game.observe(player.other()).plot_payment.is_none());

    reject(
        &mut game,
        Action::PayPlot {
            payment: [0, 0, 0, 2, 0, 0],
        },
    );
    reject(&mut game, Action::PayForetell { payment: [0; 6] });
    act(
        &mut game,
        Action::PayForetell {
            payment: [0, 0, 0, 2, 0, 0],
        },
    );
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.actor(), Some(player));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert_ne!(game.object_ref(card), object);
    assert!(visible(&game, player, card));
    assert!(!visible(&game, player.other(), card));
    assert!(visible(&game, player.other(), known_land));

    let private = game.observe(player).foretold_cards[0];
    let public = game.observe(player.other()).foretold_cards[0];
    assert_eq!(private.object, Some(game.object_ref(card)));
    assert_eq!(public.object, None);
    assert_eq!(public.serial, private.serial);
    assert_eq!(public.owner, player);
    assert_eq!(public.foretold_turn, game.state.turn);
    let exiled = game.object_ref(card);
    reject(&mut game, foretold_cast(exiled));
}

#[test]
fn foretell_cast_uses_later_timing_and_the_exact_alternative() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, SORCERY, Zone::Hand);
    let instant = put(&mut game, player, INSTANT, Zone::Hand);
    let other_card = put(&mut game, player, INSTANT, Zone::Hand);
    let exiled = foretell(&mut game, card);
    let exiled_instant = foretell(&mut game, instant);
    let turn = game.state.turn;
    reject(&mut game, foretold_cast(exiled));
    reject(&mut game, foretold_cast(exiled_instant));
    advance(&mut game, |g| {
        g.state.turn > turn
            && g.actor() == Some(player.other())
            && g.state.pending == Pending::Priority
    });
    reject(&mut game, foretold_cast(exiled_instant));
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    game.state.players[player.index()].mana = [0, 2, 0, 1, 0, 0];
    reject(&mut game, foretold_cast(exiled));
    let hand = game.object_ref(other_card);
    reject(&mut game, Action::AnnounceForetell { object: hand });
    let mut instant_branch = game.clone();
    act(&mut instant_branch, foretold_cast(exiled_instant));
    pay(&mut instant_branch);
    assert_eq!(
        instant_branch
            .state
            .stack
            .last()
            .unwrap()
            .cast
            .unwrap()
            .method,
        CastMethod::Foretell
    );

    advance(&mut game, |g| {
        g.state.active == player && g.state.step == Step::Main1 && g.actor() == Some(player)
    });
    reject(&mut game, Action::AnnounceExile { card });
    let permission = game
        .grant_exile_play_permission_with_expiry(player, exiled, None, ExilePlayExpiry::WhileExiled)
        .unwrap();
    permission.spend_any_mana = true;
    permission.exile_on_graveyard = true;
    game.state.players[player.index()].mana = [0, 4, 0, 0, 0, 0];
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceExile { card })
    );
    reject(&mut game, foretold_cast(exiled));

    game.state.players[player.index()].mana = [0, 2, 0, 1, 0, 0];
    let before = game.snapshot();
    let mut removed = game.clone();
    removed.move_object(card, Zone::Hand);
    removed.move_object(card, Zone::Exile);
    reject(&mut removed, foretold_cast(exiled));
    assert_eq!(removed.observe(player).foretold_cards.len(), 1);
    game.restore(&before).unwrap();
    act(&mut game, foretold_cast(exiled));
    pay(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.origin, Zone::Exile);
    assert_eq!(cast.method, CastMethod::Foretell);
    assert_eq!(cast.payment, [0, 2, 0, 1, 0, 0]);
    assert!(!cast.spend_any_mana && !cast.exile_on_graveyard);
    assert!(visible(&game, player.other(), card));
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);

    assert!(!visible(&game, player.other(), instant));
    game.apply(player, Action::Concede).unwrap();
    assert!(visible(&game, player.other(), instant));
    assert_eq!(
        game.observe(player.other()).foretold_cards[0].object,
        Some(exiled_instant)
    );
}

const MANA_SACRIFICE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        cost: ActivationCost {
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 1, 0, 0,
        ]))],
    }],
    ..CardRules::EMPTY
};
const OPTIONAL_COPY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 17,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn foretell_rewinds_an_unpayable_foreign_mana_choice_to_the_exact_hand_card() {
    use rustmage_core::engine::characteristic_layers::AnimationTypeMode;

    let _rules = fixture_rules::activate(&[
        (Card::from_name(SORCERY).unwrap(), &FORETELL),
        (Card::from_name("Bear Cub").unwrap(), &MANA_SACRIFICE),
        (Card::GrizzlyBears, &OPTIONAL_COPY),
    ]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let card = put(&mut game, player, SORCERY, Zone::Hand);
    let sacrifice = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let green = put(&mut game, player, "Druid of the Cowl", Zone::Battlefield);
    let enchanted = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    let aura = put(&mut game, other, "Witness Protection", Zone::Battlefield);
    let blueprint = game.object_ref(aura);
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(enchanted));
    game.change_object_form(
        blueprint,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::PreserveCardTypes,
    );
    let returning = put(&mut game, other, "Grizzly Bears", Zone::Exile);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: game.object_ref(sacrifice),
        exiled: game.object_ref(returning),
        return_zone: Zone::Battlefield,
    });
    game.state.players[player.index()].mana = [0; 6];
    let object = game.object_ref(card);
    let foretell = Action::AnnounceForetell { object };
    act(&mut game, foretell.clone());
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, ability: 0, .. } if *card == sacrifice),
    );
    assert_eq!(game.actor(), Some(other));
    act(
        &mut game,
        Action::ChooseEntryCopy {
            object: Some(blueprint),
        },
    );
    let green = game.object_ref(green);
    act(&mut game, Action::ChooseAuraAttachment { object: green });
    assert!(matches!(game.state.pending, Pending::ManaRollback { .. }));
    for keep in [false, true] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseManaRollback { keep });
        assert_eq!(branch.actor(), Some(player));
        assert_eq!(branch.object_ref(card), object);
        assert_eq!(branch.object_view(card).zone, Zone::Hand);
        assert!(branch.observe(other).foretold_cards.is_empty());
        assert_eq!(branch.legal_actions().contains(&foretell), !keep);
    }
}
