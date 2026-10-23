//! Shared flexible-color capacity, with an independent native-action oracle.
use super::super::ability_timeline_tests::{named, setup};
use super::*;

fn brute(game: &Game, goal: &FundingGoal, seen: &mut HashSet<String>) -> bool {
    let pool = game.state.players[goal.actor.index()].mana;
    if payable(pool, goal.required, goal.generic) {
        return true;
    }
    let objects: Vec<_> = (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .collect();
    if !seen.insert(serde_json::to_string(&(pool, objects)).unwrap()) {
        return false;
    }
    for action in game
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
    {
        let mut child = game.clone();
        child.apply_legal(goal.actor, action);
        if brute(&child, goal, seen) {
            return true;
        }
    }
    false
}

#[test]
fn joint_color_funding_agrees_with_native_weather_and_converter_sequences() {
    for weather_count in 0..=2 {
        for converter_count in 0..=1 {
            let p = Player::P0;
            let mut game = setup(p);
            for _ in 0..2 {
                named(&mut game, p, "Forest", Zone::Battlefield);
            }
            for _ in 0..weather_count {
                named(&mut game, p, "Weather Maker", Zone::Battlefield);
            }
            for _ in 0..converter_count {
                named(&mut game, p, "Omni-Cheese Pizza", Zone::Battlefield);
            }
            game.state.players[p.index()].mana = [0; 6];
            for required in [[1, 1, 0, 0, 0, 0], [1, 0, 0, 1, 0, 0], [2, 1, 0, 0, 0, 0]] {
                for generic in 0..=1 {
                    let goal = FundingGoal {
                        actor: p,
                        required,
                        generic,
                        reserve: None,
                        source: None,
                    };
                    let before = game.snapshot();
                    let expected = brute(&game, &goal, &mut HashSet::new());
                    let world = FundingState::from_game(&game, &goal);
                    assert!(!expected || world.joint_colors_possible(&goal));
                    assert_eq!(
                        game.can_fund(goal),
                        expected,
                        "weather={weather_count} converters={converter_count} required={required:?} generic={generic}"
                    );
                    assert_eq!(game.snapshot(), before);
                }
            }
        }
    }
}

#[test]
fn joint_color_funding_dense_weather_and_converters_reject_without_dfs() {
    for p in [Player::P0, Player::P1] {
        let mut game = setup(p);
        for _ in 0..24 {
            named(&mut game, p, "Forest", Zone::Battlefield);
        }
        let weather = named(&mut game, p, "Weather Maker", Zone::Battlefield);
        game.add_counter(weather, CounterKind::Charge, 8);
        for _ in 0..2 {
            named(&mut game, p, "Omni-Cheese Pizza", Zone::Battlefield);
        }
        for _ in 0..12 {
            named(&mut game, p, "Skateboard", Zone::Battlefield);
        }
        game.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [1, 1, 1, 1, 1, 0],
            generic: 2,
            reserve: None,
            source: None,
        };
        let before = game.snapshot();
        let world = FundingState::from_game(&game, &goal);
        assert!(
            world.spendable_mana_upper().is_none(),
            "a converter must disable the previous closed-world bound"
        );
        let mut visited = HashSet::new();
        let start = std::time::Instant::now();
        assert!(!world.reachable(&goal, &mut visited));
        eprintln!(
            "TMT_JOINT dense39battlefield visited={} elapsed={:?}",
            visited.len(),
            start.elapsed()
        );
        assert!(visited.is_empty());
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn joint_color_funding_defers_dynamic_and_mana_bonus_worlds() {
    for name in ["Mona Lisa, Science Geek", "Groundchuck & Dirtbag"] {
        let p = Player::P0;
        let mut game = setup(p);
        named(&mut game, p, name, Zone::Battlefield);
        game.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [1, 1, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&game, &goal);
        assert!(
            world.joint_colors_possible(&goal),
            "unsupported bound domain must preserve exact search: {name}"
        );
    }
}

fn creature_purpose(exile_any_mana: bool) -> mana_lots::Purpose {
    let card = Card::from_name("Bear Cub").unwrap();
    mana_lots::Purpose::Spell(mana_lots::Source {
        card,
        kinds: card.kind_mask(),
        subtypes: GrantedSubtypes::default(),
        foreign_stash: false,
        exile_any_mana,
        mana_value: card.fixed_mana_value(),
    })
}

#[test]
fn joint_color_funding_preserves_convoke_and_any_mana_substitution() {
    let p = Player::P0;
    let goal = FundingGoal {
        actor: p,
        required: [1, 1, 0, 0, 0, 0],
        generic: 0,
        reserve: None,
        source: None,
    };
    let mut game = setup(p);
    named(&mut game, p, "Serra Angel", Zone::Battlefield);
    named(&mut game, p, "Donatello, Gadget Master", Zone::Battlefield);
    game.state.players[p.index()].mana = [0; 6];
    let mut world = FundingState::from_game_with_convoke(&game, &goal, true);
    world.purpose = creature_purpose(false);
    if let mana_lots::Purpose::Spell(source) = &mut world.purpose {
        source.card = Card::from_name("Disruptor of Currents").unwrap();
    }
    assert!(world.convoke_available());
    assert!(world.joint_colors_possible(&goal));
    assert!(world.reachable(&goal, &mut HashSet::new()));

    let mut game = setup(p);
    for _ in 0..2 {
        named(&mut game, p, "Forest", Zone::Battlefield);
    }
    game.state.players[p.index()].mana = [0; 6];
    let mut world = FundingState::from_game(&game, &goal);
    world.purpose = creature_purpose(true);
    assert!(world.joint_colors_possible(&goal));
    assert!(world.reachable(&goal, &mut HashSet::new()));

    named(&mut game, p, "Vizier of the Menagerie", Zone::Battlefield);
    let mut world = FundingState::from_game(&game, &goal);
    world.purpose = creature_purpose(false);
    assert!(world.joint_colors_possible(&goal));
    assert!(world.reachable(&goal, &mut HashSet::new()));
}

#[test]
fn joint_color_funding_preserves_attached_mana_and_control_worlds() {
    let p = Player::P0;
    let mut game = setup(p);
    let a = named(&mut game, p, "Forest", Zone::Battlefield);
    named(&mut game, p, "Forest", Zone::Battlefield);
    let aura = named(&mut game, p, "New Horizons", Zone::Battlefield);
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(a));
    game.state.players[p.index()].mana = [0; 6];
    let goal = FundingGoal {
        actor: p,
        required: [1, 0, 0, 0, 1, 0],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    assert!(world.joint_colors_possible(&goal));
    assert!(world.reachable(&goal, &mut HashSet::new()));

    let mut game = setup(p);
    named(&mut game, p, "Plains", Zone::Battlefield);
    let island = named(&mut game, p.other(), "Island", Zone::Battlefield);
    game.record_control(game.object_ref(island), p, None);
    game.refresh_controllers();
    game.state.players[p.index()].mana = [0; 6];
    let goal = FundingGoal {
        actor: p,
        required: [1, 1, 0, 0, 0, 0],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    assert!(world.joint_colors_possible(&goal));
    assert!(world.reachable(&goal, &mut HashSet::new()));
}

#[test]
fn joint_color_funding_defers_linked_return_and_repeatable_positive_production() {
    let p = Player::P0;
    let mut game = setup(p);
    let source = named(&mut game, p, "Omni-Cheese Pizza", Zone::Battlefield);
    let island = named(&mut game, p, "Island", Zone::Exile);
    // A generic CR610.3 link can release a resource during a mana sacrifice.
    // The existing linked-funding suite exercises paid reachable links.
    game.state.exile_links.push(ExileLink {
        source: game.object_ref(source),
        exiled: game.object_ref(island),
        return_zone: Zone::Battlefield,
    });
    game.state.players[p.index()].mana = [0; 6];
    let goal = FundingGoal {
        actor: p,
        required: [1, 1, 0, 0, 0, 0],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    assert!(world.joint_colors_possible(&goal));

    // Explicit future-cost IR proof case, not a new admitted physical card:
    // a repeatable positive converter has no finite one-use upper bound.
    let mut ability = Card::from_name("Omni-Cheese Pizza").unwrap().activations()[0];
    ability.cost.tap = false;
    ability.cost.sacrifice = false;
    ability.once_per_turn = false;
    ability.program = &[Instruction::AddMana(ManaProduction::Choice {
        mask: 31,
        amount: 2,
    })];
    assert_eq!(
        activation_gain_upper(
            &ability,
            ability.mana_production().unwrap(),
            &Counters::default(),
            20,
            |cost| cost
        ),
        u128::MAX
    );
    assert!(FundingState::finite_joint_production(&ability).is_none());
}
