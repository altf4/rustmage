//! Paid gameplay outcomes for inherited Wilds of Eldraine cards from M through R.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FOOD: Card = Card::from_catalogue_id(5246);
const RAT: Card = Card::from_catalogue_id(5323);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn tokens(game: &Game, controller: Player, recipe: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield
                && object.controller == controller
                && object.is_token()
                && object.card == recipe
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

fn role(game: &Game, controller: Player, recipient: CardId, kind: RoleKind) -> CardId {
    let recipient = game.object_ref(recipient);
    game.state
        .objects
        .iter()
        .position(|object| {
            object.zone == Zone::Battlefield
                && object.controller == controller
                && object.card.role() == Some(kind)
                && object.attached_to == Some(recipient)
        })
        .expect("role on the exact recipient") as CardId
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    game.begin_step(Step::End);
    for _ in 0..12 {
        if game.state.turn != turn {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("turn did not reach cleanup");
}

fn reach_attackers(game: &mut Game) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::Attackers);
}

fn attack(game: &mut Game, card: CardId) {
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

#[test]
fn ride_the_rails_and_desperate_parry_pay_the_instant_face_then_the_exiled_creature() {
    for (name, player, altered, front, keywords) in [
        (
            "Minecart Daredevil",
            Player::P0,
            (5, 4),
            (4, 2),
            Keywords::NONE,
        ),
        (
            "Obyra's Attendants",
            Player::P1,
            (-1, 3),
            (3, 4),
            Keywords::FLYING,
        ),
    ] {
        let mut game = scenario(player.other());
        let spell = put(&mut game, player, name, Zone::Hand);
        let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
        act(&mut game, Action::Pass);
        reject(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::AnnounceAdventure { card: spell });
        act(&mut game, target(giant));
        pay(&mut game);

        let mut stale = game.clone();
        act(&mut stale, Action::Pass);
        cast(&mut stale, bounce, Some(Target::Card(giant)));
        settle(&mut stale);
        assert_eq!(stale.object_view(giant).zone, Zone::Hand);
        assert_eq!(stale.object_view(spell).zone, Zone::Graveyard);

        settle(&mut game);
        assert_eq!(stats(&game, giant), altered);
        assert_eq!(game.object_view(spell).zone, Zone::Exile);
        let exiled = game.object_ref(spell);
        reject(&mut game, Action::AnnounceExile { card: spell });
        finish_turn(&mut game);
        assert_eq!(stats(&game, giant), (3, 3));
        game.begin_step(Step::Main1);
        game.state.players[player.index()].mana = [8; 6];
        act(&mut game, Action::AnnounceExile { card: spell });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        assert_ne!(game.object_ref(spell), exiled);
        assert_eq!(stats(&game, spell), front);
        assert_eq!(game.object_view(spell).keywords, keywords);
    }
}

#[test]
fn free_the_fae_takes_one_of_three_eligible_milled_cards_and_exiles_the_prankster() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Picklock Prankster", Zone::Hand);
    let old = put(&mut game, player, "Shock", Zone::Graveyard);
    let adventurer = put(&mut game, player, "Minecart Daredevil", Zone::Library);
    let faerie = put(&mut game, player, "Obyra's Attendants", Zone::Library);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let sorcery = put(&mut game, player, "Divination", Zone::Library);
    let before = game.state.players[player.index()].library.len();

    act(&mut game, Action::AnnounceAdventure { card: source });
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].library.len(), before - 4);
    assert_eq!(game.state.pending, Pending::MilledSelection { player });
    assert!(game.observe(player.other()).milled_choice.is_none());
    reject(&mut game, Action::FinishMilledSelection);
    for invalid in [adventurer, old] {
        let object = game.object_ref(invalid);
        reject(&mut game, Action::SelectMilled { object });
    }

    let eligible = [faerie, instant, sorcery];
    for selected in eligible {
        let mut branch = game.clone();
        let object = branch.object_ref(selected);
        act(&mut branch, Action::SelectMilled { object });
        settle(&mut branch);
        for card in [faerie, instant, sorcery, adventurer, old] {
            assert_eq!(
                branch.object_view(card).zone,
                if card == selected {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
        }
        assert!(
            branch
                .observe(player.other())
                .known_hand
                .contains(&branch.object_ref(selected))
        );
        assert_eq!(branch.object_view(source).zone, Zone::Exile);
        act(&mut branch, Action::AnnounceExile { card: source });
        pay(&mut branch);
        settle(&mut branch);
        let front = branch.object_view(source);
        assert_eq!(front.zone, Zone::Battlefield);
        assert_eq!((front.power, front.toughness), (1, 3));
        assert!(
            front
                .keywords
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
    }

    let mut empty = scenario(player);
    let source = put(&mut empty, player, "Picklock Prankster", Zone::Hand);
    let before = empty.state.players[player.index()].library.len();
    act(&mut empty, Action::AnnounceAdventure { card: source });
    pay(&mut empty);
    settle(&mut empty);
    assert_eq!(
        empty.state.players[player.index()].library.len(),
        before - 4
    );
    assert_eq!(empty.object_view(source).zone, Zone::Exile);
}

#[test]
fn pest_problem_makes_two_no_block_rats_then_trainee_has_first_strike_on_its_turn() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let trainee = put(&mut game, player, "Ratcatcher Trainee", Zone::Hand);
    let bear = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceAdventure { card: trainee });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(trainee).zone, Zone::Exile);
    let rats = tokens(&game, player, RAT);
    assert_eq!(rats.len(), 2);
    for rat in &rats {
        let rat = game.object_view(*rat);
        assert_eq!((rat.power, rat.toughness, rat.colors), (1, 1, 4));
        assert!(rat.has_subtype("Rat") && rat.keywords.contains(Keywords::CANT_BLOCK));
    }
    reach_attackers(&mut game);
    attack(&mut game, bear);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: rats[0],
            attacker: bear,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    finish_turn(&mut game);

    game.begin_step(Step::Main1);
    game.state.players[player.index()].mana = [8; 6];
    act(&mut game, Action::AnnounceExile { card: trainee });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, trainee), (2, 1));
    assert!(
        game.object_view(trainee)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    finish_turn(&mut game);
    assert!(
        !game
            .object_view(trainee)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
}

#[test]
fn coralsmith_can_pay_three_activations_and_scries_two_on_death() {
    let player = Player::P0;
    let mut game = scenario(player);
    let smith = put(&mut game, player, "Merfolk Coralsmith", Zone::Hand);
    let second = put(&mut game, player, "Island", Zone::Library);
    let first = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, smith, None);
    settle(&mut game);
    assert_eq!(stats(&game, smith), (2, 3));
    for power in [3, 4] {
        act(&mut game, activate(smith, 0));
        assert_eq!(game.mana_payment_domain().unwrap().generic, 1);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, smith), (power, 5 - power));
    }
    let mut cleanup = game.clone();
    finish_turn(&mut cleanup);
    assert_eq!(stats(&cleanup, smith), (2, 3));

    act(&mut game, activate(smith, 0));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(smith).zone, Zone::Graveyard);
    pass_pair(&mut game);
    let look = game.observe(player).library_choice.unwrap();
    assert_eq!(
        look.remaining
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
        [first, second]
    );
    assert!(game.observe(player.other()).library_choice.is_none());
    for (card, destination) in [
        (first, LibraryDestination::Bottom),
        (second, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    assert_eq!(game.top_library_card(player), Some(second));
}

#[test]
fn merry_bards_resolves_its_optional_payment_before_choosing_a_young_hero() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let bard = put(&mut game, player, "Merry Bards", Zone::Hand);
        let hero = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
        cast(&mut game, bard, None);
        pass_pair(&mut game);
        assert!(game.state.stack.last().unwrap().targets.is_empty());
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::ResolutionMana { player });
        let mut declined = game.clone();
        act(&mut declined, Action::DeclineResolutionMana);
        settle(&mut declined);
        assert!(
            declined
                .observe(player)
                .objects
                .iter()
                .all(|object| !object.is_token())
        );

        let payment = Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 1],
        };
        act(&mut game, payment);
        reject(&mut game, target(enemy));
        act(&mut game, target(hero));
        let mut stale = game.clone();
        act(&mut stale, Action::Pass);
        cast(&mut stale, bounce, Some(Target::Card(hero)));
        settle(&mut stale);
        assert!(
            stale
                .observe(player)
                .objects
                .iter()
                .all(|object| !object.is_token())
        );

        settle(&mut game);
        role(&game, player, hero, RoleKind::YoungHero);
        reach_attackers(&mut game);
        attack(&mut game, hero);
        settle(&mut game);
        assert_eq!(game.counter_count(hero, CounterKind::PlusOne), 1);
    }
}

