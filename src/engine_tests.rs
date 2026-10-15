use super::*;

fn config(deck: Vec<Card>) -> Config {
    Config {
        decks: [deck.clone(), deck],
        seed: 42,
        starting_player: Player::P0,
        deck_rules: DeckRules::Freeform,
    }
}

fn fresh() -> Game {
    Game::new(config(vec![Card::Forest; 60])).unwrap()
}

pub(super) fn act(game: &mut Game, action: Action) {
    game.apply(game.actor().unwrap(), action).unwrap();
}
pub(super) fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

pub(super) fn spawn(game: &mut Game, p: Player, card: Card, zone: Zone) -> CardId {
    let id = game.state.objects.len() as CardId;
    game.state.objects.push(Object {
        card,
        owner: p,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        damage: 0,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: Counters::default(),
        used_abilities: 0,
    });
    if zone == Zone::Library {
        game.state.players[p.index()].library.push(id);
    }
    if zone == Zone::Battlefield {
        game.register_static_source(id);
    }
    id
}

pub(super) fn scenario() -> Game {
    let mut game = fresh();
    game.state.objects.clear();
    game.state.static_sources.clear();
    for player in &mut game.state.players {
        player.library.clear();
        player.kept = true;
    }
    for p in [Player::P0, Player::P1] {
        for _ in 0..20 {
            spawn(&mut game, p, Card::Forest, Zone::Library);
        }
    }
    game.state.pending = Pending::Priority;
    game.state.step = Step::Main1;
    game.state.turn = 3;
    game
}

pub(super) fn cast(game: &mut Game, card: CardId, target: Option<Target>) {
    let action = game
        .legal_actions()
        .into_iter()
        .find(
            |a| matches!(a, Action::Cast { card: c, target: t, .. } if *c == card && *t == target),
        )
        .expect("cast available");
    act(game, action);
}

fn assert_invariants(game: &Game) {
    let s = &game.state;
    for (i, obj) in s.objects.iter().enumerate() {
        let libraries = s
            .players
            .iter()
            .flat_map(|p| &p.library)
            .filter(|&&id| id == i as CardId)
            .count();
        assert_eq!(libraries, usize::from(obj.zone == Zone::Library));
        assert_eq!(
            s.stack
                .iter()
                .filter(|spell| spell.kind == StackKind::Spell && spell.card == i as CardId)
                .count()
                + usize::from(
                    s.announcing.as_ref().is_some_and(
                        |a| a.item.kind == StackKind::Spell && a.item.card == i as CardId
                    )
                )
                + usize::from(
                    s.resolving.as_ref().is_some_and(
                        |r| r.item.kind == StackKind::Spell && r.item.card == i as CardId
                    )
                ),
            usize::from(obj.zone == Zone::Stack)
        );
    }
    let mut stack_ids = std::collections::BTreeSet::new();
    for item in s
        .stack
        .iter()
        .chain(&s.pending_triggers)
        .chain(&s.ordering_triggers)
        .chain(s.resolving.iter().map(|r| &r.item))
        .chain(s.announcing.iter().map(|a| &a.item))
    {
        assert!(stack_ids.insert(item.id));
        assert!(item.id < s.next_stack_id);
        if item.kind == StackKind::Spell {
            assert_eq!(game.object_ref(item.card), item.object);
            assert_eq!(s.objects[item.card as usize].zone, Zone::Stack);
        }
    }
    for (i, player) in s.players.iter().enumerate() {
        assert!(
            player
                .library
                .iter()
                .all(|&id| s.objects[id as usize].owner.index() == i)
        );
    }
    if s.outcome.is_none() {
        assert!(!game.legal_actions().is_empty());
    }
}

#[test]
fn seed_is_local_and_clone_restores_future_randomness() {
    let deck = Card::ALL.into_iter().cycle().take(60).collect();
    let mut a = Game::new(config(deck)).unwrap();
    let mut b = a.clone();
    let checkpoint = a.snapshot();
    act(&mut a, Action::Mulligan);
    act(&mut a, Action::Mulligan);
    act(&mut b, Action::Mulligan);
    act(&mut b, Action::Mulligan);
    assert_eq!(a.snapshot(), b.snapshot());
    let expected = a.snapshot();
    a.restore(&checkpoint);
    act(&mut a, Action::Mulligan);
    act(&mut a, Action::Mulligan);
    assert_eq!(a.snapshot(), expected);
    // Unrelated games cannot change this state's RNG.
    let _ = Game::new(config(vec![Card::Mountain; 60])).unwrap();
    assert_eq!(a.snapshot(), b.snapshot());
}

