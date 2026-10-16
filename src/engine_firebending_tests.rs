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
                    | Action::DeclineEffectPayment
                    | Action::FinishSearch
                    | Action::PutTrigger { .. }
                    | Action::Pass
                    | Action::ArrangeLibrary { .. }
                    | Action::Sacrifice { .. }
                    | Action::FinishTargets => true,
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
fn empty(g: &mut Game) {
    for p in &mut g.state.players {
        p.mana = [0; 6];
        p.combat_red_mana = 0;
    }
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in ids {
        replay(g, Action::Attack { card });
    }
    replay(g, Action::FinishAttackers);
}
fn advance(g: &mut Game, step: Step) {
    for _ in 0..150 {
        if g.state.step == step
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::Discard { .. }
                        | Action::PutTrigger { .. }
                        | Action::KeepLegend { .. }
                )
            })
            .expect("step action");
        replay(g, a);
    }
    panic!("step not reached {step:?}");
}
fn pool(g: &Game, p: Player) -> (Mana, u32) {
    let state = &g.state.players[p.index()];
    assert!(state.combat_red_mana <= state.mana[3]);
    for viewer in [p, p.other()] {
        let o = g.observe(viewer);
        assert_eq!(o.schema, crate::engine::OBSERVATION_SCHEMA);
        assert_eq!(o.players[p.index()].mana, state.mana);
        assert_eq!(o.players[p.index()].combat_red_mana, state.combat_red_mana);
    }
    (state.mana, state.combat_red_mana)
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn announce(g: &mut Game, id: CardId, ability: u16) {
    replay(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn full_cast(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
    {
        replay(g, a);
    }
    pay(g);
    settle(g, None);
}
const BODIES: [&str; 8] = [
    "Azula, On the Hunt",
    "Fire Sages",
    "Namazu Trader",
    "Rough Rhino Cavalry",
    "Sun Warriors",
    "Uncle Iroh",
    "Vindictive Warden",
    "Zhao, Ruthless Admiral",
];
#[test]
fn all_eight_bodies_cast_with_exact_keywords_stats_hybrid_costs_and_complete_entries() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in BODIES.iter().enumerate() {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            full_cast(&mut g, id);
            let o = g.object_view(id);
            assert_eq!(o.zone, Zone::Battlefield);
            assert_eq!(
                (o.power, o.toughness),
                [
                    (4, 3),
                    (2, 2),
                    (3, 4),
                    (5, 5),
                    (3, 5),
                    (4, 2),
                    (2, 3),
                    (3, 4)
                ][i]
            );
            assert_eq!(o.card.is_legendary(), matches!(i, 0 | 5 | 7));
            assert_eq!(o.keywords.contains(Keywords::MENACE), i == 6);
            if i == 2 {
                assert_eq!(g.state.players[p.index()].life, 19);
                assert_eq!(tokens(&g, p, "Treasure").len(), 1);
            } else {
                assert_eq!(g.state.players[p.index()].life, 20);
            }
            assert_eq!(pool(&g, p).1, 0);
        }
    }
}
#[test]
fn fixed_firebending_uses_stack_does_not_require_live_source_and_cannot_fund_its_own_pending_trigger()
 {
    for p in [Player::P0, Player::P1] {
        for (name, n) in [
            ("Fire Sages", 1),
            ("Rough Rhino Cavalry", 2),
            ("Uncle Iroh", 1),
            ("Vindictive Warden", 1),
            ("Zhao, Ruthless Admiral", 2),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Battlefield);
            let spell = named(&mut g, p, "Shock", Zone::Hand);
            empty(&mut g);
            attack(&mut g, &[id]);
            assert_eq!(g.state.stack.len(), 1);
            assert_eq!(pool(&g, p).1, 0);
            let before = g.snapshot();
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceCast { card: spell })
            );
            assert_eq!(before, g.snapshot());
            g.move_object(id, Zone::Graveyard);
            settle(&mut g, None);
            assert_eq!(pool(&g, p), ([0, 0, 0, n, 0, 0], n));
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: spell })
            );
        }
    }
}
#[test]
fn retained_red_survives_every_combat_step_ordinary_mana_expires_and_end_combat_is_spendable() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let rhino = named(&mut g, p, "Rough Rhino Cavalry", Zone::Battlefield);
        let first = named(&mut g, p, "Reckless Lackey", Zone::Battlefield);
        empty(&mut g);
        attack(&mut g, &[rhino, first]);
        settle(&mut g, None);
        g.add_mana(p, [1, 1, 1, 1, 1, 1]);
        assert_eq!(pool(&g, p), ([1, 1, 1, 3, 1, 1], 2));
        for step in [
            Step::Blockers,
            Step::FirstStrikeDamage,
            Step::CombatDamage,
            Step::EndCombat,
        ] {
            advance(&mut g, step);
            assert_eq!(pool(&g, p), ([0, 0, 0, 2, 0, 0], 2));
        }
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        assert_eq!(pool(&g, p), ([0, 0, 0, 1, 0, 0], 1));
        settle(&mut g, None);
        advance(&mut g, Step::Main2);
        assert_eq!(pool(&g, p), ([0; 6], 0));
        assert_eq!(pool(&g, p.other()), ([0; 6], 0));
    }
}
#[test]
fn direct_and_staged_spell_payments_spend_ordinary_red_before_combat_subset() {
    for p in [Player::P0, Player::P1] {
        for staged in [false, true] {
            let mut g = setup(p);
            let rhino = named(&mut g, p, "Rough Rhino Cavalry", Zone::Battlefield);
            empty(&mut g);
            attack(&mut g, &[rhino]);
            settle(&mut g, None);
            g.add_mana(p, [0, 0, 0, 1, 0, 0]);
            let spell = named(&mut g, p, "Lightning Strike", Zone::Hand);
            if staged {
                replay(&mut g, Action::AnnounceCast { card: spell });
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                reject(
                    &mut g,
                    Action::PayCosts {
                        payment: [0, 0, 0, 1, 0, 0],
                    },
                );
                pay(&mut g);
            } else {
                cast(&mut g, spell, Some(Target::Player(p.other())));
            }
            assert_eq!(pool(&g, p), ([0, 0, 0, 1, 0, 0], 1));
            settle(&mut g, None);
            assert_eq!(g.state.players[p.other().index()].life, 17);
            advance(&mut g, Step::Blockers);
            assert_eq!(pool(&g, p).1, 1);
        }
    }
}
#[test]
fn direct_and_staged_activation_payments_preserve_only_unspent_retained_red() {
    for p in [Player::P0, Player::P1] {
        for staged in [false, true] {
            let mut g = setup(p);
            let a = named(&mut g, p, "Fire Sages", Zone::Battlefield);
            let b = named(&mut g, p, "Fire Sages", Zone::Battlefield);
            empty(&mut g);
            attack(&mut g, &[a, b]);
            settle(&mut g, None);
            g.add_mana(p, [0, 0, 0, 2, 0, 0]);
            if staged {
                announce(&mut g, a, 0);
                pay(&mut g);
            } else {
                activate(&mut g, a, 0);
            }
            assert_eq!(pool(&g, p), ([0, 0, 0, 1, 0, 0], 1));
            settle(&mut g, None);
            assert_eq!(g.object_view(a).counters.plus_one, 1);
            advance(&mut g, Step::Blockers);
            assert_eq!(pool(&g, p).1, 1);
        }
    }
}
#[test]
fn retained_red_funds_mana_conversion_with_pure_virtual_funding_and_owned_spell_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let rhino = named(&mut g, p, "Rough Rhino Cavalry", Zone::Battlefield);
        let filter = named(&mut g, p, "Crystal Grotto", Zone::Battlefield);
        let spell = named(&mut g, p, "Opt", Zone::Hand);
        empty(&mut g);
        attack(&mut g, &[rhino]);
        settle(&mut g, None);
        let saved = g.snapshot();
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        assert_eq!(saved, g.snapshot());
        replay(&mut g, Action::AnnounceCast { card: spell });
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,payment,..}if *card==filter&&production[1]==1&&payment[3]==1)).unwrap();
        replay(&mut g, a);
        assert_eq!(pool(&g, p), ([0, 1, 0, 1, 0, 0], 1));
        pay(&mut g);
        assert_eq!(pool(&g, p), ([0, 0, 0, 1, 0, 0], 1));
        settle(&mut g, None);
        advance(&mut g, Step::Blockers);
        assert_eq!(pool(&g, p).1, 1);
    }
}
#[test]
fn resolution_owned_ward_payments_consume_retained_red_and_declining_preserves_it() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let rhino = named(&mut g, p, "Rough Rhino Cavalry", Zone::Battlefield);
            let target = named(&mut g, p.other(), "Hulking Raptor", Zone::Battlefield);
            empty(&mut g);
            attack(&mut g, &[rhino]);
            settle(&mut g, None);
            g.add_mana(p, [0, 0, 0, 1, 0, 0]);
            let spell = named(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            assert_eq!(pool(&g, p), ([0, 0, 0, 2, 0, 0], 2));
            pass_pair(&mut g);
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.observe(p.other()).resolution_payment.unwrap().payer, p);
            assert!(g.observe(p.other()).actions.is_empty());
            replay(
                &mut g,
                if accept {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 2, 0, 0],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            assert_eq!(pool(&g, p).1, if accept { 0 } else { 2 });
            settle(&mut g, None);
            assert_eq!(g.object_view(target).damage, if accept { 2 } else { 0 });
        }
    }
}
#[test]
fn sun_warriors_counts_current_owned_creatures_at_resolution_even_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Sun Warriors", Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let leave = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        empty(&mut g);
        attack(&mut g, &[source]);
        g.move_object(leave, Zone::Graveyard);
        g.move_object(source, Zone::Hand);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        settle(&mut g, None);
        assert_eq!(pool(&g, p), ([0, 0, 0, 2, 0, 0], 2));
    }
}
#[test]
fn sun_warriors_can_spend_its_scaled_combat_mana_for_complete_ally_token() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Sun Warriors", Zone::Battlefield);
        for _ in 0..5 {
            named(&mut g, p, "Bear Cub", Zone::Battlefield);
        }
        empty(&mut g);
        attack(&mut g, &[source]);
        settle(&mut g, None);
        assert_eq!(pool(&g, p).1, 6);
        activate(&mut g, source, 0);
        assert_eq!(pool(&g, p).1, 1);
        settle(&mut g, None);
        let t = tokens(&g, p, "Ally");
        assert_eq!(t.len(), 1);
        assert_eq!((t[0].power, t[0].toughness), (1, 1));
        assert_eq!(t[0].card.colors(), 1);
        assert!(t[0].summoning_sick);
        advance(&mut g, Step::Blockers);
        assert_eq!(pool(&g, p).1, 1);
    }
}
#[test]
fn azula_orders_two_independent_attack_triggers_and_pays_clue_from_retained_mana() {
    for p in [Player::P0, Player::P1] {
        for first in [0, 1] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Azula, On the Hunt", Zone::Battlefield);
            empty(&mut g);
            attack(&mut g, &[source]);
            assert_eq!(g.state.ordering_triggers.len(), 2);
            let id = g
                .state
                .ordering_triggers
                .iter()
                .find(|s| s.kind == (StackKind::TriggeredAbility { ability: first }))
                .unwrap()
                .id;
            replay(&mut g, Action::PutTrigger { id });
            settle(&mut g, None);
            assert_eq!(g.state.players[p.index()].life, 19);
            assert_eq!(pool(&g, p).1, 2);
            let clue = tokens(&g, p, "Clue")[0].id;
            activate(&mut g, clue, 0);
            assert_eq!(pool(&g, p).1, 0);
            settle(&mut g, None);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .all(|o| o.owner != p || o.zone != Zone::Hand)
            );
        }
    }
}
#[test]
fn rough_rhino_exhaust_is_paid_once_per_incarnation_and_temporary_trample_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Rough Rhino Cavalry", Zone::Battlefield);
        empty(&mut g);
        attack(&mut g, &[source]);
        settle(&mut g, None);
        g.add_mana(p, [0, 0, 0, 6, 0, 0]);
        activate(&mut g, source, 0);
        assert_eq!(pool(&g, p).1, 0);
        assert_eq!(g.object_view(source).exhausted_abilities, 1);
        settle(&mut g, None);
        assert_eq!(g.object_view(source).counters.plus_one, 2);
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        g.add_mana(p, [0, 0, 0, 8, 0, 0]);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        advance(&mut g, Step::Main2);
        for _ in 0..100 {
            if !g.keywords(source).contains(Keywords::TRAMPLE) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::Discard { .. }))
                .unwrap();
            replay(&mut g, a);
        }
        assert!(!g.keywords(source).contains(Keywords::TRAMPLE));
        assert_eq!(g.object_view(source).exhausted_abilities, 1);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        assert_eq!(g.object_view(source).exhausted_abilities, 0);
    }
}
#[test]
fn vindictive_warden_and_zhao_retain_their_full_activation_and_sacrifice_growth_bodies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let warden = named(&mut g, p, "Vindictive Warden", Zone::Battlefield);
        empty(&mut g);
        attack(&mut g, &[warden]);
        settle(&mut g, None);
        g.add_mana(p, [0, 0, 0, 2, 0, 0]);
        activate(&mut g, warden, 0);
        assert_eq!(pool(&g, p).1, 0);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        let zhao = named(&mut g, p, "Zhao, Ruthless Admiral", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let cost = named(&mut g, p, "Forest", Zone::Battlefield);
        g.sacrifice_batch(&[(p, cost)]);
        settle(&mut g, None);
        assert_eq!(g.object_view(warden).power, 3);
        assert_eq!(g.object_view(zhao).power, 4);
        assert_eq!(g.object_view(foreign).power, 2);
        let late = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.object_view(late).power, 2);
        g.sacrifice_batch(&[(p, zhao)]);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn uncle_iroh_discounts_only_owned_lesson_spells_without_changing_colored_cost_or_changeling() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Uncle Iroh", Zone::Battlefield);
        for (who, name, wanted) in [
            (p, "Shared Roots", 0),
            (p.other(), "Shared Roots", 1),
            (p, "Lightning Strike", 1),
        ] {
            let id = named(&mut g, who, name, Zone::Hand);
            let (colored, generic) = g.state.objects[id as usize].card.cost();
            let reduced = g.reduced_spell_cost(who, id, ManaCost { colored, generic });
            assert_eq!(reduced.colored, colored);
            assert_eq!(reduced.generic, wanted);
        }
        let changeling = Card::catalogue()
            .find(|c| {
                c.is_supported() && c.keywords().contains(Keywords::CHANGELING) && c.cost().1 > 0
            })
            .unwrap();
        assert!(!changeling.has_subtype("Lesson"));
        let id = spawn(&mut g, p, changeling, Zone::Hand);
        let (colored, generic) = changeling.cost();
        assert_eq!(
            g.reduced_spell_cost(p, id, ManaCost { colored, generic })
                .generic,
            generic
        );
        g.move_object(source, Zone::Hand);
        let id = named(&mut g, p, "Shared Roots", Zone::Hand);
        let (colored, generic) = g.state.objects[id as usize].card.cost();
        assert_eq!(
            g.reduced_spell_cost(p, id, ManaCost { colored, generic })
                .generic,
            1
        );
    }
}
#[test]
fn namazu_attack_can_decline_or_sacrifice_its_treasure_for_owned_private_surveil() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Namazu Trader", Zone::Hand);
        full_cast(&mut g, source);
        g.state.objects[source as usize].summoning_sick = false;
        empty(&mut g);
        attack(&mut g, &[source]);
        pass_pair(&mut g);
        let saved = g.snapshot();
        replay(&mut g, Action::DeclineEffectPayment);
        assert_eq!(tokens(&g, p, "Treasure").len(), 1);
        assert!(g.observe(p).library_choice.is_none());
        g.restore(&saved).unwrap();
        let object = g.object_ref(source);
        reject(&mut g, Action::Sacrifice { object });
        let treasure = tokens(&g, p, "Treasure")[0].id;
        let object = g.object_ref(treasure);
        replay(&mut g, Action::Sacrifice { object });
        assert_eq!(g.object_view(treasure).zone, Zone::Graveyard);
        assert_eq!(g.observe(p).library_choice.unwrap().remaining.len(), 2);
        assert!(g.observe(p.other()).library_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        settle(&mut g, None);
        assert_eq!(g.object_view(treasure).zone, Zone::Outside);
        assert_eq!(pool(&g, p).1, 0);
    }
}
#[test]
fn combat_mana_overflow_restores_pending_trigger_and_both_pool_components() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = named(&mut g, p, "Rough Rhino Cavalry", Zone::Battlefield);
    empty(&mut g);
    attack(&mut g, &[source]);
    g.state.players[0].mana[3] = u32::MAX;
    replay(&mut g, Action::Pass);
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
        assert_eq!(pool(&g, p), ([0, 0, 0, u32::MAX, 0, 0], 0));
    }
}

#[test]
fn prepaid_multitarget_cast_spends_retained_red_with_or_without_available_targets() {
    for p in [Player::P0, Player::P1] {
        for has_target in [false, true] {
            let mut g = setup(p);
            let rhino = named(&mut g, p, "Rough Rhino Cavalry", Zone::Battlefield);
            empty(&mut g);
            attack(&mut g, &[rhino]);
            settle(&mut g, None);
            if !has_target {
                g.move_object(rhino, Zone::Graveyard);
            }
            g.add_mana(p, [0, 1, 0, 1, 0, 0]);
            let spell = named(&mut g, p, "Out Cold", Zone::Hand);
            replay(
                &mut g,
                Action::BeginCast {
                    card: spell,
                    payment: [0, 1, 0, 3, 0, 0],
                },
            );
            if has_target {
                assert_eq!(pool(&g, p).1, 2);
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(rhino),
                    },
                );
                replay(&mut g, Action::FinishTargets);
            }
            assert_eq!(pool(&g, p), ([0; 6], 0));
            settle(&mut g, None);
            assert_eq!(tokens(&g, p, "Clue").len(), 1);
            advance(&mut g, Step::Blockers);
            assert_eq!(pool(&g, p), ([0; 6], 0));
        }
    }
}
