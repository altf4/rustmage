//! Deterministic FIN-only Standard games through the public Rust API.
use rustmage::{
    Action, Card, CastFace, CastMethod, Config, CounterKind, DeckRules, EffectChoiceKind, Error,
    Game, Observation, Outcome, Player, SpellView, StackKind, Zone,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

const BASICS: [Card; 5] = [
    Card::Plains,
    Card::Island,
    Card::Swamp,
    Card::Mountain,
    Card::Forest,
];
const COLORS: &str = "WUBRG";
const POOL_SEEDS: &[u64] = &[0, 7];
const THEME_SEEDS: &[u64] = &[0, 7, 19, 31];
const ACTION_LIMIT: usize = 16_384;

#[derive(serde::Deserialize)]
struct Theme {
    name: String,
    colors: String,
    cards: [String; 9],
}

struct Deck {
    name: String,
    cards: Vec<Card>,
}

struct Case {
    scope: &'static str,
    decks: [String; 2],
    config: Config,
}

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap_or_else(|| panic!("missing FIN identity {name}"))
}

fn standard_members() -> BTreeSet<Card> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../data/sets/fin.json")).unwrap();
    assert_eq!(manifest["card_count"].as_u64(), Some(312));
    let mut cards: BTreeSet<_> = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| card(row["name"].as_str().unwrap()))
        .collect();
    assert_eq!(cards.len(), 312);
    for member in &cards {
        assert!(
            member.is_supported(),
            "gated FIN member {}",
            member.identity()
        );
    }
    for excluded in [card("Ragnarok, Divine Deliverance"), card("Vivi Ornitier")] {
        assert!(cards.remove(&excluded));
        let mut deck = vec![Card::Forest; 59];
        deck.push(excluded);
        assert!(
            matches!(
                Game::new(Config {
                    decks: [deck, vec![Card::Forest; 60]],
                    seed: 0,
                    starting_player: Player::P0,
                    deck_rules: DeckRules::Standard,
                }),
                Err(Error::InvalidDeck(_))
            ),
            "Standard accepted {}",
            excluded.identity()
        );
    }
    assert_eq!(cards.len(), 310);
    assert!(cards.iter().all(|card| card.is_standard_legal()));
    cards
}

