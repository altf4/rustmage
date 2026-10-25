//! Whole paid cards and normal land origins; bounded setup uses existing helpers.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const KAZAR: &str = "Ka-Zar of the Savage Land";
const MOLE: &str = "Mole Man, Moloid Master";
fn zabu() -> Card {
    Card::from_name("@token/msh-zabu").unwrap()
}
fn moloid() -> Card {
    Card::from_name("@token/msh-moloid").unwrap()
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn mana(g: &mut Game, card: CardId) {
    let action = g.legal_actions().into_iter().find(|a| {
        matches!(a, Action::TapMana { card: id } | Action::ActivateMana { card: id, .. } if *id == card)
    }).expect("fixture mana source must have its actual legal production action");
    checked(g, action);
}
fn fund(g: &mut Game, p: Player, card: Card) {
    priority(g, p);
    g.state.players[p.index()].mana = [0; 6];
    let (colored, generic) = card.cost();
    let mut sources = Vec::new();
    for (i, n) in colored.into_iter().enumerate() {
        for _ in 0..n {
            sources.push(named(
                g,
                p,
                ["Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"][i],
                Zone::Battlefield,
            ));
        }
    }
    for _ in 0..generic {
        sources.push(named(g, p, "Wastes", Zone::Battlefield));
    }
    assert!(g.can_fund(FundingGoal {
        actor: p,
        required: colored,
        generic,
        reserve: None,
        source: None
    }));
    for card in sources {
        mana(g, card);
    }
}
pub(super) fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    let definition = g.object_view(id).card;
    fund(g, p, definition);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..220 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::FinishTargets
                        | Action::PutTrigger { .. }
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("land-origin continuation stalled {:?}", g.state.pending);
}
fn ready(g: &mut Game, p: Player) {
    next_turn(g);
    settle(g);
    next_turn(g);
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.actor() == Some(p)
    });
}
fn tokens(g: &Game, p: Player, card: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.controller == p && o.card == card && o.is_token()
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
fn top(g: &Game, viewer: Player, owner: Player) -> Option<ObjectRef> {
    g.observe(viewer).known_library_top[owner.index()].map(|v| v.object)
}
pub(super) fn observation(g: &Game, p: Player) -> String {
    serde_json::to_string(&g.observe(p)).unwrap()
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    let views = [observation(g, Player::P0), observation(g, Player::P1)];
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(
        [observation(g, Player::P0), observation(g, Player::P1)],
        views
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let actions = g.legal_actions();
    let i = actions.iter().position(|x| *x == a).unwrap();
    let before = g.snapshot();
    let mut expected_views = [g.observe(Player::P0), g.observe(Player::P1)];
    let before_bytes = g.state_bytes();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, i).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state_bytes(), indexed.state_bytes());
    assert_eq!(
        [observation(&raw, Player::P0), observation(&raw, Player::P1)],
        [
            observation(&indexed, Player::P0),
            observation(&indexed, Player::P1)
        ]
    );
    let result = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(raw.snapshot(), before);
    assert_eq!(raw.state_bytes(), before_bytes);
    // Restore opens a new revision epoch. Compare every observation field with
    // that explicitly expected revision; nothing is removed from either view.
    assert!(raw.revision() > rev);
    for view in &mut expected_views {
        view.revision = raw.revision();
    }
    assert_eq!(
        [observation(&raw, Player::P0), observation(&raw, Player::P1)],
        expected_views.map(|v| serde_json::to_string(&v).unwrap())
    );
    checked(&mut raw, a);
    assert_eq!(raw.snapshot(), result);
    raw
}
fn play(g: &mut Game, p: Player, id: CardId) {
    priority(g, p);
    checked(g, Action::PlayLand { card: id });
    settle(g);
}
fn attack(g: &mut Game, p: Player, ids: &[CardId]) {
    until(g, |g| {
        g.state.active == p && g.state.pending == Pending::Attackers
    });
    for &card in ids {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
fn optional(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
}
pub(super) fn moved(g: &mut Game, ids: &[CardId], zone: Zone) {
    let events = g.move_objects_raw(ids, zone);
    g.finish_zone_events(&events, &[]);
}
pub(super) fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn msh_land_origins_paid_kazar_body_zabu_and_top_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = paid(&mut g, p, KAZAR, &[]);
        let z = tokens(&g, p, zabu())[0];
        let v = g.object_view(k);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert!(
            v.card.is_legendary()
                && v.has_subtype("Human")
                && v.has_subtype("Barbarian")
                && v.has_subtype("Hero")
        );
        assert_eq!(g.colors(k), 16);
        let v = g.object_view(z);
        assert_eq!(v.name(), "Zabu");
        assert!(v.card.is_legendary() && v.has_subtype("Cat"));
        assert_eq!(g.colors(z), 16);
        assert_eq!(
            g.stats(z),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.object_view(z).summoning_sick);
        let x = named(&mut g, p, "Shock", Zone::Library);
        assert_eq!(top(&g, p, p), Some(g.object_ref(x)));
        assert!(top(&g, p.other(), p).is_none());
        priority(&mut g, p.other());
        assert_eq!(top(&g, p, p), Some(g.object_ref(x)));
    }
}
#[test]
fn msh_land_origins_paid_mole_body_no_etb_and_exact_moloid() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let m = paid(&mut g, p, MOLE, &[]);
        assert!(tokens(&g, p, moloid()).is_empty());
        let v = g.object_view(m);
        assert_eq!(
            g.stats(m),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(v.card.is_legendary() && v.has_subtype("Human") && v.has_subtype("Villain"));
        assert_eq!(g.colors(m), 16);
        let x = named(&mut g, p, "Forest", Zone::Graveyard);
        play(&mut g, p, x);
        let id = tokens(&g, p, moloid())[0];
        let v = g.object_view(id);
        assert_eq!(v.name(), "Moloid");
        assert!(!v.card.is_legendary());
        assert!(v.has_subtype("Minion"));
        assert_eq!(
            g.stats(id),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert_eq!(g.colors(id), 16);
        assert!(g.object_view(id).summoning_sick);
    }
}
#[test]
fn msh_land_origins_shared_normal_allowance_has_no_per_source_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        paid(&mut g, p, "Loot, Exuberant Explorer", &[]);
        paid(&mut g, p, "Loot, Exuberant Explorer", &[]); // legend rule leaves one extra allowance
        let a = named(&mut g, p, "Forest", Zone::Graveyard);
        let b = named(&mut g, p, "Island", Zone::Graveyard);
        let c = named(&mut g, p, "Swamp", Zone::Graveyard);
        play(&mut g, p, a);
        play(&mut g, p, b);
        priority(&mut g, p);
        reject(&mut g, Action::PlayLand { card: c });
        assert_eq!(tokens(&g, p, moloid()).len(), 2);
        assert!(g.state.graveyard_type_uses.is_none());
        ready(&mut g, p);
        play(&mut g, p, c);
        assert_eq!(tokens(&g, p, moloid()).len(), 3);
    }
}
#[test]
fn msh_land_origins_library_graveyard_and_hand_share_the_allowance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        paid(&mut g, p, MOLE, &[]);
        let a = named(&mut g, p, "Forest", Zone::Library);
        let b = named(&mut g, p, "Island", Zone::Graveyard);
        let c = named(&mut g, p, "Swamp", Zone::Hand);
        play(&mut g, p, a);
        priority(&mut g, p);
        reject(&mut g, Action::PlayLand { card: b });
        reject(&mut g, Action::PlayLand { card: c });
        assert_eq!(
            g.counter_count(tokens(&g, p, zabu())[0], CounterKind::PlusOne),
            1
        );
        assert_eq!(tokens(&g, p, moloid()).len(), 1);
    }
}
#[test]
fn msh_land_origins_only_exact_top_owned_land_and_no_spell_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        let lower = named(&mut g, p, "Island", Zone::Library);
        let nonland = named(&mut g, p, "Bear Cub", Zone::Library);
        let enemy = named(&mut g, p.other(), "Forest", Zone::Library);
        let gy = named(&mut g, p, "Forest", Zone::Graveyard);
        priority(&mut g, p);
        for card in [lower, nonland, enemy, gy] {
            reject(&mut g, Action::PlayLand { card });
        }
        reject(&mut g, Action::AnnounceCast { card: nonland });
        assert!(!g.has_graveyard_cast(p, gy, GraveyardCastKind::Cast));
        moved(&mut g, &[nonland], Zone::Exile);
        play(&mut g, p, lower);
        assert_eq!(g.object_view(lower).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_land_origins_timing_priority_and_stack_are_normal() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let x = named(&mut g, p, "Forest", origin);
            priority(&mut g, p.other());
            reject(&mut g, Action::PlayLand { card: x });
            priority(&mut g, p);
            let spell = named(&mut g, p, "Shock", Zone::Hand);
            let definition = g.object_view(spell).card;
            fund(&mut g, p, definition);
            cast(&mut g, spell, None, &[Target::Player(p.other())]);
            reject(&mut g, Action::PlayLand { card: x });
            settle(&mut g);
            priority(&mut g, p);
            g.state.step = Step::End;
            reject(&mut g, Action::PlayLand { card: x });
            g.state.step = Step::Main1;
            g.state.active = p.other();
            reject(&mut g, Action::PlayLand { card: x });
            g.state.active = p;
            play(&mut g, p, x);
        }
    }
}
#[test]
fn msh_land_origins_mole_never_grants_graveyard_spell_or_cycling() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        let spell = named(&mut g, p, "Shock", Zone::Graveyard);
        let land = named(&mut g, p, "Hobbit Hole", Zone::Graveyard);
        priority(&mut g, p);
        reject(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        play(&mut g, p, land);
        assert!(!g.object_view(land).tapped);
    }
}
#[test]
fn msh_land_origins_adventure_lands_allow_front_and_never_adventure_spell() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let id = named(&mut g, p, "Ishgard, the Holy See", origin);
            priority(&mut g, p);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::AnnounceAdventure{card} if *card==id))
            );
            play(&mut g, p, id);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(g.object_view(id).has_subtype("Town"));
        }
    }
}
#[test]
fn msh_land_origins_live_provider_suppression_and_departure_with_paid_answers() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            let source = paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let x = named(&mut g, p, "Forest", origin);
            paid(&mut g, p, "Stop Cold", &[Target::Card(source)]);
            priority(&mut g, p);
            reject(&mut g, Action::PlayLand { card: x });
            assert!(!g.printed_abilities_available(source));
            let stop = g
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| o.zone == Zone::Battlefield && o.card.name() == "Stop Cold")
                .unwrap()
                .0 as CardId;
            paid(&mut g, p, "Disenchant", &[Target::Card(stop)]);
            priority(&mut g, p);
            assert!(g.legal_actions().contains(&Action::PlayLand { card: x }));
            paid(&mut g, p, "Murder", &[Target::Card(source)]);
            priority(&mut g, p);
            reject(&mut g, Action::PlayLand { card: x });
        }
    }
}
#[test]
fn msh_land_origins_provider_theft_changes_only_controllers_library_or_graveyard() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            let source = paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let mine = named(&mut g, p, "Forest", origin);
            g.change_controller(g.object_ref(source), p.other());
            priority(&mut g, p);
            reject(&mut g, Action::PlayLand { card: mine });
            assert!(cast_origins::land_origin_available(&g, p.other(), origin));
            assert!(!cast_origins::land_origin_available(&g, p, origin));
            next_turn(&mut g);
            until(&mut g, |g| {
                g.state.active == p.other()
                    && g.state.step == Step::Main1
                    && g.actor() == Some(p.other())
            });
            let theirs = named(&mut g, p.other(), "Island", origin);
            play(&mut g, p.other(), theirs);
            assert_eq!(g.object_view(theirs).controller, p.other());
        }
    }
}
#[test]
fn msh_land_origins_duplicate_sources_deduplicate_actions_without_use_rows() {
    for p in [Player::P0, Player::P1] {
        for name in [KAZAR, MOLE] {
            let mut g = setup(p);
            let source = paid(&mut g, p, name, &[]);
            let _duplicate = named(&mut g, p, name, Zone::Battlefield); // bounded pre-SBA duplicate legend: two providers still yield one action
            let _ = source;
            let origin = if name == KAZAR {
                Zone::Library
            } else {
                Zone::Graveyard
            };
            let x = named(&mut g, p, "Forest", origin);
            priority(&mut g, p);
            assert_eq!(
                g.legal_actions()
                    .iter()
                    .filter(|a| **a == Action::PlayLand { card: x })
                    .count(),
                1
            );
            g.state_based_actions();
            settle(&mut g);
            play(&mut g, p, x);
            assert!(g.state.graveyard_type_uses.is_none());
        }
    }
}
#[test]
fn msh_land_origins_kazar_etb_source_gone_or_stolen_captures_controller() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, KAZAR, Zone::Hand);
            let definition = g.object_view(source).card;
            fund(&mut g, p, definition);
            cast(&mut g, source, None, &[]);
            until(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
            if depart {
                moved(&mut g, &[source], Zone::Exile);
            } else {
                g.change_controller(g.object_ref(source), p.other());
            }
            settle(&mut g);
            assert_eq!(tokens(&g, p, zabu()).len(), 1);
            assert!(tokens(&g, p.other(), zabu()).is_empty());
        }
    }
}
#[test]
fn msh_land_origins_zabu_doubled_creation_legend_choice_is_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        let source = named(&mut g, p, KAZAR, Zone::Hand);
        let definition = g.object_view(source).card;
        fund(&mut g, p, definition);
        cast(&mut g, source, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        let zs = tokens(&g, p, zabu());
        assert_eq!(zs.len(), 2);
        let keep = g.object_ref(zs[1]);
        let wrong = g.object_ref(source);
        reject(&mut g, Action::KeepLegend { object: wrong });
        let mut g = orphan(g, Action::KeepLegend { object: keep });
        settle(&mut g);
        assert_eq!(tokens(&g, p, zabu()), vec![zs[1]]);
    }
}
#[test]
fn msh_land_origins_landfall_hand_library_graveyard_and_put_entries() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        paid(&mut g, p, MOLE, &[]);
        let z = tokens(&g, p, zabu())[0];
        for origin in [Zone::Hand, Zone::Library, Zone::Graveyard] {
            let x = named(&mut g, p, "Forest", origin);
            if g.state.land_played {
                ready(&mut g, p);
            }
            play(&mut g, p, x);
        }
        let x = named(&mut g, p, "Island", Zone::Hand);
        moved(&mut g, &[x], Zone::Battlefield);
        settle(&mut g);
        assert_eq!(tokens(&g, p, moloid()).len(), 4);
        assert_eq!(g.counter_count(z, CounterKind::PlusOne), 4);
    }
}
#[test]
fn msh_land_origins_simultaneous_new_source_and_two_lands_see_entire_batch() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, MOLE, Zone::Hand);
            let a = named(&mut g, p, "Forest", Zone::Hand);
            let b = named(&mut g, p, "Island", Zone::Hand);
            let mut ids = vec![source, a, b];
            if reverse {
                ids.reverse();
            }
            // Bounded simultaneous put event: the newly entering source sees both lands.
            moved(&mut g, &ids, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(tokens(&g, p, moloid()).len(), 2);
            let mut g = setup(p);
            // Bounded legitimate token-copy batch, rather than an impossible token in hand.
            let mut definitions = [zabu(), Card::Forest, Card::Island];
            if reverse {
                definitions.reverse();
            }
            let groups: Vec<_> = definitions
                .iter()
                .map(|&token| TokenGroupView {
                    token,
                    amount: 1,
                    tapped: false,
                })
                .collect();
            let recipes: Vec<_> = definitions
                .iter()
                .map(|&c| Some(CopyRecipe::new(c)))
                .collect();
            let refs = g.create_copy_groups_inner(p, &groups, &recipes, None);
            let source = refs
                .iter()
                .find(|r| g.object_view(r.card).card == zabu())
                .unwrap()
                .card;
            g.advance_entries();
            settle(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
        }
    }
}
#[test]
fn msh_land_origins_landfall_opponent_and_type_change_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        paid(&mut g, p, MOLE, &[]);
        let z = tokens(&g, p, zabu())[0];
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        moved(&mut g, &[land], Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(z, CounterKind::PlusOne), 0);
        assert!(tokens(&g, p, moloid()).is_empty());
        let creature = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Imprisoned in the Moon",
            &[Target::Card(creature)],
        );
        assert_ne!(g.kinds(creature) & 2, 0);
        assert_eq!(g.counter_count(z, CounterKind::PlusOne), 0);
        assert!(tokens(&g, p, moloid()).is_empty());
    }
}
#[test]
fn msh_land_origins_zabu_exact_departure_and_reused_slot_never_get_old_counter() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        let z = tokens(&g, p, zabu())[0];
        let old = g.object_ref(z);
        let x = named(&mut g, p, "Forest", Zone::Hand);
        priority(&mut g, p);
        checked(&mut g, Action::PlayLand { card: x });
        moved(&mut g, &[z], Zone::Exile);
        g.state_based_actions();
        g.create_tokens(p, zabu(), 1, false);
        settle(&mut g);
        let new = tokens(&g, p, zabu())[0];
        assert_ne!(g.object_ref(new), old);
        assert_eq!(g.counter_count(new, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_land_origins_zabu_counter_replacements_and_trigger_source_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        paid(&mut g, p, "Doubling Season", &[]);
        let z = tokens(&g, p, zabu())[0];
        let x = named(&mut g, p, "Forest", Zone::Hand);
        play(&mut g, p, x);
        assert_eq!(g.counter_count(z, CounterKind::PlusOne), 2);
        paid(&mut g, p, "Stop Cold", &[Target::Card(z)]);
        let y = named(&mut g, p, "Island", Zone::Hand);
        moved(&mut g, &[y], Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(z, CounterKind::PlusOne), 2);
    }
}
#[test]
fn msh_land_origins_moloid_attack_accept_or_decline_is_public_only_after_mill() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, MOLE, &[]);
            let x = named(&mut g, p, "Forest", Zone::Graveyard);
            play(&mut g, p, x);
            let m = tokens(&g, p, moloid())[0];
            ready(&mut g, p);
            let topcard = named(&mut g, p, "Shock", Zone::Library);
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            attack(&mut g, p, &[m]);
            optional(&mut g);
            assert!(top(&g, p.other(), p).is_none());
            let mut g = orphan(g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.object_view(topcard).zone,
                if accept {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws);
            assert!(!g.state.players[p.index()].attempted_empty_draw);
        }
    }
}
#[test]
fn msh_land_origins_moloid_empty_library_skips_impossible_choice_without_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        let x = named(&mut g, p, "Forest", Zone::Graveyard);
        play(&mut g, p, x);
        let m = tokens(&g, p, moloid())[0];
        ready(&mut g, p);
        let ids = g.state.players[p.index()].library.clone();
        moved(&mut g, &ids, Zone::Exile);
        attack(&mut g, p, &[m]);
        settle(&mut g);
        assert!(!matches!(g.state.pending, Pending::EffectOptional { .. }));
        assert!(!g.state.players[p.index()].attempted_empty_draw);
        assert!(g.outcome().is_none());
    }
}
#[test]
fn msh_land_origins_moloid_attack_packet_survives_source_departure_and_theft() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, MOLE, &[]);
            let x = named(&mut g, p, "Forest", Zone::Graveyard);
            play(&mut g, p, x);
            let m = tokens(&g, p, moloid())[0];
            ready(&mut g, p);
            let mine = named(&mut g, p, "Shock", Zone::Library);
            let theirs = named(&mut g, p.other(), "Murder", Zone::Library);
            attack(&mut g, p, &[m]);
            optional(&mut g);
            if depart {
                moved(&mut g, &[m], Zone::Exile);
            } else {
                g.change_controller(g.object_ref(m), p.other());
            }
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert_eq!(g.object_view(mine).zone, Zone::Graveyard);
            assert_eq!(g.object_view(theirs).zone, Zone::Library);
        }
    }
}
#[test]
fn msh_land_origins_multiple_moloids_have_independent_optional_attack_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        let a = named(&mut g, p, "Forest", Zone::Hand);
        let b = named(&mut g, p, "Island", Zone::Hand);
        moved(&mut g, &[a, b], Zone::Battlefield);
        settle(&mut g);
        let ms = tokens(&g, p, moloid());
        ready(&mut g, p);
        let topcard = named(&mut g, p, "Shock", Zone::Library);
        attack(&mut g, p, &ms);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: false });
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_view(topcard).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_land_origins_moloid_milled_land_can_later_use_mole_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        let x = named(&mut g, p, "Forest", Zone::Graveyard);
        play(&mut g, p, x);
        let m = tokens(&g, p, moloid())[0];
        ready(&mut g, p);
        let y = named(&mut g, p, "Island", Zone::Library);
        attack(&mut g, p, &[m]);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.actor() == Some(p)
        });
        play(&mut g, p, y);
        assert_eq!(tokens(&g, p, moloid()).len(), 2);
    }
}
#[test]
fn msh_land_origins_top_land_life_choice_is_public_owned_and_next_top_private() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, KAZAR, &[]);
            let next = named(&mut g, p, "Shock", Zone::Library);
            let land = named(&mut g, p, "Breeding Pool", Zone::Library);
            priority(&mut g, p);
            checked(&mut g, Action::PlayLand { card: land });
            assert!(g.state.entry_batch.is_some());
            assert!(g.observe(p.other()).entry_life.is_some());
            assert_eq!(
                g.observe(p.other()).entry_life.unwrap().object,
                g.object_ref(land)
            );
            assert_eq!(top(&g, p.other(), p), Some(g.object_ref(land)));
            assert!(!g.state.known_hand.contains(&g.object_ref(land)));
            assert!(
                g.state
                    .library_top
                    .as_ref()
                    .unwrap()
                    .operation
                    .as_ref()
                    .unwrap()
                    .land
            );
            assert_ne!(top(&g, p, p), Some(g.object_ref(next)));
            let mut g = orphan(g, Action::ChooseEntryLife { pay });
            settle(&mut g);
            assert_eq!(g.object_view(land).zone, Zone::Battlefield);
            assert_eq!(g.object_view(land).tapped, !pay);
            assert_eq!(top(&g, p, p), Some(g.object_ref(next)));
            assert!(top(&g, p.other(), p).is_none());
        }
    }
}
#[test]
fn msh_land_origins_top_land_entry_value_is_public_without_revealing_deeper_cards() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        let next = named(&mut g, p, "Shock", Zone::Library);
        let land = named(&mut g, p, "Secluded Courtyard", Zone::Library);
        priority(&mut g, p);
        checked(&mut g, Action::PlayLand { card: land });
        assert!(g.observe(p.other()).entry_value.is_some());
        assert_eq!(
            g.observe(p.other()).entry_value.as_ref().unwrap().object,
            g.object_ref(land)
        );
        assert_ne!(top(&g, p.other(), p), Some(g.object_ref(next)));
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        let mut g = orphan(g, a);
        settle(&mut g);
        assert_eq!(top(&g, p, p), Some(g.object_ref(next)));
        assert!(top(&g, p.other(), p).is_none());
    }
}
#[test]
fn msh_land_origins_private_library_put_remains_private_during_entry_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        let land = named(&mut g, p, "Breeding Pool", Zone::Library);
        let m = g.entry_move(land, Zone::Battlefield, false);
        g.stage_entries(vec![m], None);
        g.advance_entries();
        assert!(g.observe(p.other()).entry_life.is_none());
        assert!(g.observe(p).entry_life.is_some());
        checked(&mut g, Action::ChooseEntryLife { pay: false });
        settle(&mut g);
    }
}
#[test]
fn msh_land_origins_graveyard_land_life_choice_is_public_and_allowance_consumed_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        let land = named(&mut g, p, "Breeding Pool", Zone::Graveyard);
        priority(&mut g, p);
        checked(&mut g, Action::PlayLand { card: land });
        assert!(g.observe(p.other()).entry_life.is_some());
        assert!(g.state.land_played);
        let mut g = orphan(g, Action::ChooseEntryLife { pay: true });
        settle(&mut g);
        assert_eq!(tokens(&g, p, moloid()).len(), 1);
        let other = named(&mut g, p, "Forest", Zone::Graveyard);
        priority(&mut g, p);
        reject(&mut g, Action::PlayLand { card: other });
    }
}
#[test]
fn msh_land_origins_owned_playland_and_illegal_failed_cast_do_not_change_permissions() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let x = named(&mut g, p, "Forest", origin);
            priority(&mut g, p);
            g.state.players[p.index()].mana = [0; 6];
            for o in &mut g.state.objects {
                if o.zone == Zone::Battlefield && o.card.is_land() {
                    o.tapped = true;
                }
            }
            let spell = named(&mut g, p, "Ghalta, Primal Hunger", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: spell });
            let mut g = orphan(g, Action::PlayLand { card: x });
            settle(&mut g);
            assert_eq!(g.object_view(x).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn msh_land_origins_daredevil_exile_land_then_mole_and_zabu_normal_landfall() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        paid(&mut g, p, MOLE, &[]);
        let dare = paid(&mut g, p, "Daredevil, Man Without Fear", &[]);
        let next = named(&mut g, p, "Shock", Zone::Library);
        let land = named(&mut g, p, "Island", Zone::Library);
        attack(&mut g, p, &[dare]);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        assert!(g.can_play_exiled(p, land));
        assert_eq!(top(&g, p, p), Some(g.object_ref(next)));
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.actor() == Some(p)
        });
        play(&mut g, p, land);
        assert_eq!(tokens(&g, p, moloid()).len(), 1);
        assert_eq!(
            g.counter_count(tokens(&g, p, zabu())[0], CounterKind::PlusOne),
            1
        );
        assert!(!g.can_play_exiled(p, land));
    }
}
#[test]
fn msh_land_origins_vision_draw_replaces_private_top_before_ordinary_land_play() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        paid(&mut g, p, MOLE, &[]);
        paid(&mut g, p, "The Vision", &[]);
        let next = named(&mut g, p, "Island", Zone::Library);
        let old = named(&mut g, p, "Forest", Zone::Library);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        let definition = g.object_view(shock).card;
        fund(&mut g, p, definition);
        checked(&mut g, Action::AnnounceCast { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        while g
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.card == shock && a.item.kind == StackKind::Spell)
        {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        until(&mut g, |g| matches!(g.state.pending, Pending::Modes { .. }));
        checked(&mut g, Action::ChooseMode { mode: 2 });
        settle(&mut g);
        assert_eq!(g.object_view(old).zone, Zone::Hand);
        assert!(!g.observe(p.other()).known_hand.contains(&g.object_ref(old)));
        assert_eq!(top(&g, p, p), Some(g.object_ref(next)));
        play(&mut g, p, next);
        priority(&mut g, p);
        reject(&mut g, Action::PlayLand { card: old });
        assert_eq!(tokens(&g, p, moloid()).len(), 1);
    }
}
#[test]
fn msh_land_origins_copied_artifact_moloid_improvises_and_retains_attack_mill() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        let land = named(&mut g, p, "Forest", Zone::Graveyard);
        play(&mut g, p, land);
        let original = tokens(&g, p, moloid())[0];
        // Bounded copiable ARTIFACT exception, using the existing copy engine; no new copier admitted.
        let definition = moloid();
        let recipe = CopyRecipe {
            definition,
            exceptions: CopyExceptions::ARTIFACT,
            overrides: None,
        };
        let copy = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(recipe)],
            None,
        )[0]
        .card;
        g.advance_entries();
        settle(&mut g);
        assert!(g.object_view(copy).card == definition);
        assert_ne!(g.kinds(copy) & 16, 0);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        for _ in 0..4 {
            let card = named(&mut g, p, "Wastes", Zone::Battlefield);
            mana(&mut g, card);
        }
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        let bad = g.object_ref(original);
        reject(&mut g, Action::ChooseImproviser { object: bad });
        let object = g.object_ref(copy);
        checked(&mut g, Action::ChooseImproviser { object });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        let mut g = orphan(g, pay);
        settle(&mut g);
        assert!(g.object_view(copy).tapped);
        assert_eq!(g.object_view(arc).zone, Zone::Battlefield);
        ready(&mut g, p);
        let x = named(&mut g, p, "Shock", Zone::Library);
        attack(&mut g, p, &[copy]);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_land_origins_actual_memory_vessel_hand_ban_keeps_other_land_origins() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let vessel = paid(&mut g, p, "Memory Vessel", &[]);
            priority(&mut g, p);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: vessel,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            while g.state.announcing.is_some() {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            settle(&mut g);
            assert!(g.hand_play_prohibited());
            let hand = named(&mut g, p, "Forest", Zone::Hand);
            let x = named(&mut g, p, "Island", origin);
            priority(&mut g, p);
            reject(&mut g, Action::PlayLand { card: hand });
            play(&mut g, p, x);
        }
    }
}
#[test]
fn msh_land_origins_bounded_cast_prohibition_never_prohibits_normal_land_action() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let x = named(&mut g, p, "Forest", origin);
            let spell = named(&mut g, p, "Bear Cub", Zone::Hand);
            g.state.casting_prohibited = true;
            priority(&mut g, p);
            reject(&mut g, Action::AnnounceCast { card: spell });
            play(&mut g, p, x);
            assert_eq!(g.object_view(x).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn msh_land_origins_moloid_put_attacking_does_not_trigger_declared_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, MOLE, &[]);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, p, &[bear]);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        let before = g.state.players[p.index()].library.len();
        g.create_token_batch_capture(
            p,
            moloid(),
            1,
            true,
            Some(&[CombatDefender::Player(p.other())]),
            false,
        );
        g.advance_entries();
        g.prepare_triggers();
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .any(|i| i.source_card == moloid())
        );
        assert!(!g.state.stack.iter().any(|i| i.source_card == moloid()));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}
