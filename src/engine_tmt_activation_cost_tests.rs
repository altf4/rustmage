//! Printed activation discounts and non-targeting specific-kind counter costs.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tmt_departure_tests::{candidate, cast_candidate, food, mutagens, reject};
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
fn finish(g: &mut Game) {
    let id = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..12 {
        if g.state.announcing.as_ref().map(|a| a.item.id) != id || g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseActivationCost { .. }
                )
            })
            .unwrap_or_else(|| {
                panic!(
                    "payment continuation {:?}, {:?}, domain {:?}, mana {:?}",
                    g.state.pending,
                    g.legal_actions(),
                    g.mana_payment_domain(),
                    g.state.players[g.actor().unwrap().index()].mana
                )
            });
        checked(g, a);
    }
    panic!("unfinished activation");
}
fn announce(g: &mut Game, id: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn copy(g: &mut Game, p: Player, name: &str) -> CardId {
    g.stage_copy_creation(
        p,
        CopyRecipe {
            faces: copies::CopyFaces::Definition,
            definition: Card::from_name(name).unwrap(),
            exceptions: CopyExceptions::NONE,
        },
        1,
    );
    settle(g);
    g.state
        .copy_recipes
        .as_ref()
        .unwrap()
        .iter()
        .rev()
        .find(|r| {
            r.recipe.definition == Card::from_name(name).unwrap() && g.on_battlefield(r.object.card)
        })
        .unwrap()
        .object
        .card
}
fn hand_cast_x(g: &mut Game, id: CardId, x: u32) {
    if g.state.objects[id as usize].card.is_supported() {
        checked(g, Action::AnnounceCast { card: id });
    } else {
        let item = g.cast_item(g.actor().unwrap(), id, [0; 6]);
        g.open_payment_announcement(item, Counters::default(), None);
    }
    checked(g, Action::ChooseX { value: x });
    finish(g);
}
fn choose_counter(g: &mut Game, id: CardId, counter: CounterKind) {
    let object = g.object_ref(id);
    checked(g, Action::ChooseCreatureCounterCost { object, counter });
}
fn mana_action(g: &Game, id: CardId, color: usize) -> Action {
    g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==id && production[color]==1)).expect("mana action")
}
#[test]
fn tmt_activation_man_paid_x_and_trample_fixed_body_and_reanimation_x_zero() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 3] {
            let mut g = setup(p);
            let man = candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 2, x];
            hand_cast_x(&mut g, man, x);
            settle(&mut g);
            assert_eq!(mutagens(&g, p), x as usize);
            assert_eq!((g.stats(man).power, g.stats(man).toughness), (2, 3));
            assert!(g.keywords(man).contains(Keywords::TRAMPLE));
            g.move_object(man, Zone::Graveyard);
            g.state.players[p.index()].mana = [30; 6];
            let zombify = named(&mut g, p, "Zombify", Zone::Hand);
            cast(&mut g, zombify, None, &[Target::Card(man)]);
            settle(&mut g);
            assert_eq!(mutagens(&g, p), x as usize);
            assert_eq!(g.state.objects[man as usize].zone, Zone::Battlefield);
        }
    }
}
#[test]
fn tmt_activation_man_mutagen_zero_fee_still_taps_sacrifices_and_places_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Battlefield);
        let token = named(
            &mut g,
            p,
            "@token/7506aa37538b239479a880e5",
            Zone::Battlefield,
        );
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        announce(&mut g, token, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            0
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert!(!g.on_battlefield(token));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn tmt_activation_man_food_discount_and_nontoken_artifact_cost_is_unchanged() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Battlefield);
        let token = food(&mut g, p);
        let life = g.state.players[p.index()].life;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, token, 0);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let archive = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        assert_eq!(
            g.reduced_activation_cost(
                p,
                archive,
                ManaCost {
                    colored: [0; 6],
                    generic: 2
                }
            )
            .generic,
            2
        );
        let copied = copy(&mut g, p, "Hedron Archive");
        assert!(g.state.objects[copied as usize].is_token());
        assert!(
            !g.copy_recipe(g.object_ref(copied))
                .unwrap()
                .definition
                .is_token()
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let before = g.state.players[p.index()].library.len();
        announce(&mut g, copied, 1);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn tmt_activation_man_generic_only_live_source_control_and_ability_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let man = candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Battlefield);
        let own = food(&mut g, p);
        let enemy = food(&mut g, p.other());
        let raw = ManaCost {
            colored: [1, 1, 0, 0, 0, 0],
            generic: 2,
        };
        let reduced = g.reduced_activation_cost(p, own, raw);
        assert_eq!(reduced.colored, raw.colored);
        assert_eq!(reduced.generic, 1);
        assert_eq!(
            g.reduced_activation_cost(p.other(), own, raw),
            reduced,
            "source controller determines the modifier even if another player could activate the ability"
        );
        assert_eq!(g.reduced_activation_cost(p.other(), enemy, raw), raw);
        let r = g.object_ref(man);
        g.remove_abilities_exact(&[r], AnimationDuration::Permanent);
        assert_eq!(g.reduced_activation_cost(p, own, raw), raw);
    }
}
#[test]
fn tmt_activation_man_mana_converter_discount_is_seen_by_borrowed_funding_and_paid_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Battlefield);
        let grotto = copy(&mut g, p, "Three Tree Mascot");
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        assert!(g.can_fund(FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            source: None,
            reserve: None
        }));
        checked(&mut g, Action::AnnounceCast { card: growth });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        let action = mana_action(&g, grotto, 4);
        checked(&mut g, action);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 5);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn tmt_activation_man_cost_stays_locked_when_provider_returns_during_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let archive = copy(&mut g, p, "Hedron Archive");
        let treasure = super::tests::spawn(
            &mut g,
            p,
            Card::catalogue()
                .find(|c| c.token_info().is_some_and(|t| t.name == "Treasure Token"))
                .unwrap(),
            Zone::Battlefield,
        );
        let man = candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Exile);
        g.state.exile_links.push(ExileLink {
            source: g.object_ref(treasure),
            exiled: g.object_ref(man),
            return_zone: Zone::Battlefield,
        });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, archive, 1);
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            2
        );
        let action = mana_action(&g, treasure, 4);
        checked(&mut g, action);
        assert!(g.on_battlefield(man));
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            2
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn tmt_activation_ray_paid_entry_flying_and_actual_counter_cost_draws_for_both_players() {
    for p in [Player::P0, Player::P1] {
        for self_counter in [true, false] {
            let mut g = setup(p);
            let ray = candidate(&mut g, p, "Ray Fillet, Man Ray", Zone::Hand);
            cast_candidate(&mut g, ray, &[]);
            settle(&mut g);
            assert_eq!(mutagens(&g, p), 1);
            assert!(g.keywords(ray).contains(Keywords::FLYING));
            let donor = if self_counter {
                ray
            } else {
                named(&mut g, p, "Grizzly Bears", Zone::Battlefield)
            };
            g.set_counters(donor, Counters::of(CounterKind::PlusOne, 1));
            let before = g.state.players[p.index()].library.len();
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            announce(&mut g, ray, 0);
            assert_eq!(g.state.pending, Pending::CreatureCounterCost { player: p });
            choose_counter(&mut g, donor, CounterKind::PlusOne);
            finish(&mut g);
            assert_eq!(g.counter_count(donor, CounterKind::PlusOne), 0);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        }
    }
}
#[test]
fn tmt_activation_ray_cannot_use_wrong_kind_noncreature_or_opponent_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ray = candidate(&mut g, p, "Ray Fillet, Man Ray", Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let artifact = food(&mut g, p);
        g.set_counters(bear, Counters::of(CounterKind::Stun, 1));
        g.set_counters(enemy, Counters::of(CounterKind::PlusOne, 1));
        g.set_counters(artifact, Counters::of(CounterKind::PlusOne, 1));
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: ray,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.set_counters(bear, Counters::of(CounterKind::PlusOne, 1));
        announce(&mut g, ray, 0);
        for (id, kind) in [
            (bear, CounterKind::Stun),
            (enemy, CounterKind::PlusOne),
            (artifact, CounterKind::PlusOne),
        ] {
            let reference = g.object_ref(id);
            reject(
                &mut g,
                Action::ChooseCreatureCounterCost {
                    object: reference,
                    counter: kind,
                },
            );
        }
        choose_counter(&mut g, bear, CounterKind::PlusOne);
        finish(&mut g);
        settle(&mut g);
    }
}
#[test]
fn tmt_activation_ray_counter_cost_ignores_hexproof_and_can_kill_zero_base_donor_after_announcement()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ray = candidate(&mut g, p, "Ray Fillet, Man Ray", Zone::Battlefield);
        let source = g.activation_item(p, ray, 0);
        g.animate_source(
            &source,
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((0, 0)),
                keywords: Keywords::HEXPROOF,
                duration: AnimationDuration::Permanent,
            },
        );
        // Ray is now also a 0/0 protected only by its one counter.
        g.set_counters(ray, Counters::of(CounterKind::PlusOne, 1));
        let before = g.state.players[p.index()].library.len();
        announce(&mut g, ray, 0);
        choose_counter(&mut g, ray, CounterKind::PlusOne);
        finish(&mut g);
        assert_eq!(g.state.objects[ray as usize].zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
#[test]
fn tmt_activation_ray_selected_counter_reserves_exact_incarnation_and_rejects_stale_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ray = candidate(&mut g, p, "Ray Fillet, Man Ray", Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.set_counters(bear, Counters::of(CounterKind::PlusOne, 1));
        announce(&mut g, ray, 0);
        let old = g.object_ref(bear);
        checked(
            &mut g,
            Action::ChooseCreatureCounterCost {
                object: old,
                counter: CounterKind::PlusOne,
            },
        );
        g.move_object(bear, Zone::Hand);
        reject(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        assert!(
            g.legal_actions()
                .iter()
                .all(|a| !matches!(a, Action::PayCosts { .. }))
        );
    }
}

#[test]
fn tmt_activation_man_x_ability_discount_applies_after_declared_x_and_cost_is_fixed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Battlefield);
        let hellkite = copy(&mut g, p, "Steel Hellkite");
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, hellkite, 1);
        assert_eq!(g.x_choice().unwrap().intervals[0].maximum, 2);
        checked(&mut g, Action::ChooseX { value: 2 });
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            1
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: hellkite,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn tmt_activation_man_granted_ability_discount_uses_actual_artifact_token_recipient() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Battlefield);
        let recipient = named(
            &mut g,
            p,
            "@token/667dad32c9f20b70d5e9bda3",
            Zone::Battlefield,
        );
        let pole = named(&mut g, p, "Fishing Pole", Zone::Battlefield);
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        finish(&mut g);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        let grantor = g.object_ref(pole);
        checked(
            &mut g,
            Action::AnnounceAttachedAbility {
                grantor,
                ability: 1,
            },
        );
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            0
        );
        finish(&mut g);
        settle(&mut g);
        assert!(g.state.objects[recipient as usize].tapped);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn tmt_activation_ray_repeatable_instant_cost_and_foreign_owned_controlled_donor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ray = candidate(&mut g, p, "Ray Fillet, Man Ray", Zone::Battlefield);
        let donor = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let aura = named(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(donor)]);
        settle(&mut g);
        g.set_counters(donor, Counters::of(CounterKind::PlusOne, 2));
        g.state.step = Step::End;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        let before = g.state.players[p.index()].library.len();
        for _ in 0..2 {
            announce(&mut g, ray, 0);
            choose_counter(&mut g, donor, CounterKind::PlusOne);
            finish(&mut g);
            settle(&mut g);
        }
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert_eq!(g.counter_count(donor, CounterKind::PlusOne), 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(!g.state.objects[ray as usize].tapped);
    }
}
#[test]
fn tmt_activation_man_multiple_providers_stack_only_generic_reduction_and_source_loss_is_current() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = candidate(&mut g, p, "Mutagen Man, Living Ooze", Zone::Battlefield);
        // A pre-SBA kernel fixture isolates additive continuous effects; normal
        // legend checks still apply to real duplicate legendary entries.
        let definition = Card::from_name("Mutagen Man, Living Ooze").unwrap();
        let second = super::tests::spawn(&mut g, p, definition, Zone::Battlefield);
        let token = food(&mut g, p);
        let cost = ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        };
        assert_eq!(
            g.reduced_activation_cost(p, token, cost),
            ManaCost { generic: 0, ..cost }
        );
        g.move_object(first, Zone::Graveyard);
        assert_eq!(g.reduced_activation_cost(p, token, cost).generic, 0);
        g.move_object(second, Zone::Graveyard);
        assert_eq!(g.reduced_activation_cost(p, token, cost), cost);
    }
}

#[test]
fn tmt_activation_ray_generates_mona_power_mana_before_removing_reserved_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ray = candidate(&mut g, p, "Ray Fillet, Man Ray", Zone::Battlefield);
        let mona = candidate(&mut g, p, "Mona Lisa, Science Geek", Zone::Battlefield);
        g.set_counters(mona, Counters::of(CounterKind::PlusOne, 1));
        g.state.players[p.index()].mana = [0; 6];
        let before = g.state.players[p.index()].library.len();
        announce(&mut g, ray, 0);
        choose_counter(&mut g, mona, CounterKind::PlusOne);
        assert_eq!(g.stats(mona).power, 2);
        checked(
            &mut g,
            Action::ActivateMana {
                card: mona,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_eq!(g.counter_count(mona, CounterKind::PlusOne), 1);
        finish(&mut g);
        assert_eq!(g.counter_count(mona, CounterKind::PlusOne), 0);
        assert_eq!(g.stats(mona).power, 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