#[test]
fn mulligan_round_waits_for_both_declarations_then_bottoms_before_next_round() {
    let mut game = fresh();
    let rng = game.state.rng;
    act(&mut game, Action::Mulligan);
    assert_eq!(game.state.rng, rng);
    assert_eq!(game.actor(), Some(Player::P1));
    act(&mut game, Action::Keep);
    assert_eq!(
        game.state.pending,
        Pending::Bottom {
            player: Player::P0,
            remaining: 1
        }
    );
    assert_eq!(game.hand_count(Player::P0), 7);
    let card = game.observe(Player::P0).objects[0].id;
    act(&mut game, Action::Bottom { card });
    assert_eq!(game.hand_count(Player::P0), 6);
    assert_eq!(game.state.players[0].library[0], card);
    assert_eq!(game.state.pending, Pending::Mulligan(Player::P0));
    act(&mut game, Action::Mulligan);
    assert_eq!(
        game.state.pending,
        Pending::Bottom {
            player: Player::P0,
            remaining: 2
        }
    );
    for _ in 0..2 {
        let action = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Bottom { .. }))
            .unwrap();
        act(&mut game, action);
    }
    assert_eq!(game.hand_count(Player::P0), 5);
    act(&mut game, Action::Keep);
    assert_eq!(game.state.step, Step::Upkeep);
    assert_invariants(&game);
}

#[test]
fn seven_mulligans_end_with_zero_cards_and_no_eighth_mulligan() {
    let mut game = fresh();
    act(&mut game, Action::Mulligan);
    act(&mut game, Action::Keep);
    for round in 1..=7 {
        for _ in 0..round {
            let action = game
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Bottom { .. }))
                .unwrap();
            act(&mut game, action);
        }
        assert_eq!(game.hand_count(Player::P0), 7 - round);
        if round < 7 {
            act(&mut game, Action::Mulligan);
        }
    }
    assert_eq!(game.state.step, Step::Upkeep);
    assert!(!game.legal_actions().contains(&Action::Mulligan));
    assert_invariants(&game);
}

#[test]
fn first_player_skips_only_first_draw_and_other_player_draws() {
    let mut game = fresh();
    act(&mut game, Action::Keep);
    act(&mut game, Action::Keep);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::Main1);
    assert_eq!(game.hand_count(Player::P0), 7);
    for _ in 0..100 {
        if game.state.turn == 2 && game.state.step == Step::Draw {
            break;
        }
        let action = if game.state.pending == Pending::Attackers {
            Action::FinishAttackers
        } else {
            Action::Pass
        };
        act(&mut game, action);
    }
    assert_eq!(game.state.step, Step::Draw);
    assert_eq!(game.state.active, Player::P1);
    assert_eq!(game.hand_count(Player::P1), 8);
}

#[test]
fn rejected_actions_are_atomic_and_restore_invalidates_indices() {
    let mut game = scenario();
    let before = game.snapshot();
    let rev = game.revision();
    assert_eq!(game.apply(Player::P1, Action::Pass), Err(Error::WrongActor));
    assert_eq!(
        game.apply(Player::P0, Action::PlayLand { card: u32::MAX }),
        Err(Error::IllegalAction)
    );
    assert_eq!(
        game.apply(
            Player::P0,
            Action::Cast {
                card: u32::MAX,
                target: None,
                payment: [0; 6]
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    assert_eq!(game.revision(), rev);
    game.restore(&before);
    assert_eq!(
        game.apply_index(Player::P0, rev, 0),
        Err(Error::StaleRevision)
    );
    assert_eq!(game.snapshot(), before);
}

#[test]
fn land_and_sorcery_timing_are_enforced() {
    let mut game = scenario();
    let forest = spawn(&mut game, Player::P0, Card::Forest, Zone::Hand);
    let another = spawn(&mut game, Player::P0, Card::Forest, Zone::Hand);
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Hand);
    game.state.players[0].mana = [0, 0, 0, 0, 2, 0];
    act(&mut game, Action::PlayLand { card: forest });
    assert!(
        !game
            .legal_actions()
            .contains(&Action::PlayLand { card: another })
    );
    act(&mut game, Action::Pass);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::PlayLand { .. }))
    );
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::BeginCombat);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Cast { card, .. } if *card == bears))
    );
}

