//! Focused printed outcomes; shared engine invariants live in core tests.
//! Whole-card activated acquisition, exact lifetimes and private draw/discard.
use super::*;
const FIXTURES: &[&str] = &[
    "Thranduil, the Elvenking",
    "Forest",
    "Llanowar Elves",
    "Elvenking's Harper",
    "Iron-Shield Elf",
    "Morcant's Eyes",
    "Greenbelt Guardian",
    "Three Tree Mascot",
    "Topiary Lecturer",
    "Stoic Grove-Guide",
    "Bear Cub",
    "Elvish Archdruid",
    "Fíli the Pathfinder",
    "Elrond, Moon-Reader",
    "Thranduil, Sindarin Liege",
    "Witness Protection",
    "Negate",
    "Shock",
    "Giant Growth",
    "Rest in Peace",
    "Lotus Ring",
    "Bard, King of Dale",
    "Guardian of the Halls",
    "The Wondrous Wasp",
];
fn fixture(g: &mut Game, p: Player, name: &str, z: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, z)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn finish(g: &mut Game) {
    for _ in 0..96 {
        if quiet(g) || g.outcome().is_some() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseEntryTimestamp { .. }
                    | Action::Discard { .. }
            )
        });
    }
    panic!("Thranduil resolution budget {:?}", g.state.pending);
}
fn until_discard(g: &mut Game) {
    for _ in 0..64 {
        if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("mandatory discard not reached {:?}", g.state.pending);
}
fn bind(g: &mut Game, t: Target) {
    act(g, Action::ChooseTarget { target: t });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let t = fixture(g, p, "Thranduil, the Elvenking", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 1, 0, 1, 2];
    cast(g, t, None);
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, t), (5, 6));
    assert!(g.object_view(t).is_legendary());
    assert!(g.has_subtype(t, "Elf") && g.has_subtype(t, "Noble"));
    assert_eq!(g.hand_count(p), 0, "self entry is excluded");
    t
}
fn view(g: &Game, t: CardId, donor: CardId, ability: u16) -> GraveyardBorrowedAbilityView {
    g.observe(g.state.objects[t as usize].controller)
        .graveyard_borrowed_abilities
        .unwrap_or_default()
        .into_iter()
        .find(|v| {
            v.source == g.object_ref(t)
                && v.graveyard == g.object_ref(donor)
                && v.ability == ability
        })
        .expect("exact borrowed ability")
}
fn announce(g: &mut Game, t: CardId, donor: CardId, ability: u16) {
    let graveyard = g.object_ref(donor);
    act(
        g,
        Action::AnnounceGraveyardBorrowedAbility {
            card: t,
            graveyard,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn mana(g: &mut Game, t: CardId, donor: CardId, ability: u16, production: Mana) {
    let graveyard = g.object_ref(donor);
    act(
        g,
        Action::AnnounceGraveyardBorrowedMana {
            card: t,
            graveyard,
            ability,
            counter_payment: Counters::default(),
            production,
        },
    );
    while g.state.announcing.is_some() {
        pay(g);
    }
}
#[test]
fn thranduil_paid_body_owns_graveyard_abilities_mana_nonmana_and_not_donor_triggers_both_seats() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let elf = fixture(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let harper = fixture(&mut g, p, "Elvenking's Harper", Zone::Graveyard);
        let elrond = fixture(&mut g, p, "Elrond, Moon-Reader", Zone::Graveyard);
        let enemy = fixture(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        let hand = fixture(&mut g, p, "Llanowar Elves", Zone::Hand);
        g.move_object(hand, Zone::Library);
        let t = paid(&mut g, p);
        assert_eq!(
            view(&g, t, elf, u16::MAX).definition.name(),
            "Llanowar Elves"
        );
        assert_eq!(
            view(&g, t, harper, 0).definition.name(),
            "Elvenking's Harper"
        );
        view(&g, t, elrond, 0);
        assert!(g.observe(p).borrowed_abilities.is_empty());
        assert!(
            g.observe(p)
                .graveyard_borrowed_abilities
                .unwrap()
                .iter()
                .all(|v| v.graveyard.card != enemy && v.graveyard.card != hand)
        );
        let action = Action::AnnounceGraveyardBorrowedMana {
            card: t,
            graveyard: g.object_ref(elf),
            ability: u16::MAX,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        reject(&mut g, action.clone()); // summoning sickness belongs to Thranduil
        g.state.objects[t as usize].summoning_sick = false;
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(g.object_view(t).tapped);
        assert_eq!(
            g.hand_count(p),
            0,
            "Elrond's triggered ability in graveyard was not borrowed"
        );
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
        announce(&mut g, t, harper, 0);
        bind(&mut g, Target::Card(t));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.keywords(t).contains(Keywords::CANT_BE_BLOCKED));
        assert_eq!(g.object_view(harper).zone, Zone::Graveyard);
        // The direct payment-window route is distinct from the priority
        // announcement above and preserves the enclosing paid creature cast.
        g.state.objects[t as usize].tapped = false;
        let body = fixture(&mut g, p, "Guardian of the Halls", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: body });
        let graveyard = g.object_ref(elf);
        act(
            &mut g,
            Action::ActivateGraveyardBorrowedMana {
                card: t,
                graveyard,
                ability: u16::MAX,
                counter_payment: Counters::default(),
                payment: [0; 6],
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(body).zone, Zone::Battlefield);
        assert!(g.object_view(t).tapped);
    }
}
#[test]
fn thranduil_all_elf_cards_includes_kindred_noncreature_and_sacrifices_actual_recipient() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let eyes = fixture(&mut g, p, "Morcant's Eyes", Zone::Graveyard);
        fixture(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let t = paid(&mut g, p);
        assert_eq!(g.object_view(eyes).kinds & 1, 0);
        view(&g, t, eyes, 0);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 4];
        announce(&mut g, t, eyes, 0);
        pay(&mut g);
        assert_eq!(g.object_view(t).zone, Zone::Graveyard);
        assert_eq!(g.object_view(eyes).zone, Zone::Graveyard);
        finish(&mut g);
        let tokens: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(tokens.len(), 3, "includes the Elf just sacrificed as cost");
        for id in tokens {
            assert!(g.has_subtype(id, "Elf"));
            assert_eq!(stats(&g, id), (2, 2));
        }
        assert!(g.observe(p).graveyard_borrowed_abilities.is_none());
    }
}
#[test]
fn thranduil_self_reference_uses_actual_power_counters_and_recipient_not_donor() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let lecturer = fixture(&mut g, p, "Topiary Lecturer", Zone::Graveyard);
        let guard = fixture(&mut g, p, "Guardian of the Halls", Zone::Graveyard);
        let t = paid(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 5];
        announce(&mut g, t, guard, 0);
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(t, CounterKind::PlusOne), 3);
        assert_eq!(g.counter_count(guard, CounterKind::PlusOne), 0);
        assert_eq!(stats(&g, t), (8, 9));
        g.state.objects[t as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        mana(&mut g, t, lecturer, 0, [0, 0, 0, 0, 8, 0]);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 8, 0]);
        assert_eq!(g.object_view(lecturer).zone, Zone::Graveyard);
    }
}
#[test]
fn thranduil_changeling_mana_once_per_turn_instances_and_exhaust_have_exact_acquisition_lifetimes()
{
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let mascot = fixture(&mut g, p, "Three Tree Mascot", Zone::Graveyard);
        let guardian = fixture(&mut g, p, "Greenbelt Guardian", Zone::Graveyard);
        let t = paid(&mut g, p);
        let old = view(&g, t, mascot, 0).slot;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        mana(&mut g, t, mascot, 0, [0, 0, 0, 0, 1, 0]);
        assert!(view(&g, t, mascot, 0).used);
        let stale = Action::AnnounceGraveyardBorrowedMana {
            card: t,
            graveyard: g.object_ref(mascot),
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        reject(&mut g, stale.clone());
        g.move_object(mascot, Zone::Hand);
        g.give_priority();
        g.refresh_copy_grant_availability();
        g.move_object(mascot, Zone::Graveyard);
        g.refresh_copy_grant_availability();
        assert!(view(&g, t, mascot, 0).slot > old);
        reject(&mut g, stale);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        mana(&mut g, t, mascot, 0, [0, 0, 0, 0, 1, 0]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        announce(&mut g, t, guardian, 1);
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(t, CounterKind::PlusOne), 3);
        let first = view(&g, t, guardian, 1).slot;
        assert!(view(&g, t, guardian, 1).used);
        let again = Action::AnnounceGraveyardBorrowedAbility {
            card: t,
            graveyard: g.object_ref(guardian),
            ability: 1,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [8; 6];
        reject(&mut g, again);
        let aura = fixture(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(t)));
        finish(&mut g);
        assert!(g.observe(p).graveyard_borrowed_abilities.is_none());
        g.move_object(aura, Zone::Graveyard);
        g.give_priority();
        g.refresh_copy_grant_availability();
        assert!(view(&g, t, guardian, 1).slot > first);
        assert!(!view(&g, t, guardian, 1).used);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        announce(&mut g, t, guardian, 1);
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(t, CounterKind::PlusOne), 6);
    }
}

