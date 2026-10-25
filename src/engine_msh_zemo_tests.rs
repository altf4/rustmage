//! Ordinary supported paid boast/connive scenarios and labelled generic card-copy fixtures.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
const ZEMO: &str = "Baron Helmut Zemo";
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Hand)
}

fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, mode: Option<u16>, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = hand(g, p, name);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, mode, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    if g.state.active != p {
        next_turn(g);
        until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    }
    let id = packet(g, p, name, None, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..200 {
        if done(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("continuation did not complete");
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let bytes = g.state_bytes();
    let views = serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap(),
        views
    );
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let before_views = views(&g);
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
}
fn action(z: CardId) -> Action {
    Action::AnnounceActivate {
        card: z,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn ready(p: Player) -> (Game, CardId) {
    let mut g = setup(p);
    let z = paid(&mut g, p, ZEMO);
    next_turn(&mut g);
    next_turn(&mut g);
    until(&mut g, |g| {
        g.state.active == p && g.state.step == Step::Main1
    });
    (g, z)
}
fn attack(g: &mut Game, p: Player, z: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: z });
    checked(g, Action::FinishAttackers);
    priority(g, p);
}
fn grave(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n)
        .map(|_| named(g, p, "Murder", Zone::Graveyard))
        .collect()
}
fn choose(g: &mut Game, c: CardId) {
    let object = g.object_ref(c);
    checked(g, Action::ChooseAdditionalCost { object });
}
fn boast(g: &mut Game, p: Player, z: CardId, cards: &[CardId]) {
    priority(g, p);
    checked(g, action(z));
    until(g, |g| {
        matches!(g.state.pending, Pending::AdditionalCost { .. })
    });
    for &c in cards {
        choose(g, c);
    }
    checked(g, Action::FinishGraveyardSymbolCost);
    finish(g);
}
fn offer(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
}
#[test]
fn msh_zemo_paid_body_and_boast_requires_actual_attack_not_just_fifteen_symbols() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        grave(&mut g, p, 8);
        assert_eq!(
            g.stats(z),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.object_view(z).is_legendary());
        for ty in ["Human", "Noble", "Villain"] {
            assert!(g.has_subtype(z, ty));
        }
        assert_eq!(g.object_view(z).colors, 4);
        reject(&mut g, action(z));
        attack(&mut g, p, z);
        assert!(g.legal_actions().contains(&action(z)));
    }
}
#[test]
fn msh_zemo_symbol_cost_rejects_fourteen_wrong_color_owner_and_duplicate_atomically() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let cards = grave(&mut g, p, 7);
        attack(&mut g, p, z);
        reject(&mut g, action(z));
        let eighth = named(&mut g, p, "Murder", Zone::Graveyard);
        let white = named(&mut g, p, "Disenchant", Zone::Graveyard);
        let other = named(&mut g, p.other(), "Murder", Zone::Graveyard);
        checked(&mut g, action(z));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::AdditionalCost { .. })
        });
        for c in cards {
            choose(&mut g, c);
        }
        reject(&mut g, Action::FinishGraveyardSymbolCost);
        for c in [white, other] {
            let object = g.object_ref(c);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        choose(&mut g, eighth);
        let object = g.object_ref(eighth);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        g = orphan(g, Action::FinishGraveyardSymbolCost);
        finish(&mut g);
        offer(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            8
        );
        g = orphan(g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.object_view(white).zone, Zone::Graveyard);
        assert_eq!(g.object_view(other).zone, Zone::Graveyard);
        reject(&mut g, action(z));
    }
}
#[test]
fn msh_zemo_more_than_minimum_selection_is_permitted_and_all_paid_cards_stay_exiled() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let cards = grave(&mut g, p, 9);
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            9
        );
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert!(cards.iter().all(|&c| g.object_view(c).zone == Zone::Exile));
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.copy_kind == CopyKind::Card && o.zone == Zone::Exile)
        );
    }
}
#[test]
fn msh_zemo_free_casts_are_real_spells_up_to_three_and_cost_cards_remain_exiled() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let cards = grave(&mut g, p, 8);
        let victims = (0..3)
            .map(|_| named(&mut g, p.other(), "Hill Giant", Zone::Battlefield))
            .collect::<Vec<_>>();
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        for (n, &victim) in victims.iter().enumerate() {
            let object = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
            checked(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            finish(&mut g);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .casts_remaining,
                Some(2 - n as u16)
            );
        }
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::CastExiledDuringResolution { .. }))
        );
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert!(victims.iter().all(|&c| !g.on_battlefield(c)));
        assert!(cards.iter().all(|&c| g.object_view(c).zone == Zone::Exile));
        assert_eq!(g.object_view(z).counters.plus_one, 0);
    }
}
#[test]
fn msh_zemo_connive_black_hand_only_with_real_draw_discard_and_nonland_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let z = paid(&mut g, p, ZEMO);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Murder", None, &[Target::Card(target)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        let d = hand(&mut g, p, "Negate");
        g = orphan(g, Action::Discard { card: d });
        settle(&mut g);
        assert_eq!(g.object_view(z).counters.plus_one, 1);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(&mut g, p, "Shock", None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(z).counters.plus_one, 1);
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(z).counters.plus_one, 1);
    }
}
#[test]
fn msh_zemo_cost_branch_full_nonactor_privacy_owned_restore_and_cancelled_snapshot() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let cards = grave(&mut g, p, 8);
        let hidden = hand(&mut g, p.other(), "Murder");
        attack(&mut g, p, z);
        let before = g.snapshot();
        checked(&mut g, action(z));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::AdditionalCost { .. })
        });
        let mut alt = g.clone();
        alt.state.objects[hidden as usize].card = Card::from_name("Negate").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(alt.observe(p)).unwrap()
        );
        let object = g.object_ref(cards[0]);
        g = orphan(g, Action::ChooseAdditionalCost { object });
        g.restore(&before).unwrap();
        assert!(
            cards
                .iter()
                .all(|&c| g.object_view(c).zone == Zone::Graveyard)
        );
        assert!(g.legal_actions().contains(&action(z)));
    }
}
#[test]
fn msh_zemo_cast_permanent_copies_become_tokens_with_printed_keywords_and_loyalty() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let mut cards = grave(&mut g, p, 8);
        for name in ["Vampire Nighthawk", "Liliana, Dreadhorde General"] {
            cards.push(named(&mut g, p, name, Zone::Graveyard));
        }
        named(&mut g, p, "Hinterland Sanctifier", Zone::Battlefield);
        let life = g.life(p);
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        let mut casted = Vec::new();
        for name in ["Vampire Nighthawk", "Liliana, Dreadhorde General"] {
            let object = *g
                .state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .iter()
                .find(|r| g.state.objects[r.card as usize].card.name() == name)
                .unwrap();
            checked(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            finish(&mut g);
            casted.push(object.card);
        }
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        for &c in &casted {
            assert!(g.on_battlefield(c));
            assert!(g.object_view(c).is_token());
            assert_eq!(g.object_view(c).owner, p);
        }
        assert!(
            g.keywords(casted[0]).contains(
                Keywords::FLYING
                    .union(Keywords::DEATHTOUCH)
                    .union(Keywords::LIFELINK)
            )
        );
        assert_eq!(g.counter_count(casted[1], CounterKind::Loyalty), 6);
        assert_eq!(g.object_view(z).counters.plus_one, 0);
        assert_eq!(g.life(p), life + 1);
    }
}
#[test]
fn msh_zemo_free_copy_pays_mandatory_additional_cost_and_quota_only_on_successful_cast() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let mut cards = grave(&mut g, p, 8);
        cards.push(named(&mut g, p, "Final Vengeance", Zone::Graveyard));
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        let object = *g
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .iter()
            .find(|r| g.state.objects[r.card as usize].card.name() == "Final Vengeance")
            .unwrap();
        checked(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .casts_remaining,
            Some(3)
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::AdditionalCost { .. })
        });
        let source = g.object_ref(z);
        g = orphan(g, Action::ChooseAdditionalCost { object: source });
        finish(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .casts_remaining,
            Some(2)
        );
        assert!(!g.on_battlefield(z));
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert!(cards.iter().all(|&c| g.object_view(c).zone == Zone::Exile));
    }
}
#[test]
fn msh_zemo_only_exact_successful_cost_exile_incarnations_are_copied() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let cards = grave(&mut g, p, 8);
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        let packet = g
            .state
            .stack
            .iter()
            .find(|s| s.card == z && matches!(s.kind, StackKind::ActivatedAbility { .. }))
            .unwrap()
            .clone();
        assert_eq!(packet.exiled_symbol_cards.as_ref().unwrap().len(), 8);
        // Bounded independent movement invalidates one paid exact exile reference.
        g.move_object(cards[0], Zone::Graveyard);
        g.move_object(cards[0], Zone::Exile);
        offer(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            7
        );
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
    }
}
#[test]
fn msh_zemo_paid_source_departure_and_controller_change_do_not_redirect_copies() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let (mut g, z) = ready(p);
            let cards = grave(&mut g, p, 8);
            attack(&mut g, p, z);
            boast(&mut g, p, z, &cards);
            if leave {
                packet(&mut g, p.other(), "Murder", None, &[Target::Card(z)]);
            } else {
                // Bounded established controller transfer after the ability was paid.
                g.state.objects[z as usize].controller = p.other();
            }
            offer(&mut g);
            assert_eq!(g.state.resolution_free_casts.as_ref().unwrap().player, p);
            assert!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining
                    .iter()
                    .all(|r| g.state.objects[r.card as usize].owner == p)
            );
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
        }
    }
}
#[test]
fn msh_zemo_attacked_exact_incarnation_and_each_turn_limit_survive_control_but_not_blink() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let cards = grave(&mut g, p, 8);
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        let second_cards = grave(&mut g, p, 8);
        priority(&mut g, p);
        reject(&mut g, action(z));
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other() && g.state.step == Step::Main1
        });
        priority(&mut g, p);
        reject(&mut g, action(z));
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        attack(&mut g, p, z);
        assert!(g.legal_actions().contains(&action(z)));
        boast(&mut g, p, z, &second_cards);
        offer(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            8
        );
        assert!(
            cards
                .iter()
                .chain(&second_cards)
                .all(|&c| g.object_view(c).zone == Zone::Exile)
        );
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        // Bounded blink has no right to inherit the old object's attack history.
        let old = g.object_ref(z);
        g.move_object(z, Zone::Exile);
        g.move_object(z, Zone::Battlefield);
        assert_ne!(g.object_ref(z), old);
        priority(&mut g, p);
        reject(&mut g, action(z));
    }
}
#[test]
fn msh_zemo_connive_land_discard_no_counter_and_departed_source_still_draws_discards() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let z = paid(&mut g, p, ZEMO);
            let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let before = g.hand_count(p);
            packet(&mut g, p, "Murder", None, &[Target::Card(victim)]);
            until(&mut g, |g| {
                g.state.pending == Pending::Priority
                    && g.state.stack.iter().any(|s| {
                        s.card == z && matches!(s.kind, StackKind::TriggeredAbility { .. })
                    })
            });
            if leave {
                packet(&mut g, p.other(), "Murder", None, &[Target::Card(z)]);
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.hand_count(p), before + 1);
            let land = hand(&mut g, p, "Forest");
            g = orphan(g, Action::Discard { card: land });
            settle(&mut g);
            assert_eq!(g.object_view(z).counters.plus_one, 0);
            assert_eq!(g.on_battlefield(z), !leave);
        }
    }
}
#[test]
fn msh_zemo_free_x_copy_uses_zero_x_and_remains_a_real_exile_cast() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let mut cards = grave(&mut g, p, 8);
        cards.push(named(&mut g, p, "Exsanguinate", Zone::Graveyard));
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        let object = *g
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .iter()
            .find(|r| g.state.objects[r.card as usize].card.name() == "Exsanguinate")
            .unwrap();
        checked(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        reject(&mut g, Action::ChooseX { value: 1 });
        finish(&mut g);
        let s = g
            .state
            .stack
            .iter()
            .find(|s| s.card == object.card)
            .unwrap();
        assert_eq!(s.x, Some(0));
        assert_eq!(s.cast.unwrap().origin, Zone::Exile);
        let life = [g.life(p), g.life(p.other())];
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!([g.life(p), g.life(p.other())], life);
        assert_eq!(g.object_view(z).counters.plus_one, 0);
    }
}
#[test]
fn msh_zemo_real_echo_ability_copy_reuses_paid_refs_but_has_independent_three_spell_quota() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = paid(&mut g, p, "Echo, Perceptive Prodigy");
        let z = paid(&mut g, p, ZEMO);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        let cards = grave(&mut g, p, 8);
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        let ability = g
            .state
            .stack
            .iter()
            .find(|s| s.card == z && matches!(s.kind, StackKind::ActivatedAbility { .. }))
            .unwrap()
            .id;
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(ability),
            },
        );
        finish(&mut g);
        offer(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .casts_remaining,
            Some(3)
        );
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            8
        );
        checked(&mut g, Action::FinishResolutionCasts);
        offer(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .casts_remaining,
            Some(3)
        );
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            8
        );
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert!(cards.iter().all(|&c| g.object_view(c).zone == Zone::Exile));
        assert!(g.on_battlefield(echo) && g.on_battlefield(z));
    }
}
#[test]
fn msh_zemo_monocolored_hybrid_symbols_count_once_independent_of_generic_payment_components() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let cards = (0..15)
            .map(|_| named(&mut g, p, "Reigning Victor", Zone::Graveyard))
            .collect::<Vec<_>>();
        assert_eq!(Card::from_name("Reigning Victor").unwrap().cost().0, [0; 6]);
        attack(&mut g, p, z);
        checked(&mut g, action(z));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::AdditionalCost { .. })
        });
        for &c in cards.iter().take(14) {
            choose(&mut g, c);
        }
        reject(&mut g, Action::FinishGraveyardSymbolCost);
        choose(&mut g, cards[14]);
        checked(&mut g, Action::FinishGraveyardSymbolCost);
        finish(&mut g);
        offer(&mut g);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining
                .len(),
            15
        );
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
    }
}
#[test]
fn msh_zemo_actual_offering_after_attack_allows_opponent_turn_boast_without_resetting_once() {
    for p in [Player::P0, Player::P1] {
        for used in [false, true] {
            let (mut g, z) = ready(p);
            let own = grave(&mut g, p, 8);
            let other = grave(&mut g, p.other(), 8);
            attack(&mut g, p, z);
            if used {
                boast(&mut g, p, z, &own);
                offer(&mut g);
                checked(&mut g, Action::FinishResolutionCasts);
                settle(&mut g);
            }
            until(&mut g, |g| {
                g.state.step == Step::Main2 && g.state.pending == Pending::Priority
            });
            packet(
                &mut g,
                p,
                "Harmless Offering",
                None,
                &[Target::Player(p.other()), Target::Card(z)],
            );
            settle(&mut g);
            assert_eq!(g.object_view(z).controller, p.other());
            priority(&mut g, p.other());
            if used {
                reject(&mut g, action(z));
            } else {
                boast(&mut g, p.other(), z, &other);
                offer(&mut g);
                assert_eq!(
                    g.state.resolution_free_casts.as_ref().unwrap().player,
                    p.other()
                );
                checked(&mut g, Action::FinishResolutionCasts);
                settle(&mut g);
            }
        }
    }
}
#[test]
fn msh_zemo_copied_card_token_recipe_survives_real_recursive_copies_and_flat_exceptions() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let mut cards = grave(&mut g, p, 8);
        cards.push(named(&mut g, p, "Vampire Nighthawk", Zone::Graveyard));
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        let object = *g
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .iter()
            .find(|r| g.state.objects[r.card as usize].card.name() == "Vampire Nighthawk")
            .unwrap();
        checked(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        finish(&mut g);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        let h = object.card;
        assert!(g.object_view(h).is_token());
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        packet(&mut g, p, "Giant Growth", None, &[Target::Card(h)]);
        settle(&mut g);
        packet(&mut g, p, "Molten Duplication", None, &[Target::Card(h)]);
        settle(&mut g);
        let molten = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (i != h as usize
                    && o.zone == Zone::Battlefield
                    && o.is_token()
                    && g.object_view(i as CardId).name() == "Vampire Nighthawk")
                    .then_some(i as CardId)
            })
            .unwrap();
        assert_eq!(
            g.stats(molten),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert!(g.copiable_recipe(g.object_ref(molten)).exceptions.artifact);
        packet(&mut g, p, "Self-Reflection", None, &[Target::Card(molten)]);
        settle(&mut g);
        let recursive = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (i != h as usize
                    && i != molten as usize
                    && o.zone == Zone::Battlefield
                    && o.is_token()
                    && g.object_view(i as CardId).name() == "Vampire Nighthawk")
                    .then_some(i as CardId)
            })
            .unwrap();
        assert_eq!(
            g.stats(recursive),
            Stats {
                power: 2,
                toughness: 3
            }
        );
        assert!(
            g.copiable_recipe(g.object_ref(recursive))
                .exceptions
                .artifact
        );
        assert!(g.current_kinds(recursive) & 16 != 0);
        assert!(!g.keywords(recursive).contains(Keywords::HASTE));
    }
}
// Bounded generic card-copy permission fixture. This extends an existing paused
// continuation with one directly allocated copy; the Zemo paid-exile packet is
// untouched and does not claim that a blue/colorless card paid its black cost.
fn bounded_copy_offer(p: Player, name: &str) -> (Game, ObjectRef) {
    let (mut g, z) = ready(p);
    let cards = grave(&mut g, p, 8);
    attack(&mut g, p, z);
    boast(&mut g, p, z, &cards);
    offer(&mut g);
    let definition = Card::from_name(name).unwrap();
    assert!(definition.is_supported());
    let copy = g.allocate_card_copy(definition, p);
    g.state
        .resolution_free_casts
        .as_mut()
        .unwrap()
        .remaining
        .push(copy);
    (g, copy)
}
#[test]
fn msh_zemo_bounded_generic_copied_card_selected_faces_preserve_physical_dfc_token_shape() {
    for p in [Player::P0, Player::P1] {
        for back in [false, true] {
            let (mut g, object) = bounded_copy_offer(p, "Monica Rambeau // Photon, Living Light");
            let front = g.state.objects[object.card as usize].card;
            let selected = if back {
                front.back_face().unwrap()
            } else {
                front
            };
            g = orphan(
                g,
                Action::CastExiledDuringResolution {
                    object,
                    face: if back {
                        CastFace::Back
                    } else {
                        CastFace::Normal
                    },
                },
            );
            finish(&mut g);
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
            let view = g.object_view(object.card);
            assert!(view.is_token());
            assert_eq!(view.card, selected);
            assert_eq!(view.face.as_ref().unwrap().back, back);
            assert_eq!(g.copiable_recipe(view.reference).definition, selected);
            assert!(g.can_transform_permanent(view.reference));
            g.transform_permanent(view.reference);
            assert_eq!(
                g.object_view(object.card).face.as_ref().unwrap().back,
                !back
            );
        }
    }
}
#[test]
fn msh_zemo_bounded_generic_copied_card_entry_choice_and_failed_entry_are_owned() {
    for p in [Player::P0, Player::P1] {
        for countered in [false, true] {
            let (mut g, object) = bounded_copy_offer(p, "Adaptive Automaton");
            checked(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            finish(&mut g);
            let stack_ref = g.object_ref(object.card);
            checked(&mut g, Action::FinishResolutionCasts);
            if countered {
                packet(
                    &mut g,
                    p.other(),
                    "Cancel",
                    None,
                    &[Target::Spell(object.card)],
                );
                settle(&mut g);
                assert!(!g.on_battlefield(object.card));
                assert!(g.copy_recipe(stack_ref).is_none());
            } else {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EntryValue { .. })
                });
                let value = EntryChoiceValue::CreatureType(
                    crate::cards::CreatureType::from_name("Robot").unwrap(),
                );
                g = orphan(g, Action::ChooseEntryValue { value });
                settle(&mut g);
                let view = g.object_view(object.card);
                assert!(view.is_token());
                assert_eq!(view.entry_choice, Some(value));
                assert!(g.has_subtype(object.card, "Robot"));
                assert!(g.state.entry_batch.is_none());
            }
        }
    }
}
#[test]
fn msh_zemo_actual_planeswalker_copy_entry_replacements_are_owned_and_not_token_creation() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        let mut cards = grave(&mut g, p, 8);
        cards.push(named(
            &mut g,
            p,
            "Liliana, Dreadhorde General",
            Zone::Graveyard,
        ));
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        attack(&mut g, p, z);
        boast(&mut g, p, z, &cards);
        offer(&mut g);
        let object = *g
            .state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .iter()
            .find(|r| g.state.objects[r.card as usize].card.name() == "Liliana, Dreadhorde General")
            .unwrap();
        checked(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        finish(&mut g);
        checked(&mut g, Action::FinishResolutionCasts);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        g = orphan(g, a);
        settle(&mut g);
        assert_eq!(g.counter_count(object.card, CounterKind::Loyalty), 24);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card.name() == "Liliana, Dreadhorde General")
                .count(),
            1
        );
    }
}

