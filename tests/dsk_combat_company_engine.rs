//! Declaration-wide attack and block restrictions through the action surface.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TOKEN: &str = "@token/cat_beast_2_2_white";
const COMPANY: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantAttackOrBlockAlone],
    }],
    ..CardRules::EMPTY
};

fn token() -> Card {
    Card::from_name(TOKEN).unwrap()
}

fn attacks(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

fn attack(card: CardId) -> Action {
    Action::Attack { card }
}

fn attack_walker(game: &Game, card: CardId, walker: CardId) -> Action {
    Action::AttackPlaneswalker {
        card,
        defender: game.object_ref(walker),
    }
}

fn block(blocker: CardId, attacker: CardId) -> Action {
    Action::Block { blocker, attacker }
}

fn choose_blockers(game: &mut Game, attackers: &[CardId]) {
    attacks(game);
    for &card in attackers {
        act(game, attack(card));
    }
    act(game, Action::FinishAttackers);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::Blockers);
}

#[test]
fn two_restricted_attackers_can_attack_different_defenders() {
    let _rules = fixture_rules::activate(&[(token(), &COMPANY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, TOKEN, Zone::Battlefield);
    let walker = put(
        &mut game,
        player.other(),
        "Liliana, Dreadhorde General",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 6);
    attacks(&mut game);
    reject(&mut game, attack(first));
    let solo_walker = attack_walker(&game, first, walker);
    reject(&mut game, solo_walker);
    assert!(game.legal_actions().contains(&Action::FinishAttackers));

    let other = put(&mut game, player, TOKEN, Zone::Battlefield);
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == attack(first))
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, attack(first));
    assert_eq!(game.snapshot(), indexed.snapshot());
    reject(&mut game, Action::FinishAttackers);

    let other_walker = attack_walker(&game, other, walker);
    act(&mut game, other_walker);
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.attackers.len(), 2);
    assert_eq!(
        game.state.attackers[0].defender,
        Some(CombatDefender::Player(player.other()))
    );
    assert_eq!(
        game.state.attackers[1].defender,
        Some(CombatDefender::Planeswalker(game.object_ref(walker)))
    );
    game.move_object(other, Zone::Graveyard);
    assert_eq!(game.state.attackers.len(), 1);
    assert_eq!(game.state.attackers[0].card, first);

    game.restore(&before).unwrap();
    assert!(game.state.attackers.is_empty());
    assert!(game.legal_actions().contains(&attack(first)));
}

#[test]
fn losing_the_restriction_allows_a_creature_token_to_attack_alone() {
    let _rules = fixture_rules::activate(&[(token(), &COMPANY)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let creature = put(&mut game, player, TOKEN, Zone::Battlefield);
    let aura = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (1, 1));
    attacks(&mut game);
    act(&mut game, attack(creature));
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.attackers.len(), 1);
    assert_eq!(game.state.attackers[0].card, creature);
}

