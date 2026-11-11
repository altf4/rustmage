use super::*;

#[test]
fn metamorphic_blast_sets_base_stats_type_and_color_while_preserving_abilities_and_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let target = board(&mut game, player, "Gingerbrute");
    game.add_counter(target, CounterKind::PlusOne, 1);
    let spell = hand(&mut game, player, "Metamorphic Blast");
    modal(
        &mut game,
        spell,
        2,
        &[Target::Card(target), Target::Player(player)],
    );
    quiet(&mut game);
    let rabbit = game.object_view(target);
    assert_eq!((rabbit.power, rabbit.toughness), (1, 2));
    assert_eq!(rabbit.colors, 1);
    assert!(rabbit.has_subtype("Rabbit"));
    assert!(!rabbit.has_subtype("Golem"));
    assert!(rabbit.keywords.contains(Keywords::HASTE));
    assert_eq!(rabbit.kinds & 17, 17);
    assert_eq!(rabbit.counters.plus_one, 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);

    reach(&mut game, |g| {
        g.state.active == player.other() && g.state.step == Step::Main1
    });
    assert_eq!(stats(&game, target), (2, 2));
    assert_eq!(game.object_view(target).colors, 0);
    assert!(game.object_view(target).has_subtype("Golem"));
}

#[test]
fn neutralize_the_guards_shrinks_only_the_opponents_current_creatures_and_surveils() {
    let player = Player::P1;
    let mut game = scenario(player);
    let own = board(&mut game, player, "Bear Cub");
    let small = board(&mut game, player.other(), "Bear Cub");
    let bigger = board(&mut game, player.other(), "Grizzly Bears");
    let buried = top(&mut game, player, "Forest");
    let kept = top(&mut game, player, "Island");
    let spell = hand(&mut game, player, "Neutralize the Guards");
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    bind_targets(&mut game, &[Target::Player(player.other())]);
    pay(&mut game);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectLibrary { .. })
    });
    arrange(&mut game, kept, LibraryDestination::Top);
    arrange(&mut game, buried, LibraryDestination::Graveyard);
    quiet(&mut game);
    assert_eq!(zone(&game, small), Zone::Graveyard);
    assert_eq!(zone(&game, buried), Zone::Graveyard);
    assert_eq!(stats(&game, own), (1, 1));
    assert_eq!(stats(&game, bigger), (1, 1));
}

#[test]
fn quick_draw_uses_both_target_roles_and_late_entrants_keep_their_abilities() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = board(&mut game, player, "Grizzly Bears");
    let strike = board(&mut game, player.other(), "Reckless Lackey");
    let double = board(&mut game, player.other(), "Swiftblade Vindicator");
    let spell = hand(&mut game, player, "Quick Draw");
    announce(
        &mut game,
        spell,
        &[Target::Card(creature), Target::Player(player.other())],
    );
    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    let bounce = hand(&mut stale, player.other(), "Unsummon");
    cast(&mut stale, bounce, Some(Target::Card(creature)));
    quiet(&mut stale);
    assert_eq!(zone(&stale, creature), Zone::Hand);
    for branch in [&mut game, &mut stale] {
        quiet(branch);
        assert!(!keywords(branch, strike).contains(Keywords::FIRST_STRIKE));
        assert!(!keywords(branch, double).contains(Keywords::DOUBLE_STRIKE));
        assert!(keywords(branch, double).contains(Keywords::TRAMPLE.union(Keywords::VIGILANCE)));
        let late = board(branch, player.other(), "Swiftblade Vindicator");
        assert!(keywords(branch, late).contains(Keywords::DOUBLE_STRIKE));
    }
    assert_eq!(stats(&game, creature), (3, 3));
    assert!(keywords(&game, creature).contains(Keywords::FIRST_STRIKE));
}

