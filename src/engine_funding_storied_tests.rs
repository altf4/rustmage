//! Storied gained during realizable mana funding must enable later mana moves.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

fn act(g: &mut Game, action: Action) {
    g.apply(g.actor().unwrap(), action).unwrap();
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, action);
}
fn settle(g: &mut Game) {
    for _ in 0..32 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("resolution budget exhausted");
}
fn cast(g: &mut Game, id: CardId) {
    act(g, Action::AnnounceCast { card: id });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    g.state.players[p.index()].mana = [12; 6];
    let id = named(g, p, name, Zone::Hand);
    cast(g, id);
    settle(g);
    id
}
fn equip(g: &mut Game, ring: CardId, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: ring,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    settle(g);
}
struct Fixture {
    game: Game,
    oin: CardId,
    wasp: CardId,
    rings: Vec<CardId>,
}
fn fixture(p: Player, second_ring: bool) -> Fixture {
    let mut g = setup(p);
    let oin = paid(&mut g, p, "Óin the Brave");
    let wasp = named(&mut g, p, "The Wondrous Wasp", Zone::Hand);
    cast(&mut g, wasp);
    let target = Target::Card(oin);
    for _ in 0..24 {
        if g.legal_actions().contains(&Action::ChooseTarget { target }) {
            act(&mut g, Action::ChooseTarget { target });
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            break;
        }
        act(&mut g, Action::Pass);
    }
    settle(&mut g);
    assert!(!g.printed_abilities_available(oin));
    let first = paid(&mut g, p, "Lotus Ring");
    equip(&mut g, first, wasp);
    let mut rings = vec![first];
    if second_ring {
        let second = paid(&mut g, p, "Lotus Ring");
        equip(&mut g, second, oin);
        rings.push(second);
    }
    // Wasp's entry tapped Oin. Untap it by an effect while retaining sickness,
    // so the tested missing permission is haste rather than an unrelated tap.
    g.untap_objects(&[oin], untap_rules::UntapCause::Effect);
    assert!(!g.object_view(oin).tapped);
    g.state.objects[wasp as usize].summoning_sick = false;
    g.state.players[p.index()].mana = [0; 6];
    assert!(g.state.objects[oin as usize].summoning_sick);
    assert!(!g.state.players[p.index()].enduring_story);
    Fixture {
        game: g,
        oin,
        wasp,
        rings,
    }
}
fn goal(p: Player, amount: u32) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0, 0, 0, amount, 0, 0],
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn hob_storied_funding_new_designation_enables_second_sick_mana_source() {
    for p in [Player::P0, Player::P1] {
        let Fixture { game: g, .. } = fixture(p, true);
        let saved = g.snapshot();
        assert!(
            g.can_fund(goal(p, 6)),
            "sacrifice Wasp, acquire enduring story, then haste permits Oin"
        );
        assert_eq!(
            g.snapshot(),
            saved,
            "funding cannot mutate the real designation"
        );
    }
}

fn tap(g: &Game, source: CardId, ring: CardId) -> Action {
    Action::TapGrantedMana {
        card: source,
        grantor: g.object_ref(ring),
        production: [0, 0, 0, 3, 0, 0],
    }
}
fn projected_step<'g>(
    world: &FundingState<'g>,
    goal: &FundingGoal,
    source: CardId,
) -> FundingState<'g> {
    let step = world
        .moves(goal)
        .into_iter()
        .find(|step| {
            world.resources[step.resource].id == source && step.production == [0, 0, 0, 3, 0, 0]
        })
        .expect("realizable three-red mana move");
    let mut result = None;
    world.visit_applied(&step, &mut |next| {
        result = Some(next.clone());
        true
    });
    result.expect("completed projected mana step")
}
fn make_nonlegendary(g: &mut Game, id: CardId) {
    // Existing copiable nonlegendary exception, preserving the same typed abilities.
    let object = g.object_ref(id);
    let mut recipe = g.copiable_recipe(object);
    recipe.overrides = Some(CopyOverrides {
        legendary: Some(false),
        ..CopyOverrides::NONE
    });
    g.set_copy_recipe(object, recipe);
    g.refresh_controllers();
    assert!(!CharacteristicWorld::legendary(g, id));
}

