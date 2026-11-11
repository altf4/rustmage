//! Paid outcome proofs for the inherited Wilds definitions from T through Z.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FOOD: Card = Card::from_catalogue_id(5246);
const WICKED: Card = Card::from_catalogue_id(5248);
const RAT: Card = Card::from_catalogue_id(5323);

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance(game: &mut Game, reached: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if reached(game) {
            return;
        }
        let action = match game.state.pending {
            Pending::Priority => Action::Pass,
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            _ => panic!("unexpected pending state {:?}", game.state.pending),
        };
        act(game, action);
    }
    panic!("the expected turn or step did not arrive");
}

fn later_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    advance(game, |game| {
        game.state.turn > turn && game.state.active == player && game.state.step == Step::Main1
    });
}

fn scry(game: &mut Game, player: Player, count: usize) {
    let view = game.observe(player).library_choice.unwrap();
    assert_eq!(view.remaining.len(), count);
    assert!(!view.surveil);
    assert!(game.observe(player.other()).library_choice.is_none());
    for object in view.remaining {
        act(
            game,
            Action::ArrangeLibrary {
                card: object.id,
                destination: LibraryDestination::Top,
            },
        );
    }
}

fn tokens(game: &Game, player: Player, card: Card) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.card == card
        })
        .collect()
}

fn adventure(game: &mut Game, card: CardId, recipient: Option<CardId>) {
    act(game, Action::AnnounceAdventure { card });
    if let Some(recipient) = recipient {
        act(game, target(recipient));
    }
    pay(game);
}

fn from_adventure(game: &mut Game, card: CardId) {
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    act(game, Action::AnnounceExile { card });
    pay(game);
}

fn bargain(game: &mut Game, card: CardId, payment: Option<CardId>) {
    act(game, Action::AnnounceCast { card });
    let sacrifice = payment.map(|card| game.object_ref(card));
    act(game, Action::ChooseBargain { sacrifice });
}

fn activation(game: &mut Game, card: CardId, slot: u16) {
    choose(game, |action| match action {
        Action::Activate {
            card: source,
            ability,
            ..
        }
        | Action::AnnounceActivate {
            card: source,
            ability,
            ..
        } => *source == card && *ability == slot,
        _ => false,
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn settle_ordered(game: &mut Game) {
    for _ in 0..36 {
        match game.state.pending {
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            Pending::Priority if game.state.stack.is_empty() => return,
            Pending::Priority => act(game, Action::Pass),
            _ => panic!("unexpected pending state {:?}", game.state.pending),
        }
    }
    panic!("the ordered triggers did not resolve");
}

#[test]
fn taken_by_nightmares_checks_the_enchantment_when_it_resolves() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let enchantment = put(&mut game, player, "Virtue of Strength", Zone::Battlefield);
    let taken = put(&mut game, player, "Taken by Nightmares", Zone::Hand);
    let disenchant = put(&mut game, opponent, "Disenchant", Zone::Hand);
    cast(&mut game, taken, Some(Target::Card(victim)));

    let mut stripped = game.clone();
    act(&mut stripped, Action::Pass);
    cast(&mut stripped, disenchant, Some(Target::Card(enchantment)));
    settle(&mut stripped);
    assert_eq!(stripped.object_view(enchantment).zone, Zone::Graveyard);
    assert_eq!(stripped.object_view(victim).zone, Zone::Exile);

    pass_pair(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    scry(&mut game, player, 2);
    settle(&mut game);
}

#[test]
fn water_wings_sets_base_stats_and_titanic_growth_adds_to_either_creature() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let friend = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let wings = put(&mut game, player, "Water Wings", Zone::Hand);
    let growth = put(&mut game, player, "Titanic Growth", Zone::Hand);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    game.add_counter(friend, CounterKind::PlusOne, 1);
    act(&mut game, Action::AnnounceCast { card: wings });
    reject(&mut game, target(enemy));
    act(&mut game, target(friend));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, friend), (5, 5));
    let keywords = Keywords::FLYING.union(Keywords::HEXPROOF);
    assert!(game.object_view(friend).keywords.contains(keywords));

    let mut opposition = game.clone();
    act(&mut opposition, Action::Pass);
    act(&mut opposition, Action::AnnounceCast { card: shock });
    reject(&mut opposition, target(friend));

    cast(&mut game, growth, Some(Target::Card(enemy)));
    settle(&mut game);
    assert_eq!(stats(&game, enemy), (6, 6));
    later_main(&mut game, opponent);
    assert_eq!(stats(&game, friend), (3, 3));
    assert_eq!(stats(&game, enemy), (2, 2));
    for keyword in [Keywords::FLYING, Keywords::HEXPROOF] {
        assert!(!game.object_view(friend).keywords.contains(keyword));
    }
}

