//! The negative bound must never reject a payable cost; unknown worlds fall back.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

fn context(game: &Game, actor: Player, source: CardId, kind: EffectChoiceKind) -> CostContext {
    let mut additional = game.state.objects[source as usize]
        .card
        .additional_cost()
        .unwrap();
    additional.kind = kind;
    CostContext {
        actor,
        source: game.object_ref(source),
        source_counter_reservation: Counters::default(),
        clauses: vec![CastCostRow {
            clause: CastCostClause::Additional,
            paid: false,
        }],
        active: None,
        casualty: Vec::new(),
        additional: Some(additional),
        additional_life: false,
        forage: None,
        selected: Vec::new(),
        exact: false,
        counters: Vec::new(),
        mana: None,
        teammates: Vec::new(),
        teamwork_exact: false,
    }
}

fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        source: None,
        reserve: None,
    }
}

#[test]
fn additional_bounds_cover_distinct_resource_costs_and_never_reject_payable_choices() {
    for actor in [Player::P0, Player::P1] {
        for (hand, battlefield, graveyard, life) in [
            (0, 0, 0, 2),
            (1, 0, 0, 2),
            (0, 1, 0, 2),
            (0, 0, 2, 2),
            (0, 0, 0, 3),
        ] {
            let mut game = setup(actor);
            game.state.players[actor.index()].life = life;
            let source = named(&mut game, actor, "Bitter Triumph", Zone::Hand);
            for _ in 0..hand {
                named(&mut game, actor, "Island", Zone::Hand);
            }
            for _ in 0..battlefield {
                named(&mut game, actor, "Bear Cub", Zone::Battlefield);
            }
            for _ in 0..graveyard {
                named(&mut game, actor, "Bear Cub", Zone::Graveyard);
            }
            let kinds = [
                EffectChoiceKind::Discard,
                EffectChoiceKind::DiscardNamed("Island"),
                EffectChoiceKind::DiscardOrLife(3),
                EffectChoiceKind::Sacrifice,
                EffectChoiceKind::SacrificeOrDiscard { nonland: false },
                EffectChoiceKind::TapPermanents(2),
                EffectChoiceKind::SacrificeDifferentNames(2),
                EffectChoiceKind::ExileGraveyard,
                EffectChoiceKind::ExileGraveyardCards(2),
                EffectChoiceKind::CollectEvidence(3),
                EffectChoiceKind::RemoveCreatureCounters(1),
            ];
            for kind in kinds {
                let mut world = FundingState::from_game(&game, &goal(actor));
                world.install_cost_context(Some(Box::new(context(&game, actor, source, kind))));
                let deficit = world.additional_cost_deficit();
                if deficit {
                    assert!(!world.cost_completion(), "false negative for {kind:?}");
                }
                let expected = match kind {
                    EffectChoiceKind::Discard | EffectChoiceKind::DiscardNamed(_) => hand == 0,
                    EffectChoiceKind::DiscardOrLife(_) => hand == 0 && life < 3,
                    EffectChoiceKind::Sacrifice => battlefield == 0,
                    EffectChoiceKind::SacrificeOrDiscard { .. } => battlefield == 0 && hand == 0,
                    EffectChoiceKind::TapPermanents(_)
                    | EffectChoiceKind::SacrificeDifferentNames(_) => battlefield < 2,
                    EffectChoiceKind::ExileGraveyard => battlefield + graveyard == 0,
                    EffectChoiceKind::ExileGraveyardCards(_) => battlefield + graveyard < 2,
                    EffectChoiceKind::CollectEvidence(_) => battlefield + graveyard == 0,
                    EffectChoiceKind::RemoveCreatureCounters(_) => battlefield == 0,
                    _ => unreachable!(),
                };
                assert_eq!(deficit, expected, "wrong bound for {kind:?}");
            }
        }
    }
}