#[test]
fn mintstrosity_only_creates_food_for_dying_and_the_food_can_be_paid_and_sacrificed() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Mintstrosity", Zone::Hand);
    let kill = put(&mut game, player, "Murder", Zone::Hand);
    let exile = put(&mut game, player, "Final Vengeance", Zone::Hand);
    let offering = put(&mut game, player, "A Tale for the Ages", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 1));
    assert!(tokens(&game, player, FOOD).is_empty());

    let mut banished = game.clone();
    // A paid exile spell with a cost of sacrificing a separate enchantment.
    act(&mut banished, Action::AnnounceCast { card: exile });
    act(&mut banished, target(source));
    let object = banished.object_ref(offering);
    act(&mut banished, Action::ChooseAdditionalCost { object });
    pay(&mut banished);
    settle(&mut banished);
    assert_eq!(banished.object_view(source).zone, Zone::Exile);
    assert!(tokens(&banished, player, FOOD).is_empty());

    cast(&mut game, kill, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let food = tokens(&game, player, FOOD);
    assert_eq!(food.len(), 1);
    act(&mut game, activate(food[0], 0));
    assert_eq!(game.mana_payment_domain().unwrap().generic, 2);
    pay(&mut game);
    assert_ne!(game.object_view(food[0]).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn misleading_motes_lets_the_creatures_owner_place_it_on_top_or_bottom() {
    for player in [Player::P0, Player::P1] {
        let owner = player.other();
        let mut game = scenario(player);
        let source = put(&mut game, player, "Misleading Motes", Zone::Hand);
        let victim = put(&mut game, owner, "Hill Giant", Zone::Battlefield);
        let top = game.top_library_card(owner).unwrap();
        let prior = game.object_ref(victim);
        game.change_controller(prior, player);
        cast(&mut game, source, Some(Target::Card(victim)));
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::TargetLibrary { player: owner });
        assert_eq!(game.observe(owner).target_library.unwrap().target, prior);
        assert!(game.observe(player).target_library.is_none());
        reject(
            &mut game,
            Action::ChooseLibraryDestination {
                object: prior,
                destination: LibraryDestination::SecondFromTop,
            },
        );

        for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
            let mut branch = game.clone();
            act(
                &mut branch,
                Action::ChooseLibraryDestination {
                    object: prior,
                    destination,
                },
            );
            settle(&mut branch);
            assert_eq!(branch.object_view(victim).zone, Zone::Library);
            assert_ne!(branch.object_ref(victim), prior);
            let library = &branch.state.players[owner.index()].library;
            if destination == LibraryDestination::Top {
                assert_eq!(library.last(), Some(&victim));
            } else {
                assert_eq!(library.first(), Some(&victim));
                assert_eq!(library.last(), Some(&top));
            }
            let movement = branch.observe(player).last_target_library_move.unwrap();
            assert_eq!(movement.owner, owner);
            assert_eq!(movement.destination, destination);
        }
    }
}

