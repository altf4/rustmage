use super::tests::{act, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, name: &str, zone: Zone) -> CardId {
    spawn(game, Player::P0, Card::from_name(name).unwrap(), zone)
}
fn treasure(game: &mut Game) -> CardId {
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.name == "Treasure Token"))
        .unwrap();
    game.create_tokens(Player::P0, factory, 1, false);
    game.state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.zone == Zone::Battlefield && o.card == factory)
        .unwrap()
        .0 as CardId
}
fn choose(game: &mut Game, predicate: impl Fn(&Action) -> bool) -> Action {
    game.legal_actions()
        .into_iter()
        .find(predicate)
        .expect("legal action")
}

#[test]
fn target_treasure_then_sacrifice_it_to_pay_preserves_the_whole_declaration() {
    let mut game = scenario();
    let token = treasure(&mut game);
    let spell = named(&mut game, "Disenchant", Zone::Hand);
    game.state.players[0].mana[4] = 1;
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Cast { .. }))
    );
    act(&mut game, Action::AnnounceCast { card: spell });
    let announced = game.snapshot();
    let target = Action::ChooseTarget {
        target: Target::Card(token),
    };
    act(&mut game, target.clone());
    assert_eq!(game.state.pending, Pending::PayCosts { player: Player::P0 });
    let selected = game.snapshot();
    let request = game.observe(Player::P0).payment_request.unwrap();
    assert_eq!(request.required, [1, 0, 0, 0, 0, 0]);
    assert_eq!(request.generic, 1);
    assert!(game.observe(Player::P1).payment_request.is_none());
    let options: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::ActivateMana { .. }))
        .collect();
    assert_eq!(options.len(), 1); // Other colors would strand the white payment.
    assert!(matches!(
        options[0],
        Action::ActivateMana {
            production: [1, 0, 0, 0, 0, 0],
            ..
        }
    ));
    let mana = options[0].clone();
    act(&mut game, mana.clone());
    assert_eq!(game.object_view(token).zone, Zone::Graveyard); // No SBA inside casting.
    assert!(game.state.free_tokens.is_empty());
    assert!(game.state.stack.is_empty());
    assert_eq!(
        game.state.announcing.as_ref().unwrap().item.targets[0]
            .object
            .unwrap()
            .generation
            + 1,
        game.object_ref(token).generation
    );
    let funded = game.snapshot();
    let pay = Action::PayCosts {
        payment: [1, 0, 0, 0, 1, 0],
    };
    act(&mut game, pay.clone());
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.object_view(token).zone, Zone::Outside);
    pass_pair(&mut game); // Its only target is now illegal.
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    let after = game.snapshot();
    for (checkpoint, actions) in [
        (announced, vec![target.clone(), mana.clone(), pay.clone()]),
        (selected, vec![mana.clone(), pay.clone()]),
        (funded, vec![pay.clone()]),
    ] {
        game.restore(&checkpoint);
        for action in actions {
            act(&mut game, action);
        }
        pass_pair(&mut game);
        assert_eq!(game.snapshot(), after);
    }
}

#[test]
fn payment_cannot_tap_or_sacrifice_the_source_needed_for_the_enclosing_cost() {
    let mut game = scenario();
    let archive = named(&mut game, "Hedron Archive", Zone::Battlefield);
    assert!(
        !game.legal_actions().iter().any(
            |a| matches!(a,Action::AnnounceActivate { card, ability:1, .. } if *card == archive)
        )
    );
    let first = spawn(&mut game, Player::P0, Card::Forest, Zone::Battlefield);
    let second = spawn(&mut game, Player::P0, Card::Forest, Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: archive,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana { card, .. } if *card == archive))
    );
    let before = game.snapshot();
    assert_eq!(
        game.apply(
            Player::P0,
            Action::ActivateMana {
                card: archive,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 2]
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    act(&mut game, Action::TapMana { card: first });
    act(&mut game, Action::TapMana { card: second });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 2, 0],
        },
    );
    assert_eq!(game.object_view(archive).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 2);
}

#[test]
fn unfunded_mana_ability_can_get_mana_from_another_source_without_using_the_stack() {
    let mut game = scenario();
    let prism = named(&mut game, "Prophetic Prism", Zone::Battlefield);
    let land = spawn(&mut game, Player::P0, Card::Forest, Zone::Battlefield);
    act(
        &mut game,
        Action::AnnounceMana {
            card: prism,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 1, 0, 0, 0],
        },
    );
    assert!(game.state.stack.is_empty());
    let before = game.snapshot();
    act(&mut game, Action::TapMana { card: land });
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana { card, .. } if *card == prism))
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[0].mana, [0, 0, 1, 0, 0, 0]);
    assert_eq!(game.actor(), Some(Player::P0));
    assert!(game.object_view(prism).tapped);
    let after = game.snapshot();
    game.restore(&before);
    act(&mut game, Action::TapMana { card: land });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    assert_eq!(game.snapshot(), after);
}