#[test]
fn additional_bounds_handle_combined_costs_and_preserve_unknowns_and_linked_returns() {
    for actor in [Player::P0, Player::P1] {
        let mut game = setup(actor);
        game.state.players[actor.index()].life = 2;
        let source = named(&mut game, actor, "Bitter Triumph", Zone::Hand);
        let mut world = FundingState::from_game(&game, &goal(actor));
        world.install_cost_context(Some(Box::new(context(
            &game,
            actor,
            source,
            EffectChoiceKind::Discard,
        ))));
        assert!(world.additional_cost_deficit());
        world
            .cast_costs
            .as_mut()
            .unwrap()
            .clauses
            .push(CastCostRow {
                clause: CastCostClause::Life(1),
                paid: false,
            });
        assert!(world.additional_cost_deficit());
        world.cast_costs.as_mut().unwrap().clauses.pop();
        world
            .cast_costs
            .as_mut()
            .unwrap()
            .additional
            .as_mut()
            .unwrap()
            .kind = EffectChoiceKind::HandToTop;
        assert!(!world.additional_cost_deficit());
        world
            .cast_costs
            .as_mut()
            .unwrap()
            .additional
            .as_mut()
            .unwrap()
            .kind = EffectChoiceKind::ForageOrBlack;
        assert!(!world.additional_cost_deficit());

        let prison = named(&mut game, actor, "Bolg's Company", Zone::Battlefield);
        let exiled = named(&mut game, actor, "Island", Zone::Exile);
        game.state.exile_links.push(ExileLink {
            source: game.object_ref(prison),
            exiled: game.object_ref(exiled),
            return_zone: Zone::Hand,
        });
        let mut world = FundingState::from_game(&game, &goal(actor));
        world.install_cost_context(Some(Box::new(context(
            &game,
            actor,
            source,
            EffectChoiceKind::Discard,
        ))));
        assert!(!world.additional_cost_deficit());
    }
}

