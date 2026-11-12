//! Authored blocked-attacker permissions and live creature-target restrictions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn start_attacking(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    pass_pair(game);
}

#[test]
fn norin_cannot_block_and_gives_each_blocked_creature_one_exact_play_offer() {
    let player = Player::P0;
    let opponent = player.other();
    let mut defending = scenario(opponent);
    let norin = put(
        &mut defending,
        player,
        "Norin, Swift Survivalist",
        Zone::Battlefield,
    );
    let incoming = put(&mut defending, opponent, "Bear Cub", Zone::Battlefield);
    start_attacking(&mut defending, &[incoming]);
    reject(
        &mut defending,
        Action::Block {
            blocker: norin,
            attacker: incoming,
        },
    );

    let mut game = scenario(player);
    let norin = put(&mut game, player, "Norin, Swift Survivalist", Zone::Hand);
    cast(&mut game, norin, None);
    settle(&mut game);
    let attackers =
        ["Bear Cub", "Hill Giant"].map(|name| put(&mut game, player, name, Zone::Battlefield));
    let blockers =
        ["Grizzly Bears"; 3].map(|name| put(&mut game, opponent, name, Zone::Battlefield));
    start_attacking(&mut game, &attackers);
    for (blocker, attacker) in blockers
        .into_iter()
        .zip([attackers[0], attackers[0], attackers[1]])
    {
        act(&mut game, Action::Block { blocker, attacker });
    }
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    assert_eq!(game.state.stack.len(), 2);
    let subject = game.state.stack.last().unwrap().entering_object.unwrap();
    game.move_object(norin, Zone::Graveyard);

    let mut blinked = game.clone();
    blinked.move_object(subject.card, Zone::Exile);
    blinked.move_object(subject.card, Zone::Battlefield);
    pass_pair(&mut blinked);
    assert_eq!(blinked.state.pending, Pending::Priority);
    assert!(blinked.observe(player).exile_permissions.is_empty());

    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    let exiled = game.object_ref(subject.card);
    assert_eq!(game.object_view(subject.card).zone, Zone::Exile);
    assert!(
        game.observe(player)
            .exile_permissions
            .iter()
            .any(|permission| {
                permission.object == exiled
                    && permission.expiry == ExilePlayExpiry::Turn(game.state.turn)
            })
    );
    let recast = Action::AnnounceExile { card: subject.card };
    reject(&mut game, recast.clone());
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.observe(player).exile_permissions.len(), 1);
    for _ in 0..16 {
        if game.state.step == Step::Main2 {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::AssignDamage { .. })
        });
    }
    assert_eq!(game.state.step, Step::Main2);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 3, 0];
    act(&mut game, recast);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(subject.card).zone, Zone::Battlefield);
    assert!(game.observe(player).exile_permissions.is_empty());
}

#[test]
fn nowhere_to_run_flashes_in_and_changes_hexproof_and_ward_at_the_right_times() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let source = put(&mut game, player, "Nowhere to Run", Zone::Hand);
    let hexproof = put(
        &mut game,
        opponent,
        "Sphinx of the Final Word",
        Zone::Battlefield,
    );
    let ward = put(&mut game, opponent, "Tolarian Terror", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    cast(&mut game, source, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert!(
        game.object_view(hexproof)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    let target = |card| Action::ChooseTarget {
        target: Target::Card(card),
    };
    reject(&mut game, target(own));

    let mut hexproof_branch = game.clone();
    act(&mut hexproof_branch, target(hexproof));
    let mut removed = hexproof_branch.clone();
    removed.move_object(source, Zone::Graveyard);
    settle(&mut removed);
    assert_eq!(stats(&removed, hexproof), (5, 5));
    settle(&mut hexproof_branch);
    assert_eq!(stats(&hexproof_branch, hexproof), (2, 2));

    act(&mut game, target(ward));
    assert_eq!(game.state.stack.len(), 1);
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(stats(&game, ward), (2, 2));
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    if game.actor() == Some(opponent) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, removal, Some(Target::Card(ward)));
    assert!(game.state.stack.iter().any(|item| {
        matches!(item.kind, StackKind::TriggeredAbility { .. })
            && item.source.is_some_and(|source| source.id == ward)
    }));
}
