use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved);
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision;
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision, rev);
}
fn choose(g: &mut Game, colored: Mana) {
    let a = g
        .observe(g.actor().unwrap())
        .cost_choices
        .iter()
        .find(|c| c.cost.colored == colored)
        .map(|c| Action::ChooseSpellCost { cost: c.index })
        .expect("cost available");
    replay(g, a);
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<Target>) {
    for _ in 0..100 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            return;
        }
        let actions = g.legal_actions();
        let a = target
            .and_then(|t| {
                actions
                    .iter()
                    .find(|a| matches!(a,Action::ChooseTarget{target}if *target==t))
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. }
                            | Action::Pass
                            | Action::Discard { .. }
                            | Action::DeclineEffectPayment
                    )
                })
            })
            .unwrap()
            .clone();
        replay(g, a);
    }
    panic!("fixture didn't settle");
}
fn cast_funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(g, id, None);
    id
}
fn tokens(g: &Game, name: &str) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.card
                    .token_info()
                    .is_some_and(|t| t.name == format!("{name} Token"))
        })
        .count()
}
fn attack(g: &mut Game, id: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[id as usize].summoning_sick = false;
    replay(g, Action::Attack { card: id });
    replay(g, Action::FinishAttackers);
}

#[test]
fn cost_equivalents_are_unique_same_mana_value_and_fixed_costs_stay_single() {
    for card in Card::catalogue().filter(|c| c.is_supported() && !c.is_land()) {
        let options = card.cost_options().collect::<Vec<_>>();
        assert!(!options.is_empty());
        for (i, cost) in options.iter().enumerate() {
            assert!(!options[..i].contains(cost), "{}", card.name());
            assert_eq!(
                u64::from(cost.generic) + cost.colored.iter().map(|n| u64::from(*n)).sum::<u64>(),
                card.fixed_mana_value()
            );
        }
        assert_eq!(options.len() > 1, card.has_hybrid_cost());
    }
    let c = Card::from_name("Gallant Citizen").unwrap();
    assert_eq!(
        c.cost_options().map(|c| c.colored).collect::<Vec<_>>(),
        vec![[0, 0, 0, 0, 2, 0], [1, 0, 0, 0, 1, 0], [2, 0, 0, 0, 0, 0]]
    );
    assert_eq!(c.fixed_mana_value(), 2);
    assert_eq!(c.info().mana_cost, "{G/W}{G/W}");
    assert_eq!(Card::from_name("Large Bear").unwrap().fixed_mana_value(), 5);
}

#[test]
fn prefunded_hybrid_allocations_match_exhaustive_pool_spending_without_duplicates() {
    let c = Card::from_name("Moonrise Cleric").unwrap(); // {1}{W/B}{W/B}
    for w in 0..=3 {
        for b in 0..=3 {
            for g in 0..=3 {
                for colorless in 0..=2 {
                    let pool = [w, 0, b, 0, g, colorless];
                    let mut actual = Vec::new();
                    c.spell_payments(pool, |p| actual.push(p));
                    let mut expected = Vec::new();
                    for pw in 0..=w {
                        for pb in 0..=b {
                            for pg in 0..=g {
                                for pc in 0..=colorless {
                                    if pw + pb + pg + pc == 3 && pw + pb >= 2 {
                                        expected.push([pw, 0, pb, 0, pg, pc]);
                                    }
                                }
                            }
                        }
                    }
                    actual.sort();
                    expected.sort();
                    assert_eq!(actual, expected, "pool {pool:?}");
                }
            }
        }
    }
}