#[test]
fn tempest_hart_loots_two_and_grows_only_for_a_committed_expensive_cast() {
    let player = Player::P1;
    let mut game = scenario(player);
    let hart = put(&mut game, player, "Tempest Hart", Zone::Hand);
    let wurm = put(&mut game, player, "Beanstalk Wurm", Zone::Hand);
    let old = put(&mut game, player, "Opt", Zone::Hand);
    let first = put(&mut game, player, "Plains", Zone::Library);
    let second = put(&mut game, player, "Island", Zone::Library);
    adventure(&mut game, hart, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert!(game.observe(player.other()).actions.is_empty());
    act(&mut game, Action::Discard { card: old });
    act(&mut game, Action::Discard { card: second });
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    for discarded in [old, second] {
        assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    }

    from_adventure(&mut game, hart);
    settle(&mut game);
    assert_eq!(stats(&game, hart), (3, 4));
    assert!(game.object_view(hart).keywords.contains(Keywords::TRAMPLE));
    adventure(&mut game, wurm, None);
    settle(&mut game);
    assert_eq!(game.counter_count(hart, CounterKind::PlusOne), 0);
    from_adventure(&mut game, wurm);
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(game.counter_count(hart, CounterKind::PlusOne), 1);
}

#[test]
fn tomeseeker_and_ouphe_require_bargain_and_their_distinct_target_filters() {
    let player = Player::P0;
    let opponent = player.other();
    for (name, legal_name, legal_zone, legal_owner, destination) in [
        (
            "Tenacious Tomeseeker",
            "Opt",
            Zone::Graveyard,
            player,
            Zone::Hand,
        ),
        (
            "Troublemaker Ouphe",
            "Candy Trail",
            Zone::Battlefield,
            opponent,
            Zone::Exile,
        ),
    ] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, name, Zone::Hand);
        let payment = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let valid = put(&mut game, legal_owner, legal_name, legal_zone);
        let wrong_owner = put(&mut game, legal_owner.other(), legal_name, legal_zone);
        let wrong_kind = put(&mut game, legal_owner, "Beanstalk Wurm", legal_zone);
        let nontoken = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let mut declined = game.clone();
        bargain(&mut declined, creature, None);
        pay(&mut declined);
        settle(&mut declined);
        assert_eq!(declined.object_view(valid).zone, legal_zone);
        assert_eq!(declined.object_view(payment).zone, Zone::Battlefield);

        act(&mut game, Action::AnnounceCast { card: creature });
        let illegal = game.object_ref(nontoken);
        reject(
            &mut game,
            Action::ChooseBargain {
                sacrifice: Some(illegal),
            },
        );
        let sacrifice = game.object_ref(payment);
        act(
            &mut game,
            Action::ChooseBargain {
                sacrifice: Some(sacrifice),
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(payment).zone, Zone::Graveyard);
        pass_pair(&mut game);
        reject(&mut game, target(wrong_owner));
        reject(&mut game, target(wrong_kind));
        act(&mut game, target(valid));
        settle(&mut game);
        assert_eq!(game.object_view(valid).zone, destination);
    }
}