#[test]
fn funding_can_sacrifice_a_permanent_then_pay_a_graveyard_cost() {
    for actor in [Player::P0, Player::P1] {
        let mut game = setup(actor);
        game.state.players[actor.index()].mana = [0; 6];
        let source = named(&mut game, actor, "Bitter Triumph", Zone::Hand);
        named(&mut game, actor, "Bolg's Company", Zone::Battlefield);
        named(&mut game, actor, "Goblin Piker", Zone::Battlefield);
        let card = game.state.objects[source as usize].card;
        let mut cost = card.additional_cost().unwrap();
        cost.kind = EffectChoiceKind::ExileGraveyardCards(1);
        let request = FundingGoal {
            actor,
            required: [0, 0, 1, 1, 0, 0],
            generic: 0,
            source: None,
            reserve: None,
        };
        let before = game.snapshot();
        assert_eq!(
            game.preview_optional_cast_costs(
                request,
                game.spell_mana_purpose_for_card(actor, source, card),
                game.object_ref(source),
                Some(cost),
                None,
                &[],
                false,
                &[],
                &[CastCostClause::Additional],
            ),
            Some(true),
        );
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn group_cost_requires_distinct_eligible_resources_and_preserves_state() {
    for actor in [Player::P0, Player::P1] {
        let mut game = setup(actor);
        let source = named(&mut game, actor, "Bitter Triumph", Zone::Hand);
        let mut spec = game.state.objects[source as usize]
            .card
            .additional_cost()
            .unwrap();
        spec.kind = EffectChoiceKind::TapPermanents(3);
        spec.filter.kinds = 1;
        named(&mut game, actor, "Bear Cub", Zone::Battlefield);
        named(&mut game, actor, "Bear Cub", Zone::Battlefield);
        named(&mut game, actor.other(), "Bear Cub", Zone::Battlefield);
        named(&mut game, actor, "Forest", Zone::Battlefield);
        let before = game.snapshot();
        assert!(!game.group_cost_fundable(
            goal(actor),
            mana_lots::Purpose::Other,
            spec,
            game.object_ref(source),
            &[]
        ));
        assert_eq!(game.snapshot(), before);
        let third = named(&mut game, actor, "Bear Cub", Zone::Battlefield);
        let before = game.snapshot();
        assert!(game.group_cost_fundable(
            goal(actor),
            mana_lots::Purpose::Other,
            spec,
            game.object_ref(source),
            &[]
        ));
        assert_eq!(game.snapshot(), before);
        game.state.objects[third as usize].tapped = true;
        assert!(!game.group_cost_fundable(
            goal(actor),
            mana_lots::Purpose::Other,
            spec,
            game.object_ref(source),
            &[]
        ));
    }
}

#[test]
fn group_cost_rejects_missing_third_creature_before_funding() {
    let actor = Player::P0;
    let mut game = setup(actor);
    game.state.players[actor.index()].mana = [0; 6];
    let source = named(&mut game, actor, "Bitter Triumph", Zone::Hand);
    let mut spec = game.state.objects[source as usize]
        .card
        .additional_cost()
        .unwrap();
    spec.kind = EffectChoiceKind::TapPermanents(3);
    spec.filter.kinds = 1;
    for _ in 0..2 {
        named(&mut game, actor, "Bear Cub", Zone::Battlefield);
    }
    for _ in 0..5 {
        named(&mut game, actor, "Starting Town", Zone::Battlefield);
    }
    let request = FundingGoal {
        generic: 4,
        ..goal(actor)
    };
    let before = game.snapshot();
    let iterations = 100;
    let started = std::time::Instant::now();
    for _ in 0..iterations {
        assert!(game.can_fund_for(request.clone(), mana_lots::Purpose::Other));
    }
    let funding = started.elapsed();
    let started = std::time::Instant::now();
    for _ in 0..iterations {
        assert!(!game.group_cost_fundable(
            request.clone(),
            mana_lots::Purpose::Other,
            spec,
            game.object_ref(source),
            &[]
        ));
    }
    eprintln!(
        "group-cost sample: {iterations} initial mana searches {funding:?}; \
        {iterations} bounded group queries {:?}",
        started.elapsed()
    );
    assert_eq!(game.snapshot(), before);
}

#[test]
fn group_cost_handles_graveyard_prefixes_and_activation_source_reservation() {
    for actor in [Player::P0, Player::P1] {
        let mut game = setup(actor);
        let spell = named(&mut game, actor, "Bitter Triumph", Zone::Hand);
        let mut spec = game.state.objects[spell as usize]
            .card
            .additional_cost()
            .unwrap();
        spec.kind = EffectChoiceKind::ExileGraveyardCards(2);
        let first = named(&mut game, actor, "Bear Cub", Zone::Graveyard);
        named(&mut game, actor.other(), "Bear Cub", Zone::Graveyard);
        let source = game.object_ref(spell);
        let prefix = [game.object_ref(first)];
        let before = game.snapshot();
        assert!(!game.group_cost_fundable(
            goal(actor),
            mana_lots::Purpose::Other,
            spec,
            source,
            &prefix
        ));
        assert_eq!(game.snapshot(), before);
        named(&mut game, actor, "Bear Cub", Zone::Graveyard);
        let before = game.snapshot();
        assert!(game.group_cost_fundable(
            goal(actor),
            mana_lots::Purpose::Other,
            spec,
            source,
            &prefix
        ));
        assert_eq!(game.snapshot(), before);

        let ability = named(
            &mut game,
            actor,
            "Lathril, Blade of the Elves",
            Zone::Battlefield,
        );
        game.state.objects[ability as usize].summoning_sick = false;
        let definition = game.state.objects[ability as usize].card;
        let source = game.object_ref(ability);
        let activation_goal = FundingGoal {
            source: Some((source, 0, Counters::default(), definition, None)),
            ..goal(actor)
        };
        spec.kind = EffectChoiceKind::TapPermanents(1);
        spec.filter.kinds = 1;
        let before = game.snapshot();
        assert!(!game.group_cost_fundable(
            activation_goal.clone(),
            mana_lots::Purpose::Other,
            spec,
            source,
            &[]
        ));
        assert_eq!(game.snapshot(), before);
        named(&mut game, actor, "Bear Cub", Zone::Battlefield);
        let before = game.snapshot();
        assert!(game.group_cost_fundable(
            activation_goal,
            mana_lots::Purpose::Other,
            spec,
            source,
            &[]
        ));
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn creature_counter_selection_preserves_reserved_loyalty_payment() {
    for actor in [Player::P0, Player::P1] {
        let mut game = setup(actor);
        let source = named(&mut game, actor, "Silence the Echo", Zone::Battlefield);
        let mut context = context(&game, actor, source, EffectChoiceKind::RemoveCreatureCounters(1));
        game.state.objects[source as usize].card = Card::from_name("Bear Cub").unwrap();
        game.state.objects[source as usize].counters.loyalty = 1;
        context.source_counter_reservation = Counters::of(CounterKind::Loyalty, 1);
        context.counters = vec![(game.object_ref(source), CounterKind::Loyalty)];
        let run = |g: &Game, c: CostContext| {
            let mut world = FundingState::from_game(g, &goal(actor));
            world.install_cost_context(Some(Box::new(c)));
            world.cost_completion()
        };
        assert!(!run(&game, context.clone()));
        game.state.objects[source as usize].counters.loyalty = 2;
        assert!(run(&game, context.clone()));
        game.state.objects[source as usize].counters.loyalty = 1;
        let other = named(&mut game, actor, "Bear Cub", Zone::Battlefield);
        game.state.objects[other as usize].counters.plus_one = 1;
        context.counters = vec![(game.object_ref(other), CounterKind::PlusOne)];
        assert!(run(&game, context));
    }
}