#[test]
fn plunge_into_winter_and_quick_study_distinguish_optional_targets_and_private_draws() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Plunge into Winter", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
    let bottom = put(&mut game, player, "Island", Zone::Library);
    let top = put(&mut game, player, "Shock", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: source });
    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    pay(&mut zero);
    act(&mut game, target(victim));
    pay(&mut game);

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    cast(&mut stale, bounce, Some(Target::Card(victim)));
    settle(&mut stale);
    assert_eq!(stale.top_library_card(player), Some(top));
    assert_eq!(stale.object_view(top).zone, Zone::Library);

    for (branch, tapped) in [(&mut zero, false), (&mut game, true)] {
        pass_pair(branch);
        assert_eq!(branch.object_view(victim).tapped, tapped);
        assert_eq!(
            branch.observe(player).library_choice.unwrap().remaining[0].id,
            top
        );
        assert!(branch.observe(player.other()).library_choice.is_none());
        act(
            branch,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(branch);
        assert_eq!(branch.object_view(bottom).zone, Zone::Hand);
        assert!(
            !branch
                .observe(player.other())
                .known_hand
                .contains(&branch.object_ref(bottom))
        );
    }

    let mut game = scenario(player.other());
    let study = put(&mut game, player, "Quick Study", Zone::Hand);
    let still_hidden = put(&mut game, player, "Forest", Zone::Library);
    let drawn = [
        put(&mut game, player, "Island", Zone::Library),
        put(&mut game, player, "Shock", Zone::Library),
    ];
    act(&mut game, Action::Pass);
    announce(&mut game, study, &[]);
    settle(&mut game);
    for card in drawn {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
        assert!(
            !game
                .observe(player.other())
                .known_hand
                .contains(&game.object_ref(card))
        );
    }
    assert_eq!(game.top_library_card(player), Some(still_hidden));
}