#[test]
fn huntsmans_redemption_creates_the_beast_searches_after_payment_and_boosts_two_targets() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let saga = put(&mut game, player, "The Huntsman's Redemption", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let creature = put(&mut game, player, "Hill Giant", Zone::Library);
    let land = put(&mut game, player, "Plains", Zone::Library);
    let illegal = put(&mut game, player, "Opt", Zone::Library);
    cast(&mut game, saga, None);
    settle(&mut game);
    let beast = tokens(&game, player, Card::from_catalogue_id(5271))
        .pop()
        .unwrap();
    assert_eq!((beast.power, beast.toughness, beast.colors), (3, 3, 16));
    assert!(beast.has_subtype("Beast"));

    later_main(&mut game, player);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    settle(&mut declined);
    assert_eq!(declined.object_view(beast.id).zone, Zone::Battlefield);

    let sacrifice = game.object_ref(beast.id);
    act(&mut game, Action::Sacrifice { object: sacrifice });
    assert_ne!(game.object_view(beast.id).zone, Zone::Battlefield);
    assert!(game.observe(opponent).search_choice.is_none());
    assert!(game.observe(player).search_choice.unwrap().reveal);
    reject(&mut game, Action::ChooseSearch { card: illegal });
    for found in [creature, land] {
        let mut branch = game.clone();
        let seed = branch.state.rng;
        act(&mut branch, Action::ChooseSearch { card: found });
        act(&mut branch, Action::FinishSearch);
        settle(&mut branch);
        assert_eq!(branch.object_view(found).zone, Zone::Hand);
        assert_ne!(branch.state.rng, seed);
    }
    act(&mut game, Action::FinishSearch);
    settle(&mut game);

    later_main(&mut game, player);
    act(&mut game, target(own));
    act(&mut game, target(enemy));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    for recipient in [own, enemy] {
        assert_eq!(stats(&game, recipient), (4, 4));
        assert!(
            game.object_view(recipient)
                .keywords
                .contains(Keywords::TRAMPLE)
        );
    }
    later_main(&mut game, opponent);
    assert_eq!(stats(&game, own), (2, 2));
    assert_eq!(stats(&game, enemy), (2, 2));
}

#[test]
fn vanity_and_sweettooth_finish_their_chapters_with_exact_food_and_role_recipients() {
    let player = Player::P1;
    let opponent = player.other();
    let mut vanity = scenario(player);
    let saga = put(&mut vanity, player, "The Witch's Vanity", Zone::Hand);
    let own = put(&mut vanity, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut vanity, opponent, "Bear Cub", Zone::Battlefield);
    let large = put(&mut vanity, opponent, "Hill Giant", Zone::Battlefield);
    cast(&mut vanity, saga, None);
    pass_pair(&mut vanity);
    reject(&mut vanity, target(own));
    reject(&mut vanity, target(large));
    act(&mut vanity, target(enemy));
    settle(&mut vanity);
    assert_eq!(vanity.object_view(enemy).zone, Zone::Graveyard);
    later_main(&mut vanity, player);
    settle(&mut vanity);
    assert_eq!(tokens(&vanity, player, FOOD).len(), 1);
    later_main(&mut vanity, player);
    reject(&mut vanity, target(large));
    act(&mut vanity, target(own));
    settle(&mut vanity);
    let roles = tokens(&vanity, player, WICKED);
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].attached_to, Some(vanity.object_ref(own)));
    assert_eq!(vanity.object_view(saga).zone, Zone::Graveyard);

    let mut sweet = scenario(player);
    let saga = put(&mut sweet, player, "Welcome to Sweettooth", Zone::Hand);
    let enemy = put(&mut sweet, opponent, "Bear Cub", Zone::Battlefield);
    spawn(&mut sweet, opponent, FOOD, Zone::Battlefield);
    spawn(&mut sweet, player, FOOD, Zone::Battlefield);
    cast(&mut sweet, saga, None);
    settle(&mut sweet);
    let human = tokens(&sweet, player, Card::from_catalogue_id(5254))
        .pop()
        .unwrap();
    assert!(human.has_subtype("Human"));
    assert_eq!((human.power, human.toughness, human.colors), (1, 1, 1));
    later_main(&mut sweet, player);
    settle(&mut sweet);
    assert_eq!(tokens(&sweet, player, FOOD).len(), 2);
    later_main(&mut sweet, player);
    reject(&mut sweet, target(enemy));
    act(&mut sweet, target(human.id));
    settle(&mut sweet);
    assert_eq!(sweet.counter_count(human.id, CounterKind::PlusOne), 3);
    assert_eq!(sweet.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn toadstool_admirer_pays_for_growth_and_collects_ward_from_the_caster() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let admirer = put(&mut game, player, "Toadstool Admirer", Zone::Hand);
    let murder = put(&mut game, opponent, "Murder", Zone::Hand);
    cast(&mut game, admirer, None);
    settle(&mut game);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 3];
    activation(&mut game, admirer, 0);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(stats(&game, admirer), (2, 2));
    act(&mut game, Action::Pass);
    cast(&mut game, murder, Some(Target::Card(admirer)));
    pass_pair(&mut game);

    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.object_view(murder).zone, Zone::Graveyard);
    assert_eq!(decline.object_view(admirer).zone, Zone::Battlefield);

    assert_eq!(game.actor(), Some(opponent));
    let before: u32 = game.state.players[opponent.index()].mana.iter().sum();
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let after: u32 = game.state.players[opponent.index()].mana.iter().sum();
    assert_eq!(before - after, 2);
    settle(&mut game);
    assert_eq!(game.object_view(admirer).zone, Zone::Graveyard);
}