#[test]
fn phantom_interference_combines_spirit_creation_and_a_payable_counter() {
    let active = Player::P0;
    let player = active.other();
    let mut game = scenario(active);
    let shock = hand(&mut game, active, "Shock");
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    let phantom = hand(&mut game, player, "Phantom Interference");
    modal(&mut game, phantom, 2, &[Target::Spell(shock)]);

    let mut stale = game.clone();
    let negate = hand(&mut stale, player, "Negate");
    cast(&mut stale, negate, Some(Target::Spell(shock)));
    quiet(&mut stale);
    assert!(tokens(&stale, player, SPIRIT).is_empty());
    until_action(&mut game, &Action::DeclineResolutionMana);
    let spirit = tokens(&game, player, SPIRIT)[0];
    assert_eq!(stats(&game, spirit), (2, 2));
    assert_eq!(game.object_view(spirit).colors, 1);
    assert!(keywords(&game, spirit).contains(Keywords::FLYING));

    let mut paid = game.clone();
    choose(&mut paid, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    quiet(&mut paid);
    assert_eq!(life(&paid, player), 18);
    act(&mut game, Action::DeclineResolutionMana);
    quiet(&mut game);
    assert_eq!(life(&game, player), 20);
}

#[test]
fn one_last_job_attaches_aura_to_the_creature_its_earlier_mode_returned() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = graveyard(&mut game, player, "Grizzly Bears");
    let vehicle = graveyard(&mut game, player, "Mobile Homestead");
    let aura = graveyard(&mut game, player, "Pacifism");
    let equipment = graveyard(&mut game, player, "Lavaspur Boots");
    let foreign = board(&mut game, player.other(), "Grizzly Bears");
    let job = hand(&mut game, player, "One Last Job");

    for (card, zone_after) in [(aura, Zone::Graveyard), (equipment, Zone::Battlefield)] {
        let mut branch = game.clone();
        modal(&mut branch, job, 3, &[Target::Card(card)]);
        quiet(&mut branch);
        assert_eq!(zone(&branch, card), zone_after);
        assert!(branch.object_view(card).attached_to.is_none());
    }
    modal(
        &mut game,
        job,
        6,
        &[
            Target::Card(creature),
            Target::Card(vehicle),
            Target::Card(aura),
        ],
    );
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(zone(&game, creature), Zone::Battlefield);
    assert_eq!(zone(&game, vehicle), Zone::Battlefield);
    let exact = game.object_ref(creature);
    let wrong = game.object_ref(foreign);
    reject(&mut game, Action::ChooseEffectObject { object: wrong });
    act(&mut game, Action::ChooseEffectObject { object: exact });
    quiet(&mut game);
    assert_eq!(zone(&game, aura), Zone::Battlefield);
    assert_eq!(game.object_view(aura).attached_to, Some(exact));
}

