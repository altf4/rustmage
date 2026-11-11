//! Printed Class, pawprint, sacrifice, and private selection scenarios.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    for finish in [Action::FinishTargets, Action::FinishStackTargets] {
        if game.legal_actions().contains(&finish) {
            act(game, finish);
        }
    }
}

fn begin_step(game: &mut Game, active: Player, step: Step) {
    game.state.active = active;
    game.state.priority = active;
    game.begin_step(step);
    game.prepare_triggers();
}

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

fn activate(game: &mut Game, source: CardId, ability: u16) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability: index, .. } | Action::AnnounceActivate { card, ability: index, .. } if *card == source && *index == ability),
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn cast_cost(game: &mut Game, wanted: CastCostClause) {
    let index = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == wanted && !row.paid)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause: index });
}

fn paid_offspring(game: &mut Game, source: CardId) {
    act(game, Action::AnnounceCast { card: source });
    act(game, Action::ChooseOffspring { pay: true });
    pay(game);
}

fn choose_cost(game: &mut Game, source: CardId) {
    let object = game.object_ref(source);
    act(game, Action::ChooseAdditionalCost { object });
}

#[test]
fn bandits_talent_offers_a_private_alternative_and_checks_class_levels() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let talent = put(&mut game, player, "Bandit's Talent", Zone::Hand);
    let first = put(&mut game, opponent, "Forest", Zone::Hand);
    let second = put(&mut game, opponent, "Swamp", Zone::Hand);
    let nonland = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
    cast(&mut game, talent, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).effect_choice.is_none());
    reject(&mut game, Action::Discard { card: first });
    let mut one = game.clone();
    act(&mut one, Action::Discard { card: nonland });
    assert_eq!(one.object_view(nonland).zone, Zone::Graveyard);
    assert_eq!(one.object_view(first).zone, Zone::Hand);
    act(&mut game, Action::DeclineEffectPayment);
    for card in [first, second] {
        act(&mut game, Action::Discard { card });
        assert_eq!(
            game.object_view(card).zone,
            if card == first {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
    }
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(nonland).zone, Zone::Hand);
    activate(&mut game, talent, 0);
    settle(&mut game);
    assert_eq!(game.object_view(talent).class_level, Some(2));
    let mut above_limit = game.clone();
    put(&mut above_limit, opponent, "Forest", Zone::Hand);
    begin_step(&mut above_limit, opponent, Step::Upkeep);
    assert!(above_limit.state.stack.is_empty());
    let mut upkeep = game.clone();
    begin_step(&mut upkeep, opponent, Step::Upkeep);
    assert_eq!(upkeep.state.stack.len(), 1);
    let mut response = upkeep.clone();
    put(&mut response, opponent, "Forest", Zone::Hand);
    settle(&mut response);
    assert_eq!(life(&response, opponent), 20);
    settle(&mut upkeep);
    assert_eq!(life(&upkeep, opponent), 18);
    activate(&mut game, talent, 1);
    settle(&mut game);
    assert_eq!(game.object_view(talent).class_level, Some(3));
    game.state.step = Step::Upkeep;
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::Draw);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn scavengers_talent_creates_one_food_mills_each_sacrifice_and_returns_with_finality() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let talent = put(&mut game, player, "Scavenger's Talent", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.sacrifice_batch(&[(player, first), (player, second)]);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let food = game.controlled_token_ids(player).next().unwrap();
    assert!(game.has_subtype(food, "Food"));
    let third = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.sacrifice_batch(&[(player, third)]);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    for ability in [0, 1] {
        activate(&mut game, talent, ability);
        settle(&mut game);
    }
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    begin_step(&mut game, player, Step::End);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert_eq!(declined.object_view(creature).zone, Zone::Battlefield);
    let object = game.object_ref(talent);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    for source in [food, creature, equipment] {
        choose_cost(&mut game, source);
    }
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    let object = game.object_ref(creature);
    act(&mut game, Action::SelectGraveyardReturn { object });
    assert_eq!(game.counter_count(creature, CounterKind::Finality), 1);
    for _ in 0..24 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
            break;
        }
        let player_target = Action::ChooseTarget {
            target: Target::Player(opponent),
        };
        if game.legal_actions().contains(&player_target) {
            target(&mut game, Target::Player(opponent));
        } else {
            choose(&mut game, |action| {
                matches!(
                    action,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
                )
            });
        }
    }
    assert_eq!(game.state.players[opponent.index()].library.len(), 2);
    assert_eq!(game.object_view(talent).zone, Zone::Battlefield);
}