#[test]
fn torch_the_tower_changes_damage_and_scries_with_bargain_and_exiles_later_deaths() {
    let player = Player::P0;
    let opponent = player.other();
    for bargained in [false, true] {
        let mut game = scenario(player);
        let torch = put(&mut game, player, "Torch the Tower", Zone::Hand);
        let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let victim = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let murder = put(&mut game, player, "Murder", Zone::Hand);
        bargain(&mut game, torch, bargained.then_some(offering));
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        act(&mut game, target(victim));
        pay(&mut game);
        pass_pair(&mut game);
        assert_eq!(
            game.object_view(victim).damage,
            if bargained { 3 } else { 2 }
        );
        if bargained {
            scry(&mut game, player, 1);
        }
        settle(&mut game);
        cast(&mut game, murder, Some(Target::Card(victim)));
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Exile);
    }
}

#[test]
fn witchstalker_frenzy_counts_declared_attackers_that_left_combat() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attackers = [0; 2].map(|_| put(&mut game, player, "Bear Cub", Zone::Battlefield));
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    let generic = put(&mut game, player, "Forest", Zone::Battlefield);
    let giant = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let frenzy = put(&mut game, player, "Witchstalker Frenzy", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, Action::AnnounceCast { card: frenzy });
    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    for card in attackers {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::TapMana { card: island });
    cast(&mut game, bounce, Some(Target::Card(attackers[0])));
    settle(&mut game);
    assert_eq!(game.object_view(attackers[0]).zone, Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: frenzy });
    act(&mut game, target(giant));
    for card in [mountain, generic] {
        act(&mut game, Action::TapMana { card });
    }
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 1, 0],
        },
    );
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 4);
    settle(&mut game);
    assert_eq!(game.object_view(giant).damage, 5);
}

#[test]
fn tuinvale_guide_keeps_celebration_after_an_entrant_leaves_and_deals_lifelink_damage() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let guide = put(&mut game, player, "Tuinvale Guide", Zone::Battlefield);
    let artifact = put(&mut game, player, "Short Bow", Zone::Hand);
    let entrant = put(&mut game, player, "Bear Cub", Zone::Hand);
    let destroy = put(&mut game, player, "Disenchant", Zone::Hand);
    assert_eq!(stats(&game, guide), (2, 3));
    assert!(game.object_view(guide).keywords.contains(Keywords::FLYING));
    cast(&mut game, artifact, None);
    settle(&mut game);
    assert_eq!(stats(&game, guide), (2, 3));
    assert!(
        !game
            .object_view(guide)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    cast(&mut game, entrant, None);
    settle(&mut game);
    cast(&mut game, destroy, Some(Target::Card(artifact)));
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(stats(&game, guide), (3, 3));
    assert!(
        game.object_view(guide)
            .keywords
            .contains(Keywords::LIFELINK)
    );

    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: guide });
    act(&mut game, Action::FinishAttackers);
    advance(&mut game, |game| game.state.step == Step::EndCombat);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    later_main(&mut game, opponent);
    assert_eq!(stats(&game, guide), (2, 3));
    assert!(
        !game
            .object_view(guide)
            .keywords
            .contains(Keywords::LIFELINK)
    );
}