#[test]
fn staged_hybrid_choice_precedes_mana_window_and_commit_is_exact() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Gallant Citizen", Zone::Hand);
    for _ in 0..2 {
        spawn(&mut g, Player::P0, Card::Plains, Zone::Battlefield);
        spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
    }
    replay(&mut g, Action::AnnounceCast { card: c });
    assert_eq!(g.state.pending, Pending::SpellCost { player: Player::P0 });
    let view = g.observe(Player::P0);
    assert_eq!(view.schema, 33);
    assert_eq!(view.cost_choices.len(), 3);
    assert!(view.payment_request.is_none());
    assert!(
        view.actions
            .iter()
            .all(|a| matches!(a, Action::ChooseSpellCost { .. } | Action::Concede))
    );
    reject(&mut g, Action::Pass);
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 2, 0],
        },
    );
    reject(&mut g, Action::ChooseSpellCost { cost: 3 });
    reject(&mut g, Action::ChooseSpellCost { cost: u16::MAX });
    let saved = g.snapshot();
    let rev = g.revision;
    assert_eq!(
        g.apply(Player::P1, Action::ChooseSpellCost { cost: 0 }),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision, rev);
    choose(&mut g, [2, 0, 0, 0, 0, 0]);
    assert_eq!(g.state.pending, Pending::PayCosts { player: Player::P0 });
    assert_eq!(
        g.observe(Player::P1).announcing_cost,
        Some(ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 0
        })
    );
    assert_eq!(
        g.observe(Player::P0).payment_request.unwrap().required,
        [2, 0, 0, 0, 0, 0]
    );
    // Extra mana may legally be floated, but it cannot change the committed equivalent.
    for _ in 0..2 {
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::TapMana{card}if g.state.objects[*card as usize].card==Card::Forest)).unwrap();
        replay(&mut g, a);
    }
    reject(&mut g, Action::ChooseSpellCost { cost: 0 });
    for _ in 0..2 {
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::TapMana{card}if g.state.objects[*card as usize].card==Card::Plains)).unwrap();
        replay(&mut g, a);
    }
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 2, 0],
        },
    );
    reject(
        &mut g,
        Action::PayCosts {
            payment: [3, 0, 0, 0, 0, 0],
        },
    );
    pay(&mut g);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    settle(&mut g, None);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(g.object_view(c).zone, Zone::Battlefield);
}

#[test]
fn hybrid_choices_filter_by_exact_funding_and_no_cost_is_advertised_without_completion() {
    for (w, green, expected) in [
        (0, 2, vec![[0, 0, 0, 0, 2, 0]]),
        (1, 1, vec![[1, 0, 0, 0, 1, 0]]),
        (2, 0, vec![[2, 0, 0, 0, 0, 0]]),
    ] {
        let mut g = scenario();
        let c = named(&mut g, Player::P0, "Gallant Citizen", Zone::Hand);
        for _ in 0..w {
            spawn(&mut g, Player::P0, Card::Plains, Zone::Battlefield);
        }
        for _ in 0..green {
            spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
        }
        replay(&mut g, Action::AnnounceCast { card: c });
        assert_eq!(
            g.observe(Player::P0)
                .cost_choices
                .iter()
                .map(|c| c.cost.colored)
                .collect::<Vec<_>>(),
            expected
        );
    }
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Gallant Citizen", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 1, 5];
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceCast{card}if *card==c))
    );
    reject(&mut g, Action::AnnounceCast { card: c });
}

#[test]
fn same_snapshot_can_branch_to_three_fixed_equivalents_and_replay_after_restoration() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Gallant Citizen", Zone::Hand);
    g.state.players[0].mana = [2, 0, 0, 0, 2, 0];
    replay(&mut g, Action::AnnounceCast { card: c });
    let saved = g.snapshot();
    let stale = g.observe(Player::P0).revision;
    for colored in [[0, 0, 0, 0, 2, 0], [1, 0, 0, 0, 1, 0], [2, 0, 0, 0, 0, 0]] {
        choose(&mut g, colored);
        assert_eq!(
            g.observe(Player::P0).announcing_cost.unwrap().colored,
            colored
        );
        let pays = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| {
                if let Action::PayCosts { payment } = a {
                    Some(payment)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(pays, vec![colored]);
        pay(&mut g);
        settle(&mut g, None);
        assert_eq!(g.state.players[0].library.len(), 19);
        assert_eq!(
            g.state.players[0].mana,
            std::array::from_fn(|i| [2, 0, 0, 0, 2, 0][i] - colored[i])
        );
        g.restore(&saved);
        assert_eq!(
            g.apply_index(Player::P0, stale, 0),
            Err(Error::StaleRevision)
        );
    }
}

#[test]
fn hybrid_payment_choice_can_pay_generic_with_any_type_and_rejects_wrong_colored_parts() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Large Bear", Zone::Hand);
    g.state.players[0].mana = [1, 1, 2, 1, 2, 1];
    replay(&mut g, Action::AnnounceCast { card: c });
    choose(&mut g, [0, 0, 2, 0, 0, 0]);
    let request = g.observe(Player::P0).payment_request.unwrap();
    assert_eq!(request.generic, 3);
    reject(
        &mut g,
        Action::PayCosts {
            payment: [1, 1, 1, 1, 1, 0],
        },
    );
    replay(
        &mut g,
        Action::PayCosts {
            payment: [1, 1, 2, 0, 0, 1],
        },
    );
    settle(&mut g, None);
    assert!(
        g.keywords(c).contains(
            Keywords::REACH
                .union(Keywords::TRAMPLE)
                .union(Keywords::HASTE)
        )
    );
}