#[test]
fn return_the_favor_copies_one_spell_and_changes_the_other_spells_single_target() {
    let active = Player::P0;
    let player = active.other();
    let mut game = scenario(active);
    let first = hand(&mut game, active, "Shock");
    let second = hand(&mut game, active, "Shock");
    cast(&mut game, first, Some(Target::Player(player)));
    cast(&mut game, second, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    let favor = hand(&mut game, player, "Return the Favor");
    modal(
        &mut game,
        favor,
        2,
        &[Target::Spell(first), Target::Spell(second)],
    );
    let new_target = Action::SetStackTarget {
        index: 0,
        target: Target::Player(active),
    };
    until_action(&mut game, &new_target);
    assert!(game.observe(player).retarget.unwrap().item.is_copy);
    act(&mut game, new_target.clone());
    act(&mut game, Action::FinishStackTargets);
    until_action(&mut game, &new_target);
    assert!(!game.observe(player).retarget.unwrap().item.is_copy);
    reject(&mut game, Action::FinishStackTargets);
    act(&mut game, new_target);
    act(&mut game, Action::FinishStackTargets);
    quiet(&mut game);
    assert_eq!(life(&game, active), 16);
    assert_eq!(life(&game, player), 18);
}

#[test]
fn riku_counts_selected_modes_including_repeated_and_zero_selections() {
    let player = Player::P1;
    let cases: &[(&str, u16, &[u16], u16)] = &[
        ("Call Damage Control", 0, &[0], 0),
        ("Metamorphic Blast", 1, &[0, 1, 2, 4], 4),
        ("Lively Dirge", 2, &[0, 1, 2, 3, 4, 5, 6], 6),
        ("Cosmium Confluence", 0, &[0, 1, 2, 3, 4, 5, 6, 7], 7),
    ];
    for &(name, original_mode, expected, chosen) in cases {
        let mut game = scenario(player);
        let riku = board(&mut game, player, "Riku of Many Paths");
        let forest = top(&mut game, player, "Forest");
        let spell = hand(&mut game, player, name);
        let targets = if name == "Metamorphic Blast" {
            vec![Target::Player(player)]
        } else {
            Vec::new()
        };
        modal(&mut game, spell, original_mode, &targets);
        until_action(&mut game, &Action::ChooseMode { mode: 0 });
        let actual: Vec<_> = game
            .legal_actions()
            .into_iter()
            .filter_map(|action| match action {
                Action::ChooseMode { mode } => Some(mode),
                _ => None,
            })
            .collect();
        assert_eq!(actual, expected, "{name}");
        act(&mut game, Action::ChooseMode { mode: chosen });
        reach(&mut game, |g| {
            g.state.pending == Pending::Priority && g.state.stack.len() == 1
        });
        assert_eq!(counters(&game, riku), u64::from(chosen & 2 != 0));
        assert_eq!(
            keywords(&game, riku).contains(Keywords::TRAMPLE),
            chosen & 2 != 0
        );
        let birds = tokens(&game, player, BLUE_BIRD);
        assert_eq!(birds.len(), usize::from(chosen & 4 != 0));
        if let Some(&bird) = birds.first() {
            assert_eq!(stats(&game, bird), (1, 1));
            assert_eq!(game.object_view(bird).colors, 2);
            assert!(game.object_view(bird).has_subtype("Bird"));
            assert!(keywords(&game, bird).contains(Keywords::FLYING));
        }
        assert_eq!(
            zone(&game, forest),
            if chosen & 1 != 0 {
                Zone::Exile
            } else {
                Zone::Library
            }
        );
    }
}

#[test]
fn rush_of_dread_resolves_its_three_modes_in_order_and_rounds_all_counts_up() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creatures: Vec<_> = (0..3)
        .map(|_| board(&mut game, opponent, "Grizzly Bears"))
        .collect();
    let cards: Vec<_> = (0..3)
        .map(|_| hand(&mut game, opponent, "Forest"))
        .collect();
    let noncreature = board(&mut game, opponent, "Swamp");
    game.state.players[opponent.index()].life = 21;
    let spell = hand(&mut game, player, "Rush of Dread");
    modal(&mut game, spell, 6, &[Target::Player(opponent); 3]);

    for &card in &creatures[..2] {
        let sacrifice = Action::Sacrifice {
            object: game.object_ref(card),
        };
        until_action(&mut game, &sacrifice);
        let wrong = game.object_ref(noncreature);
        reject(&mut game, Action::Sacrifice { object: wrong });
        act(&mut game, sacrifice);
    }
    assert_eq!(zone(&game, creatures[0]), Zone::Graveyard);
    assert_eq!(zone(&game, creatures[2]), Zone::Battlefield);
    for &card in &cards[..2] {
        assert!(game.observe(player).effect_choice.is_none());
        act(&mut game, Action::Discard { card });
    }
    quiet(&mut game);
    assert_eq!(zone(&game, cards[0]), Zone::Graveyard);
    assert_eq!(zone(&game, cards[2]), Zone::Hand);
    assert_eq!(life(&game, opponent), 10);
}