#[test]
fn sewer_witch_roles_vermin_and_tabby_count_their_distinct_departures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let witch = put(&mut game, player, "Twisted Sewer-Witch", Zone::Hand);
    let vermin = put(&mut game, player, "Voracious Vermin", Zone::Hand);
    let tabby = put(&mut game, player, "Warehouse Tabby", Zone::Battlefield);
    let visitor = put(&mut game, player, "Wicked Visitor", Zone::Battlefield);
    let mark = put(&mut game, player, "Witch's Mark", Zone::Hand);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let enemy = spawn(&mut game, opponent, RAT, Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, vermin, None);
    settle(&mut game);
    let rats = tokens(&game, player, RAT);
    assert_eq!(rats.len(), 1);
    assert_eq!(
        (rats[0].power, rats[0].toughness, rats[0].colors),
        (1, 1, 4)
    );
    assert!(rats[0].keywords.contains(Keywords::CANT_BLOCK));
    cast(&mut game, witch, None);
    settle(&mut game);
    let rats = tokens(&game, player, RAT);
    assert_eq!(rats.len(), 2);
    let recipients: Vec<_> = [game.object_ref(vermin)]
        .into_iter()
        .chain(rats.iter().map(|rat| rat.reference))
        .collect();
    let roles = tokens(&game, player, WICKED);
    assert_eq!(roles.len(), recipients.len());
    assert!(recipients.iter().all(|recipient| {
        roles
            .iter()
            .any(|role| role.attached_to == Some(*recipient))
    }));
    assert_eq!(stats(&game, enemy), (1, 1));
    assert_eq!(stats(&game, visitor), (2, 2));

    activation(&mut game, tabby, 0);
    settle(&mut game);
    assert!(
        game.object_view(tabby)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    cast(&mut game, murder, Some(Target::Card(own)));
    settle(&mut game);
    assert_eq!(game.counter_count(vermin, CounterKind::PlusOne), 1);

    act(&mut game, Action::AnnounceCast { card: mark });
    reject(&mut game, target(enemy));
    act(&mut game, target(vermin));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    pass_pair(&mut game);
    if game.legal_actions().contains(&Action::DeclineEffectPayment) {
        act(&mut game, Action::DeclineEffectPayment);
    }
    settle_ordered(&mut game);
    assert_eq!(tokens(&game, player, WICKED).len(), 3);
    assert_eq!(tokens(&game, player, RAT).len(), 3);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.counter_count(vermin, CounterKind::PlusOne), 1);
    later_main(&mut game, opponent);
    assert!(
        !game
            .object_view(tabby)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
}

#[test]
fn witchs_mark_can_loot_without_a_target_but_fizzles_with_a_lost_chosen_target() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let mark = put(&mut game, player, "Witch's Mark", Zone::Hand);
    let discard = put(&mut game, player, "Opt", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    let mut stale = game.clone();
    announce(&mut stale, mark, &[Target::Card(bear)]);
    act(&mut stale, Action::Pass);
    cast(&mut stale, bounce, Some(Target::Card(bear)));
    settle(&mut stale);
    assert_eq!(stale.object_view(discard).zone, Zone::Hand);
    assert_eq!(stale.state.players[player.index()].cards_drawn_this_turn, 0);
    assert!(tokens(&stale, player, WICKED).is_empty());

    act(&mut game, Action::AnnounceCast { card: mark });
    act(&mut game, Action::FinishTargets);
    pay(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(opponent).actions.is_empty());
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert!(tokens(&game, player, WICKED).is_empty());
}

#[test]
fn two_headed_hunter_grants_two_damage_steps_and_then_enters_with_menace() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let hunter = put(&mut game, player, "Two-Headed Hunter", Zone::Hand);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    adventure(&mut game, hunter, Some(bear));
    settle(&mut game);
    assert!(
        game.object_view(bear)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );
    from_adventure(&mut game, hunter);
    settle(&mut game);
    assert!(game.object_view(hunter).keywords.contains(Keywords::MENACE));
    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: bear });
    act(&mut game, Action::FinishAttackers);
    advance(&mut game, |game| game.state.step == Step::EndCombat);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    later_main(&mut game, opponent);
    assert!(
        !game
            .object_view(bear)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );
}

#[test]
fn threadbind_requires_a_tapped_creature_and_its_normal_face_flies() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let clique = put(&mut game, player, "Threadbind Clique", Zone::Hand);
    let tapped = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let ready = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    act(&mut game, Action::AnnounceAdventure { card: clique });
    reject(&mut game, target(ready));
    act(&mut game, target(tapped));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(tapped).zone, Zone::Graveyard);
    from_adventure(&mut game, clique);
    settle(&mut game);
    assert!(game.object_view(clique).keywords.contains(Keywords::FLYING));
}

#[test]
fn unassuming_sage_pays_for_a_nontargeted_role_that_scries_on_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sage = put(&mut game, player, "Unassuming Sage", Zone::Hand);
    let sorcerer = Card::from_catalogue_id(5321);
    cast(&mut game, sage, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(stats(&declined, sage), (2, 2));
    assert!(tokens(&declined, player, sorcerer).is_empty());

    let before: u32 = game.state.players[player.index()].mana.iter().sum();
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let after: u32 = game.state.players[player.index()].mana.iter().sum();
    assert_eq!(before - after, 2);
    settle(&mut game);
    let roles = tokens(&game, player, sorcerer);
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].attached_to, Some(game.object_ref(sage)));
    assert_eq!(stats(&game, sage), (3, 3));
    later_main(&mut game, player);
    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: sage });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    scry(&mut game, player, 1);
}