#[test]
fn thranduil_discard_additional_cost_pays_from_hand_and_grants_actual_source_indestructible() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let iron = fixture(&mut g, p, "Iron-Shield Elf", Zone::Graveyard);
        let t = paid(&mut g, p);
        let graveyard = g.object_ref(iron);
        reject(
            &mut g,
            Action::AnnounceGraveyardBorrowedAbility {
                card: t,
                graveyard,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let discarded = fixture(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, t, iron, 0);
        let object = g.object_ref(discarded);
        act(&mut g, Action::ChooseAdditionalCost { object });
        while g.state.announcing.is_some() {
            pay(&mut g);
        }
        finish(&mut g);
        assert_eq!(g.object_view(discarded).zone, Zone::Graveyard);
        assert!(g.object_view(t).tapped);
        assert!(g.keywords(t).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(iron).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn thranduil_etb_filters_self_opponent_nonlegend_and_nonelf_and_observes_draw_replacement() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        paid(&mut g, p);
        for (owner, name) in [
            (p, "Llanowar Elves"),
            (p.other(), "Elrond, Moon-Reader"),
            (p, "Bard, King of Dale"),
        ] {
            let id = fixture(&mut g, owner, name, Zone::Hand);
            g.state.priority = owner;
            g.state.active = owner;
            g.state.players[owner.index()].mana = [8; 6];
            cast(&mut g, id, None);
            finish(&mut g);
            assert_eq!(g.hand_count(p), 0, "entry filter {name}");
        }
        g.state.active = p;
        g.state.priority = p;
        let elf = fixture(&mut g, p, "Elrond, Moon-Reader", Zone::Hand);
        g.state.players[p.index()].mana = [8; 6];
        cast(&mut g, elf, None);
        until_discard(&mut g);
        assert_eq!(
            g.hand_count(p),
            4,
            "Bard doubles each of the two draws in main phase"
        );
        assert_eq!(g.state.players[p.index()].library.len(), 4);
        choose(&mut g, |a| matches!(a, Action::Discard { .. }));
        finish(&mut g);
        assert_eq!(g.hand_count(p), 3);
    }
}