#[test]
fn payments_include_every_generic_color_choice_and_reject_underpayment() {
    let mut game = scenario();
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Hand);
    game.state.players[0].mana = [1, 1, 0, 0, 2, 0];
    let castings: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::Cast { .. }))
        .collect();
    assert_eq!(castings.len(), 3);
    let before = game.snapshot();
    assert_eq!(
        game.apply(
            Player::P0,
            Action::Cast {
                card: bears,
                target: None,
                payment: [0, 0, 0, 0, 1, 0]
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(before, game.snapshot());
    act(
        &mut game,
        Action::Cast {
            card: bears,
            target: None,
            payment: [0, 1, 0, 0, 1, 0],
        },
    );
    assert_eq!(game.state.players[0].mana, [1, 0, 0, 0, 1, 0]);
}

#[test]
fn stack_retains_caster_priority_then_resolves_lifo_with_counterspell() {
    let mut game = scenario();
    let bolt = spawn(&mut game, Player::P0, Card::LightningBolt, Zone::Hand);
    let counter = spawn(&mut game, Player::P1, Card::Counterspell, Zone::Hand);
    game.state.players[0].mana[3] = 1;
    game.state.players[1].mana[1] = 2;
    cast(&mut game, bolt, Some(Target::Player(Player::P1)));
    assert_eq!(game.actor(), Some(Player::P0));
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(bolt)));
    assert_eq!(game.actor(), Some(Player::P1));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(Player::P0));
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.objects[bolt as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[counter as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.players[1].life, 20);
    assert_invariants(&game);
}