#[test]
fn moment_of_valor_pays_both_modes_and_rechecks_the_destroy_targets_power() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Moment of Valor", Zone::Hand);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let dragon = put(&mut game, opponent, "Shivan Dragon", Zone::Battlefield);
    let shrink = put(&mut game, opponent, "Candy Grapple", Zone::Hand);
    let destroy = put(&mut game, opponent, "Murder", Zone::Hand);
    game.state.objects[giant as usize].tapped = true;
    act(&mut game, Action::AnnounceCast { card: source });

    let mut protect = game.clone();
    act(&mut protect, Action::ChooseMode { mode: 0 });
    act(&mut protect, target(giant));
    pay(&mut protect);
    settle(&mut protect);
    assert!(!protect.object_view(giant).tapped);
    assert_eq!(stats(&protect, giant), (4, 3));
    assert!(
        protect
            .object_view(giant)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
    act(&mut protect, Action::Pass);
    cast(&mut protect, destroy, Some(Target::Card(giant)));
    settle(&mut protect);
    assert_eq!(protect.object_view(giant).zone, Zone::Battlefield);
    finish_turn(&mut protect);
    assert_eq!(stats(&protect, giant), (3, 3));
    assert!(
        !protect
            .object_view(giant)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );

    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(&mut game, target(giant));
    act(&mut game, target(dragon));
    pay(&mut game);
    let mut reduced = game.clone();
    act(&mut reduced, Action::Pass);
    act(&mut reduced, Action::AnnounceCast { card: shrink });
    act(&mut reduced, Action::ChooseBargain { sacrifice: None });
    act(&mut reduced, target(dragon));
    pay(&mut reduced);
    settle(&mut reduced);
    assert_eq!(stats(&reduced, dragon), (2, 2));
    assert_eq!(reduced.object_view(dragon).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(dragon).zone, Zone::Graveyard);
}

#[test]
fn not_dead_after_all_returns_under_the_owner_and_wicked_drains_its_controllers_opponent() {
    let player = Player::P1;
    let owner = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, owner, "Bear Cub", Zone::Battlefield);
    let steal = put(&mut game, player, "Goatnap", Zone::Hand);
    let protection = put(&mut game, player, "Not Dead After All", Zone::Hand);
    let destroy = put(&mut game, player, "Murder", Zone::Hand);
    let after_cleanup = put(&mut game, owner, "Murder", Zone::Hand);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, steal, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.object_view(victim).controller, player);
    let stolen = game.object_ref(victim);
    cast(&mut game, protection, Some(Target::Card(victim)));
    settle(&mut game);

    let mut expired = game.clone();
    finish_turn(&mut expired);
    expired.state.players[owner.index()].mana = [8; 6];
    cast(&mut expired, after_cleanup, Some(Target::Card(victim)));
    settle(&mut expired);
    assert_eq!(expired.object_view(victim).zone, Zone::Graveyard);

    cast(&mut game, destroy, Some(Target::Card(victim)));
    settle(&mut game);
    let returned = game.object_view(victim);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, owner);
    assert!(returned.tapped);
    assert_ne!(returned.reference, stolen);
    assert_eq!((returned.power, returned.toughness), (3, 3));
    let wicked = role(&game, player, victim, RoleKind::Wicked);
    cast(&mut game, disenchant, Some(Target::Card(wicked)));
    settle(&mut game);
    assert_eq!(game.state.players[owner.index()].life, 19);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(stats(&game, victim), (2, 2));
}

#[test]
fn obyra_flashes_in_and_drains_once_for_each_other_own_faerie() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let obyra = put(&mut game, player, "Obyra, Dreaming Duelist", Zone::Hand);
    let foe = put(&mut game, opponent, "Barrow Naughty", Zone::Hand);
    let nonfaerie = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let trick = put(&mut game, player, "Faebloom Trick", Zone::Hand);
    act(&mut game, Action::Pass);
    announce(&mut game, obyra, &[]);
    settle(&mut game);
    assert!(game.object_view(obyra).keywords.contains(Keywords::FLYING));
    assert_eq!(game.state.players[opponent.index()].life, 20);
    cast(&mut game, foe, None);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 20);

    act(&mut game, Action::Pass);
    cast(&mut game, trick, None);
    for _ in 0..24 {
        if game.state.stack.is_empty() && game.state.pending == Pending::Priority {
            break;
        }
        let wanted = if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            });
            continue;
        } else if game.legal_actions().contains(&target(nonfaerie)) {
            target(nonfaerie)
        } else {
            assert_eq!(game.state.pending, Pending::Priority);
            Action::Pass
        };
        act(&mut game, wanted);
    }
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(
        game.observe(player)
            .objects
            .iter()
            .filter(|object| {
                object.controller == player
                    && object.zone == Zone::Battlefield
                    && object.is_token()
                    && object.has_subtype("Faerie")
            })
            .count(),
        2
    );
}