#[test]
fn no_responses_or_unfundable_mana_choices_are_offered_during_payment() {
    let mut game = scenario();
    let token = treasure(&mut game);
    let growth = spawn(&mut game, Player::P0, Card::GiantGrowth, Zone::Hand);
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: growth });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bears),
        },
    );
    assert!(!game.legal_actions().contains(&Action::Pass));
    assert!(game.observe(Player::P1).actions.is_empty());
    let before = game.snapshot();
    let revision = game.revision;
    let wrong = Action::ActivateMana {
        card: token,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [1, 0, 0, 0, 0, 0],
    };
    assert_eq!(game.apply(Player::P0, wrong), Err(Error::IllegalAction));
    assert_eq!(game.snapshot(), before);
    assert_eq!(game.revision, revision);
    assert_eq!(game.apply(Player::P1, Action::Pass), Err(Error::WrongActor));
    let mana = choose(&mut game, |a| matches!(a, Action::ActivateMana { .. }));
    act(&mut game, mana);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.object_view(bears).power, 5);
}

#[test]
fn queued_triggers_wait_until_the_enclosing_cast_has_committed() {
    let mut game = scenario();
    let bear = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let growth = spawn(&mut game, Player::P0, Card::GiantGrowth, Zone::Hand);
    let land = spawn(&mut game, Player::P0, Card::Forest, Zone::Battlefield);
    let teddy = named(&mut game, "Friendly Teddy", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: growth });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    // Kernel fixture: a death event occurring inside a future mana-cost effect.
    game.move_object(teddy, Zone::Graveyard);
    assert_eq!(game.state.pending_triggers.len(), 1);
    act(&mut game, Action::TapMana { card: land });
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.pending_triggers.len(), 1);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    assert_eq!(game.state.stack.len(), 2);
    assert_eq!(game.state.stack[0].kind, StackKind::Spell);
    assert!(matches!(
        game.state.stack[1].kind,
        StackKind::TriggeredAbility { .. }
    ));
}

#[test]
fn impossible_colored_cost_is_rejected_without_mutation_among_many_identical_sources() {
    let mut game = scenario();
    for _ in 0..64 {
        spawn(&mut game, Player::P0, Card::Forest, Zone::Battlefield);
    }
    let before = game.snapshot();
    let revision = game.revision;
    assert!(!game.can_fund(FundingGoal {
        reserve: None,
        actor: Player::P0,
        required: [0, 2, 0, 0, 0, 0],
        generic: 0,
        source: None
    }));
    assert_eq!(game.snapshot(), before);
    assert_eq!(game.revision, revision);
}