#[test]
fn target_is_checked_again_on_resolution() {
    let mut game = scenario();
    let bears = spawn(&mut game, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let growth = spawn(&mut game, Player::P0, Card::GiantGrowth, Zone::Hand);
    let bolt = spawn(&mut game, Player::P1, Card::LightningBolt, Zone::Hand);
    game.state.players[0].mana[4] = 1;
    game.state.players[1].mana[3] = 1;
    cast(&mut game, growth, Some(Target::Card(bears)));
    act(&mut game, Action::Pass);
    cast(&mut game, bolt, Some(Target::Card(bears)));
    pass_pair(&mut game);
    assert_eq!(game.state.objects[bears as usize].zone, Zone::Graveyard);
    pass_pair(&mut game);
    assert_eq!(game.state.objects[bears as usize].growth, Stats::default());
    assert_eq!(game.state.objects[growth as usize].zone, Zone::Graveyard);
}

#[test]
fn summoning_sickness_blocks_attacks_and_elf_mana_but_not_blocking() {
    let mut game = scenario();
    let elf = spawn(&mut game, Player::P0, Card::LlanowarElves, Zone::Hand);
    game.state.players[0].mana[4] = 1;
    cast(&mut game, elf, None);
    pass_pair(&mut game);
    assert!(
        !game
            .legal_actions()
            .contains(&Action::TapMana { card: elf })
    );
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    assert!(!game.legal_actions().contains(&Action::Attack { card: elf }));
    game.state.active = Player::P1;
    let attacker = spawn(&mut game, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    game.state.attackers.push(Attacker {
        card: attacker,
        blocked: false,
    });
    game.state.step = Step::Blockers;
    game.state.pending = Pending::Blockers;
    assert!(game.legal_actions().contains(&Action::Block {
        blocker: elf,
        attacker
    }));
}

#[test]
fn combat_damage_is_simultaneous_and_multiblock_damage_is_a_real_choice() {
    let mut game = scenario();
    let attacker = spawn(&mut game, Player::P0, Card::HillGiant, Zone::Battlefield);
    let b1 = spawn(&mut game, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let b2 = spawn(&mut game, Player::P1, Card::GoblinPiker, Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    assert!(!game.state.objects[attacker as usize].tapped); // declaration is atomic
    act(&mut game, Action::FinishAttackers);
    assert!(game.state.objects[attacker as usize].tapped);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::Block {
            blocker: b1,
            attacker,
        },
    );
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Block { blocker, .. } if *blocker == b1))
    );
    act(
        &mut game,
        Action::Block {
            blocker: b2,
            attacker,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Damage);
    let checkpoint = game.snapshot();
    assert_eq!(game.state.objects[attacker as usize].damage, 0);
    // Current rules permit every division, without an ordering/lethal requirement.
    act(
        &mut game,
        Action::AssignDamage {
            target: Target::Card(b1),
            amount: 2,
        },
    );
    assert_eq!(game.state.objects[attacker as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[b1 as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[b2 as usize].zone, Zone::Graveyard);
    game.restore(&checkpoint);
    act(
        &mut game,
        Action::AssignDamage {
            target: Target::Card(b1),
            amount: 0,
        },
    );
    assert_eq!(game.state.objects[b1 as usize].zone, Zone::Battlefield);
    assert_eq!(game.state.objects[b2 as usize].zone, Zone::Graveyard);
    assert_invariants(&game);
}

#[test]
fn killing_a_blocker_does_not_unblock_attacker() {
    let mut game = scenario();
    let attacker = spawn(&mut game, Player::P0, Card::HillGiant, Zone::Battlefield);
    let blocker = spawn(&mut game, Player::P1, Card::GoblinPiker, Zone::Battlefield);
    game.state.attackers.push(Attacker {
        card: attacker,
        blocked: true,
    });
    game.state.blocks.push((blocker, attacker));
    game.move_object(blocker, Zone::Graveyard);
    game.state.step = Step::Blockers;
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 20);
    assert_eq!(game.state.objects[attacker as usize].damage, 0);
}

#[test]
fn cleanup_discards_then_expires_damage_and_growth_together() {
    let mut game = scenario();
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    game.state.objects[bears as usize].growth = (3).into();
    game.state.objects[bears as usize].damage = 4;
    for _ in 0..9 {
        spawn(&mut game, Player::P0, Card::Mountain, Zone::Hand);
    }
    game.state.players[0].mana[3] = 3;
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Discard);
    for _ in 0..2 {
        let action = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        act(&mut game, action);
    }
    assert_eq!(game.state.active, Player::P1);
    assert_eq!(game.state.objects[bears as usize].damage, 0);
    assert_eq!(game.state.objects[bears as usize].growth, Stats::default());
    assert_eq!(game.state.objects[bears as usize].zone, Zone::Battlefield);
    assert_eq!(game.state.players[0].mana, [0; 6]);
}

#[test]
fn decking_waits_for_failed_draw_and_simultaneous_losses_draw() {
    let mut game = scenario();
    game.state.players[0].library.truncate(1);
    game.draw(Player::P0);
    game.state_based_actions();
    assert!(game.outcome().is_none());
    game.draw(Player::P0);
    game.state_based_actions();
    assert_eq!(game.outcome(), Some(Outcome::Winner(Player::P1)));
    let mut game = scenario();
    game.state.players[0].life = 0;
    game.state.players[1].life = 0;
    game.state_based_actions();
    assert_eq!(game.outcome(), Some(Outcome::Draw));
    assert!(game.legal_actions().is_empty());
}

#[test]
fn observations_do_not_reveal_opponent_hand_library_rng_or_choices() {
    let mut game = fresh();
    let observation = game.observe(Player::P1);
    assert_eq!(observation.objects.len(), 7);
    assert!(
        observation
            .objects
            .iter()
            .all(|o| o.owner == Player::P1 && o.zone == Zone::Hand)
    );
    assert!(observation.actions.is_empty());
    let original = serde_json::to_value(observation).unwrap();
    // Change both order and identities of unseen cards, leaving public counts fixed.
    game.state.players[0].library.reverse();
    game.state.rng.next();
    for o in &mut game.state.objects {
        if o.zone == Zone::Library || (o.owner == Player::P0 && o.zone == Zone::Hand) {
            o.card = Card::Counterspell;
        }
    }
    assert_eq!(
        serde_json::to_value(game.observe(Player::P1)).unwrap(),
        original
    );
}

#[test]
fn constructed_and_unknown_card_rejection_are_explicit() {
    let mut cfg = config(vec![Card::LightningBolt; 60]);
    cfg.deck_rules = DeckRules::Constructed;
    assert!(matches!(Game::new(cfg), Err(Error::InvalidDeck(_))));
    let mut cfg = config(vec![Card::Forest; 59]);
    cfg.deck_rules = DeckRules::Constructed;
    assert!(Game::new(cfg).is_err());
    assert!(serde_json::from_str::<Card>("\"BlackLotus\"").is_err());
    assert!(Game::new(config(vec![Card::Forest; 60])).is_ok());
}

#[test]
fn random_legal_play_preserves_invariants_and_every_replay_checkpoint() {
    for seed in 0..12 {
        let mut cfg = config(Card::ALL.into_iter().cycle().take(60).collect());
        cfg.seed = seed;
        let mut game = Game::new(cfg).unwrap();
        let mut rng = Rng(seed + 100);
        for step in 0..2500 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            let actions: Vec<_> = game
                .legal_actions()
                .into_iter()
                .filter(|a| *a != Action::Concede)
                .collect();
            let action = actions[rng.below(actions.len() as u64) as usize].clone();
            let before = game.snapshot();
            let mut branch = game.clone();
            game.apply(actor, action.clone()).unwrap();
            branch.apply(actor, action.clone()).unwrap();
            assert_eq!(game.snapshot(), branch.snapshot());
            if step % 13 == 0 {
                let after = game.snapshot();
                game.restore(&before);
                game.apply(actor, action).unwrap();
                assert_eq!(game.snapshot(), after);
            }
            assert_invariants(&game);
        }
    }
}

