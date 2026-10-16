use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.outcome().is_some()
            || (g.state.stack.is_empty() && g.state.pending == Pending::Priority)
        {
            return;
        }
        let p = g.actor().unwrap();
        if g.observe(p).library_choice.is_some() {
            assert!(g.observe(p.other()).library_choice.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::ChooseSearch { .. }))
            .cloned()
            .or_else(|| {
                actions.into_iter().find(|a| match a {
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    } => target.is_none_or(|t| t == *id),
                    Action::ChooseTarget {
                        target: Target::Player(who),
                    } => *who == p.other(),
                    Action::ChooseSearch { .. }
                    | Action::FinishSearch
                    | Action::PutTrigger { .. }
                    | Action::Pass
                    | Action::ArrangeLibrary { .. }
                    | Action::Sacrifice { .. }
                    | Action::FinishTargets
                    | Action::ChooseOptional { accept: true } => true,
                    _ => false,
                })
            })
            .expect("fixture resolution choice");
        replay(g, a);
    }
    panic!("not settled");
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability:a,target:None,..}if *card==id&&*a==ability)).expect("fixture activation");
    replay(g, a);
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<ObjectView> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.owner == p
                && o.card.is_token()
                && o.card.has_subtype(subtype)
        })
        .map(|(i, _)| g.object_view(i as CardId))
        .collect()
}
fn hit(g: &Game, id: CardId, p: Player, amount: i128, combat: bool) -> DamageEvent {
    DamageEvent {
        source: g.damage_source(id),
        target: Target::Player(p),
        amount,
        combat,
    }
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|s| s.card == id)
        .count()
}
fn combat(g: &mut Game, ids: &[CardId], p: Player) {
    for &id in ids {
        g.state.attackers.push(Attacker {
            defender: Some(CombatDefender::Player(p.other())),
            card: id,
            blocked: false,
        });
    }
    g.state.step = Step::Blockers;
    g.give_priority();
    pass_pair(g);
}
fn ward(g: &mut Game, p: Player) -> (CardId, CardId) {
    let source = named(g, p.other(), "The Serpent Society", Zone::Battlefield);
    let spell = named(g, p, "Shock", Zone::Hand);
    cast(g, spell, Some(Target::Card(source)));
    assert_eq!(g.state.stack.len(), 2);
    (spell, source)
}
#[test]
fn all_four_bodies_cast_with_complete_stats_keywords_legend_and_clue_entry() {
    for p in [Player::P0, Player::P1] {
        for (name, pt, dt, legend) in [
            ("Fynn, the Fangbearer", (1, 3), true, true),
            ("Persuasive Interrogators", (5, 6), false, false),
            ("The Serpent Society", (3, 4), true, true),
            ("Virulent Silencer", (2, 3), false, false),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, None);
            settle(&mut g, None);
            let o = g.object_view(id);
            assert_eq!((o.power, o.toughness), pt);
            assert_eq!(o.keywords.contains(Keywords::DEATHTOUCH), dt);
            assert_eq!(o.card.is_legendary(), legend);
            assert_eq!(o.card.is_artifact(), name == "Virulent Silencer");
            assert_eq!(
                tokens(&g, p, "Clue").len(),
                usize::from(name == "Persuasive Interrogators")
            );
            for viewer in [p, p.other()] {
                let o = g.observe(viewer);
                assert_eq!(o.schema, 50);
                assert_eq!(o.players[0].poison_counters, 0);
                assert_eq!(o.players[1].poison_counters, 0);
            }
        }
    }
}
#[test]
fn fynn_filters_frozen_deathtouch_controller_combat_and_positive_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Fynn, the Fangbearer", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Pirate Peddlers", Zone::Battlefield);
        for (id, amount, combat) in [
            (source, 0, true),
            (source, 1, false),
            (own, 1, true),
            (foreign, 1, true),
        ] {
            g.player_damage_triggers(&[hit(&g, id, p.other(), amount, combat)]);
            assert_eq!(count(&g, source), 0);
        }
        g.state.objects[own as usize].gained_keywords = Keywords::DEATHTOUCH;
        let events = [
            hit(&g, own, p.other(), 1, true),
            hit(&g, own, p.other(), 5, true),
        ];
        g.state.objects[own as usize].gained_keywords = Keywords::NONE;
        g.player_damage_triggers(&events);
        assert_eq!(count(&g, source), 1);
        assert_eq!(g.state.pending_triggers[0].damage_event.unwrap().amount, 6);
        assert!(g.state.pending_triggers[0].targets.is_empty());
        settle(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 2);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life_lost_this_turn, 0);
    }
}
#[test]
fn silencer_filters_nontoken_artifact_creatures_and_uses_exact_damaged_player() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Virulent Silencer", Zone::Battlefield);
        let artifact = named(&mut g, p, "Gravestone Strider", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Gravestone Strider", Zone::Battlefield);
        let factory = Card::catalogue()
            .find(|c| c.is_token() && c.is_artifact() && c.is_creature())
            .unwrap();
        g.create_tokens(p, factory, 1, false);
        let token = g
            .state
            .objects
            .iter()
            .position(|o| o.zone == Zone::Battlefield && o.card == factory)
            .unwrap() as CardId;
        for (id, combat) in [
            (bear, true),
            (foreign, true),
            (token, true),
            (artifact, false),
        ] {
            g.player_damage_triggers(&[hit(&g, id, p.other(), 1, combat)]);
            assert_eq!(count(&g, source), 0);
        }
        // The printed recipient is a player, even when that player is the controller.
        g.player_damage_triggers(&[hit(&g, artifact, p, 10, true)]);
        assert_eq!(count(&g, source), 1);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].poison_counters, 2);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 0);
    }
}
#[test]
fn actual_combat_poison_uses_stack_and_reaches_ten_after_source_departure() {
    for p in [Player::P0, Player::P1] {
        for (name, damage) in [("Fynn, the Fangbearer", 1), ("Virulent Silencer", 2)] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Battlefield);
            g.state.players[p.other().index()].poison_counters = 8;
            combat(&mut g, &[id], p);
            assert_eq!(g.state.players[p.other().index()].life, 20 - damage);
            assert_eq!(g.state.players[p.other().index()].poison_counters, 8);
            assert_eq!(g.outcome(), None);
            assert_eq!(g.state.stack.len(), 1);
            assert_eq!(g.state.stack[0].damage_event.unwrap().player, p.other());
            g.move_object(id, Zone::Graveyard);
            settle(&mut g, None);
            assert_eq!(g.state.players[p.other().index()].poison_counters, 10);
            assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
            assert!(g.actor().is_none());
            for viewer in [p, p.other()] {
                assert_eq!(
                    g.observe(viewer).players[p.other().index()].poison_counters,
                    10
                );
                assert!(g.observe(viewer).actions.is_empty());
            }
        }
    }
}
#[test]
fn multiple_fynn_combat_dealers_trigger_once_each_instead_of_once_per_batch_or_damage_point() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fynn = named(&mut g, p, "Fynn, the Fangbearer", Zone::Battlefield);
        let pirate = named(&mut g, p, "Pirate Peddlers", Zone::Battlefield);
        combat(&mut g, &[fynn, pirate], p);
        assert_eq!(g.state.ordering_triggers.len(), 2);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 4);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
