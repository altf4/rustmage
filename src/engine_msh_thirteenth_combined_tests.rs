//! Combined assurance over the integrated nine-card cohort.
//! Final producer interfaces rechecked against 7a34fe15 at base6cb1ab80.
//! Two groups: a bounded paid both-seat scenario and32 natural Standard games.
//! Shuri slot0 has two own-artifact groups; first-main copies use0..1 target headers.
//! Runtime outcomes and natural coverage are established only when these tests pass.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{checked, named, setup};
use super::*;
use std::collections::{BTreeMap, BTreeSet};
const NINE: [&str; 9] = [
    "Red Hulk",
    "Grim Reaper, Lethal Legionnaire",
    "Baxter Building",
    "Raft Security Officer",
    "Hex Magic",
    "Moonstone, Harsh Mistress",
    "Shuri, Wakandan Inventor",
    "Absorbing Man",
    "Taskmaster, Mercenary Mimic",
];
const SHURI_COPY_SLOT: u16 = 0; // Confirmed staged slot; recheck final integrated producer.
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let before_views = views(g);
    let revision = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(views(g), before_views);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
pub(super) fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut observation = g.observe(p);
        // Restoring deliberately advances the branch revision.
        observation.revision = 0;
        serde_json::to_value(observation).unwrap()
    })
}
fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    let before_views = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(indexed.snapshot(), after);
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn finish_packet(g: &mut Game, packet: u64) {
    for _ in 0..20 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.id != packet)
        {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("packet {packet} stalled at {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("packet {packet} did not finish");
}
// Nontrivial packets stop explicitly. This is deterministic scenario progression,
// not a policy or injected game-state transition. Optional branches default to decline.
fn drive(g: &mut Game, stop: impl Fn(&Game) -> bool) -> usize {
    let mut declined = 0;
    g.prepare_triggers();
    for _ in 0..600 {
        if stop(g) || g.outcome().is_some() {
            return declined;
        }
        let action = if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            declined += 1;
            Action::ChooseOptional { accept: false }
        } else if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            Action::DeclineResolutionMana
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                    ) || matches!(a, Action::FinishTargets)
                        && g.state.announcing.as_ref().is_some_and(|a| a.spec.max == 0)
                })
                .unwrap_or_else(|| {
                    panic!(
                        "unhandled combined decision {:?} at {:?}",
                        g.state.pending, g.state.step
                    )
                })
        };
        checked(g, action);
    }
    panic!("combined progression stalled");
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn settle(g: &mut Game) {
    drive(g, quiet);
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}

// The external decision maker may see another player's private information.
// Counterfactuals alter only identities/order absent from that viewer's full view.
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub(super) struct PrivacyProbeCounts {
    pub hand_identity_swaps: usize,
    pub library_positions_changed: usize,
}
fn observed_references(value: &serde_json::Value, refs: &mut BTreeSet<ObjectRef>) {
    match value {
        serde_json::Value::Object(fields) => {
            if fields.get("card").is_some_and(serde_json::Value::is_u64)
                && fields
                    .get("generation")
                    .is_some_and(serde_json::Value::is_u64)
            {
                refs.insert(serde_json::from_value(value.clone()).unwrap());
            }
            for nested in fields.values() {
                observed_references(nested, refs);
            }
        }
        serde_json::Value::Array(values) => {
            for nested in values {
                observed_references(nested, refs);
            }
        }
        _ => (),
    }
}
pub(super) fn hidden_information_contract(game: &Game, viewer: Player) -> PrivacyProbeCounts {
    let observation = serde_json::to_value(game.observe(viewer)).unwrap();
    let mut visible = BTreeSet::new();
    observed_references(&observation, &mut visible);
    let mut alternative = game.clone();
    let mut counts = PrivacyProbeCounts::default();
    let hidden: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let reference = game.object_ref(i as CardId);
            (o.zone == Zone::Hand
                && !game.shares_information(viewer, o.owner)
                && !game.state.known_hand.contains(&reference)
                && !visible.contains(&reference))
            .then_some(i)
        })
        .collect();
    if let Some(&first) = hidden.first()
        && let Some(&second) = hidden
            .iter()
            .skip(1)
            .find(|&&id| game.state.objects[id].card != game.state.objects[first].card)
    {
        let card = alternative.state.objects[first].card;
        alternative.state.objects[first].card = alternative.state.objects[second].card;
        alternative.state.objects[second].card = card;
        counts.hand_identity_swaps = 1;
    }
    let owner = viewer.other();
    let before = &game.state.players[owner.index()].library;
    // Keep every currently visible exact reference at its old position, including
    // private selections/searches revealed through player control. Always retain
    // the top card, matching the existing below-top counterfactual scope.
    let slots: Vec<_> = before
        .iter()
        .enumerate()
        .filter_map(|(index, &id)| {
            (index + 1 < before.len() && !visible.contains(&game.object_ref(id))).then_some(index)
        })
        .collect();
    let library = &mut alternative.state.players[owner.index()].library;
    for (&slot, &opposite) in slots.iter().zip(slots.iter().rev()) {
        library[slot] = before[opposite];
        counts.library_positions_changed += usize::from(slot != opposite);
    }
    for (index, &id) in before.iter().enumerate() {
        if visible.contains(&game.object_ref(id)) || index + 1 == before.len() {
            assert_eq!(library[index], id);
        }
    }
    assert_eq!(
        observation,
        serde_json::to_value(alternative.observe(viewer)).unwrap()
    );
    assert_eq!(game.legal_actions(), alternative.legal_actions());
    counts
}

