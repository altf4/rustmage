use super::*;

fn transform(game: &mut Game, card: CardId, back: Card) {
    let object = game.object_ref(card);
    announce_ability(game, card, 0);
    pay(game);
    settle(game);
    assert_eq!(game.object_ref(card), object);
    assert_eq!(game.object_view(card).card, back);
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn choose_clause(game: &mut Game, want: impl Fn(CastCostClause) -> bool) {
    let clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && want(row.clause))
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
}

#[test]
fn miles_front_counters_then_back_camouflage_and_attack_doubling() {
    let player = Player::P0;
    let mut game = scenario(player);
    let iron = put(
        &mut game,
        player,
        "Iron Spider, Stark Upgrade",
        Zone::Battlefield,
    );
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let miles = put(
        &mut game,
        player,
        "Miles Morales // Ultimate Spider-Man",
        Zone::Hand,
    );
    cast(&mut game, miles, None);
    pass_pair(&mut game);
    choose_target(&mut game, Target::Card(iron));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(iron),
        },
    );
    choose_target(&mut game, Target::Card(enemy));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.counter_count(iron, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);
    game.add_counter(iron, CounterKind::Shield, 1);
    transform(&mut game, miles, Card(5541));
    let view = game.object_view(miles);
    assert!(
        view.keywords
            .contains(Keywords::FIRST_STRIKE.union(Keywords::HASTE))
    );
    assert_eq!(stats(&game, miles), (4, 3));
    announce_ability(&mut game, miles, 0);
    pay(&mut game);
    settle(&mut game);
    let view = game.object_view(miles);
    assert_eq!(view.colors, 0);
    assert!(view.keywords.contains(Keywords::HEXPROOF));
    assert_eq!(game.counter_count(miles, CounterKind::PlusOne), 1);

    declare(&mut game, &[miles]);
    settle(&mut game);
    assert_eq!(game.counter_count(miles, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(iron, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(iron, CounterKind::Shield), 2);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);
    advance_until(&mut game, |g| {
        g.state.active != player && g.state.step == Step::Main1
    });
    let view = game.object_view(miles);
    assert_eq!(view.colors, 1 | 8 | 16);
    assert!(!view.keywords.contains(Keywords::HEXPROOF));
}