#[test]
fn flying_requires_flying_or_reach_and_vigilance_does_not_tap() {
    let mut game = scenario();
    let angel = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Serra Angel").unwrap(),
        Zone::Battlefield,
    );
    let ground = spawn(&mut game, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let reach = spawn(
        &mut game,
        Player::P1,
        Card::from_name("Magnigoth Sentry").unwrap(),
        Zone::Battlefield,
    );
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: angel });
    act(&mut game, Action::FinishAttackers);
    assert!(!game.state.objects[angel as usize].tapped);
    pass_pair(&mut game);
    assert!(!game.legal_actions().contains(&Action::Block {
        blocker: ground,
        attacker: angel
    }));
    assert!(game.legal_actions().contains(&Action::Block {
        blocker: reach,
        attacker: angel
    }));
    let before = game.snapshot();
    assert_eq!(
        game.apply(
            Player::P1,
            Action::Block {
                blocker: ground,
                attacker: angel
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
}

#[test]
fn haste_ignores_sickness_and_defender_still_cannot_attack() {
    let mut game = scenario();
    let haste = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Brazen Scourge").unwrap(),
        Zone::Battlefield,
    );
    let defender = spawn(
        &mut game,
        Player::P0,
        Card::from_name("Monoist Sentry").unwrap(),
        Zone::Battlefield,
    );
    game.state.objects[haste as usize].summoning_sick = true;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    assert!(
        game.legal_actions()
            .contains(&Action::Attack { card: haste })
    );
    assert!(
        !game
            .legal_actions()
            .contains(&Action::Attack { card: defender })
    );
}

#[test]
fn flash_creature_can_be_cast_in_opponents_upkeep() {
    let mut game = scenario();
    let flash = spawn(
        &mut game,
        Player::P1,
        Card::from_name("Dawn's Light Archer").unwrap(),
        Zone::Hand,
    );
    let bears = spawn(&mut game, Player::P1, Card::GrizzlyBears, Zone::Hand);
    game.state.step = Step::Upkeep;
    game.state.priority = Player::P1;
    game.state.players[1].mana[4] = 3;
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Cast { card, .. } if *card == bears))
    );
    cast(&mut game, flash, None);
    pass_pair(&mut game);
    assert_eq!(game.state.objects[flash as usize].zone, Zone::Battlefield);
    assert!(game.state.objects[flash as usize].summoning_sick);
    assert_eq!(game.actor(), Some(Player::P0));
}
