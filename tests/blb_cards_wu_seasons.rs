mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn choose_mode(game: &mut Game, card: CardId, mode: u16) {
    act(game, Action::AnnounceCast { card });
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|a| matches!(a, Action::ChooseMode { .. }))
            .count(),
        16
    );
    act(game, Action::ChooseMode { mode });
}

fn progress(game: &mut Game) {
    choose(game, |a| {
        matches!(a, Action::Pass | Action::PutTrigger { .. })
    });
}

fn until(game: &mut Game, action: Action) {
    for _ in 0..24 {
        if game.legal_actions().contains(&action) {
            return;
        }
        progress(game);
    }
    panic!("missing {action:?} in {:?}", game.state.pending);
}

fn token_ids(game: &Game, definition: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token() && object.card == definition)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn burrow_can_choose_zero_five_rabbits_repeated_targets_and_mixed_roles() {
    let player = Player::P0;
    let other = player.other();
    let mut base = scenario(player);
    let spell = put(&mut base, player, "Season of the Burrow", Zone::Hand);
    let victim = put(&mut base, other, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut base, player, "Forest", Zone::Graveyard);
    let expensive = put(&mut base, player, "Hill Giant", Zone::Graveyard);
    for (mode, count) in [(0, 0), (5, 5)] {
        let mut branch = base.clone();
        choose_mode(&mut branch, spell, mode);
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(token_ids(&branch, Card(5285)).len(), count);
        assert_eq!(branch.object_view(victim).zone, Zone::Battlefield);
    }

    let mut two = base.clone();
    choose_mode(&mut two, spell, 10);
    act(&mut two, target(victim));
    act(&mut two, target(victim));
    pay(&mut two);
    settle(&mut two);
    assert_eq!(two.object_view(victim).zone, Zone::Exile);
    assert_eq!(two.state.players[other.index()].cards_drawn_this_turn, 2);

    let mut all_invalid = base.clone();
    choose_mode(&mut all_invalid, spell, 7);
    act(&mut all_invalid, target(victim));
    pay(&mut all_invalid);
    all_invalid.move_object(victim, Zone::Hand);
    settle(&mut all_invalid);
    assert!(token_ids(&all_invalid, Card(5285)).is_empty());

    choose_mode(&mut base, spell, 15);
    act(&mut base, target(victim));
    reject(&mut base, target(expensive));
    act(&mut base, target(land));
    pay(&mut base);
    base.move_object(victim, Zone::Hand);
    settle(&mut base);
    assert_eq!(base.state.players[other.index()].cards_drawn_this_turn, 0);
    assert_eq!(base.object_view(land).zone, Zone::Battlefield);
    assert_eq!(base.counter_count(land, CounterKind::Indestructible), 1);
}

#[test]
fn weaving_rechooses_each_copy_then_returns_live_nonland_nontoken_permanents() {
    let player = Player::P1;
    let mut base = scenario(player);
    let spell = put(&mut base, player, "Season of Weaving", Zone::Hand);
    let boots = put(&mut base, player, "Swiftfoot Boots", Zone::Battlefield);
    let enemy = put(
        &mut base,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let land = put(&mut base, player, "Forest", Zone::Battlefield);
    let blueprint = base.object_view(boots).card;
    let selection = |g: &Game, card| Action::ChooseEffectObject {
        object: g.object_ref(card),
    };

    let mut copies = base.clone();
    choose_mode(&mut copies, spell, 11);
    pay(&mut copies);
    let pick = selection(&copies, boots);
    until(&mut copies, pick.clone());
    assert_eq!(
        copies.state.players[player.index()].cards_drawn_this_turn,
        1
    );
    let wrong = selection(&copies, enemy);
    reject(&mut copies, wrong);
    act(&mut copies, pick);
    let new = token_ids(&copies, blueprint)[0];
    let again = selection(&copies, new);
    act(&mut copies, again);
    settle(&mut copies);
    assert_eq!(token_ids(&copies, blueprint).len(), 2);

    choose_mode(&mut base, spell, 15);
    pay(&mut base);
    let choice = selection(&base, boots);
    until(&mut base, choice.clone());
    act(&mut base, choice);
    settle(&mut base);
    assert_eq!(token_ids(&base, blueprint).len(), 1);
    assert_eq!(base.object_view(boots).zone, Zone::Hand);
    assert_eq!(base.object_view(enemy).zone, Zone::Hand);
    assert_eq!(base.object_view(land).zone, Zone::Battlefield);
}

#[test]
fn warren_warleader_and_offspring_offer_both_modes_on_one_attack() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let parent = put(&mut game, player, "Warren Warleader", Zone::Hand);
    let definition = game.object_view(parent).card;
    act(&mut game, Action::AnnounceCast { card: parent });
    act(&mut game, Action::ChooseOffspring { pay: true });
    pay(&mut game);
    settle(&mut game);
    let child = token_ids(&game, definition)[0];
    assert_eq!(stats(&game, child), (1, 1));
    let attack = Action::Attack { card: attacker };
    until(&mut game, attack.clone());
    act(&mut game, attack);
    act(&mut game, Action::FinishAttackers);
    for mode in [0, 1] {
        let action = Action::ChooseMode { mode };
        until(&mut game, action.clone());
        act(&mut game, action);
    }
    for _ in 0..20 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            break;
        }
        let defender = Action::ChooseEntryAttackingDefender {
            defender: CombatDefender::Player(other),
        };
        if game.legal_actions().contains(&defender) {
            act(&mut game, defender);
        } else {
            progress(&mut game);
        }
    }
    assert_eq!(stats(&game, attacker), (3, 3));
    assert_eq!(stats(&game, parent), (4, 4));
    assert_eq!(stats(&game, child), (1, 1));
    let rabbits = token_ids(&game, Card(5285));
    assert_eq!(rabbits.len(), 1);
    assert!(game.object_view(rabbits[0]).tapped);
    assert!(game.state.attackers.iter().any(|a| a.card == rabbits[0]));
}