#[test]
fn prefunded_shortcuts_do_not_duplicate_actions_and_mana_value_is_printed_not_payment_color() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Moonrise Cleric", Zone::Hand);
    g.state.players[0].mana = [3, 0, 3, 0, 0, 0];
    let actions = g
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a,Action::Cast{card,..}if *card==c))
        .collect::<Vec<_>>();
    for (i, a) in actions.iter().enumerate() {
        assert!(!actions[..i].contains(a));
    }
    assert_eq!(actions.len(), 4); // WWW, WWB, WBB, BBB
    replay(&mut g, actions[1].clone());
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.stack[0].source_card.fixed_mana_value(), 3);
    settle(&mut g, None);
    let before = g.state.players[0].life;
    attack(&mut g, c);
    settle(&mut g, None);
    assert_eq!(g.state.players[0].life, before + 1);
}

#[test]
fn chosen_cost_is_public_and_other_viewer_has_no_choices_or_private_hand() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Gallant Citizen", Zone::Hand);
    let hidden = spawn(&mut g, Player::P0, Card::Forest, Zone::Hand);
    g.state.players[0].mana = [2, 0, 0, 0, 2, 0];
    replay(&mut g, Action::AnnounceCast { card: c });
    let other = g.observe(Player::P1);
    assert!(other.cost_choices.is_empty() && other.actions.is_empty());
    assert!(other.announcing_cost.is_none());
    assert!(other.objects.iter().all(|o| o.id != hidden));
    choose(&mut g, [1, 0, 0, 0, 1, 0]);
    assert_eq!(
        g.observe(Player::P1).announcing_cost,
        g.observe(Player::P0).announcing_cost
    );
    assert!(g.observe(Player::P1).payment_request.is_none());
}

#[test]
fn hybrid_token_makers_produce_exact_factories_and_noggle_death_also_triggers() {
    for (name, token, amount) in [
        ("Head of the Homestead", "Rabbit", 2),
        ("Mechanized Ninja Cavalry", "Robot", 1),
        ("Slithering Cryptid", "Mutagen", 1),
        ("Noggle Robber", "Treasure", 1),
    ] {
        let mut g = scenario();
        let c = cast_funded(&mut g, name);
        settle(&mut g, None);
        assert_eq!(tokens(&g, token), amount, "{name}");
        if name == "Noggle Robber" {
            g.move_object(c, Zone::Graveyard);
            settle(&mut g, None);
            assert_eq!(tokens(&g, token), 2);
        }
    }
    let mut g = scenario();
    let c = cast_funded(&mut g, "Pretending Poxbearers");
    settle(&mut g, None);
    assert_eq!(tokens(&g, "Ally"), 0);
    g.move_object(c, Zone::Graveyard);
    settle(&mut g, None);
    assert_eq!(tokens(&g, "Ally"), 1);
}

#[test]
fn pond_prophet_draws_and_mary_jane_uses_limited_owned_spider_entries() {
    let mut g = scenario();
    cast_funded(&mut g, "Pond Prophet");
    settle(&mut g, None);
    assert_eq!(g.state.players[0].library.len(), 19);
    let mary = cast_funded(&mut g, "Mary Jane Watson");
    settle(&mut g, None);
    let opponent = named(&mut g, Player::P1, "Attercop", Zone::Hand);
    g.move_object(opponent, Zone::Battlefield);
    assert!(g.state.triggered_this_turn.is_empty());
    for _ in 0..2 {
        let own = named(&mut g, Player::P0, "Attercop", Zone::Hand);
        g.move_object(own, Zone::Battlefield);
    }
    settle(&mut g, None);
    assert_eq!(g.state.players[0].library.len(), 18);
    assert_eq!(
        g.state.triggered_this_turn,
        vec![TriggerUse {
            source: g.object_ref(mary),
            ability: 0
        }]
    );
}