fn deck(colors: u8, featured: &[Card]) -> Vec<Card> {
    assert_eq!(featured.len(), 9);
    let basics: Vec<_> = BASICS
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(i, card)| (colors & (1 << i) != 0).then_some(card))
        .collect();
    let mut cards: Vec<_> = basics.into_iter().cycle().take(24).collect();
    for &card in featured {
        cards.extend([card; 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}

fn pool_decks(members: &BTreeSet<Card>) -> Vec<Deck> {
    let (lands, mut spells): (Vec<_>, Vec<_>) =
        members.iter().copied().partition(|card| card.is_land());
    spells.sort_by_key(|card| (card.colors(), card.fixed_mana_value(), card.identity()));
    let anchors = [
        "Zack Fair",
        "Adelbert Steiner",
        "Aerith Gainsborough",
        "Ambrosia Whiteheart",
        "Il Mheg Pixie",
        "Sahagin",
        "Quistis Trepe",
        "Jill, Shiva's Dominant",
        "Hecteyes",
        "Dark Confidant",
        "Fang, Fearless l'Cie",
        "Kain, Traitorous Dragoon",
        "Blazing Bomb",
        "Freya Crescent",
        "Mysidian Elder",
        "Vaan, Street Thief",
        "Goobbue Gardener",
        "Torgal, A Fine Hound",
        "Traveling Chocobo",
        "Loporrit Scout",
    ]
    .map(card);
    let mut decks: Vec<_> = spells
        .chunks(7)
        .enumerate()
        .map(|(i, group)| {
            let colors = group.iter().fold(0, |mask, card| mask | card.colors());
            let colors = if colors == 0 {
                1 << (i % BASICS.len())
            } else {
                colors
            };
            let mut featured = group.to_vec();
            for &candidate in anchors.iter().chain(&spells) {
                if featured.len() == 9 {
                    break;
                }
                if candidate.is_creature()
                    && candidate.colors() & !colors == 0
                    && !featured.contains(&candidate)
                {
                    featured.push(candidate);
                }
            }
            Deck {
                name: format!("pool {i}"),
                cards: deck(colors, &featured),
            }
        })
        .collect();
    // Make room for each utility land without crowding out the decks' spell colors.
    let count = decks.len();
    for (i, land) in lands
        .into_iter()
        .filter(|land| !BASICS.contains(land))
        .enumerate()
    {
        let slot = 2 * (i / count);
        assert!(slot + 2 <= 24);
        decks[i % count].cards[slot..slot + 2].fill(land);
    }
    assert_eq!(
        decks
            .iter()
            .flat_map(|deck| deck.cards.iter().copied())
            .collect::<BTreeSet<_>>(),
        *members
    );
    decks
}

fn cases(members: &BTreeSet<Card>, themes: &[Theme]) -> Vec<Case> {
    let themes = themes
        .iter()
        .map(|theme| {
            let mask = theme.colors.bytes().fold(0, |mask, color| {
                mask | (1 << COLORS.find(char::from(color)).unwrap())
            });
            let featured = theme.cards.each_ref().map(|name| card(name));
            assert!(featured.iter().all(|card| card.colors() & !mask == 0));
            Deck {
                name: theme.name.clone(),
                cards: deck(mask, &featured),
            }
        })
        .collect();
    let mut cases = Vec::new();
    for (scope, decks, seeds) in [
        ("entire deck-legal pool", pool_decks(members), POOL_SEEDS),
        ("mechanic themes", themes, THEME_SEEDS),
    ] {
        assert!(
            decks
                .iter()
                .all(|deck| deck.cards.iter().all(|card| members.contains(card)))
        );
        for i in 0..decks.len() {
            let pair = [&decks[i], &decks[(i + 1) % decks.len()]];
            for &seed in seeds {
                for starting_player in [Player::P0, Player::P1] {
                    cases.push(Case {
                        scope,
                        decks: pair.map(|deck| deck.name.clone()),
                        config: Config {
                            decks: pair.map(|deck| deck.cards.clone()),
                            seed,
                            starting_player,
                            deck_rules: DeckRules::Standard,
                        },
                    });
                }
            }
        }
    }
    cases
}

#[derive(Default)]
struct Evidence {
    games: usize,
    decisions: usize,
    wins: [usize; 2],
    draws: usize,
    empty_library_endings: usize,
    replayed: usize,
    owned_branches: usize,
    distinct_siblings: usize,
    private_choices: usize,
    spent_colors: u8,
    casts: BTreeSet<&'static str>,
    cast_faces: BTreeSet<&'static str>,
    ability_sources: BTreeSet<&'static str>,
    mechanics: BTreeMap<&'static str, u64>,
}

fn source_definition(packet: &SpellView) -> Option<Card> {
    packet
        .attachment_trigger
        .as_ref()
        .filter(|grant| grant.granted)
        .map(|grant| grant.granting_card)
        .or_else(|| packet.source().map(|source| source.card))
}

impl Evidence {
    fn add(&mut self, mechanic: &'static str, count: u64) {
        if count != 0 {
            *self.mechanics.entry(mechanic).or_default() += count;
        }
    }

    fn transition(&mut self, before: &Observation, after: &Observation, action: &Action) {
        match action {
            Action::ArrangeLibrary { .. } => {
                if let Some(choice) = &before.library_choice {
                    self.add(
                        if choice.surveil {
                            "surveil choices"
                        } else {
                            "scry choices"
                        },
                        1,
                    );
                }
            }
            Action::PlayLand { card } => {
                if let Some(land) = after
                    .objects
                    .iter()
                    .find(|o| o.id == *card && o.zone == Zone::Battlefield)
                {
                    self.add("land plays", 1);
                    if land.has_subtype("Town") {
                        self.add("Town land plays", 1);
                    }
                    if before
                        .adventures
                        .iter()
                        .any(|permission| permission.object.card == *card)
                    {
                        self.add("Adventure land plays", 1);
                    }
                }
            }
            Action::FinishAttackers if !after.attackers.is_empty() => {
                self.add("attack declarations", 1)
            }
            Action::PayResolutionMana { .. } => self.add("resolution mana payments", 1),
            _ => (),
        }
        if after.melded_permanents.len() > before.melded_permanents.len() {
            self.add(
                "melded permanents",
                (after.melded_permanents.len() - before.melded_permanents.len()) as u64,
            );
        }
        for packet in &after.stack {
            if before.stack.iter().any(|old| old.id == packet.id) {
                continue;
            }
            let Some(source) = packet.source() else {
                continue;
            };
            if packet.kind == StackKind::Spell && packet.is_copy {
                self.add("spell copies on stack", 1);
            }
            if let Some(cast) = packet
                .cast
                .filter(|_| packet.kind == StackKind::Spell && !packet.is_copy)
            {
                self.casts
                    .insert(cast.physical_card.unwrap_or(source.card).identity());
                self.cast_faces.insert(source.card.name());
                for (i, amount) in cast.payment[..5].iter().enumerate() {
                    if *amount > 0 {
                        self.spent_colors |= 1 << i;
                    }
                }
                if cast.face == CastFace::Adventure {
                    self.add("Adventure casts", 1);
                }
                if matches!(
                    cast.method,
                    CastMethod::Flashback | CastMethod::GrantedFlashback
                ) {
                    self.add("flashback casts", 1);
                }
                if cast.origin == Zone::Exile {
                    self.add("exile casts", 1);
                }
                if cast.origin == Zone::Graveyard {
                    self.add("graveyard casts", 1);
                }
                if cast.treasure_mana_spent {
                    self.add("Treasure-funded casts", 1);
                }
                if packet.mode.is_some()
                    && matches!(
                        source.card.name(),
                        "Fire Magic" | "Ice Magic" | "Thunder Magic" | "Vincent's Limit Break"
                    )
                {
                    self.add("tiered casts", 1);
                }
            }
            if matches!(packet.kind, StackKind::ActivatedAbility { .. }) {
                if packet.crew.is_some() {
                    self.add("crew abilities paid", 1);
                }
                if matches!(
                    source.card.name(),
                    "Hill Gigas" | "Ice Flan" | "Malboro" | "Sandworm"
                ) && before
                    .objects
                    .iter()
                    .any(|o| o.id == source.id && o.zone == Zone::Hand)
                {
                    self.add("typecycling abilities paid", 1);
                }
            }
        }

        // Only the resolving packet or stack top could complete on this action.
        // A lower countered packet is never credited as having resolved.
        let resolving = before.resolving.as_ref().or_else(|| {
            matches!(action, Action::Pass)
                .then(|| before.stack.last())
                .flatten()
        });
        if let Some(packet) = resolving
            && matches!(
                packet.kind,
                StackKind::ActivatedAbility { .. } | StackKind::TriggeredAbility { .. }
            )
            && after
                .resolving
                .as_ref()
                .is_none_or(|current| current.id != packet.id)
            && after.stack.iter().all(|current| current.id != packet.id)
            && let Some(card) = source_definition(packet)
        {
            self.ability_sources.insert(card.identity());
        }

        let previous: BTreeMap<_, _> = before
            .objects
            .iter()
            .map(|object| (object.id, object))
            .collect();
        for object in after.objects.iter().filter(|o| o.zone == Zone::Battlefield) {
            let old = previous
                .get(&object.id)
                .copied()
                .filter(|old| old.reference == object.reference && old.zone == Zone::Battlefield);
            if object.is_token() && old.is_none() {
                self.add("token entries", 1);
                for (subtype, mechanic) in [
                    ("Hero", "Hero entries"),
                    ("Bird", "Bird entries"),
                    ("Wizard", "Wizard entries"),
                    ("Treasure", "Treasure entries"),
                    ("Food", "Food entries"),
                ] {
                    if object.has_subtype(subtype) {
                        self.add(mechanic, 1);
                    }
                }
                if object.copy_recipe.is_some() {
                    self.add("copy-token entries", 1);
                }
            }
            if object.has_subtype("Equipment")
                && object.attached_to.is_some()
                && old.and_then(|o| o.attached_to) != object.attached_to
            {
                self.add("Equipment attachments", 1);
            }
            if old.is_some_and(|prior| prior.controller != object.controller) {
                self.add("permanent control changes", 1);
            }
            if let Some(face) = &object.face {
                if old.is_none() && face.back {
                    self.add("transformed entries", 1);
                }
                let previous = old
                    .and_then(|prior| prior.face.as_ref())
                    .map_or(0, |f| f.transforms);
                self.add(
                    "transforms in place",
                    face.transforms.saturating_sub(previous),
                );
            }
            for (kind, mechanic) in [
                (CounterKind::Lore, "lore counters placed"),
                (CounterKind::Finality, "finality counters placed"),
                (CounterKind::Stun, "stun counters placed"),
            ] {
                self.add(
                    mechanic,
                    object
                        .counters
                        .get(kind)
                        .saturating_sub(old.map_or(0, |prior| prior.counters.get(kind))),
                );
            }
        }
    }
}

fn private_choice(view: &Observation) -> bool {
    view.library_choice.is_some()
        || view
            .library_selection
            .as_ref()
            .is_some_and(|choice| !choice.public)
        || view.search_choice.is_some()
        || view.library_cast_look.is_some()
        || view.hand_library_search.is_some()
        || view.connive_choice.is_some()
        || view.hand_choice.is_some()
        || view.effect_choice.as_ref().is_some_and(|choice| {
            choice.inspected_cards.is_some()
                || matches!(
                    choice.kind,
                    EffectChoiceKind::Discard
                        | EffectChoiceKind::HandToTop
                        | EffectChoiceKind::BottomHand
                        | EffectChoiceKind::SacrificeOrDiscard { .. }
                )
        })
}

fn check_privacy(view: &Observation, actor: Player) {
    assert_eq!(view.actor, Some(actor));
    assert!(view.objects.iter().all(|o| {
        !matches!(o.zone, Zone::Library | Zone::Outside)
            && (o.zone != Zone::Hand
                || o.owner == view.viewer
                || view.known_hand.contains(&o.reference))
    }));
    if view.viewer != actor {
        assert!(view.actions.is_empty());
        assert!(!private_choice(view));
    }
}

fn semantic_view(game: &Game, player: Player) -> Vec<u8> {
    let mut view = game.observe(player);
    view.revision = 0;
    serde_json::to_vec(&view).unwrap()
}

fn verify_branch(game: &mut Game, view: &Observation, index: usize, evidence: &mut Evidence) {
    let actor = view.viewer;
    let action = &view.actions[index];
    let root = game.snapshot();
    assert_eq!(game.legal_actions(), view.actions);
    assert_eq!(
        game.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(
        game.apply(actor, Action::ChooseMode { mode: u16::MAX }),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), root);
    assert_eq!(game.revision(), view.revision);

    let mut indexed = {
        let parent = game.clone();
        parent.clone()
    };
    if let Some((sibling_index, _)) = view
        .actions
        .iter()
        .enumerate()
        .find(|(i, other)| *i != index && !matches!(other, Action::Concede))
    {
        let mut sibling = indexed.clone();
        sibling
            .apply_index(actor, view.revision, sibling_index)
            .unwrap();
        assert_eq!(game.snapshot(), root);
        assert_eq!(indexed.snapshot(), root);
        indexed.apply_index(actor, view.revision, index).unwrap();
        evidence.distinct_siblings += usize::from(sibling.snapshot() != indexed.snapshot());
    } else {
        indexed.apply_index(actor, view.revision, index).unwrap();
    }

    game.apply(actor, action.clone()).unwrap();
    let committed = game.snapshot();
    assert_eq!(indexed.snapshot(), committed);
    for seat in [Player::P0, Player::P1] {
        assert_eq!(semantic_view(&indexed, seat), semantic_view(game, seat));
    }
    game.restore(&root).unwrap();
    assert_eq!(
        game.apply_index(actor, view.revision, index),
        Err(Error::StaleRevision)
    );
    game.apply(actor, action.clone()).unwrap();
    assert_eq!(game.snapshot(), committed);
    evidence.owned_branches += 1;
}

fn play(case: &Case, verify: bool, evidence: &mut Evidence) {
    let mut game = Game::new(case.config.clone()).unwrap();
    let mut journal = Vec::new();
    let mut private_boundaries = BTreeSet::new();
    let mut decisions = 0usize;
    while game.outcome().is_none() {
        assert!(
            decisions < ACTION_LIMIT,
            "FIN stalled in {} {:?}, seed {}, starter {:?}, after {decisions} actions",
            case.scope,
            case.decks,
            case.config.seed,
            case.config.starting_player,
        );
        let actor = game.actor().unwrap();
        let before = game.observe(actor);
        let index = rustmage::policy::smoke_index_from_observation(&before, &before.actions);
        let action = before.actions[index].clone();
        assert!(
            !matches!(action, Action::Concede),
            "FIN policy dead end in {} {:?}, seed {}, starter {:?}, turn {}, {}",
            case.scope,
            case.decks,
            case.config.seed,
            case.config.starting_player,
            before.turn,
            before.decision,
        );
        if verify {
            check_privacy(&before, actor);
            check_privacy(&game.observe(actor.other()), actor);
            let private = private_choice(&before);
            evidence.private_choices += usize::from(private);
            if decisions.is_multiple_of(128)
                || (private && private_boundaries.insert(before.decision))
            {
                verify_branch(&mut game, &before, index, evidence);
            } else {
                game.apply(actor, action.clone()).unwrap();
            }
            evidence.transition(&before, &game.observe(actor), &action);
            journal.push((actor, action));
        } else {
            game.apply(actor, action).unwrap();
        }
        decisions += 1;
    }

    if verify {
        let mut replay = Game::new(case.config.clone()).unwrap();
        for (actor, action) in journal {
            replay.apply(actor, action).unwrap();
            evidence.replayed += 1;
        }
        assert_eq!(replay.snapshot(), game.snapshot());
        for seat in [Player::P0, Player::P1] {
            assert_eq!(semantic_view(&replay, seat), semantic_view(&game, seat));
        }
        evidence.empty_library_endings += usize::from(
            game.observe(Player::P0)
                .players
                .iter()
                .any(|player| player.library_count == 0),
        );
    }
    match game.outcome().unwrap() {
        Outcome::Winner(winner) => evidence.wins[winner.index()] += 1,
        Outcome::Draw => evidence.draws += 1,
    }
    evidence.games += 1;
    evidence.decisions += decisions;
}

#[allow(
    clippy::assertions_on_constants,
    reason = "the measured benchmark requires an optimized binary"
)]
fn main() -> Result<(), &'static str> {
    let verify = match std::env::args().nth(1).as_deref() {
        Some("verify") => true,
        Some("benchmark") => {
            assert!(
                !cfg!(debug_assertions),
                "run the FIN benchmark with --release"
            );
            false
        }
        _ => return Err("use verify or benchmark"),
    };
    let members = standard_members();
    let themes: Vec<Theme> = serde_json::from_str(include_str!("support/fin_themes.json")).unwrap();
    let cases = cases(&members, &themes);
    let mut evidence = Evidence::default();
    let start = Instant::now();
    for (index, case) in cases.iter().enumerate() {
        if verify && index.is_multiple_of(16) {
            eprintln!(
                "FIN progress {index}/{} games: {} {:?}",
                cases.len(),
                case.scope,
                case.decks
            );
        }
        play(case, verify, &mut evidence);
    }
    let seconds = start.elapsed().as_secs_f64();
    assert_eq!(evidence.games, cases.len());
    assert_eq!(
        evidence.wins.iter().sum::<usize>() + evidence.draws,
        evidence.games
    );
    if verify {
        assert_eq!(evidence.replayed, evidence.decisions);
        assert!(evidence.owned_branches >= evidence.games);
        assert!(evidence.distinct_siblings > 0 && evidence.private_choices > 0);
        assert_eq!(evidence.spent_colors, 0b1_1111);
        for mechanic in [
            "Adventure casts",
            "Equipment attachments",
            "Hero entries",
            "attack declarations",
            "crew abilities paid",
            "flashback casts",
            "lore counters placed",
            "surveil choices",
            "token entries",
            "transformed entries",
            "transforms in place",
            "typecycling abilities paid",
        ] {
            assert!(
                evidence.mechanics.contains_key(mechanic),
                "missing FIN game outcome: {mechanic}"
            );
        }
    }
    let report = serde_json::json!({
        "mode": if verify { "verified FIN-only Standard games" } else { "optimized FIN-only Standard simulation" },
        "set_identities": 312,
        "deck_legal_members": members.len(),
        "pool_games": cases.iter().filter(|case| case.scope == "entire deck-legal pool").count(),
        "theme_games": cases.iter().filter(|case| case.scope == "mechanic themes").count(),
        "theme_names": themes.iter().map(|theme| &theme.name).collect::<Vec<_>>(),
        "pool_seeds": POOL_SEEDS,
        "theme_seeds": THEME_SEEDS,
        "games": evidence.games,
        "wins": evidence.wins,
        "draws": evidence.draws,
        "decisions": evidence.decisions,
        "seconds": seconds,
        "games_per_second": evidence.games as f64 / seconds,
        "decisions_per_second": evidence.decisions as f64 / seconds,
        "verification": verify.then(|| serde_json::json!({
            "owned_branches": evidence.owned_branches,
            "distinct_siblings": evidence.distinct_siblings,
            "private_choices": evidence.private_choices,
            "replayed_actions": evidence.replayed,
            "end_states_with_empty_library": evidence.empty_library_endings,
            "spent_colors": COLORS.chars().enumerate().filter_map(|(i, color)| (evidence.spent_colors & (1 << i) != 0).then_some(color)).collect::<String>(),
            "distinct_cast_members": evidence.casts.len(),
            "cast_members": evidence.casts,
            "cast_faces": evidence.cast_faces,
            "completed_ability_sources": evidence.ability_sources,
            "mechanical_outcomes": evidence.mechanics,
        })),
        "timing_scope": if verify {
            "game initialization, observation, policy, apply, privacy, owned checks, outcome receipts, and full replay"
        } else {
            "game initialization, actor observation, policy, apply, and terminal checks"
        },
    });
    println!("FIN_ACCEPTANCE {report}");
    Ok(())
}