#[test]
fn norman_connives_then_transforms_and_grants_a_discounted_mayhem_cast() {
    let player = Player::P0;
    let mut game = scenario(player);
    let norman = put(
        &mut game,
        player,
        "Norman Osborn // Green Goblin",
        Zone::Battlefield,
    );
    let discarded = put(&mut game, player, "Divination", Zone::Hand);
    let milled = put(&mut game, player, "Divination", Zone::Graveyard);
    assert!(
        game.object_view(norman)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    declare(&mut game, &[norman]);
    advance_until(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    act(&mut game, Action::Discard { card: discarded });
    settle(&mut game);
    assert_eq!(game.counter_count(norman, CounterKind::PlusOne), 1);
    assert_eq!(game.state.players[player.other().index()].life, 19);
    advance_until(&mut game, |g| g.state.step == Step::Main2);
    game.state.players[player.index()].mana = [0, 1, 1, 1, 0, 1];
    transform(&mut game, norman, Card(5542));
    let view = game.object_view(norman);
    assert!(
        view.keywords
            .contains(Keywords::FLYING.union(Keywords::MENACE))
    );
    assert_eq!(stats(&game, norman), (4, 4));

    let mayhem = |card| Action::AnnounceGrantedMayhem {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut game, mayhem(milled));
    act(&mut game, mayhem(discarded));
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::GrantedMayhem
    );
    settle(&mut game);
    assert_eq!(game.hand_count(player), 3);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn peter_creates_a_spider_and_amazing_spider_man_grants_the_paid_alternative() {
    let player = Player::P0;
    let mut game = scenario(player);
    let peter = put(
        &mut game,
        player,
        "Peter Parker // Amazing Spider-Man",
        Zone::Hand,
    );
    cast(&mut game, peter, None);
    settle(&mut game);
    let spiders = token_refs(&game, Card(5544), player);
    assert_eq!(spiders.len(), 1);
    let spider = spiders[0];
    assert_eq!(stats(&game, spider.card), (2, 1));
    assert_eq!(game.object_view(spider.card).colors, 16);
    assert!(
        game.object_view(spider.card)
            .keywords
            .contains(Keywords::REACH)
    );
    transform(&mut game, peter, Card(5543));
    assert!(
        game.object_view(peter)
            .keywords
            .contains(Keywords::VIGILANCE.union(Keywords::REACH))
    );
    assert_eq!(token_refs(&game, Card(5544), player), spiders);

    let returned = put(&mut game, player, "City Pigeon", Zone::Battlefield);
    game.state.objects[returned as usize].tapped = true;
    let object = game.object_ref(returned);
    let eligible = put(&mut game, player, "Kraven, Proud Predator", Zone::Hand);
    let colorless = put(&mut game, player, "Iron Spider, Stark Upgrade", Zone::Hand);
    let nonlegendary = put(&mut game, player, "Hill Giant", Zone::Hand);
    let web = |card| Action::AnnounceWebSlinging {
        card,
        face: CastFace::Normal,
        cost: ManaCost {
            colored: [1, 1, 0, 0, 1, 0],
            generic: 0,
        },
        flash_surcharge: false,
        origin: WebSlingingOrigin::Ordinary,
    };
    game.state.players[player.index()].mana = [1, 1, 0, 0, 1, 0];
    reject(&mut game, web(colorless));
    reject(&mut game, web(nonlegendary));
    act(&mut game, web(eligible));
    act(&mut game, Action::BeginOrderedCosts);
    choose_clause(&mut game, |clause| clause == CastCostClause::Mana);
    pay(&mut game);
    choose_clause(&mut game, |clause| {
        matches!(clause, CastCostClause::WebSlingingReturn(_))
    });
    act(&mut game, Action::PayWebSlingingReturn { object });
    assert_eq!(game.object_view(returned).zone, Zone::Hand);
    let receipt = game.state.stack.iter().find_map(|item| item.cast).unwrap();
    assert_eq!(receipt.method, CastMethod::WebSlinging);
    assert_eq!(receipt.web_slinging_return_mana_value, Some(1));
    settle(&mut game);
    assert_eq!(game.object_view(eligible).zone, Zone::Battlefield);
}

#[test]
fn all_three_modal_backs_can_be_cast_without_the_front_entry() {
    let player = Player::P1;
    for (name, back, mana) in [
        (
            "Miles Morales // Ultimate Spider-Man",
            Card(5541),
            [1, 0, 0, 1, 1, 3],
        ),
        (
            "Norman Osborn // Green Goblin",
            Card(5542),
            [0, 1, 1, 1, 0, 1],
        ),
        (
            "Peter Parker // Amazing Spider-Man",
            Card(5543),
            [1, 1, 0, 0, 1, 1],
        ),
    ] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = mana;
        let card = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::AnnounceBack { card });
        pay(&mut game);
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().face,
            CastFace::Back
        );
        settle(&mut game);
        assert_eq!(game.object_view(card).card, back, "{name}");
        assert_eq!(game.state.players[player.index()].mana, [0; 6], "{name}");
        assert!(token_refs(&game, Card(5544), player).is_empty());
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
    }
}

#[test]
fn origin_makes_a_spider_then_applies_permanent_types_and_temporary_double_strike() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let saga = put(&mut game, player, "Origin of Spider-Man", Zone::Hand);
    cast(&mut game, saga, None);
    settle(&mut game);
    assert_eq!(token_refs(&game, Card(5544), player).len(), 1);

    let turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    choose_target(&mut game, Target::Card(own));
    settle(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
    assert!(game.object_view(own).is_legendary());
    for subtype in ["Bear", "Spider", "Hero"] {
        assert!(game.has_subtype(own, subtype));
    }

    let turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.active == player && g.state.turn > turn && g.state.step == Step::Main1
    });
    choose_target(&mut game, Target::Card(own));
    settle(&mut game);
    assert!(
        game.object_view(own)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    advance_until(&mut game, |g| {
        g.state.active != player && g.state.step == Step::Main1
    });
    assert!(
        !game
            .object_view(own)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );
    assert!(game.object_view(own).is_legendary());
    assert!(game.has_subtype(own, "Spider"));
}

#[test]
fn passenger_ferry_crews_and_pays_blue_before_targeting_another_attacker() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ferry = put(&mut game, player, "Passenger Ferry", Zone::Battlefield);
    let crew = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    announce_ability(&mut game, ferry, 0);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    reject(&mut game, Action::FinishCrew);
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert!(game.object_view(crew).tapped);
    assert!(game.is_creature(ferry));
    assert_eq!(stats(&game, ferry), (4, 3));

    declare(&mut game, &[ferry, attacker]);
    let blue = [0, 1, 0, 0, 0, 0];
    game.state.players[player.index()].mana = blue;
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert!(
        !declined
            .object_view(attacker)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );

    act(&mut game, Action::PayResolutionMana { payment: blue });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ferry),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(crew),
        },
    );
    choose_target(&mut game, Target::Card(attacker));
    settle(&mut game);
    assert!(
        game.object_view(attacker)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