#[test]
fn rottenmouth_viper_applies_sacrifices_and_resolves_each_blight_choice() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let viper = put(&mut game, player, "Rottenmouth Viper", Zone::Hand);
    let food = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let discard = put(&mut game, opponent, "Forest", Zone::Hand);
    let opponent_food = put(&mut game, opponent, "Gingerbrute", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 3, 0];
    act(&mut game, Action::AnnounceCast { card: viper });
    reject(&mut game, Action::ChooseX { value: 1 });
    act(&mut game, Action::ChooseX { value: 2 });
    act(&mut game, Action::BeginOrderedCosts);
    cast_cost(&mut game, CastCostClause::Additional);
    choose_cost(&mut game, food);
    choose_cost(&mut game, bear);
    cast_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(food).zone, Zone::Graveyard);
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(viper, CounterKind::Blight), 2);
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).effect_choice.is_none());
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(life(&game, opponent), 16);
    assert_eq!(game.object_view(opponent_food).zone, Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.objects[viper as usize].summoning_sick = false;
    act(&mut game, Action::Attack { card: viper });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(viper, CounterKind::Blight), 4);
    let object = game.object_ref(opponent_food);
    act(&mut game, Action::Sacrifice { object });
    for _ in 0..3 {
        act(&mut game, Action::DeclineEffectPayment);
    }
    assert_eq!(life(&game, opponent), 4);
}

#[test]
fn season_of_loss_resolves_repeated_sacrifices_and_counts_deaths_or_graveyard_cards() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let season = put(&mut game, player, "Season of Loss", Zone::Hand);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let finality = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.add_counter(finality, CounterKind::Finality, 1);
    let prior = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.sacrifice_batch(&[(player, prior)]);
    let opposing_first = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let opposing_second = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    for (mode, draws, opponent_life) in [(8, 2, 20), (14, 0, 18)] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: season });
        assert_eq!(
            branch
                .legal_actions()
                .iter()
                .filter(|action| matches!(action, Action::ChooseMode { .. }))
                .count(),
            16
        );
        act(&mut branch, Action::ChooseMode { mode });
        pay(&mut branch);
        pass_pair(&mut branch);
        for (own, opposing) in [(first, opposing_first), (finality, opposing_second)] {
            let object = branch.object_ref(own);
            act(&mut branch, Action::Sacrifice { object });
            assert_eq!(branch.object_view(own).zone, Zone::Battlefield);
            let object = branch.object_ref(opposing);
            act(&mut branch, Action::Sacrifice { object });
        }
        assert_eq!(branch.object_view(first).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(finality).zone, Zone::Exile);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
        assert_eq!(life(&branch, opponent), opponent_life);
    }
}

#[test]
fn stargaze_selects_privately_and_charges_life_with_a_short_library() {
    let player = Player::P1;
    for (x, library_size) in [(0, 4), (2, 4), (3, 1)] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Stargaze", Zone::Hand);
        let excess = game.state.players[player.index()].library.len() - library_size;
        let earlier = game.state.players[player.index()].library[..excess].to_vec();
        for card in earlier {
            game.move_object(card, Zone::Exile);
        }
        let library = game.state.players[player.index()].library.clone();
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseX { value: x });
        pay(&mut game);
        pass_pair(&mut game);
        if x == 2 {
            assert!(game.observe(player.other()).library_selection.is_none());
            assert!(game.observe(player).library_selection.is_some());
            reject(&mut game, Action::FinishLibrarySelection);
            let snapshot = game.snapshot();
            let action = Action::SelectLibrary { card: library[1] };
            let index = game
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            let mut twin = game.clone();
            twin.apply_index(player, game.revision(), index).unwrap();
            act(&mut game, action);
            assert_eq!(twin.snapshot(), game.snapshot());
            twin.restore(&snapshot).unwrap();
            assert_eq!(twin.snapshot(), snapshot);
            act(&mut game, Action::SelectLibrary { card: library[3] });
            act(&mut game, Action::FinishLibrarySelection);
        }
        settle(&mut game);
        assert_eq!(life(&game, player), 20 - i128::from(x));
        for (index, card) in library.into_iter().enumerate() {
            let expected = match (x, index) {
                (0, _) => Zone::Library,
                (2, 0 | 2) => Zone::Graveyard,
                _ => Zone::Hand,
            };
            assert_eq!(game.object_view(card).zone, expected);
        }
    }
}

#[test]
fn thornplate_intimidator_parent_and_child_each_offer_the_opponent_a_choice() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let intimidator = put(&mut game, player, "Thornplate Intimidator", Zone::Hand);
    let discard = put(&mut game, opponent, "Forest", Zone::Hand);
    let sacrifice = put(&mut game, opponent, "Gingerbrute", Zone::Battlefield);
    paid_offspring(&mut game, intimidator);
    for _ in 0..16 {
        match game.state.pending {
            Pending::EffectChoice { .. } => break,
            Pending::OrderTriggers { .. } => choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            }),
            Pending::Targets { .. } => target(&mut game, Target::Player(opponent)),
            _ => act(&mut game, Action::Pass),
        }
    }
    assert_eq!(game.actor(), Some(opponent));
    act(&mut game, Action::Discard { card: discard });
    for _ in 0..16 {
        if matches!(game.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        let choose = Action::ChooseTarget {
            target: Target::Player(opponent),
        };
        if game.legal_actions().contains(&choose) {
            target(&mut game, Target::Player(opponent));
        } else {
            act(&mut game, Action::Pass);
        }
    }
    let mut paid = game.clone();
    let object = paid.object_ref(sacrifice);
    act(&mut paid, Action::Sacrifice { object });
    assert_eq!(paid.object_view(sacrifice).zone, Zone::Graveyard);
    assert_eq!(life(&paid, opponent), 20);
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(life(&game, opponent), 17);
    let child = game.controlled_token_ids(player).next().unwrap();
    assert_eq!(
        game.object_view(child).card,
        game.object_view(intimidator).card
    );
    assert_eq!(stats(&game, child), (1, 1));
}