pub(super) fn owned_checked_step(
    mut game: Game,
    actor: Player,
    index: usize,
    action: Action,
) -> Game {
    let revision = game.revision();
    let before = game.snapshot();
    assert_eq!(
        game.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(game.snapshot(), before);
    assert_eq!(
        game.apply_index(actor, revision, usize::MAX),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    let (mut raw, mut indexed) = (game.clone(), game.clone());
    drop(game);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(raw.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}

#[derive(Default)]
struct FixtureLog {
    paid_casts: BTreeMap<String, usize>,
    observed: BTreeMap<String, usize>,
}
impl FixtureLog {
    fn event(&mut self, label: &str) {
        *self.observed.entry(label.to_owned()).or_default() += 1;
    }
}
pub(super) fn write_cache_record(variable: &str, value: serde_json::Value) {
    if let Ok(path) = std::env::var(variable) {
        let path = std::path::PathBuf::from(path);
        assert!(
            path.starts_with(
                std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache")
            )
        );
        std::fs::write(path, serde_json::to_string_pretty(&value).unwrap() + "\n").unwrap();
    }
}
fn select_target(g: &mut Game, target: Target) {
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    checked(g, Action::ChooseTarget { target });
    finish_packet(g, packet);
}
fn source_trigger(g: &Game, id: CardId) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == id && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
    })
}
fn permission(g: &Game, player: Player, id: CardId) -> bool {
    g.state
        .exile_permissions
        .iter()
        .any(|row| row.object == g.object_ref(id) && row.player == player)
}
// Payment mana is explicit bounded fixture setup. None preserves mana made by an
// actual prior mana activation; it does not grant free mana or waive a cost.
fn cast_existing(
    g: &mut Game,
    p: Player,
    id: CardId,
    fixture_mana: Option<Mana>,
    targets: &[Target],
    discard: Option<CardId>,
    log: &mut FixtureLog,
) {
    priority(g, p);
    if let Some(mana) = fixture_mana {
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.players[p.index()].mana = mana;
    }
    let name = g.object_view(id).card.identity().to_owned();
    let origin = g.object_view(id).zone;
    checked(
        g,
        if origin == Zone::Exile {
            Action::AnnounceExile { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    for &target in targets {
        while !g.legal_actions().contains(&Action::ChooseTarget { target }) {
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..24 {
        if g.state
            .announcing
            .as_ref()
            .is_none_or(|a| a.item.id != packet)
        {
            *log.paid_casts.entry(name).or_default() += 1;
            if origin == Zone::Exile {
                log.event("paid_cast_from_permission_exile");
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::ChooseAdditionalCost { object } => discard == Some(object.card),
                Action::FinishTargets
                | Action::ChooseSpellCost { .. }
                | Action::PayCosts { .. } => true,
                _ => false,
            })
            .unwrap_or_else(|| panic!("unpaid combined spell {name}: {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("combined cast stayed open");
}
fn paid_new(g: &mut Game, p: Player, name: &str, log: &mut FixtureLog) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, generic) = g.object_view(id).card.cost();
    mana[5] += generic;
    cast_existing(g, p, id, Some(mana), &[], None, log);
    settle(g);
    id
}
fn activate(g: &mut Game, id: CardId, ability: u16, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    let packet = g.state.announcing.as_ref().unwrap().item.id;
    for &target in targets {
        while !g.legal_actions().contains(&Action::ChooseTarget { target }) {
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    finish_packet(g, packet);
}
fn mana_ability(card: CardId, ability: u16, payment: Mana, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment,
        counter_payment: Counters::default(),
        production,
    }
}
// The staged producers express "up to one" with 0..1 target headers. Consuming
// Game here lets each target checkpoint dispose of its actual parent, rather than
// retaining the caller's original Game while checking a cloned surrogate.
fn next_main_with_copies(
    mut g: Game,
    p: Player,
    absorber: CardId,
    task: CardId,
    blueprints: Option<[CardId; 2]>,
) -> Game {
    let turn = g.state.turn;
    let stop = |g: &Game| {
        g.state.turn > turn
            && g.state.active == p
            && g.state.step == Step::Main1
            && (quiet(g) || source_trigger(g, absorber) || source_trigger(g, task))
    };
    drive(&mut g, stop);
    if blueprints.is_none() {
        // These observations occur after the duration expires, before choosing
        // either new optional first-main copy. Counters survive restoration.
        assert_eq!(g.object_view(absorber).name(), NINE[7]);
        assert_eq!(
            (g.stats(absorber).power, g.stats(absorber).toughness),
            (4, 4)
        );
        assert_eq!(g.object_view(task).name(), NINE[8]);
        assert_eq!((g.stats(task).power, g.stats(task).toughness), (4, 6));
    }
    let mut seen = BTreeSet::new();
    while source_trigger(&g, absorber) || source_trigger(&g, task) {
        let source = g.state.announcing.as_ref().unwrap().item.card;
        assert!(
            seen.insert(source),
            "one first-main trigger per actual source"
        );
        let announcement = g.state.announcing.as_ref().unwrap();
        assert_eq!((announcement.spec.min, announcement.spec.max), (0, 1));
        let packet = announcement.item.id;
        if let Some(blueprints) = blueprints {
            let chosen = if source == absorber {
                blueprints[0]
            } else {
                blueprints[1]
            };
            g = orphan(
                g,
                Action::ChooseTarget {
                    target: Target::Card(chosen),
                },
            );
        } else {
            checked(&mut g, Action::FinishTargets);
        }
        finish_packet(&mut g, packet);
        drive(&mut g, stop);
    }
    assert_eq!(seen.len(), 2);
    assert!(quiet(&g));
    priority(&mut g, p);
    g
}

#[test]
fn msh_thirteenth_combined_paid_nine_copy_counter_exile_funding_and_combat_win() {
    let start = std::time::Instant::now();
    let mut records = Vec::new();
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        for pl in &mut g.state.players {
            pl.mana = [0; 6];
        }
        let mut log = FixtureLog::default();
        // Bounded production fixtures: filler libraries, real Forest mana source,
        // and a normal opposing blocker. No injected life/outcome or turn changes.
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let bear = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        let baxter = named(&mut g, p, NINE[2], Zone::Hand);
        g = orphan(g, Action::PlayLand { card: baxter });
        log.event("normal_baxter_land_play");
        let raft = paid_new(&mut g, p, NINE[3], &mut log);
        let shuri = paid_new(&mut g, p, NINE[6], &mut log);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: baxter,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0; 6];
        let moon = paid_new(&mut g, p, NINE[5], &mut log);
        let absorber = paid_new(&mut g, p, NINE[7], &mut log);
        let task = paid_new(&mut g, p, NINE[8], &mut log);
        let grim = paid_new(&mut g, p, NINE[1], &mut log);
        let red = paid_new(&mut g, p, NINE[0], &mut log);

        checked(&mut g, mana_ability(baxter, 0, [0; 6], [0, 0, 0, 0, 0, 1]));
        log.event("baxter_actual_colorless_mana");
        // Actual Baxter C plus explicit three generic fixture mana pays Shuri's
        // reduced four. The intrinsic Improvise alternative is not chosen.
        g.state.players[p.index()].mana[5] += 3;
        let arc_a = named(&mut g, p, "Arc Reactor", Zone::Hand);
        cast_existing(&mut g, p, arc_a, None, &[], None, &mut log);
        settle(&mut g);
        let arc_b = named(&mut g, p, "Arc Reactor", Zone::Hand);
        cast_existing(
            &mut g,
            p,
            arc_b,
            Some([0, 0, 0, 0, 0, 4]),
            &[],
            None,
            &mut log,
        );
        settle(&mut g);
        assert!(g.object_view(arc_a).tapped && g.object_view(arc_b).tapped);

        let murder = named(&mut g, q, "Murder", Zone::Hand);
        cast_existing(
            &mut g,
            q,
            murder,
            Some([0, 0, 2, 0, 0, 1]),
            &[Target::Card(red)],
            None,
            &mut log,
        );
        settle(&mut g);
        assert_eq!(g.object_view(red).zone, Zone::Graveyard);
        let dead_red = g.object_ref(red);
        let synth = named(&mut g, p, "A.I.M. Synthoids", Zone::Hand);
        let thrill = named(&mut g, p, "Thrill of Possibility", Zone::Hand);
        cast_existing(
            &mut g,
            p,
            thrill,
            Some([0, 0, 0, 1, 0, 1]),
            &[],
            Some(synth),
            &mut log,
        );
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        assert_eq!(g.actor(), Some(p));
        assert_eq!(
            g.hand_count(p),
            0,
            "discard trigger precedes the real two draws"
        );
        let discarded = g.object_ref(synth);
        g = orphan(g, Action::ChooseOptional { accept: true });
        hidden_information_contract(&g, p);
        log.event("moon_discard_optional_accepted");
        settle(&mut g);
        assert_eq!(g.object_view(synth).zone, Zone::Exile);
        assert_ne!(g.object_ref(synth), discarded);
        assert!(permission(&g, p, synth));
        assert!(!permission(&g, q, synth));
        assert_eq!(g.hand_count(p), 2);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        let hex = named(&mut g, p, NINE[4], Zone::Hand);
        let hex_cards: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.owner == p && o.zone == Zone::Hand && i as CardId != hex).then_some(i as CardId)
            })
            .collect();
        assert_eq!(hex_cards.len(), 3);
        cast_existing(
            &mut g,
            p,
            hex,
            Some([0, 0, 0, 1, 0, 2]),
            &[],
            None,
            &mut log,
        );
        settle(&mut g); // The corrected core grants only after all three draws.
        assert_eq!(g.hand_count(p), 3);
        for &id in &hex_cards {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert!(permission(&g, p, id));
        }
        log.event("hex_three_exact_post_draw_permissions");
        assert!(
            g.state.pending_triggers.is_empty(),
            "exile does not discard"
        );
        assert!(g.on_battlefield(moon));

        g = next_main_with_copies(g, p, absorber, task, Some([arc_a, red]));
        assert_eq!(g.object_ref(red), dead_red);
        assert_eq!(g.object_view(absorber).name(), NINE[7]);
        assert_eq!(
            (g.stats(absorber).power, g.stats(absorber).toughness),
            (4, 4)
        );
        assert_eq!(g.current_kinds(absorber) & 17, 17, "artifact and creature");
        assert!(g.keywords(absorber).contains(Keywords::VIGILANCE));
        assert!(g.has_subtype(absorber, "Human") && g.has_subtype(absorber, "Villain"));
        assert_eq!(g.object_view(task).name(), NINE[8]);
        assert_eq!((g.stats(task).power, g.stats(task).toughness), (6, 7));
        assert!(
            g.keywords(task)
                .contains(Keywords::REACH.union(Keywords::TRAMPLE))
        );
        assert!(
            g.has_subtype(task, "Human")
                && g.has_subtype(task, "Mercenary")
                && g.has_subtype(task, "Villain")
        );
        assert!(!g.has_subtype(task, "Gamma"));
        log.event("absorbing_copy_chosen_and_resolved");
        log.event("taskmaster_graveyard_copy_chosen_and_resolved");

        cast_existing(
            &mut g,
            p,
            synth,
            Some([0, 0, 0, 0, 0, 1]),
            &[],
            None,
            &mut log,
        );
        assert!(!permission(&g, p, synth));
        drive(&mut g, |g| g.observe(p).library_choice.is_some());
        let choice = g.observe(p).library_choice.unwrap();
        assert!(choice.surveil);
        assert_eq!(choice.remaining.len(), 2);
        assert!(g.observe(q).library_choice.is_none());
        let low = choice.remaining[0].id;
        let high = choice.remaining[1].id;
        reject(
            &mut g,
            Action::ArrangeLibrary {
                card: bear,
                destination: LibraryDestination::Graveyard,
            },
        );
        g = orphan(
            g,
            Action::ArrangeLibrary {
                card: low,
                destination: LibraryDestination::Graveyard,
            },
        );
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: high,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(low).zone, Zone::Graveyard);
        log.event("actual_synthoids_private_surveil");
        let synth_ref = g.object_ref(synth);
        let before_library = g.state.players[p.index()].library.clone();
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        // The staged Shuri definition confirms this sole activation at slot 0.
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: shuri,
                ability: SHURI_COPY_SLOT,
                counter_payment: Counters::default(),
            },
        );
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(synth),
            },
        );
        // The two occurrences may legally choose the same object (CR115.3),
        // although this fixture deliberately copies an existing copied blueprint.
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(absorber),
            },
        );
        finish_packet(&mut g, packet);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_ref(synth), synth_ref);
        assert_eq!(g.object_view(synth).name(), NINE[7]);
        assert_eq!((g.stats(synth).power, g.stats(synth).toughness), (4, 4));
        assert!(!g.object_view(synth).is_token());
        assert!(g.has_subtype(synth, "Human") && g.has_subtype(synth, "Villain"));
        assert!(!g.has_subtype(synth, "Robot"));
        assert!(g.on_battlefield(absorber) && g.on_battlefield(synth));
        assert!(g.keywords(synth).contains(Keywords::VIGILANCE));
        assert_eq!(g.state.players[p.index()].library, before_library);
        assert!(
            g.observe(p).library_choice.is_none(),
            "copying is not entering"
        );
        log.event("shuri_copy_of_copy_resolved_without_legend_or_entry");

        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        activate(&mut g, raft, 0, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(raft).tapped && g.object_view(bear).tapped);
        log.event("raft_actual_reduced_small_creature_tap");
        cast_existing(
            &mut g,
            p,
            shock,
            Some([0, 0, 0, 1, 0, 0]),
            &[Target::Card(task)],
            None,
            &mut log,
        );
        drive(&mut g, |g| {
            source_trigger(g, task)
                && g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Player(q),
                })
        });
        assert_eq!(g.counter_count(task, CounterKind::PlusOne), 1);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(task),
            },
        );
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(q),
            },
        );
        finish_packet(&mut g, packet);
        settle(&mut g);
        assert_eq!(g.life(q), 19);
        assert_eq!(g.counter_count(red, CounterKind::PlusOne), 0);
        assert!(!permission(&g, p, shock));
        log.event("taskmaster_acquired_damage_counter_reflexive_resolved");

        drive(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        checked(&mut g, Action::Attack { card: grim });
        checked(&mut g, Action::FinishAttackers);
        drive(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
        // Real mana abilities inside the payment window: CCC + G -> Baxter
        // exact four-color split B+RRR -> successful Grim payment and later target.
        checked(&mut g, mana_ability(arc_a, 0, [0; 6], [0, 0, 0, 0, 0, 3]));
        checked(&mut g, Action::TapMana { card: forest });
        g = orphan(
            g,
            mana_ability(baxter, 1, [0, 0, 0, 0, 1, 3], [0, 0, 1, 3, 0, 0]),
        );
        log.event("baxter_actual_four_mana_split_during_grim_payment");
        g = orphan(
            g,
            Action::PayResolutionMana {
                payment: [0, 0, 1, 3, 0, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        log.event("grim_resolution_mana_paid");
        drive(&mut g, |g| {
            source_trigger(g, grim)
                && g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(red),
                })
        });
        let packet = g.state.announcing.as_ref().unwrap().item.id;
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(red),
            },
        );
        finish_packet(&mut g, packet);
        // In this fixture the opponent is the only legal defender, so entry may
        // commit without a defender menu. If offered, choose the ordinary player.
        drive(&mut g, |g| {
            g.on_battlefield(red)
                || g.legal_actions().contains(&Action::ChooseHandDefender {
                    defender: CombatDefender::Player(q),
                })
        });
        if !g.on_battlefield(red) {
            g = orphan(
                g,
                Action::ChooseHandDefender {
                    defender: CombatDefender::Player(q),
                },
            );
        }
        assert_ne!(g.object_ref(red), dead_red);
        assert!(g.object_view(red).tapped);
        assert!(g.state.attackers.iter().any(|a| a.card == red));
        assert_eq!(g.counter_count(red, CounterKind::Finality), 1);
        log.event("grim_returned_exact_red_finality_tapped_attacking");
        drive(&mut g, |g| g.state.step == Step::Main2 && quiet(g));
        assert_eq!(
            g.life(q),
            10,
            "nine actual combat damage after earlier child one"
        );
        let second_shock = named(&mut g, q, "Shock", Zone::Hand);
        cast_existing(
            &mut g,
            q,
            second_shock,
            Some([0, 0, 0, 1, 0, 0]),
            &[Target::Card(red)],
            None,
            &mut log,
        );
        drive(&mut g, |g| {
            source_trigger(g, red)
                && g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Player(q),
                })
        });
        assert_eq!(g.counter_count(red, CounterKind::PlusOne), 1);
        select_target(&mut g, Target::Player(q));
        settle(&mut g);
        assert_eq!(g.life(q), 9);
        assert_eq!(g.counter_count(task, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(red, CounterKind::Finality), 1);
        log.event("physical_red_damage_counter_reflexive_resolved");

        let turn = g.state.turn;
        drive(&mut g, |g| {
            g.state.turn > turn && g.state.active == q && quiet(g)
        });
        assert_eq!(g.object_ref(synth), synth_ref);
        assert_eq!(g.object_view(synth).name(), "A.I.M. Synthoids");
        assert_eq!((g.stats(synth).power, g.stats(synth).toughness), (1, 3));
        assert!(g.observe(p).library_choice.is_none());
        for &id in &hex_cards {
            assert!(!permission(&g, p, id));
        }
        assert_eq!((g.stats(task).power, g.stats(task).toughness), (7, 8));
        assert!(g.has_subtype(synth, "Robot") && g.has_subtype(synth, "Villain"));
        assert!(!g.has_subtype(synth, "Human"));
        log.event("shuri_expired_but_until_next_turn_copy_and_counter_persisted");
        g = next_main_with_copies(g, p, absorber, task, None);
        assert_eq!(g.counter_count(task, CounterKind::PlusOne), 1);
        log.event("next_turn_copy_expiry_retained_physical_counter");
        priority(&mut g, p);
        let hand_before = g.hand_count(p);
        checked(&mut g, mana_ability(arc_a, 0, [0; 6], [0, 0, 0, 0, 0, 3]));
        checked(&mut g, Action::TapMana { card: forest });
        activate(&mut g, baxter, 2, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand_before + 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        log.event("baxter_actual_guarded_draw_paid_from_real_sources");
        drive(&mut g, |g| {
            g.state.active == p && g.state.pending == Pending::Attackers
        });
        for id in [red, grim, absorber, task, moon, shuri, synth, raft] {
            if g.legal_actions().contains(&Action::Attack { card: id }) {
                checked(&mut g, Action::Attack { card: id });
            }
        }
        checked(&mut g, Action::FinishAttackers);
        // Opponent chooses ordinary no-blocks; Grim's later optional payment is
        // declined. No life/outcome mutation or Concede manufactures the terminal.
        drive(&mut g, |g| g.outcome().is_some());
        assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
        for name in NINE.into_iter().filter(|name| *name != NINE[2]) {
            assert!(
                log.paid_casts.contains_key(name),
                "fixture omitted paid {name}"
            );
        }
        records.push(serde_json::json!({"seat":p,"outcome":g.outcome(),
            "paid_cast_counts":log.paid_casts,"observed_asserted_interactions":log.observed}));
    }
    write_cache_record(
        "RUSTMAGE_RECORD_MSH_NINE_FIXTURE",
        serde_json::json!({
            "scope":"both-seat bounded fixture; eight paid nonlands, ordinary Baxter land play, real abilities, natural combat terminal; not a natural legal full game",
            "records":records,"checked_seconds_including_replay_and_assertions":start.elapsed().as_secs_f64(),
        }),
    );
}