#[test]
fn msh_land_origins_zabu_copy_of_copy_has_own_source_and_preserved_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, KAZAR, &[]);
        let original = tokens(&g, p, zabu())[0];
        let definition = zabu();
        let first = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(definition))],
            None,
        )[0];
        g.advance_entries();
        settle(&mut g);
        let recipe = g.copy_recipe(first).unwrap();
        let second = g.create_copy_groups_inner(
            p.other(),
            &[TokenGroupView {
                token: definition,
                amount: 1,
                tapped: false,
            }],
            &[Some(recipe)],
            None,
        )[0];
        g.advance_entries();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        checked(&mut g, Action::KeepLegend { object: second });
        settle(&mut g);
        assert_eq!(g.copy_recipe(second).unwrap().definition, definition);
        let land = named(&mut g, p.other(), "Forest", Zone::Hand);
        moved(&mut g, &[land], Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(second.card, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(original, CounterKind::PlusOne), 0);
        assert_ne!(first, second);
    }
}
#[test]
fn msh_land_origins_recast_new_provider_does_not_reset_turn_land_allowance() {
    for p in [Player::P0, Player::P1] {
        for origin in [Zone::Library, Zone::Graveyard] {
            let mut g = setup(p);
            let source = paid(
                &mut g,
                p,
                if origin == Zone::Library { KAZAR } else { MOLE },
                &[],
            );
            let old = g.object_ref(source);
            let a = named(&mut g, p, "Forest", origin);
            play(&mut g, p, a);
            paid(&mut g, p, "Unsummon", &[Target::Card(source)]);
            let definition = g.object_view(source).card;
            fund(&mut g, p, definition);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            assert_ne!(g.object_ref(source), old);
            let b = named(&mut g, p, "Island", origin);
            priority(&mut g, p);
            reject(&mut g, Action::PlayLand { card: b });
        }
    }
}
#[test]
fn msh_land_origins_top_land_tap_replacement_order_is_public_owned_and_private_next() {
    for p in [Player::P0, Player::P1] {
        for intrinsic_first in [false, true] {
            let mut g = setup(p);
            paid(&mut g, p, KAZAR, &[]);
            let spelunk = paid(&mut g, p, "Spelunking", &[]);
            let next = named(&mut g, p, "Shock", Zone::Library);
            let land = named(&mut g, p, "Breeding Pool", Zone::Library);
            priority(&mut g, p);
            checked(&mut g, Action::PlayLand { card: land });
            assert!(matches!(g.state.pending, Pending::EntryReplacement { .. }));
            let view = g.observe(p.other()).entry_replacement.unwrap();
            assert_eq!(view.object, g.object_ref(land));
            assert_eq!(view.effects.len(), 2);
            assert_ne!(top(&g, p.other(), p), Some(g.object_ref(next)));
            let effect = view
                .effects
                .iter()
                .find(|e| e.intrinsic == intrinsic_first)
                .unwrap();
            assert_eq!(
                effect.source,
                if intrinsic_first {
                    g.object_ref(land)
                } else {
                    g.object_ref(spelunk)
                }
            );
            let mut g = orphan(
                g,
                Action::ChooseEntryReplacement {
                    source: effect.source,
                    intrinsic: effect.intrinsic,
                },
            );
            assert!(matches!(g.state.pending, Pending::EntryLife { .. }));
            assert!(g.observe(p.other()).entry_life.is_some());
            checked(&mut g, Action::ChooseEntryLife { pay: false });
            settle(&mut g);
            assert_eq!(g.object_view(land).tapped, !intrinsic_first);
            assert_eq!(top(&g, p, p), Some(g.object_ref(next)));
            assert!(top(&g, p.other(), p).is_none());
        }
    }
}
