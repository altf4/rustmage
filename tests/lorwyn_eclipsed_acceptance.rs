//! Frozen ECL-only natural games. Inclusion, committed casts and executed rules are separate evidence.
//! The optimized workload uses the same policy/decks/seeds without privacy, branch or replay checks.
use rustmage::{
    Action, Card, CastMethod, Config, CounterKind, DeckRules, Error, Game, Observation, Outcome,
    Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

const POOL_SEEDS: &[u64] = &[0, 7];
const THEME_SEEDS: &[u64] = &[0, 7, 19, 31];
const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
fn members() -> BTreeSet<Card> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/ecl.json")).unwrap();
    let cards = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| Card::from_name(row["name"].as_str().unwrap()).expect("frozen member identity"))
        .collect::<BTreeSet<_>>();
    assert_eq!(cards.len(), 273, "frozen physical membership changed");
    for card in &cards {
        assert!(
            card.is_supported(),
            "full-set acceptance still gates {}",
            card.identity()
        );
    }
    cards
}
fn pool_decks(members: &BTreeSet<Card>) -> Vec<Vec<Card>> {
    let mut cards = members.iter().copied().collect::<Vec<_>>();
    cards.sort_by_key(|card| (card.colors(), card.fixed_mana_value(), card.name()));
    let (lands, spells): (Vec<_>, Vec<_>) = cards.iter().copied().partition(|card| card.is_land());
    let groups = spells.chunks(9).collect::<Vec<_>>();
    let count = groups.len().max(lands.len().div_ceil(6));
    let mut decks = Vec::new();
    for index in 0..count {
        let group = groups[index % groups.len()];
        let colors = group.iter().fold(0u8, |mask, card| mask | card.colors());
        let mut basics = BASICS
            .iter()
            .enumerate()
            .filter(|(i, _)| colors & (1 << i) != 0)
            .map(|(_, name)| Card::from_name(name).unwrap())
            .collect::<Vec<_>>();
        if basics.is_empty() {
            basics.push(Card::from_name("Forest").unwrap());
        }
        let mut deck = (0..24)
            .map(|i| basics[i % basics.len()])
            .collect::<Vec<_>>();
        for &card in group {
            deck.extend([card; 4]);
        }
        while deck.len() < 60 {
            deck.push(basics[deck.len() % basics.len()]);
        }
        decks.push(deck);
    }
    // Distribute every set land into real spell decks, four copies each.
    for (index, land) in lands.into_iter().enumerate() {
        let first = 4 * (index / count);
        decks[index % count][first..first + 4].fill(land);
    }
    assert_eq!(
        decks.iter().flatten().copied().collect::<BTreeSet<_>>(),
        cards.into_iter().collect()
    );
    decks
}
fn theme_decks(members: &BTreeSet<Card>) -> Vec<Vec<Card>> {
    let themes: Vec<(String, Vec<String>)> =
        serde_json::from_str(include_str!("data/lorwyn_eclipsed_themes.json")).unwrap();
    themes
        .into_iter()
        .map(|(colors, names)| {
            assert_eq!(names.len(), 9);
            let cards = names
                .iter()
                .map(|name| Card::from_name(name).unwrap())
                .collect::<Vec<_>>();
            assert!(
                cards
                    .iter()
                    .all(|card| members.contains(card) && card.is_supported())
            );
            let lands = colors
                .chars()
                .map(|color| Card::from_name(BASICS["WUBRG".find(color).unwrap()]).unwrap())
                .collect::<Vec<_>>();
            let mut deck = (0..24).map(|i| lands[i % lands.len()]).collect::<Vec<_>>();
            for card in cards {
                deck.extend([card; 4]);
            }
            deck
        })
        .collect()
}
type Workload = (&'static str, Vec<Vec<Card>>, &'static [u64]);
fn workloads() -> Vec<Workload> {
    let members = members();
    let pool = pool_decks(&members);
    let themes = theme_decks(&members);
    // Black mana pays every spell here, including Lluwen's and Willow's hybrids.
    let mut graveyard = vec![Card::from_name("Swamp").unwrap(); 24];
    for name in [
        "Twilight Diviner",
        "Dose of Dawnglow",
        "Bloodline Bidding",
        "Reaping Willow",
        "Lluwen, Imperfect Naturalist",
        "Bile-Vial Boggart",
        "Graveshifter",
        "Blighted Blackthorn",
        "Moonshadow",
    ] {
        graveyard.extend([Card::from_name(name).unwrap(); 4]);
    }
    // Keep the original 220 cyclic pairings intact; the extra scope adds 16 games.
    let graveyard = vec![graveyard, themes[0].clone()];
    for deck in pool.iter().chain(&themes).chain(&graveyard) {
        assert_eq!(deck.len(), 60);
        assert!(deck.iter().all(|card| members.contains(card)));
        assert!(deck.iter().any(|card| !card.is_land()));
    }
    vec![
        ("entire pool", pool, POOL_SEEDS),
        ("mechanic themes", themes, THEME_SEEDS),
        ("graveyard entry copies", graveyard, THEME_SEEDS),
    ]
}
#[derive(Default)]
struct Evidence {
    executed: BTreeMap<&'static str, u64>,
    casts: BTreeSet<&'static str>,
    triggers: BTreeSet<&'static str>,
    games: usize,
    decisions: usize,
    branches: usize,
    private: usize,
    replayed: usize,
    combat: usize,
    drawn_to_empty: usize,
    wins: [usize; 2],
    draws: usize,
    harvested: BTreeSet<rustmage::ObjectRef>,
    harvest_intents: BTreeSet<u64>,
}
impl Evidence {
    fn add(&mut self, name: &'static str, amount: u64) {
        *self.executed.entry(name).or_default() += amount;
    }
    fn transition(&mut self, before: &Observation, after: &Observation, action: &Action) {
        match action {
            Action::Blight { .. } => self.add("blight choices", 1),
            Action::PayConspire { .. } => self.add("conspire costs paid", 1),
            Action::ChooseConvoker { .. } => self.add("convoke contributions", 1),
            Action::ProliferatePermanent { .. } | Action::ProliferatePlayer { .. } => {
                self.add("proliferation choices", 1)
            }
            Action::ChooseBeholdType {
                creature_type: Some(_),
            } => self.add("behold type choices", 1),
            Action::ChooseCounterRemoval { .. } => self.add("counter removal choices", 1),
            Action::ChooseCreatureCounterCost { .. } => self.add("supplemental counter costs", 1),
            Action::CastExiledDuringResolution { object, .. } => {
                self.add("resolution cast choices", 1);
                if before.objects.iter().any(|candidate| {
                    candidate.reference == *object
                        && candidate.zone == Zone::Exile
                        && candidate.counters.get(CounterKind::Dream) > 0
                }) {
                    self.add("dream counter free cast choices", 1);
                }
            }
            Action::AnnounceLinkedFree { .. } => self.add("linked free cast declarations", 1),
            Action::ChooseExplore { .. } => self.add("explore choices", 1),
            Action::ArrangeLibrary { .. }
                if before
                    .library_choice
                    .as_ref()
                    .is_some_and(|choice| choice.surveil) =>
            {
                self.add("surveil choices committed", 1)
            }
            _ => {}
        }
        if active_packet(before).is_some_and(|item| definition(item).name() == "Dream Harvest") {
            for permission in &after.exile_permissions {
                if permission.without_mana && !before.exile_permissions.contains(permission) {
                    self.harvested.insert(permission.object);
                    self.add("Dream Harvest free permissions committed", 1);
                }
            }
        }
        // Announcement moves the card out of exile before the later cast commit.
        // Bind the exact exiled witness now, then credit only its committed stack ID.
        if let Action::AnnounceFree { card, .. } = action
            && before.objects.iter().any(|object| {
                object.id == *card
                    && object.zone == Zone::Exile
                    && self.harvested.contains(&object.reference)
            })
        {
            for packet in after.announcing.iter().chain(after.stack.iter()) {
                if packet.card == *card {
                    self.harvest_intents.insert(packet.id);
                }
            }
        }
        for row in &after.transforms {
            if !before.transforms.contains(row) {
                self.add("transform events", 1);
            }
        }
        for item in &after.stack {
            if before.stack.iter().any(|old| old.id == item.id) {
                continue;
            }
            if item.kind == StackKind::Spell {
                self.casts.insert(item.source_card.name());
                if item.is_copy {
                    self.add("spell copies created", 1);
                    if active_packet(before)
                        .is_some_and(|packet| packet.kind == StackKind::Conspire)
                    {
                        self.add("conspire spell copies created", 1);
                    }
                }
                if let Some(cast) = item.cast.filter(|_| !item.is_copy) {
                    if cast.evoked {
                        self.add("evoke casts paid", 1);
                    }
                    if cast.origin == Zone::Exile {
                        self.add("exile casts committed", 1);
                    }
                    if cast.method == CastMethod::Free {
                        self.add("free casts committed", 1);
                        if cast.origin == Zone::Exile && self.harvest_intents.contains(&item.id) {
                            self.add("Dream Harvest free casts committed", 1);
                        }
                    }
                    if cast.method == CastMethod::Normal
                        && cast.mana_spent() < item.source_card.fixed_mana_value()
                        && matches!(item.source_card.name(), "Wildvine Pummeler" | "Rime Chill")
                        && before.objects.iter().any(|o| {
                            o.zone == Zone::Battlefield
                                && o.controller == item.controller
                                && o.colors != 0
                        })
                    {
                        self.add("vivid discounted casts paid", 1);
                    }
                    if cast.method == CastMethod::Normal
                        && item.source_card.name() == "Sapling Nursery"
                        && cast.mana_spent() < item.source_card.fixed_mana_value()
                        && before.objects.iter().any(|o| {
                            o.zone == Zone::Battlefield
                                && o.controller == item.controller
                                && o.has_subtype("Forest")
                        })
                    {
                        self.add("Forest affinity casts paid", 1);
                    }
                }
            } else if item.kind == StackKind::Conspire {
                self.add("conspire triggers created", 1);
            } else if matches!(item.kind, StackKind::TriggeredAbility { .. }) {
                self.triggers.insert(definition(item).name());
            } else if matches!(item.kind, StackKind::ActivatedAbility { .. })
                && item.source_card.name() == "Stratosoarer"
                && before
                    .objects
                    .iter()
                    .any(|object| object.id == item.card && object.zone == Zone::Hand)
            {
                self.add("basic landcycling activations paid", 1);
            }
        }
        let previous = before
            .objects
            .iter()
            .map(|o| (o.id, o))
            .collect::<BTreeMap<_, _>>();
        for object in &after.objects {
            let old = previous.get(&object.id).copied();
            if object.zone == Zone::Exile
                && object.counters.get(CounterKind::Dream)
                    > old.map_or(0, |o| o.counters.get(CounterKind::Dream))
            {
                self.add("dream counters committed to exile", 1);
            }
            if object.zone == Zone::Battlefield
                && object.copy_recipe.is_some()
                && old.is_none_or(|old| {
                    old.reference != object.reference
                        || old.copy_recipe != object.copy_recipe
                        || old.copy_ability_origin != object.copy_ability_origin
                })
            {
                let entered = old.is_none_or(|old| {
                    old.reference != object.reference || old.zone != Zone::Battlefield
                });
                if object.is_token()
                    && entered
                    && active_packet(before).is_some_and(|packet| {
                        definition(packet).name() == "Twilight Diviner"
                            && packet.entry_batch_event.is_some()
                    })
                {
                    self.add("graveyard entry copy tokens created", 1);
                }
                self.add(
                    if object.is_token() && entered {
                        "copy tokens entered"
                    } else if object.is_token() {
                        "token copy effects applied"
                    } else {
                        "physical permanent copies applied"
                    },
                    1,
                );
            }
        }
        for event in &after.creature_damage_history {
            if !before.creature_damage_history.contains(event)
                && before.objects.iter().any(|source| {
                    source.reference == event.source
                        && source.keywords.contains(rustmage::Keywords::WITHER)
                })
            {
                self.add("wither creature damage events", 1);
            }
        }
        for row in &after.ability_resolutions {
            let previous = before
                .ability_resolutions
                .iter()
                .find(|old| {
                    old.source == row.source
                        && old.definition == row.definition
                        && old.kind == row.kind
                        && old.ability_origin == row.ability_origin
                })
                .map_or(0, |old| old.count);
            let count = row.count.saturating_sub(previous);
            if count == 0 {
                continue;
            }
            if matches!(row.kind, StackKind::TriggeredAbility { .. }) {
                self.add("triggered abilities resolved", count);
            }
            if row.definition.name() == "Luminollusk"
                && after
                    .players
                    .iter()
                    .map(|p| p.life_gained_this_turn)
                    .sum::<u128>()
                    > before
                        .players
                        .iter()
                        .map(|p| p.life_gained_this_turn)
                        .sum::<u128>()
            {
                self.add("vivid life gain resolutions", count);
            }
            if row.definition.name() == "Shinestriker"
                && after
                    .players
                    .iter()
                    .map(|p| p.cards_drawn_this_turn)
                    .sum::<u64>()
                    > before
                        .players
                        .iter()
                        .map(|p| p.cards_drawn_this_turn)
                        .sum::<u64>()
            {
                self.add("vivid draw resolutions", count);
            }
        }
        if let Some(item) = active_packet(before).filter(|item| {
            matches!(
                definition(item).name(),
                "Rhys, the Evermore" | "Isilu, Carrier of Twilight"
            )
        }) && let Some(link) = &item.return_link
            && after.objects.iter().any(|object| {
                object.id == link.object.card
                    && object.zone == Zone::Battlefield
                    && object.reference != link.object
                    && object.counters.get(CounterKind::MinusOne) > 0
            })
            && !before
                .objects
                .iter()
                .any(|object| object.id == link.object.card && object.zone == Zone::Battlefield)
        {
            self.add("persist returns with minus counter", 1);
        }
        if !after.attackers.is_empty() {
            self.combat += 1;
        }
    }
}
fn definition(item: &SpellView) -> Card {
    item.attachment_trigger
        .as_ref()
        .filter(|context| context.granted)
        .map_or(item.source_card, |context| context.granting_card)
}
fn active_packet(observation: &Observation) -> Option<&SpellView> {
    observation
        .resolving
        .as_ref()
        .or_else(|| observation.stack.last())
}
fn semantic_observation(game: &Game, viewer: Player) -> serde_json::Value {
    let mut value = serde_json::to_value(game.observe(viewer)).unwrap();
    value.as_object_mut().unwrap().remove("revision");
    value
}
fn private_choice(observation: &Observation) -> bool {
    observation.library_choice.is_some()
        || observation.search_choice.is_some()
        || observation.connive_choice.is_some()
        || observation
            .entry_value
            .as_ref()
            .is_some_and(|v| v.opponent_hand.is_some())
        || observation.effect_choice.as_ref().is_some_and(|choice| {
            choice.kind == rustmage::EffectChoiceKind::Discard
                || choice
                    .hand_reveal
                    .as_ref()
                    .is_some_and(|view| !view.revealed)
        })
}
fn run_workloads(workloads: Vec<Workload>, verify: bool) -> Evidence {
    let trace_steps = verify && std::env::var_os("RUSTMAGE_ECL_TRACE").is_some();
    let start = Instant::now();
    let mut evidence = Evidence::default();
    for (scope, decks, seeds) in &workloads {
        for pair in 0..decks.len() {
            for &seed in *seeds {
                for starting_player in [Player::P0, Player::P1] {
                    let config = || Config {
                        decks: [decks[pair].clone(), decks[(pair + 1) % decks.len()].clone()],
                        seed,
                        starting_player,
                        deck_rules: DeckRules::Standard,
                    };
                    if verify {
                        eprintln!(
                            "ECL_GAME start scope={scope} pair={pair} seed={seed} starter={starting_player:?}"
                        );
                    }
                    let game_start = Instant::now();
                    let mut game = Game::new(config()).unwrap();
                    let mut journal = Vec::new();
                    evidence.harvested.clear();
                    evidence.harvest_intents.clear();
                    let mut decisions = 0usize;
                    let mut last_counts = (0usize, 0usize, 0usize);
                    while game.outcome().is_none() {
                        assert!(
                            decisions < 25000,
                            "ECL stalled scope={scope} pair={pair} seed={seed} starter={starting_player:?}"
                        );
                        if verify && decisions.is_multiple_of(128) {
                            eprintln!(
                                "ECL_PROGRESS scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} seconds={:.3} visible_objects={} battlefield={} stack={}",
                                game_start.elapsed().as_secs_f64(),
                                last_counts.0,
                                last_counts.1,
                                last_counts.2
                            );
                        }
                        let trace = |stage: &str| {
                            if trace_steps {
                                eprintln!(
                                    "ECL_TRACE scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} stage={stage}"
                                );
                            }
                        };
                        let decision_start = verify.then(Instant::now);
                        let actor = game.actor().unwrap();
                        trace("legal_actions");
                        let actions = game.legal_actions();
                        assert!(!actions.is_empty());
                        trace("policy");
                        let index = rustmage::policy::smoke_index(&game, &actions);
                        let action = actions[index].clone();
                        assert!(
                            !matches!(action, Action::Concede),
                            "policy dead end scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} view={:?}",
                            game.observe(actor)
                        );
                        if trace_steps {
                            eprintln!(
                                "ECL_ACTION decision={decisions} actor={actor:?} action={action:?}"
                            );
                        }
                        if verify {
                            trace("observe_before");
                            let before_view = game.observe(actor);
                            let private = private_choice(&before_view);
                            for viewer in [Player::P0, Player::P1] {
                                let view = game.observe(viewer);
                                assert!(view.objects.iter().all(|o| o.zone != Zone::Hand
                                    || o.owner == viewer
                                    || view.known_hand.contains(&o.reference)));
                                if viewer != actor {
                                    assert!(view.actions.is_empty());
                                    assert!(view.library_choice.is_none());
                                    assert!(view.search_choice.is_none());
                                    assert!(view.connive_choice.is_none());
                                }
                            }
                            if private {
                                evidence.private += 1;
                            }
                            if decisions.is_multiple_of(64) || private {
                                trace("branch_snapshot");
                                let snapshot = game.snapshot();
                                let revision = game.revision();
                                assert_eq!(
                                    game.apply(actor.other(), action.clone()),
                                    Err(Error::WrongActor)
                                );
                                assert_eq!(game.snapshot(), snapshot);
                                let invalid = Action::ChooseMode { mode: u16::MAX };
                                assert!(!actions.contains(&invalid));
                                assert_eq!(game.apply(actor, invalid), Err(Error::IllegalAction));
                                assert_eq!(game.snapshot(), snapshot);
                                assert_eq!(game.revision(), revision);
                                let mut child = {
                                    let parent = game.clone();
                                    parent.clone()
                                };
                                trace("branch_apply_index");
                                child.apply_index(actor, revision, index).unwrap();
                                trace("branch_apply_raw");
                                game.apply(actor, action.clone()).unwrap();
                                assert_eq!(child.snapshot(), game.snapshot());
                                for viewer in [Player::P0, Player::P1] {
                                    assert_eq!(
                                        serde_json::to_value(child.observe(viewer)).unwrap(),
                                        serde_json::to_value(game.observe(viewer)).unwrap()
                                    );
                                }
                                let after = game.snapshot();
                                trace("restore");
                                game.restore(&snapshot).unwrap();
                                assert_eq!(
                                    game.apply_index(actor, revision, index),
                                    Err(Error::StaleRevision)
                                );
                                game.apply(actor, action.clone()).unwrap();
                                assert_eq!(game.snapshot(), after);
                                evidence.branches += 1;
                            } else {
                                trace("apply_raw");
                                game.apply(actor, action.clone()).unwrap();
                            }
                            trace("observe_transition");
                            let after_view = game.observe(actor);
                            last_counts = (
                                after_view.objects.len(),
                                after_view
                                    .objects
                                    .iter()
                                    .filter(|o| o.zone == Zone::Battlefield)
                                    .count(),
                                after_view.stack.len(),
                            );
                            evidence.transition(&before_view, &after_view, &action);
                            journal.push((actor, action));
                        } else {
                            game.apply(actor, action).unwrap();
                        }
                        if let Some(decision_start) = decision_start
                            && decision_start.elapsed().as_secs() >= 1
                        {
                            eprintln!(
                                "ECL_SLOW scope={scope} pair={pair} seed={seed} starter={starting_player:?} decision={decisions} seconds={:.3}",
                                decision_start.elapsed().as_secs_f64()
                            );
                        }
                        decisions += 1;
                    }
                    if verify {
                        eprintln!(
                            "ECL_REPLAY start scope={scope} pair={pair} seed={seed} starter={starting_player:?} decisions={decisions}"
                        );
                        let mut replay = Game::new(config()).unwrap();
                        for (replayed, (actor, action)) in journal.into_iter().enumerate() {
                            if trace_steps {
                                eprintln!(
                                    "ECL_REPLAY_ACTION decision={replayed} actor={actor:?} action={action:?}"
                                );
                            }
                            replay.apply(actor, action).unwrap();
                            evidence.replayed += 1;
                        }
                        assert_eq!(replay.snapshot(), game.snapshot());
                        for viewer in [Player::P0, Player::P1] {
                            assert_eq!(
                                semantic_observation(&replay, viewer),
                                semantic_observation(&game, viewer)
                            );
                        }
                    }
                    match game.outcome().unwrap() {
                        Outcome::Winner(Player::P0) => evidence.wins[0] += 1,
                        Outcome::Winner(Player::P1) => evidence.wins[1] += 1,
                        Outcome::Draw => evidence.draws += 1,
                    }
                    evidence.drawn_to_empty += usize::from(
                        game.observe(Player::P0)
                            .players
                            .iter()
                            .any(|p| p.library_count == 0),
                    );
                    if verify {
                        eprintln!(
                            "ECL_GAME end scope={scope} pair={pair} seed={seed} starter={starting_player:?} decisions={decisions} seconds={:.3}",
                            game_start.elapsed().as_secs_f64()
                        );
                    }
                    evidence.games += 1;
                    evidence.decisions += decisions;
                }
            }
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let result = serde_json::json!({
        "mode": if verify { "verified natural games" } else { "optimized policy simulation" },
        "physical_members": members().len(),
        "supported_members": members().iter().filter(|c| c.is_supported()).count(),
        "workloads": workloads.iter().map(|(scope, decks, seeds)| serde_json::json!({
            "scope": scope, "decks": decks.len(), "seeds": seeds, "starters": ["P0", "P1"]
        })).collect::<Vec<_>>(),
        "games": evidence.games,
        "decisions": evidence.decisions,
        "owned_branches": evidence.branches,
        "private_decisions": evidence.private,
        "replayed_actions": evidence.replayed,
        "combat_observations": evidence.combat,
        "wins": evidence.wins,
        "draws": evidence.draws,
        "end_states_with_empty_library": evidence.drawn_to_empty,
        "seconds": elapsed,
        "decisions_per_second": evidence.decisions as f64 / elapsed,
        "games_per_second": evidence.games as f64 / elapsed,
        "decision_definition": "one accepted policy-selected action",
        "timing_scope": if verify {
            "game initialization, legal actions, smoke policy, apply, rules evidence, privacy, owned branches, restore and complete journal replay; excludes compilation and workload construction"
        } else {
            "game initialization, legal actions, smoke policy, apply and terminal checks; excludes compilation, workload construction, rules evidence, privacy/branch/restore checks and replay"
        },
        "cast_card_names": evidence.casts,
        "triggered_card_names": evidence.triggers,
        "executed": evidence.executed
    });
    println!("ECL_ACCEPTANCE {result}");
    evidence
}
#[test]
fn lorwyn_eclipsed_entire_pool_and_theme_acceptance() {
    let evidence = run_workloads(workloads(), true);
    assert!(
        evidence.games > 100
            && evidence.decisions > 10000
            && evidence.branches > 100
            && evidence.private > 0
            && evidence.combat > 0
    );
    assert_eq!(evidence.replayed, evidence.decisions);
    assert_eq!(
        evidence.games, 236,
        "declared seeds, decks and both starters must all run"
    );
    // Rules coverage is measured by committed choices/state changes, never deck inclusion.
    for mechanic in [
        "blight choices",
        "convoke contributions",
        "proliferation choices",
        "surveil choices committed",
        "evoke casts paid",
        "Forest affinity casts paid",
        "vivid discounted casts paid",
        "physical permanent copies applied",
        "spell copies created",
        "transform events",
        "behold type choices",
        "basic landcycling activations paid",
        "persist returns with minus counter",
        "conspire costs paid",
        "conspire spell copies created",
        "graveyard entry copy tokens created",
        "dream counters committed to exile",
        "dream counter free cast choices",
        "Dream Harvest free casts committed",
        "wither creature damage events",
    ] {
        assert!(
            evidence
                .executed
                .get(mechanic)
                .is_some_and(|&count| count > 0),
            "mechanic not executed in final workload: {mechanic}"
        );
    }
}
#[test]
#[ignore = "explicit optimized workload measurement after strict full-set acceptance"]
#[allow(
    clippy::assertions_on_constants,
    reason = "the ignored measurement must reject debug runs at runtime without preventing normal debug test compilation"
)]
fn lorwyn_eclipsed_optimized_throughput() {
    assert!(
        !cfg!(debug_assertions),
        "optimized workload must run with --release"
    );
    let evidence = run_workloads(workloads(), false);
    assert!(evidence.games > 100 && evidence.decisions > 10000);
}