#[test]
fn pests_and_battle_mouse_check_two_own_entries_and_discount_only_the_second_spell() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let mouse = put(&mut game, player, "Raging Battle Mouse", Zone::Hand);
        let pests = put(&mut game, player, "Pests of Honor", Zone::Hand);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut game, player, "Plains", Zone::Hand);
        let study = put(&mut game, player, "Quick Study", Zone::Hand);

        act(&mut game, Action::AnnounceCast { card: mouse });
        assert_eq!(game.mana_payment_domain().unwrap().generic, 1);
        pay(&mut game);
        settle(&mut game);
        act(&mut game, Action::PlayLand { card: land });

        let mut short = game.clone();
        let isolated = put(&mut short, player, "Pests of Honor", Zone::Battlefield);
        reach_attackers(&mut short);
        assert_eq!(stats(&short, mouse), (2, 1));
        assert_eq!(short.counter_count(isolated, CounterKind::PlusOne), 0);

        act(&mut game, Action::AnnounceCast { card: pests });
        let cost = game.mana_payment_domain().unwrap();
        assert_eq!(cost.required, [1, 0, 0, 0, 0, 0]);
        assert_eq!(cost.generic, 1);
        pay(&mut game);
        settle(&mut game);
        act(&mut game, Action::AnnounceCast { card: study });
        assert_eq!(game.mana_payment_domain().unwrap().generic, 2);
        pay(&mut game);
        settle(&mut game);

        pass_pair(&mut game);
        while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut game, |action| {
                matches!(action, Action::PutTrigger { .. })
            });
        }
        reject(&mut game, target(foe));
        act(&mut game, target(bear));
        settle(&mut game);
        assert_eq!(game.counter_count(pests, CounterKind::PlusOne), 1);
        assert_eq!(stats(&game, bear), (3, 3));
        assert_eq!(stats(&game, mouse), (2, 1));
        finish_turn(&mut game);
        assert_eq!(stats(&game, bear), (2, 2));
        assert_eq!(game.counter_count(pests, CounterKind::PlusOne), 1);
    }
}

#[test]
fn rowdy_research_uses_distinct_declared_attackers_after_a_creature_leaves_and_draws_three() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let first = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let research = put(&mut game, player, "Rowdy Research", Zone::Hand);
    let cards = [0; 3].map(|_| put(&mut game, player, "Plains", Zone::Library));
    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: first });
    act(&mut game, Action::Attack { card: second });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: island });
    cast(&mut game, bounce, Some(Target::Card(second)));
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
    reject(&mut game, Action::AnnounceCast { card: research });
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 4];
    act(&mut game, Action::AnnounceCast { card: research });
    let cost = game.mana_payment_domain().unwrap();
    assert_eq!(cost.generic, 4);
    assert_eq!(cost.required, [0, 1, 0, 0, 0, 0]);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    for card in cards {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
}

#[test]
fn ruby_haste_supports_two_mana_choices_or_an_attack_with_trigger_time_power() {
    let player = Player::P1;
    let mut game = scenario(player);
    let ruby = put(&mut game, player, "Ruby, Daring Tracker", Zone::Hand);
    let strong = put(&mut game, player, "Shivan Dragon", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    cast(&mut game, ruby, None);
    settle(&mut game);
    assert!(game.object_view(ruby).keywords.contains(Keywords::HASTE));
    for (ability, color) in [(0, 3), (1, 4)] {
        let mut mana_branch = game.clone();
        let mut production = [0; 6];
        production[color] = 1;
        mana_branch.state.players[player.index()].mana = [0; 6];
        act(
            &mut mana_branch,
            Action::ActivateMana {
                card: ruby,
                ability,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production,
            },
        );
        assert_eq!(mana_branch.state.players[player.index()].mana, production);
        assert!(mana_branch.object_view(ruby).tapped);
    }
    let mut short = game.clone();
    cast(&mut short, bounce, Some(Target::Card(strong)));
    settle(&mut short);
    reach_attackers(&mut short);
    attack(&mut short, ruby);
    assert!(short.state.stack.is_empty());
    assert_eq!(stats(&short, ruby), (1, 2));

    reach_attackers(&mut game);
    attack(&mut game, ruby);
    act(&mut game, Action::TapMana { card: island });
    cast(&mut game, bounce, Some(Target::Card(strong)));
    settle(&mut game);
    assert_eq!(stats(&game, ruby), (3, 4));
    finish_turn(&mut game);
    assert_eq!(stats(&game, ruby), (1, 2));
}