#[test]
fn unruly_catapult_pings_and_only_its_controllers_instant_or_sorcery_untaps_it() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let catapult = put(&mut game, player, "Unruly Catapult", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    let own = put(&mut game, player, "Shock", Zone::Hand);
    let foreign = put(&mut game, opponent, "Shock", Zone::Hand);
    assert!(
        game.object_view(catapult)
            .keywords
            .contains(Keywords::DEFENDER)
    );
    activation(&mut game, catapult, 0);
    settle(&mut game);
    assert!(game.object_view(catapult).tapped);
    assert_eq!(game.state.players[opponent.index()].life, 19);

    cast(&mut game, creature, None);
    settle(&mut game);
    assert!(game.object_view(catapult).tapped);
    act(&mut game, Action::Pass);
    cast(&mut game, foreign, Some(Target::Player(player)));
    settle(&mut game);
    assert!(game.object_view(catapult).tapped);

    cast(&mut game, own, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Stack);
    assert!(!game.object_view(catapult).tapped);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 17);
}

#[test]
fn croaking_curse_changes_base_power_on_a_tapped_creature_and_releases_transmuter() {
    let player = Player::P0;
    let mut game = scenario(player);
    let transmuter = put(&mut game, player, "Vantress Transmuter", Zone::Hand);
    let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(giant, CounterKind::PlusOne, 1);
    game.state.objects[giant as usize].tapped = true;
    adventure(&mut game, transmuter, Some(giant));
    settle(&mut game);
    assert!(game.object_view(giant).tapped);
    assert_eq!(stats(&game, giant), (2, 2));
    assert!(
        game.observe(player)
            .objects
            .iter()
            .any(|object| object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype("Role")
                && object.attached_to == Some(game.object_ref(giant)))
    );
    from_adventure(&mut game, transmuter);
    settle(&mut game);
    assert_eq!(stats(&game, transmuter), (3, 4));
}

#[test]
fn verdant_outrider_pays_to_prevent_low_power_blockers() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let outrider = put(&mut game, player, "Verdant Outrider", Zone::Battlefield);
    let small = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let large = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let cost: u32 = game.state.players[player.index()].mana.iter().sum();
    activation(&mut game, outrider, 0);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].mana.iter().sum::<u32>(),
        cost - 2
    );
    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    act(&mut game, Action::Attack { card: outrider });
    act(&mut game, Action::FinishAttackers);
    advance(&mut game, |game| game.state.pending == Pending::Blockers);
    reject(
        &mut game,
        Action::Block {
            blocker: small,
            attacker: outrider,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: large,
            attacker: outrider,
        },
    );
    act(&mut game, Action::FinishBlockers);
}

#[test]
fn woodland_acolyte_draws_the_exact_permanent_placed_on_top_by_its_adventure() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let acolyte = put(&mut game, player, "Woodland Acolyte", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let instant = put(&mut game, player, "Opt", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Forest", Zone::Graveyard);
    act(&mut game, Action::AnnounceAdventure { card: acolyte });
    reject(&mut game, target(instant));
    reject(&mut game, target(foreign));
    act(&mut game, target(land));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.top_library_card(player), Some(land));
    from_adventure(&mut game, acolyte);
    settle(&mut game);
    assert_eq!(game.object_view(acolyte).zone, Zone::Battlefield);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn tanglespan_lookout_draws_for_an_aura_its_controller_creates_or_casts() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let lookout = put(&mut game, player, "Tanglespan Lookout", Zone::Battlefield);
    let target = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, player, "Bestial Bloodline", Zone::Hand);
    let foreign = put(&mut game, opponent, "Bestial Bloodline", Zone::Hand);
    let artifact = put(&mut game, player, "Short Bow", Zone::Hand);
    let mark = put(&mut game, player, "Witch's Mark", Zone::Hand);
    cast(&mut game, artifact, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    cast(&mut game, aura, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.state.active = opponent;
    game.state.priority = opponent;
    cast(&mut game, foreign, Some(Target::Card(lookout)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    game.state.active = player;
    game.state.priority = player;
    announce(&mut game, mark, &[Target::Card(lookout)]);
    pass_pair(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}
