//! Four directly authored casting-origin cards in naturally completed FDN games.
use super::{deck, run_matrix_actions_observed};
use rustmage::{CastMethod, Observation, PermanentType, StackKind, Step, Zone};
use std::collections::{BTreeMap, BTreeSet};
#[test]
fn foundations_four_cast_origins_natural_games_and_owned_permissions() {
    let mut red_green = deck(
        "Forest",
        &[
            "Etali, Primal Storm",
            "Vizier of the Menagerie",
            "Llanowar Elves",
            "Elvish Archdruid",
            "Imperious Perfect",
            "Dwynen's Elite",
            "Giant Growth",
            "Burst Lightning",
            "Solemn Simulacrum",
        ],
    );
    for land in &mut red_green[..12] {
        *land = rustmage::Card::Mountain;
    }
    let mut sultai = deck(
        "Swamp",
        &[
            "Muldrotha, the Gravetide",
            "Tinybones, Bauble Burglar",
            "Crow of Dark Tidings",
            "Solemn Simulacrum",
            "Llanowar Elves",
            "Burglar Rat",
            "Think Twice",
            "Feed the Swarm",
            "Maalfeld Twins",
        ],
    );
    for land in &mut sultai[..8] {
        *land = rustmage::Card::Forest;
    }
    for land in &mut sultai[8..16] {
        *land = rustmage::Card::Island;
    }
    let names = [
        "Etali, Primal Storm",
        "Vizier of the Menagerie",
        "Muldrotha, the Gravetide",
        "Tinybones, Bauble Burglar",
    ];
    let mut paid = BTreeSet::new();
    let mut library_seats = BTreeSet::new();
    let mut stash_seats = BTreeSet::new();
    let mut etali_seats = BTreeSet::new();
    let mut graveyard_seats = BTreeSet::new();
    let mut land_seats = BTreeSet::new();
    let mut pending_graveyard = BTreeMap::new();
    let mut pending_free = BTreeSet::new();
    let mut owned_quota = 0;
    let mut owned_free = 0;
    let mut methods = BTreeMap::new();
    let mut observer = |before: &Observation, after: &Observation, owned: bool| {
        if before.step == Step::Pregame {
            pending_graveyard.clear();
            pending_free.clear();
        }
        if owned && !before.graveyard_type_uses.is_empty() {
            owned_quota += 1;
        }
        if owned && before.resolution_free_casts.is_some() {
            owned_free += 1;
        }
        if let Some(item) = after.announcing.as_ref().filter(|item| {
            before
                .announcing
                .as_ref()
                .is_none_or(|old| old.id != item.id)
        }) {
            if item.cast.is_some_and(|cast| {
                cast.origin == Zone::Graveyard && cast.method == CastMethod::Normal
            }) {
                let used = after
                    .graveyard_type_uses
                    .iter()
                    .find(|row| {
                        row.turn == before.turn && !before.graveyard_type_uses.contains(row)
                    })
                    .expect(
                        "graveyard announcement must consume an explicit fresh type permission",
                    );
                assert!(
                    before
                        .objects
                        .iter()
                        .any(|object| object.reference == used.source
                            && object.controller == item.controller
                            && object.zone == Zone::Battlefield
                            && object.card.name() == "Muldrotha, the Gravetide")
                );
                pending_graveyard.insert(item.id, used.permanent_type);
            }
            if item
                .cast
                .is_some_and(|cast| cast.origin == Zone::Exile && cast.method == CastMethod::Free)
            {
                assert_eq!(
                    before.resolving.as_ref().unwrap().source_card.name(),
                    "Etali, Primal Storm"
                );
                assert!(
                    before
                        .resolution_free_casts
                        .as_ref()
                        .unwrap()
                        .remaining
                        .iter()
                        .any(|object| object.card == item.card)
                );
                pending_free.insert(item.id);
            }
        }
        for used in after.graveyard_type_uses.iter().filter(|row| {
            row.permanent_type == PermanentType::Land && !before.graveyard_type_uses.contains(row)
        }) {
            let controller = before
                .objects
                .iter()
                .find(|object| object.reference == used.source)
                .unwrap()
                .controller;
            assert!(after.objects.iter().any(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == controller
                    && object.card.is_land()
                    && before
                        .objects
                        .iter()
                        .any(|old| old.id == object.id && old.zone == Zone::Graveyard)
            }));
            land_seats.insert(controller.index());
        }
        for item in after.stack.iter().filter(|item| {
            item.kind == StackKind::Spell && !before.stack.iter().any(|old| old.id == item.id)
        }) {
            let cast = item.cast.unwrap();
            *methods
                .entry(format!("{:?}/{:?}", cast.method, cast.origin))
                .or_insert(0usize) += 1;
            if names.contains(&item.source_card.name())
                && cast.origin == Zone::Hand
                && cast.method == CastMethod::Normal
            {
                assert!(cast.payment.into_iter().any(|n| n > 0));
                paid.insert((item.source_card.name(), item.controller.index()));
            }
            if cast.origin == Zone::Library {
                assert!(before.objects.iter().any(|o| o.zone == Zone::Battlefield
                    && o.controller == item.controller
                    && o.card.name() == "Vizier of the Menagerie"));
                library_seats.insert(item.controller.index());
            }
            if cast.stash_origin {
                assert_eq!(cast.origin, Zone::Exile);
                stash_seats.insert(item.controller.index());
            }
            if pending_graveyard.remove(&item.id).is_some() {
                assert_eq!(cast.origin, Zone::Graveyard);
                assert_eq!(cast.method, CastMethod::Normal);
                graveyard_seats.insert(item.controller.index());
            }
            if pending_free.remove(&item.id) {
                assert_eq!(cast.method, CastMethod::Free);
                assert_eq!(cast.origin, Zone::Exile);
                etali_seats.insert(item.controller.index());
            }
        }
    };
    run_matrix_actions_observed(
        &[red_green, sultai],
        &names,
        &[],
        &[],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observer),
    );
    println!(
        "Cast-origin witnesses: paid={paid:?}, methods={methods:?}, library={library_seats:?}, stash={stash_seats:?}, Etali={etali_seats:?}, graveyard={graveyard_seats:?}, land={land_seats:?}, owned quota={owned_quota}, free={owned_free}"
    );
    for name in names {
        for seat in [0, 1] {
            assert!(
                paid.contains(&(name, seat)),
                "missing paid {name} in seat {seat}"
            );
        }
    }
    assert_eq!(library_seats.len(), 2, "Vizier library casts");
    assert_eq!(stash_seats.len(), 2, "Tinybones foreign stash casts");
    assert_eq!(etali_seats.len(), 2, "Etali completed free casts");
    assert_eq!(
        graveyard_seats.len(),
        2,
        "Muldrotha completed graveyard casts"
    );
    assert_eq!(land_seats.len(), 2, "Muldrotha graveyard land plays");
    assert!(
        owned_quota > 0 && owned_free > 0,
        "missing owned live family continuations"
    );
}