#[test]
fn wicks_patrol_targets_after_milling_and_uses_the_live_graveyard_maximum() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let patrol = put(&mut game, player, "Wick's Patrol", Zone::Hand);
    let target_creature = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let highest = put(&mut game, player, "Serra Angel", Zone::Library);
    let next = put(&mut game, player, "Grizzly Bears", Zone::Library);
    put(&mut game, player, "Forest", Zone::Library);
    cast(&mut game, patrol, None);
    pass_pair(&mut game);
    assert!(game.legal_actions().contains(&Action::Pass));
    pass_pair(&mut game);
    assert_eq!(game.object_view(highest).zone, Zone::Graveyard);
    assert_eq!(game.object_view(next).zone, Zone::Graveyard);
    target(&mut game, Target::Card(target_creature));
    let mut reduced = game.clone();
    reduced.move_object(highest, Zone::Exile);
    settle(&mut reduced);
    assert_eq!(stats(&reduced, target_creature), (1, 1));
    settle(&mut game);
    assert_eq!(game.object_view(target_creature).zone, Zone::Graveyard);
}

#[test]
fn wick_grows_a_nontargeted_snail_and_uses_its_power_after_sacrificing_it() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let wick = put(&mut game, player, "Wick, the Whorled Mind", Zone::Hand);
    cast(&mut game, wick, None);
    settle(&mut game);
    let snail = game.controlled_token_ids(player).next().unwrap();
    assert_eq!(stats(&game, snail), (1, 1));
    assert!(game.has_subtype(snail, "Snail"));
    game.state.objects[snail as usize].gained_keywords = Keywords::HEXPROOF;
    let rat = put(&mut game, player, "Burglar Rat", Zone::Hand);
    cast(&mut game, rat, None);
    pass_pair(&mut game);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    for _ in 0..8 {
        match game.state.pending {
            Pending::EffectChoice { .. } => {
                let object = game.object_ref(snail);
                act(&mut game, Action::ChooseEffectObject { object });
            }
            Pending::Priority if game.state.stack.is_empty() => break,
            Pending::Priority => act(&mut game, Action::Pass),
            _ => panic!("unexpected Rat entry choice: {:?}", game.state.pending),
        }
    }
    assert_eq!(stats(&game, snail), (2, 2));
    game.add_counter(snail, CounterKind::PlusOne, 1);
    choose(
        &mut game,
        |action| matches!(action, Action::AnnounceActivate { card, ability: 0, .. } if *card == wick),
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    choose_cost(&mut game, snail);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_ne!(game.object_view(snail).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(life(&game, opponent), 17);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
}

#[test]
fn iridescent_vinelasher_and_starscape_cleric_copies_retain_their_own_triggers() {
    let player = Player::P0;
    let opponent = player.other();
    for name in ["Iridescent Vinelasher", "Starscape Cleric"] {
        let mut game = scenario(player);
        let source = put(&mut game, player, name, Zone::Hand);
        paid_offspring(&mut game, source);
        settle(&mut game);
        let child = game.controlled_token_ids(player).next().unwrap();
        assert_eq!(game.object_view(child).card, game.object_view(source).card);
        assert_eq!(stats(&game, child), (1, 1));
        if name == "Iridescent Vinelasher" {
            let land = put(&mut game, player, "Swamp", Zone::Hand);
            act(&mut game, Action::PlayLand { card: land });
            for _ in 0..16 {
                if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
                    break;
                }
                let choose_target = Action::ChooseTarget {
                    target: Target::Player(opponent),
                };
                if game.legal_actions().contains(&choose_target) {
                    target(&mut game, Target::Player(opponent));
                } else {
                    choose(&mut game, |action| {
                        matches!(action, Action::Pass | Action::PutTrigger { .. })
                    });
                }
            }
            assert_eq!(life(&game, opponent), 18);
        } else {
            for bat in [source, child] {
                assert!(
                    game.object_view(bat)
                        .keywords
                        .contains(Keywords::FLYING.union(Keywords::CANT_BLOCK))
                );
            }
            let duo = put(&mut game, player, "Glidedive Duo", Zone::Hand);
            cast(&mut game, duo, None);
            for _ in 0..16 {
                if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
                    break;
                }
                choose(&mut game, |action| {
                    matches!(action, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!([life(&game, player), life(&game, opponent)], [22, 16]);
        }
    }
}