#[test]
fn msh_zemo_actual_witness_protection_removes_connive_and_boast_after_attack() {
    for p in [Player::P0, Player::P1] {
        let (mut g, z) = ready(p);
        grave(&mut g, p, 8);
        attack(&mut g, p, z);
        until(&mut g, |g| {
            g.state.step == Step::Main2
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
        });
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(z)]);
        settle(&mut g);
        priority(&mut g, p);
        reject(&mut g, action(z));
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let before = g.hand_count(p);
        packet(&mut g, p, "Murder", None, &[Target::Card(victim)]);
        assert!(!g.state.pending_triggers.iter().any(|t| t.card == z));
        settle(&mut g);
        assert_eq!(g.hand_count(p), before);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_zemo_actual_grim_return_attacking_does_not_satisfy_declared_attack_boast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let grim = paid(&mut g, p, "Grim Reaper, Lethal Legionnaire");
        let z = paid(&mut g, p, ZEMO);
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(z)]);
        settle(&mut g);
        assert_eq!(g.object_view(z).zone, Zone::Graveyard);
        grave(&mut g, p, 8);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1
        });
        attack(&mut g, p, grim);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        checked(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 1, 0, 0, 3],
            },
        );
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(z),
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(z),
            },
        );
        settle(&mut g);
        assert!(g.on_battlefield(z));
        assert!(g.state.attackers.iter().any(|a| a.card == z));
        assert!(!g.state.attacked_this_turn.contains(&g.object_ref(z)));
        priority(&mut g, p);
        reject(&mut g, action(z));
    }
}