// Observation-independent accounting: only records actions after smoke_index has
// chosen from the ordinary legal menu. It never changes policy input or state.
pub(super) fn natural_event_label(g: &Game, action: &Action) -> Option<String> {
    let resolving_source = || {
        g.state
            .resolving
            .as_ref()
            .map(|r| r.item.source_card.identity())
            .unwrap_or("no resolving packet")
    };
    match action {
        Action::PlayLand { card } => Some(format!(
            "land_play::{:?}::{}",
            g.object_view(*card).zone,
            g.object_view(*card).name()
        )),
        Action::AnnounceExile { card } => Some(format!(
            "permission_cast_announced::{}",
            g.object_view(*card).name()
        )),
        Action::AnnounceActivate { card, ability, .. } => Some(format!(
            "activation_announced::{}::slot{ability}",
            g.object_view(*card).name()
        )),
        Action::ActivateMana { card, ability, .. } => Some(format!(
            "mana_ability_committed::{}::slot{ability}",
            g.object_view(*card).name()
        )),
        Action::TapMana { card } => Some(format!(
            "mana_tap_committed::{}",
            g.object_view(*card).name()
        )),
        Action::ChooseOptional { accept } => {
            Some(format!("optional::{accept}::{}", resolving_source()))
        }
        Action::PayResolutionMana { .. } => {
            Some(format!("resolution_mana_paid::{}", resolving_source()))
        }
        Action::DeclineResolutionMana => {
            Some(format!("resolution_mana_declined::{}", resolving_source()))
        }
        Action::ChooseTarget { .. } => Some(format!(
            "target_selected::{}",
            g.state
                .announcing
                .as_ref()
                .map(|a| a.item.source_card.identity())
                .unwrap_or("no announcement")
        )),
        Action::FinishTargets => Some(format!(
            "target_group_finished::{}",
            g.state
                .announcing
                .as_ref()
                .map(|a| a.item.source_card.identity())
                .unwrap_or("no announcement")
        )),
        Action::ChooseHandDefender { .. } => {
            Some("independent_attacking_entry_defender_chosen".to_owned())
        }
        Action::ChooseAdditionalCost { .. } => Some("actual_additional_cost_selected".to_owned()),
        Action::ArrangeLibrary { destination, .. } => {
            Some(format!("library_choice::{destination:?}"))
        }
        Action::BeginImprovise => Some("improvise_started".to_owned()),
        Action::ChooseImproviser { .. } => Some("improvise_helper_committed".to_owned()),
        _ => None,
    }
}
fn live_names(g: &Game) -> Vec<(ObjectRef, String)> {
    (0..g.state.objects.len() as CardId)
        .filter(|&id| g.on_battlefield(id))
        .map(|id| (g.object_ref(id), g.object_view(id).name().to_owned()))
        .collect()
}

