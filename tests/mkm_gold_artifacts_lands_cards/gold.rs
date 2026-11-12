use super::support::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn exact_cost_object(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn tokens(game: &Game, player: Player, recipe: Card) -> usize {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == recipe)
        .count()
}

fn combat_with_attackers(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn until(game: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..20 {
        if condition(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::ChooseEntryTimestamp { .. }
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!(
        "scenario did not reach its expected decision: {:?}",
        game.state.pending
    );
}

#[test]
fn niv_counts_distinct_pairs_once_for_damage_draw_and_life() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let niv = put(
        &mut game,
        player,
        "Niv-Mizzet, Guildpact",
        Zone::Battlefield,
    );
    assert!(game.keywords(niv).contains(Keywords::FLYING));
    for name in [
        "Warleader's Call",
        "Warleader's Call",
        "Trostani, Three Whispers",
        "Private Eye",
        "Riku of Many Paths",
    ] {
        put(&mut game, player, name, Zone::Battlefield);
    }

    for (spell_name, legal) in [("Lightning Helix", false), ("Shock", true)] {
        let mut choice = game.clone();
        choice.state.priority = opponent;
        let spell = put(&mut choice, opponent, spell_name, Zone::Hand);
        act(&mut choice, Action::AnnounceCast { card: spell });
        let aimed = Action::ChooseTarget {
            target: Target::Card(niv),
        };
        assert_eq!(choice.legal_actions().contains(&aimed), legal);
    }

    combat_with_attackers(&mut game, &[niv]);
    until(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    assert_eq!(game.state.players[opponent.index()].life, 12);
    target(&mut game, Target::Player(opponent));
    target(&mut game, Target::Player(player));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 9);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
}

#[test]
fn officious_interrogation_locks_surcharges_and_investigates_the_live_count() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Officious Interrogation", Zone::Hand);

    let mut one = game.clone();
    one.state.players[player.index()].mana = [1, 1, 0, 0, 0, 0];
    act(&mut one, Action::AnnounceCast { card: spell });
    target(&mut one, Target::Player(opponent));
    reject(
        &mut one,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(&mut one, Action::FinishTargets);
    pay(&mut one);
    settle(&mut one);
    assert_eq!(tokens(&one, player, CLUE), 2);

    game.state.players[player.index()].mana = [2, 2, 0, 0, 0, 0];
    announce(
        &mut game,
        spell,
        &[Target::Player(opponent), Target::Player(player)],
    );
    let paid = game.state.stack.last().unwrap().cast.unwrap().payment;
    assert_eq!(paid, [2, 2, 0, 0, 0, 0]);
    game.move_object(attacker, Zone::Exile);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE), 2);
}

#[test]
fn private_eye_buffs_only_other_controlled_detectives_and_targets_on_draw_two() {
    let player = Player::P1;
    let mut game = scenario(player);
    let eye = put(&mut game, player, "Private Eye", Zone::Battlefield);
    let own = put(&mut game, player, "Novice Inspector", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Novice Inspector",
        Zone::Battlefield,
    );
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(game.stats(eye).power, 3);
    assert_eq!(game.stats(own).power, 2);
    assert_eq!(game.stats(opposing).power, 1);
    assert_eq!(game.stats(other).power, 2);
    let draw = put(&mut game, player, "Quick Study", Zone::Hand);
    cast(&mut game, draw, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    let mut branch = game.clone();
    target(&mut branch, Target::Card(opposing));
    settle(&mut branch);
    assert!(
        branch
            .keywords(opposing)
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    target(&mut game, Target::Card(own));
    settle(&mut game);
    assert!(game.keywords(own).contains(Keywords::CANT_BE_BLOCKED));
    let draw = put(&mut game, player, "Quick Study", Zone::Hand);
    cast(&mut game, draw, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
}

#[test]
fn rakdos_targets_an_opponent_who_can_pay_two_actual_eligible_permanents() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let rakdos = put(
        &mut game,
        player,
        "Rakdos, Patron of Chaos",
        Zone::Battlefield,
    );
    assert!(
        game.keywords(rakdos)
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
    let first = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    game.create_tokens(opponent, CLUE, 1, false);
    assert!(!game.advance_entries());
    let token = game.controlled_token_ids(opponent).next().unwrap();
    game.begin_step(Step::End);
    game.prepare_triggers();
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    target(&mut game, Target::Player(opponent));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    for id in [land, token] {
        let action = exact_cost_object(&game, id);
        reject(&mut game, action);
    }
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert_eq!(
        decline.state.players[player.index()].cards_drawn_this_turn,
        2
    );
    for id in [first, second] {
        let action = exact_cost_object(&game, id);
        act(&mut game, action);
    }
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    for id in [first, second] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
}

#[test]
fn relive_returns_each_legal_role_as_a_five_five_elemental() {
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let enchantment = put(&mut game, player, "Impact Tremors", Zone::Graveyard);
    let aura = put(&mut game, player, "Pacifism", Zone::Graveyard);
    let relive = put(&mut game, player, "Relive the Past", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: relive });
    target(&mut game, Target::Card(equipment));
    target(&mut game, Target::Card(land));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    target(&mut game, Target::Card(enchantment));
    pay(&mut game);
    until(&mut game, |g| {
        g.state.pending == Pending::Priority && g.state.stack.is_empty()
    });
    for (id, kinds) in [(equipment, 17), (land, 3), (enchantment, 33)] {
        let object = game.object_view(id);
        assert_eq!(object.zone, Zone::Battlefield);
        assert_eq!(object.kinds, kinds);
        assert!(object.has_subtype("Elemental"));
        assert_eq!((object.power, object.toughness), (5, 5));
    }
    assert_eq!(game.state.players[player.other().index()].life, 17);
}

#[test]
fn repulsive_mutation_counts_power_after_the_new_counters() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    game.state.priority = opponent;
    game.state.players[opponent.index()].mana = [0, 0, 0, 1, 0, 4];
    cast(&mut game, shock, Some(Target::Player(player)));
    let pending_spell = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    let mutation = put(&mut game, player, "Repulsive Mutation", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: mutation });
    act(&mut game, Action::ChooseX { value: 2 });
    target(&mut game, Target::Card(recipient));
    target(&mut game, Target::Stack(pending_spell));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 2);
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    let mut paid = game.clone();
    act(
        &mut paid,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 4],
        },
    );
    settle(&mut paid);
    assert_eq!(paid.state.players[player.index()].life, 18);
    act(&mut game, Action::DeclineResolutionMana);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[player.index()].life, 20);
}