#[test]
fn interrogators_clue_sacrifice_queues_poison_above_private_draw_and_requires_a_clue() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Persuasive Interrogators", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g, None);
        let wrong = named(&mut g, p, "Nutrient Block", Zone::Battlefield);
        g.sacrifice_batch(&[(p, wrong)]);
        assert_eq!(count(&g, source), 0);
        settle(&mut g, None);
        let id = tokens(&g, p, "Clue")[0].id;
        activate(&mut g, id, 0);
        assert_eq!(g.object_view(id).zone, Zone::Outside);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 0);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        assert_eq!(g.state.stack.len(), 2);
        let draws = g.state.players[p.index()].cards_drawn_this_turn;
        pass_pair(&mut g);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 1);
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.owner != p || o.zone != Zone::Hand)
        );
    }
}
#[test]
fn poison_ward_is_an_owned_optional_cost_and_can_be_paid_even_when_it_loses_the_game() {
    for p in [Player::P0, Player::P1] {
        for initial in [0, 4, 5, 9] {
            for accept in [false, true] {
                let mut g = setup(p);
                g.state.players[p.index()].poison_counters = initial;
                let (spell, target) = ward(&mut g, p);
                pass_pair(&mut g);
                assert_eq!(g.actor(), Some(p));
                assert_eq!(g.state.players[p.index()].poison_counters, initial);
                assert!(g.observe(p.other()).actions.is_empty());
                reject(&mut g, Action::DeclineResolutionMana);
                replay(&mut g, Action::ChooseOptional { accept });
                assert_eq!(
                    g.state.players[p.index()].poison_counters,
                    initial + if accept { 5 } else { 0 }
                );
                assert_eq!(g.state.players[p.other().index()].poison_counters, 0);
                assert_eq!(g.state.players[p.index()].life, 20);
                assert_eq!(g.state.players[p.index()].life_lost_this_turn, 0);
                assert_eq!(
                    g.object_view(spell).zone,
                    if accept { Zone::Stack } else { Zone::Graveyard }
                );
                assert_eq!(g.object_view(target).damage, 0);
                if accept && initial >= 5 {
                    assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
                } else {
                    assert_eq!(g.outcome(), None);
                    settle(&mut g, None);
                    assert_eq!(g.object_view(target).damage, if accept { 2 } else { 0 });
                }
            }
        }
    }
}
#[test]
fn poison_ward_still_charges_after_source_leaves_and_decline_uses_exact_related_item() {
    for accept in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let (spell, target) = ward(&mut g, p);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        pass_pair(&mut g);
        replay(&mut g, Action::ChooseOptional { accept });
        assert_eq!(
            g.state.players[0].poison_counters,
            if accept { 5 } else { 0 }
        );
        settle(&mut g, None);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).damage, 0);
    }
}
#[test]
fn society_uses_frozen_keyword_deaths_and_forces_only_opponents_nontoken_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "The Serpent Society", Zone::Battlefield);
        let no = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(no, Zone::Graveyard);
        assert_eq!(count(&g, source), 0);
        let foreign = named(&mut g, p.other(), "Pirate Peddlers", Zone::Battlefield);
        g.move_object(foreign, Zone::Graveyard);
        assert_eq!(count(&g, source), 0);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[own as usize].gained_keywords = Keywords::DEATHTOUCH;
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].gained_keywords =
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        let factory = Card::catalogue()
            .find(|c| c.is_token() && c.is_creature())
            .unwrap();
        g.create_tokens(p.other(), factory, 1, false);
        let token = g
            .state
            .objects
            .iter()
            .position(|o| o.zone == Zone::Battlefield && o.card == factory)
            .unwrap() as CardId;
        g.sacrifice_batch(&[(p, source), (p, own)]);
        assert_eq!(count(&g, source), 1);
        g.prepare_triggers();
        pass_pair(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        let object = g.object_ref(token);
        reject(&mut g, Action::Sacrifice { object });
        let object = g.object_ref(victim);
        replay(&mut g, Action::Sacrifice { object });
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.object_view(token).zone, Zone::Battlefield);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn poison_is_persistent_public_and_state_based_even_without_a_source_and_simultaneous_losses_draw()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].poison_counters = 9;
        g.begin_turn();
        assert_eq!(g.state.players[p.index()].poison_counters, 9);
        assert_eq!(g.observe(p.other()).players[p.index()].poison_counters, 9);
        let saved = g.snapshot();
        let mut f = g.clone();
        g.state.players[p.index()].poison_counters = 10;
        g.state_based_actions();
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
        g.restore(&saved).unwrap();
        assert_eq!(g.outcome(), None);
        assert_eq!(g.snapshot(), f.snapshot());
        for b in [&mut g, &mut f] {
            b.state.players[0].poison_counters = 10;
            b.state.players[1].poison_counters = 11;
            b.state_based_actions();
            assert_eq!(b.outcome(), Some(Outcome::Draw));
        }
        assert_eq!(g.snapshot(), f.snapshot());
    }
    let mut g = setup(Player::P0);
    g.state.players[0].poison_counters = 10;
    g.state.players[1].life = 0;
    g.state_based_actions();
    assert_eq!(g.outcome(), Some(Outcome::Draw));
}
#[test]
fn poison_capacity_failure_rolls_back_an_owned_ward_payment_and_pending_frame() {
    let p = Player::P0;
    let mut g = setup(p);
    ward(&mut g, p);
    pass_pair(&mut g);
    g.state.players[0].poison_counters = u64::MAX;
    let saved = g.snapshot();
    let r = g.revision();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(p, Action::ChooseOptional { accept: true }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), r);
    }
}
#[test]
fn declining_poison_ward_preserves_uncounterable_spell_and_its_later_effects() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p.other(), "The Serpent Society", Zone::Battlefield);
        let spell = named(&mut g, p, "Eject", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(source)));
        pass_pair(&mut g);
        replay(&mut g, Action::ChooseOptional { accept: false });
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].poison_counters, 0);
        settle(&mut g, None);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}
#[test]
fn targeted_poison_overflow_preserves_already_paid_clue_cost_and_lower_draw_frame() {
    let p = Player::P0;
    let mut g = setup(p);
    named(&mut g, p, "Persuasive Interrogators", Zone::Battlefield);
    let factory = Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype("Clue"))
        .unwrap();
    g.create_tokens(p, factory, 1, false);
    let clue = tokens(&g, p, "Clue")[0].id;
    activate(&mut g, clue, 0);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    replay(&mut g, Action::Pass);
    g.state.players[1].poison_counters = u64::MAX;
    let saved = g.snapshot();
    let r = g.revision();
    let actor = g.actor().unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), r);
        assert_eq!(g.object_view(clue).zone, Zone::Outside);
        assert_eq!(g.state.stack.len(), 2);
    }
}