#[test]
fn hob_storied_funding_projection_persists_and_keys_history_after_source_loss() {
    for p in [Player::P0, Player::P1] {
        let Fixture {
            game: g, oin, wasp, ..
        } = fixture(p, true);
        let saved = g.snapshot();
        let goal = goal(p, 6);
        let world = FundingState::from_game(&g, &goal);
        assert!(!world.has_enduring_story(p));
        let after_wasp = projected_step(&world, &goal, wasp);
        assert!(after_wasp.has_enduring_story(p));
        assert!(!after_wasp.has_enduring_story(p.other()));
        assert!(after_wasp.value(StatValue::HasEnduringStory, oin) != 0);
        assert_eq!(
            after_wasp.value(
                StatValue::Sum(&[StatValue::HasEnduringStory, StatValue::Fixed(2)]),
                oin
            ),
            3
        );
        assert_eq!(
            after_wasp.value(StatValue::Not(&StatValue::HasEnduringStory), oin),
            0
        );
        assert_eq!(
            after_wasp.value(
                StatValue::AtLeast {
                    value: &StatValue::HasEnduringStory,
                    minimum: 1
                },
                oin
            ),
            1
        );
        let query = characteristic_query::CharacteristicQuery::in_world(&after_wasp);
        assert_eq!(query.stats(oin).power, 5, "world-aware conditional power");
        assert!(query.keywords(oin).contains(Keywords::HASTE));
        assert_eq!(
            ManaProduction::TapSourcePowerChoice { mask: 8 }.in_world(&after_wasp, oin),
            ManaProduction::Choice { mask: 8, amount: 5 },
            "SourceTappedWorld forwards the projected designation"
        );
        let after_oin = projected_step(&after_wasp, &goal, oin);
        assert_eq!(after_oin.object(oin).zone, Zone::Graveyard);
        assert!(
            after_oin.has_enduring_story(p),
            "designation outlives its last source and threshold"
        );
        let mut no_history = after_oin.clone();
        no_history.enduring_story[p.index()] = false;
        assert!(
            after_oin.key(&goal) != no_history.key(&goal),
            "memo distinguishes otherwise identical worlds by designation history"
        );
        assert_eq!(g.snapshot(), saved);
        assert!(!g.observe(p).players[p.index()].enduring_story);
        assert!(!g.observe(p.other()).players[p.index()].enduring_story);
    }
}

#[test]
fn hob_storied_funding_two_qualifiers_do_not_award_but_existing_designation_survives() {
    for p in [Player::P0, Player::P1] {
        for already in [false, true] {
            let Fixture {
                mut game,
                oin,
                wasp,
                rings,
            } = fixture(p, true);
            make_nonlegendary(&mut game, oin);
            if already {
                // A separate live Storied source awards the real designation,
                // then leaves. Oin is still suppressed and summoning sick.
                let bombur = paid(&mut game, p, "Bombur, Gentle Dreamer");
                assert!(game.state.players[p.index()].enduring_story);
                game.move_object(bombur, Zone::Exile);
                game.state.players[p.index()].mana = [0; 6];
            }
            let saved = game.snapshot();
            assert!(game.can_fund(goal(p, 3)));
            assert_eq!(game.can_fund(goal(p, 6)), already);
            assert_eq!(game.snapshot(), saved);
            let world = FundingState::from_game(&game, &goal(p, 6));
            assert_eq!(world.has_enduring_story(p), already);
            let after = projected_step(&world, &goal(p, 6), wasp);
            assert_eq!(
                after.has_enduring_story(p),
                already,
                "only two Rings qualify after Wasp leaves"
            );
            let action = tap(&game, wasp, rings[0]);
            act(&mut game, action);
            assert_eq!(game.state.players[p.index()].enduring_story, already);
            assert_eq!(
                game.object_view(oin).keywords.contains(Keywords::HASTE),
                already
            );
            let second = tap(&game, oin, rings[1]);
            assert_eq!(game.legal_actions().contains(&second), already);
        }
    }
}

#[test]
fn hob_storied_funding_actual_six_mana_matches_raw_index_children_and_restore() {
    for p in [Player::P0, Player::P1] {
        let Fixture {
            game,
            oin,
            wasp,
            rings,
        } = fixture(p, true);
        let saved = game.snapshot();
        let actions = [tap(&game, wasp, rings[0]), tap(&game, oin, rings[1])];
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        for action in &actions {
            let revision = indexed.revision();
            let index = indexed
                .legal_actions()
                .iter()
                .position(|a| a == action)
                .unwrap();
            act(&mut raw, action.clone());
            indexed.apply_index(p, revision, index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert!(raw.observe(p).players[p.index()].enduring_story);
            assert!(raw.observe(p.other()).players[p.index()].enduring_story);
        }
        assert_eq!(raw.state.players[p.index()].mana, [0, 0, 0, 6, 0, 0]);
        assert_eq!(raw.object_view(oin).zone, Zone::Graveyard);
        indexed.restore(&saved).unwrap();
        assert!(!indexed.state.players[p.index()].enduring_story);
        for action in actions {
            act(&mut indexed, action);
        }
        assert_eq!(indexed.snapshot(), raw.snapshot());
    }
}