#[test]
fn mob_lookout_connive_and_duskwatch_entry_targets_enforce_creature_relations() {
    let mut g = scenario();
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    cast_funded(&mut g, "Mob Lookout");
    pass_pair(&mut g);
    assert!(g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(bear)
    }));
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    pass_pair(&mut g);
    assert!(g.state.resolving.as_ref().unwrap().connive.is_some());
    assert_eq!(g.actor(), Some(Player::P0));
    settle(&mut g, None);
    assert_eq!(g.state.players[0].library.len(), 19);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    cast_funded(&mut g, "Duskwatch Hunter");
    settle(&mut g, Some(Target::Card(enemy)));
    assert_eq!(g.object_view(enemy).counters.plus_one, 1);
}

#[test]
fn abandon_attachments_discard_payment_is_private_optional_and_not_a_cast_cost() {
    for discard in [false, true] {
        let mut g = scenario();
        let hand = spawn(&mut g, Player::P0, Card::Forest, Zone::Hand);
        cast_funded(&mut g, "Abandon Attachments");
        pass_pair(&mut g);
        assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
        assert_eq!(g.state.players[0].library.len(), 20);
        if discard {
            replay(&mut g, Action::Discard { card: hand });
        } else {
            replay(&mut g, Action::DeclineEffectPayment);
        }
        assert_eq!(
            g.state.players[0].library.len(),
            if discard { 18 } else { 20 }
        );
        assert_eq!(
            g.object_view(hand).zone,
            if discard { Zone::Graveyard } else { Zone::Hand }
        );
    }
}

#[test]
fn hybrid_attack_bodies_keep_targets_and_source_filters() {
    let mut g = scenario();
    let own = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let foe = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let foot = cast_funded(&mut g, "Foot Elite");
    settle(&mut g, None);
    attack(&mut g, foot);
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(foot)
    }));
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(foe)
    }));
    settle(&mut g, Some(Target::Card(own)));
    assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(g.stats(own).power, 3);
    let mut g = scenario();
    let teddy = named(&mut g, Player::P0, "Friendly Teddy", Zone::Battlefield);
    g.state.objects[teddy as usize].tapped = true;
    let owl = cast_funded(&mut g, "Cat-Owl");
    settle(&mut g, None);
    attack(&mut g, owl);
    settle(&mut g, Some(Target::Card(teddy)));
    assert!(!g.object_view(teddy).tapped);
    let mut g = scenario();
    let own = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let music = cast_funded(&mut g, "Wandering Musicians");
    settle(&mut g, None);
    attack(&mut g, music);
    settle(&mut g, None);
    assert_eq!(g.stats(own).power, 3);
    let Effect::Creature(base, _) = g.state.objects[music as usize].card.effect() else {
        panic!("creature")
    };
    assert_eq!(g.stats(music).power, base + 1);
}

#[test]
fn tempest_angler_counts_completed_noncreature_casts_and_punk_frogs_ward_remains_generic() {
    let mut g = scenario();
    let angler = cast_funded(&mut g, "Tempest Angler");
    settle(&mut g, None);
    cast_funded(&mut g, "Abandon Attachments");
    settle(&mut g, None);
    assert_eq!(g.object_view(angler).counters.plus_one, 1);
    let frogs = cast_funded(&mut g, "Punk Frogs");
    settle(&mut g, None);
    g.state.priority = Player::P1;
    g.state.players[1].mana = [0, 1, 0, 0, 0, 0];
    let bounce = named(&mut g, Player::P1, "Unsummon", Zone::Hand);
    cast(&mut g, bounce, Some(Target::Card(frogs)));
    assert!(
        g.state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .any(|s| s.card == frogs)
    );
    pass_pair(&mut g);
    assert_eq!(
        g.observe(Player::P1).resolution_payment.unwrap().cost,
        ManaCost {
            colored: [0; 6],
            generic: 3
        }
    );
}