#[test]
fn compact_funding_search_matches_native_exhaustive_mana_sequences_on_small_boards() {
    fn brute(
        game: &Game,
        required: Mana,
        generic: u32,
        seen: &mut std::collections::HashSet<String>,
    ) -> bool {
        let pool = game.state.players[0].mana;
        if (0..6).all(|i| pool[i] >= required[i])
            && (0..6)
                .map(|i| u64::from(pool[i] - required[i]))
                .sum::<u64>()
                >= u64::from(generic)
        {
            return true;
        }
        let objects: Vec<_> = (0..game.state.objects.len())
            .map(|id| game.object_view(id as CardId))
            .collect();
        let key = serde_json::to_string(&(pool, objects)).unwrap();
        if !seen.insert(key) {
            return false;
        }
        for action in game
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
        {
            let mut branch = game.clone();
            // Casting has no intervening SBA/trigger placement. The independent
            // oracle mutates the native engine, not the compact resource graph.
            branch.apply_legal(Player::P0, action);
            if brute(&branch, required, generic, seen) {
                return true;
            }
        }
        false
    }
    // Controlled-subtype guards share card facts, but the oracle still executes
    // real engine actions and never consults compact resources or search keys.
    for witness in [
        None,
        Some((Player::P0, "Swamp")),
        Some((Player::P1, "Swamp")),
        Some((Player::P0, "Rooftop Percher")),
        Some((Player::P1, "Rooftop Percher")),
    ] {
        let mut game = scenario();
        named(&mut game, "Blazemire Verge", Zone::Battlefield);
        named(&mut game, "Mox Jasper", Zone::Battlefield);
        if let Some((owner, name)) = witness {
            spawn(
                &mut game,
                owner,
                Card::from_name(name).unwrap(),
                Zone::Battlefield,
            );
        }
        for red in 0..=2 {
            for blue in 0..=1 {
                for generic in 0..=2 {
                    let required = [0, blue, 0, red, 0, 0];
                    assert_eq!(
                        game.can_fund(FundingGoal {
                            reserve: None,
                            actor: Player::P0,
                            required,
                            generic,
                            source: None
                        }),
                        brute(
                            &game,
                            required,
                            generic,
                            &mut std::collections::HashSet::new()
                        ),
                        "witness={witness:?} required={required:?} generic={generic}"
                    );
                }
            }
        }
    }
    // The oracle executes native mana actions; it does not consult the compact
    // funding graph. Cover new positive haste grants, opposing ownership, both
    // active players, source departure, attachments and summoning sickness.
    for owner in [Player::P0, Player::P1] {
        for active in [Player::P0, Player::P1] {
            for present in [false, true] {
                let mut game = scenario();
                game.state.active = active;
                let elf = named(&mut game, "Llanowar Elves", Zone::Battlefield);
                game.state.objects[elf as usize].summoning_sick = true;
                let druid = named(&mut game, "Druid of the Cowl", Zone::Battlefield);
                game.state.objects[druid as usize].summoning_sick = true;
                let samut = spawn(
                    &mut game,
                    owner,
                    Card::from_name("Samut, Hazoret's Champion").unwrap(),
                    Zone::Battlefield,
                );
                if !present {
                    game.move_object(samut, Zone::Hand);
                }
                named(&mut game, "Forest", Zone::Battlefield);
                for green in 0..=4 {
                    for generic in 0..=1 {
                        let required = [0, 0, 0, 0, green, 0];
                        assert_eq!(
                            game.can_fund(FundingGoal {
                                reserve: None,
                                actor: Player::P0,
                                required,
                                generic,
                                source: None
                            }),
                            brute(
                                &game,
                                required,
                                generic,
                                &mut std::collections::HashSet::new()
                            ),
                            "owner={owner:?} active={active:?} present={present} green={green} generic={generic}"
                        );
                    }
                }
            }
        }
    }
    let mut game = scenario();
    treasure(&mut game);
    named(&mut game, "Hedron Archive", Zone::Battlefield);
    named(&mut game, "Scarecrow Guide", Zone::Battlefield);
    for white in 0..=2 {
        for blue in 0..=1 {
            for colorless in 0..=2 {
                for generic in 0..=2 {
                    let required = [white, blue, 0, 0, 0, colorless];
                    let expected = brute(
                        &game,
                        required,
                        generic,
                        &mut std::collections::HashSet::new(),
                    );
                    assert_eq!(
                        game.can_fund(FundingGoal {
                            reserve: None,
                            actor: Player::P0,
                            required,
                            generic,
                            source: None
                        }),
                        expected,
                        "required={required:?}, generic={generic}"
                    );
                }
            }
        }
    }
}

#[test]
fn nonactive_player_funds_an_instant_and_keeps_priority_after_casting() {
    let mut game = scenario();
    let bolt = spawn(&mut game, Player::P0, Card::LightningBolt, Zone::Hand);
    game.state.players[0].mana[3] = 1;
    super::tests::cast(&mut game, bolt, Some(Target::Player(Player::P1)));
    act(&mut game, Action::Pass);
    let counter = spawn(&mut game, Player::P1, Card::Counterspell, Zone::Hand);
    let first = spawn(&mut game, Player::P1, Card::Island, Zone::Battlefield);
    let second = spawn(&mut game, Player::P1, Card::Island, Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: counter });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(bolt),
        },
    );
    act(&mut game, Action::TapMana { card: first });
    act(&mut game, Action::TapMana { card: second });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 2, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.actor(), Some(Player::P1));
    assert_eq!(game.state.stack.len(), 2);
    pass_pair(&mut game);
    assert_eq!(game.object_view(bolt).zone, Zone::Graveyard);
    assert_eq!(game.state.players[1].life, 20);
}

#[test]
fn mana_and_life_only_costs_remain_payable_after_the_source_leaves() {
    let mut game = scenario();
    let source = named(&mut game, "Mystic Archaeologist", Zone::Battlefield);
    let mut lands = Vec::new();
    for _ in 0..5 {
        lands.push(spawn(
            &mut game,
            Player::P0,
            Card::Island,
            Zone::Battlefield,
        ));
    }
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let original = game.state.announcing.as_ref().unwrap().item.object;
    // Kernel fixture for a source leaving through another mana ability/cost.
    game.move_object(source, Zone::Graveyard);
    let before = game.snapshot();
    for &land in &lands {
        act(&mut game, Action::TapMana { card: land });
    }
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 5, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.stack[0].object, original);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 2);
    let after = game.snapshot();
    game.restore(&before);
    for land in lands {
        act(&mut game, Action::TapMana { card: land });
    }
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 5, 0, 0, 0, 0],
        },
    );
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}