#[test]
fn msh_thirteenth_combined_32_natural_standard_five_color_games_full_replay_measured_coverage() {
    let membership: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/msh.json")).unwrap();
    let members: BTreeSet<_> = membership["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    let mut deck = Vec::new();
    let mut deck_counts = BTreeMap::new();
    for (name, count) in [
        ("Plains", 5),
        ("Island", 5),
        ("Swamp", 5),
        ("Mountain", 5),
        ("Forest", 5),
        (NINE[0], 3),
        (NINE[1], 3),
        (NINE[2], 3),
        (NINE[3], 3),
        (NINE[4], 3),
        (NINE[5], 3),
        (NINE[6], 3),
        (NINE[7], 3),
        (NINE[8], 3),
        ("A.I.M. Synthoids", 4),
        ("Arc Reactor", 4),
    ] {
        assert!(members.contains(name), "frozen MSH membership: {name}");
        let card = Card::from_name(name).unwrap();
        // Fail closed until all three complete producers are admitted by the
        // authorized final integration; this uncompiled draft admits nothing.
        assert!(card.is_supported() && card.is_standard_legal(), "{name}");
        deck.extend(std::iter::repeat_n(card, count));
        deck_counts.insert(name, count);
    }
    assert_eq!(deck.len(), 60);
    assert_eq!(deck_counts.len(), 16);
    let land_count = deck.iter().filter(|card| card.is_land()).count();
    assert_eq!(land_count, 28); // Twenty-five basics plus three Baxter Buildings.
    assert_eq!(deck.len() - land_count, 32);
    let start = std::time::Instant::now();
    let mut records = Vec::new();
    let mut paid_casts: BTreeMap<String, usize> = BTreeMap::new();
    let mut events: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_decisions = 0usize;
    for seed in 20261024..20261040 {
        for starting_player in [Player::P0, Player::P1] {
            let config = Config {
                decks: [deck.clone(), deck.clone()],
                seed,
                starting_player,
                deck_rules: DeckRules::Standard,
            };
            let mut g = Game::new(config.clone()).unwrap();
            let mut ledger = Vec::new();
            let mut checkpoints = 0;
            let mut game_casts: BTreeMap<String, usize> = BTreeMap::new();
            let mut game_events: BTreeMap<String, usize> = BTreeMap::new();
            while g.outcome().is_none() {
                let actor = g.actor().unwrap();
                let actions = g.legal_actions();
                // Exactly the existing observation-only policy. No lookahead,
                // card-specific bot, mana injection, progress injection or Concede.
                let index = crate::policy::smoke_index(&g, &actions);
                let action = actions[index].clone();
                assert!(!matches!(action, Action::Concede));
                let paid_name = if matches!(action, Action::PayCosts { .. }) {
                    g.state
                        .announcing
                        .as_ref()
                        .filter(|a| a.item.kind == StackKind::Spell && !a.item.is_copy)
                        .map(|a| a.item.source_card.identity().to_owned())
                } else {
                    None
                };
                let event = natural_event_label(&g, &action);
                let names_before = live_names(&g);
                ledger.push((actor, action.clone()));
                if ledger.len() % 97 == 1 {
                    hidden_information_contract(&g, actor);
                    g = owned_checked_step(g, actor, index, action);
                    checkpoints += 1;
                } else {
                    g.apply(actor, action).unwrap();
                }
                // Increment only after actual successful application. Announcement
                // counts are distinct from actual paid spells and resolved copies.
                if let Some(name) = paid_name {
                    *paid_casts.entry(name.clone()).or_default() += 1;
                    *game_casts.entry(name).or_default() += 1;
                }
                if let Some(label) = event {
                    *events.entry(label.clone()).or_default() += 1;
                    *game_events.entry(label).or_default() += 1;
                }
                for (old, old_name) in names_before {
                    if g.on_battlefield(old.card) && g.object_ref(old.card) == old {
                        let current = g.object_view(old.card).name();
                        if current != old_name {
                            // Conservative observable metric; name-preserving
                            // copies are not inferred from generic stat changes.
                            let label =
                                format!("in_place_effective_name_change::{old_name}=>{current}");
                            *events.entry(label.clone()).or_default() += 1;
                            *game_events.entry(label).or_default() += 1;
                        }
                    }
                }
                assert!(
                    ledger.len() < 30_000,
                    "natural policy stalled seed {seed} starter {starting_player:?}: {:?}",
                    g.state.pending
                );
            }
            assert!(matches!(g.outcome(), Some(Outcome::Winner(_))));
            let mut replay = Game::new(config).unwrap();
            for (actor, action) in &ledger {
                replay.apply(*actor, action.clone()).unwrap();
            }
            assert_eq!(g.snapshot(), replay.snapshot());
            assert_eq!(g.outcome(), replay.outcome());
            assert_eq!(views(&g), views(&replay));
            assert!(checkpoints > 1);
            total_decisions += ledger.len();
            records.push(
                serde_json::json!({"seed":seed,"starting_player":starting_player,
                "decisions":ledger.len(),"turn":g.state.turn,"outcome":g.outcome(),
                "owned_privacy_checkpoints":checkpoints,"paid_cast_counts":game_casts,
                "observed_actions_and_effective_name_changes":game_events}),
            );
        }
    }
    assert_eq!(records.len(), 32);
    let missing_paid_nonlands: Vec<_> = NINE
        .into_iter()
        .filter(|name| *name != NINE[2] && !paid_casts.contains_key(*name))
        .collect();
    // Deliberately no all-eight-paid or every-mechanic natural coverage assertion
    // before running the fixed games. Coverage gaps remain visible in the report.
    write_cache_record(
        "RUSTMAGE_RECORD_MSH_NINE_GAMES",
        serde_json::json!({
            "status":"executed only when this test actually passes; parked draft has no measured result",
            "scope":"32 natural Standard60 MSH-only five-color mirror games; unchanged smoke_index, no injected setup/progress, natural Winner, full replay and owned/privacy checks; bounded evidence, not full-set acceptance",
            "deck":deck_counts,"land_count":land_count,"nonland_count":deck.len()-land_count,
            "seed_range":{"start_inclusive":20261024,"end_exclusive":20261040,"starting_seats":2},
            "games":records,"paid_cast_counts":paid_casts,
            "new_nonland_names_without_observed_paid_cast":missing_paid_nonlands,
            "observed_actions_and_effective_name_changes":events,"decisions":total_decisions,
            "coverage_limits":"Target selections/announcements are not resolved-copy proof. Name-preserving copies are not inferred. Hex exile is not discard; Moon requires real cleanup discard in this deck. Baxter is a land, not a paid spell.",
            "checked_seconds_including_replay_and_assertions":start.elapsed().as_secs_f64(),
        }),
    );
}