#[test]
fn synthetic_target_plan_waits_for_cost_then_targets_then_exact_payment() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Abandon Attachments", Zone::Hand);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
    replay(&mut g, Action::AnnounceCast { card: c });
    // Synthetic declaration IR only: no additional printed card is admitted.
    g.state.announcing.as_mut().unwrap().plan =
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    let size = g.state_bytes();
    choose(&mut g, [0, 1, 0, 0, 0, 0]);
    assert_eq!(g.state_bytes(), size);
    assert_eq!(g.state.pending, Pending::Targets { player: Player::P0 });
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 1],
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    assert_eq!(g.state.pending, Pending::PayCosts { player: Player::P0 });
    assert_eq!(
        g.state.announcing.as_ref().unwrap().cost,
        Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1
        })
    );
    pay(&mut g);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.stack[0].targets[0].target, Target::Card(bear));
}

#[test]
fn hybrid_funding_uses_mana_abilities_and_mixed_colors_without_borrowing_opponent_resources() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Gallant Citizen", Zone::Hand);
    let elf = named(&mut g, Player::P0, "Druid of the Cowl", Zone::Battlefield);
    g.state.objects[elf as usize].summoning_sick = false;
    let land = spawn(&mut g, Player::P0, Card::Plains, Zone::Battlefield);
    for _ in 0..2 {
        spawn(&mut g, Player::P1, Card::Forest, Zone::Battlefield);
    }
    replay(&mut g, Action::AnnounceCast { card: c });
    assert_eq!(g.observe(Player::P0).cost_choices.len(), 1);
    choose(&mut g, [1, 0, 0, 0, 1, 0]);
    replay(&mut g, Action::TapMana { card: land });
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==elf))
        .unwrap();
    replay(&mut g, a);
    pay(&mut g);
    settle(&mut g, None);
    assert_eq!(g.object_view(c).zone, Zone::Battlefield);
}

#[test]
fn all_admitted_printed_mana_costs_and_values_match_frozen_oracle_facts() {
    let snapshot: serde_json::Value =
        serde_json::from_str(include_str!("../data/standard/cards.json")).unwrap();
    for row in snapshot["cards"].as_array().unwrap() {
        let card = Card::from_name(row["name"].as_str().unwrap()).unwrap();
        if card.is_supported() {
            assert_eq!(
                card.info().mana_cost,
                row["mana_cost"].as_str().unwrap(),
                "{}",
                card.name()
            );
            assert_eq!(
                card.fixed_mana_value() as f64,
                row["cmc"].as_f64().unwrap(),
                "{}",
                card.name()
            );
        }
    }
}

#[test]
fn synthetic_modal_hybrid_declaration_commits_mode_then_cost_then_targets_then_payment() {
    let mut g = scenario();
    let c = named(&mut g, Player::P0, "Abandon Attachments", Zone::Hand);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
    replay(&mut g, Action::AnnounceCast { card: c });
    // Inject a synthetic modal declaration shape into the shared continuation.
    // Current complete printed hybrid spells have no modes. Production obtains
    // these mode/target facts from the immutable definition in choose_mode.
    g.continue_announcement_choices(true);
    assert_eq!(g.state.pending, Pending::Modes { player: Player::P0 });
    // Exercise the exact payment projection used by observe(actor), without
    // inventing a static printed modal definition just for this IR fixture.
    assert!(g.announcement_payment_request(Player::P0).is_none());
    assert!(g.announcement_payment_request(Player::P1).is_none());
    // Querying a mode list requires a real modal definition; this IR fixture
    // instead checks the routing state, then declares its explicit target plan.
    let saved = g.snapshot();
    let mut fork = g.clone();
    let plan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
    g.declare_mode(0, plan);
    fork.declare_mode(0, plan);
    assert_eq!(g.snapshot(), fork.snapshot());
    assert_eq!(g.state.pending, Pending::SpellCost { player: Player::P0 });
    assert!(g.announcement_payment_request(Player::P0).is_none());
    assert_eq!(g.state.announcing.as_ref().unwrap().item.mode, Some(0));
    let expected = g.snapshot();
    g.restore(&saved);
    g.declare_mode(0, plan);
    assert_eq!(g.snapshot(), expected);
    choose(&mut g, [0, 1, 0, 0, 0, 0]);
    assert_eq!(g.state.pending, Pending::Targets { player: Player::P0 });
    assert_eq!(
        g.announcement_payment_request(Player::P0).unwrap().required,
        [0, 1, 0, 0, 0, 0]
    );
    assert!(g.announcement_payment_request(Player::P1).is_none());
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    assert_eq!(g.state.pending, Pending::PayCosts { player: Player::P0 });
    pay(&mut g);
    assert_eq!(g.state.stack[0].mode, Some(0));
}
