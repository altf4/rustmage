//! The LCI creature lands retain mana abilities and get their distinct attack effects.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn next_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..35 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("next turn not reached");
}

fn mana(card: CardId, ability: u16, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn restless_lands_enter_tapped_and_retain_their_two_mana_choices_during_animation() {
    let cases = [
        (
            "Restless Anchorage",
            [0, 1],
            [1, 1, 0, 0, 0, 1],
            (2, 3),
            "Bird",
            Keywords::FLYING,
        ),
        (
            "Restless Prairie",
            [0, 4],
            [1, 0, 0, 0, 1, 2],
            (3, 3),
            "Llama",
            Keywords::NONE,
        ),
        (
            "Restless Reef",
            [1, 2],
            [0, 1, 1, 0, 0, 2],
            (4, 4),
            "Shark",
            Keywords::DEATHTOUCH,
        ),
        (
            "Restless Ridgeline",
            [3, 4],
            [0, 0, 0, 1, 1, 2],
            (3, 4),
            "Dinosaur",
            Keywords::NONE,
        ),
        (
            "Restless Vents",
            [2, 3],
            [0, 0, 1, 1, 0, 1],
            (2, 3),
            "Insect",
            Keywords::MENACE,
        ),
    ];
    for player in [Player::P0, Player::P1] {
        for (name, colors, payment, body, subtype, keywords) in cases {
            let mut game = scenario(player);
            let land = spawn(
                &mut game,
                player,
                Card::from_name(name).unwrap(),
                Zone::Hand,
            );
            let reef = name == "Restless Reef";
            act(&mut game, Action::PlayLand { card: land });
            let entered = game.object_view(land);
            assert!(entered.tapped && !entered.is_creature(), "{name}");
            assert_eq!(entered.colors, 0, "{name}");
            reject(&mut game, mana(land, 0, colors[0]));
            next_turn(&mut game);
            next_turn(&mut game);
            for _ in 0..8 {
                if game.state.step == Step::Main1 {
                    break;
                }
                act(&mut game, Action::Pass);
            }
            assert_eq!(game.state.step, Step::Main1);
            assert!(!game.object_view(land).tapped);
            game.state.players[player.index()].mana = payment;
            act(
                &mut game,
                Action::Activate {
                    card: land,
                    ability: if reef { 2 } else { 1 },
                    target: None,
                    payment,
                    counter_payment: Counters::default(),
                },
            );
            settle(&mut game);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
            let animated = game.object_view(land);
            assert_eq!(animated.reference, entered.reference, "{name}");
            assert_eq!(animated.kinds & 3, 3, "{name}");
            assert_eq!(animated.colors, (1 << colors[0]) | (1 << colors[1]));
            assert_eq!(stats(&game, land), body);
            assert_eq!(animated.keywords, keywords);
            assert!(animated.has_subtype(subtype));

            for (index, color) in colors.into_iter().enumerate() {
                let mut branch = game.clone();
                let ability = if reef { index as u16 } else { 0 };
                reject(&mut branch, mana(land, ability, 5));
                act(&mut branch, mana(land, ability, color));
                assert!(branch.object_view(land).tapped);
                assert_eq!(branch.state.players[player.index()].mana[color], 1);
            }
            next_turn(&mut game);
            let reverted = game.object_view(land);
            assert_eq!(reverted.kinds & 3, 2, "{name}");
            assert_eq!(reverted.colors, 0);
            assert!(!reverted.has_subtype(subtype));
        }
    }
}

fn attack_with(
    game: &mut Game,
    player: Player,
    name: &str,
    other_attacker: Option<CardId>,
) -> CardId {
    let land = spawn(
        game,
        player,
        Card::from_name(name).unwrap(),
        Zone::Battlefield,
    );
    act(
        game,
        Action::AnnounceActivate {
            card: land,
            ability: if name == "Restless Reef" { 2 } else { 1 },
            counter_payment: Counters::default(),
        },
    );
    pay(game);
    settle(game);
    pass_pair(game);
    pass_pair(game);
    act(game, Action::Attack { card: land });
    if let Some(card) = other_attacker {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    land
}

#[test]
fn each_restless_attack_has_its_own_target_and_outcome() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let mut anchorage = game.clone();
        attack_with(&mut anchorage, player, "Restless Anchorage", None);
        settle(&mut anchorage);
        assert!(anchorage.observe(player).objects.iter().any(|o| {
            o.zone == Zone::Battlefield
                && o.controller == player
                && o.card == Card::from_catalogue_id(5284)
        }));

        let mut prairie = game.clone();
        let land = attack_with(&mut prairie, player, "Restless Prairie", None);
        settle(&mut prairie);
        assert_eq!(stats(&prairie, ally), (3, 3));
        assert_eq!(stats(&prairie, foe), (2, 2));
        assert_eq!(stats(&prairie, land), (3, 3));

        let mut reef = game.clone();
        let before = reef.state.players[player.index()].library.len();
        attack_with(&mut reef, player, "Restless Reef", None);
        act(
            &mut reef,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
        settle(&mut reef);
        assert_eq!(reef.state.players[player.index()].library.len(), before - 4);

        let mut ridgeline = game.clone();
        let land = attack_with(&mut ridgeline, player, "Restless Ridgeline", Some(ally));
        reject(
            &mut ridgeline,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        reject(
            &mut ridgeline,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        act(
            &mut ridgeline,
            Action::ChooseTarget {
                target: Target::Card(ally),
            },
        );
        settle(&mut ridgeline);
        assert_eq!(stats(&ridgeline, ally), (4, 2));
        assert!(!ridgeline.object_view(ally).tapped);
        assert!(ridgeline.object_view(ally).attacking);

        let discard = put(&mut game, player, "Mountain", Zone::Hand);
        let before = game.state.players[player.index()].library.len();
        attack_with(&mut game, player, "Restless Vents", None);
        pass_pair(&mut game);
        let choice = Action::Discard { card: discard };
        assert!(game.legal_actions().contains(&choice));
        let mut decline = game.clone();
        act(&mut decline, Action::DeclineEffectPayment);
        assert_eq!(decline.object_view(discard).zone, Zone::Hand);
        assert_eq!(decline.state.players[player.index()].library.len(), before);
        act(&mut game, choice);
        assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].library.len(), before - 1);
    }
}