#[test]
fn attack_requirements_count_company_and_the_defender_capacity() {
    let _rules = fixture_rules::activate(&[(token(), &COMPANY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let restricted = put(&mut game, player, TOKEN, Zone::Battlefield);
    let saga = put(&mut game, player, "Maximum Carnage", Zone::Hand);
    cast(&mut game, saga, None);
    settle(&mut game);
    attacks(&mut game);
    assert!(game.legal_actions().contains(&Action::FinishAttackers));
    reject(&mut game, attack(restricted));

    let companion = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let walker = put(
        &mut game,
        player.other(),
        "Liliana, Dreadhorde General",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 6);
    reject(&mut game, Action::FinishAttackers);
    // Maximum Carnage also requires an attack on the opponent when able.
    let inferior = attack_walker(&game, restricted, walker);
    reject(&mut game, inferior);
    let mut limited = game.clone();
    for creature in [restricted, companion] {
        limited.state.objects[creature as usize].gained_keywords = Keywords::FLYING;
    }
    put(
        &mut limited,
        player.other(),
        "Storm, Windrider",
        Zone::Battlefield,
    );
    put(
        &mut limited,
        player.other(),
        "Tomik, Orzhov Lawmage",
        Zone::Battlefield,
    );
    let unsupported = attack_walker(&limited, restricted, walker);
    reject(&mut limited, unsupported);
    reject(&mut limited, Action::FinishAttackers);
    let single = attack_walker(&limited, companion, walker);
    act(&mut limited, single);
    act(&mut limited, Action::FinishAttackers);
    assert_eq!(limited.state.attackers.len(), 1);

    act(&mut game, attack(companion));
    reject(&mut game, Action::FinishAttackers);
    act(&mut game, attack(restricted));
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.attackers.len(), 2);
}

#[test]
fn a_taxed_companion_must_leave_enough_mana_after_the_attackers_tap() {
    let _rules = fixture_rules::activate(&[(token(), &COMPANY)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let restricted = put(&mut game, player, TOKEN, Zone::Battlefield);
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Archangel of Tithes",
        Zone::Battlefield,
    );
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    attacks(&mut game);
    reject(&mut game, attack(restricted));
    assert!(game.legal_actions().contains(&attack(elf)));
    assert!(game.legal_actions().contains(&Action::FinishAttackers));

    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    act(&mut game, attack(restricted));
    reject(&mut game, Action::FinishAttackers);
    act(&mut game, attack(elf));
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.observe(player).attack_payment.unwrap().generic, 2);
    assert!(game.object_view(elf).tapped);
    act(&mut game, Action::TapMana { card: forest });
    choose(&mut game, |action| {
        matches!(action, Action::PayAttackCost { .. })
    });
    assert_eq!(game.state.attackers.len(), 2);
}

#[test]
fn the_required_block_can_use_a_companion_blocking_another_attacker() {
    let _rules = fixture_rules::activate(&[(token(), &COMPANY)]);
    let player = Player::P0;
    let defender = player.other();
    let mut game = scenario(player);
    let required = put(
        &mut game,
        player,
        "Raphael, Ninja Destroyer",
        Zone::Battlefield,
    );
    game.state.objects[required as usize].gained_keywords = Keywords::PROTECTION_GREEN;
    let ordinary = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let restricted = put(&mut game, defender, TOKEN, Zone::Battlefield);
    choose_blockers(&mut game, &[required, ordinary]);
    reject(&mut game, block(restricted, required));
    assert!(game.legal_actions().contains(&Action::FinishBlockers));

    let companion = put(&mut game, defender, "Grizzly Bears", Zone::Battlefield);
    reject(&mut game, Action::FinishBlockers);
    reject(&mut game, block(restricted, ordinary));
    reject(&mut game, block(companion, required));
    act(&mut game, block(restricted, required));
    reject(&mut game, Action::FinishBlockers);
    act(&mut game, block(companion, ordinary));
    act(&mut game, Action::FinishBlockers);
    assert_eq!(
        game.state.blocks,
        [(restricted, required), (companion, ordinary)]
    );
    game.move_object(companion, Zone::Graveyard);
    assert_eq!(game.state.blocks, [(restricted, required)]);
}

#[test]
fn block_tax_accounts_for_the_companions_entire_menace_assignment() {
    let _rules = fixture_rules::activate(&[(token(), &COMPANY)]);
    let player = Player::P0;
    let defender = player.other();
    let mut game = scenario(player);
    let archangel = put(&mut game, player, "Archangel of Tithes", Zone::Battlefield);
    let menace = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.objects[menace as usize].gained_keywords = Keywords::MENACE;
    let restricted = put(&mut game, defender, TOKEN, Zone::Battlefield);
    game.state.objects[restricted as usize].gained_keywords = Keywords::REACH;
    let first = put(&mut game, defender, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, defender, "Bear Cub", Zone::Battlefield);
    choose_blockers(&mut game, &[archangel, menace]);
    game.state.players[defender.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, block(restricted, archangel));
    assert!(game.legal_actions().contains(&block(first, menace)));

    game.state.players[defender.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut game, block(restricted, archangel));
    reject(&mut game, Action::FinishBlockers);
    act(&mut game, block(first, menace));
    reject(&mut game, Action::FinishBlockers);
    act(&mut game, block(second, menace));
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.observe(defender).block_payment.unwrap().generic, 3);
    choose(&mut game, |action| {
        matches!(action, Action::PayBlockCost { .. })
    });
    assert_eq!(game.state.blocks.len(), 3);
}
