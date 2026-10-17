//! Legal Standard Adventure games with paid faces, complete effect witnesses,
//! remaining permanent abilities, real attacks, and owned branch replay.
use rustmage::{
    Action, BoundTarget, Card, Config, DeckRules, Error, Game, Keywords, Observation, Player,
    StackKind, Target, Zone,
};

const NAMES: [&str; 19] = [
    "Bilbo Baggins, Burglar // Take a Glance",
    "Frolicking Familiar // Blow Off Steam",
    "Galvanic Giant // Storm Reading",
    "Gingerbread Hunter // Puny Snack",
    "Glóin the Mighty // Easy Pickings",
    "Gollum, Silent Slinker // Meager Meal",
    "Grabby Giant // That's Mine",
    "Hollow Scavenger // Bakery Raid",
    "Imodane's Recruiter // Train Troops",
    "Midgar, City of Mako // Reactor Raid",
    "Minecart Daredevil // Ride the Rails",
    "Obyra's Attendants // Desperate Parry",
    "Ratcatcher Trainee // Pest Problem",
    "Shrouded Shepherd // Cleave Shadows",
    "Smaug, the Great Calamity // Spew Flame",
    "Stormkeld Vanguard // Bear Down",
    "Tempest Hart // Scan the Clouds",
    "Threadbind Clique // Rip the Seams",
    "Two-Headed Hunter // Twice the Rage",
];
fn deck(i: usize) -> Vec<Card> {
    let mut cards = Vec::new();
    for (name, count) in [
        ("Plains", 6),
        ("Island", 6),
        ("Swamp", 6),
        ("Mountain", 6),
        ("Forest", 8),
    ] {
        cards.extend(vec![Card::from_name(name).unwrap(); count]);
    }
    for name in [
        NAMES[i],
        "Bear Cub",
        "Chrome Companion",
        "Llanowar Elves",
        "Bygone Colossus",
        "Thrill of Possibility",
        "Disenchant",
    ] {
        cards.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(cards.len(), 60);
    cards
}
#[derive(Debug)]
struct Flight {
    before: Observation,
    targets: Vec<BoundTarget>,
    scry: u32,
}
#[derive(Default, Debug)]
struct Pilot {
    inset: Option<u32>,
    setup_ready: bool,
    permission: bool,
    normal: bool,
    second: bool,
    second_id: Option<u32>,
    first_effect: Option<Flight>,
    main_effect: Option<Flight>,
    second_effect: Option<Flight>,
    activation: Option<Flight>,
    main_verified: bool,
    extra: bool,
    attacked: bool,
    high_paid: bool,
    own_strike: bool,
    other_strike: bool,
    mana_trigger: Option<(u64, u32)>,
}
fn complete(p: &Pilot) -> bool {
    body_ready(p) && p.attacked
}
fn body_ready(p: &Pilot) -> bool {
    p.permission && p.normal && p.main_verified && p.second && p.extra
}
fn quiet(o: &Observation) -> bool {
    o.decision == "priority"
        && o.stack.is_empty()
        && o.announcing.is_none()
        && o.resolving.is_none()
}
fn token_count(o: &Observation, p: Player, subtype: &str) -> usize {
    o.objects
        .iter()
        .filter(|v| {
            v.owner == p
                && v.zone == Zone::Battlefield
                && v.card.is_token()
                && v.card.has_subtype(subtype)
        })
        .count()
}
fn creature_target(f: &Flight) -> u32 {
    f.targets
        .iter()
        .find_map(|t| {
            if let Target::Card(id) = t.target {
                Some(id)
            } else {
                None
            }
        })
        .expect("chosen creature")
}
fn damage(f: &Flight, o: &Observation, target: Target, amount: i128) {
    match target {
        Target::Player(p) => assert_eq!(
            o.players[p.index()].life,
            f.before.players[p.index()].life - amount
        ),
        Target::Card(id) => {
            let before = f.before.objects.iter().find(|v| v.id == id).unwrap();
            let after = o.objects.iter().find(|v| v.id == id).unwrap();
            if after.zone == Zone::Battlefield {
                assert_eq!(after.damage, before.damage + amount)
            } else {
                assert!(matches!(after.zone, Zone::Graveyard | Zone::Outside));
                assert!(before.damage + amount >= before.toughness);
            }
        }
        Target::Spell(_) => panic!("damage cannot target a spell"),
    }
}
fn verify_adventure(i: usize, p: Player, f: &Flight, o: &Observation) {
    let before = &f.before;
    let changed = |id| {
        let b = before.objects.iter().find(|v| v.id == id).unwrap();
        let a = o.objects.iter().find(|v| v.id == id).unwrap();
        (b, a)
    };
    match i {
        0 => assert_eq!(f.scry, 2, "both Scry2 card decisions"),
        1 | 14 => damage(f, o, f.targets[0].target, if i == 1 { 1 } else { 5 }),
        2 | 16 => {
            let n = if i == 2 { 4 } else { 2 };
            assert_eq!(
                o.players[p.index()].cards_drawn_this_turn,
                before.players[p.index()].cards_drawn_this_turn + n
            );
            assert_eq!(
                o.players[p.index()].cards_discarded_this_turn,
                before.players[p.index()].cards_discarded_this_turn + 2
            );
        }
        3 | 10 | 11 => {
            let (b, a) = changed(creature_target(f));
            let (dp, dt) = match i {
                3 => (-2, -2),
                10 => (2, 1),
                _ => (-4, 0),
            };
            if a.zone == Zone::Battlefield {
                assert_eq!((a.power, a.toughness), (b.power + dp, b.toughness + dt))
            } else {
                assert_eq!(i, 3);
                assert!(b.toughness + dt <= 0);
                assert_eq!(a.zone, Zone::Graveyard);
            }
        }
        4 | 13 => {
            let victims: Vec<_> = before
                .objects
                .iter()
                .filter(|v| v.owner == p.other() && v.zone == Zone::Battlefield && v.is_creature())
                .collect();
            assert!(!victims.is_empty());
            for v in victims {
                if i == 4 {
                    damage(f, o, Target::Card(v.id), 1)
                } else {
                    let (_, a) = changed(v.id);
                    if v.toughness > 1 {
                        assert_eq!(a.zone, Zone::Battlefield);
                        assert_eq!((a.power, a.toughness), (v.power - 1, v.toughness - 1));
                    } else {
                        assert!(matches!(a.zone, Zone::Graveyard | Zone::Outside));
                    }
                }
            }
        }
        5 => {
            let (b, a) = changed(creature_target(f));
            assert_eq!(a.counters.plus_one, b.counters.plus_one + 1);
            let target = f
                .targets
                .iter()
                .find_map(|t| {
                    if let Target::Player(q) = t.target {
                        Some(q)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(
                o.players[target.index()].life,
                before.players[target.index()].life + 2
            );
        }
        6 | 7 => {
            let sub = if i == 6 { "Treasure" } else { "Food" };
            assert_eq!(token_count(o, p, sub), token_count(before, p, sub) + 1);
        }
        8 | 12 => {
            let (sub, pt, colors, keyword) = if i == 8 {
                ("Knight", (2, 2), 1, Keywords::VIGILANCE)
            } else {
                ("Rat", (1, 1), 4, Keywords::CANT_BLOCK)
            };
            assert_eq!(token_count(o, p, sub), token_count(before, p, sub) + 2);
            let new: Vec<_> = o
                .objects
                .iter()
                .filter(|v| {
                    v.owner == p
                        && v.zone == Zone::Battlefield
                        && v.card.is_token()
                        && v.card.has_subtype(sub)
                        && !before.objects.iter().any(|b| b.reference == v.reference)
                })
                .collect();
            assert_eq!(new.len(), 2);
            for v in new {
                assert_eq!((v.power, v.toughness), pt);
                assert_eq!(v.card.token_info().unwrap().colors, colors);
                assert!(v.keywords.contains(keyword));
            }
        }
        9 => {
            assert_eq!(
                o.players[p.index()].cards_drawn_this_turn,
                before.players[p.index()].cards_drawn_this_turn + 2
            );
            assert!(before.objects.iter().any(|b| {
                b.owner == p
                    && b.zone == Zone::Battlefield
                    && (b.is_creature() || b.card.is_artifact())
                    && o.objects
                        .iter()
                        .any(|a| a.id == b.id && matches!(a.zone, Zone::Graveyard | Zone::Outside))
            }));
        }
        15 | 17 => {
            let (b, a) = changed(creature_target(f));
            if i == 15 {
                assert!(b.kinds & (16 | 32) != 0);
            } else {
                assert!(b.tapped && b.is_creature());
            }
            assert!(matches!(a.zone, Zone::Graveyard | Zone::Outside));
        }
        18 => {
            let (_, a) = changed(creature_target(f));
            assert!(a.keywords.contains(Keywords::DOUBLE_STRIKE));
        }
        _ => unreachable!(),
    }
}
fn verify_main(i: usize, p: Player, id: u32, f: &Flight, o: &Observation) {
    let source = o.objects.iter().find(|v| v.id == id).unwrap();
    assert_eq!(source.zone, Zone::Battlefield);
    assert!(!o.adventures.iter().any(|r| r.object.card == id));
    let stats = [
        (2, 1),
        (2, 2),
        (3, 3),
        (5, 5),
        (4, 3),
        (4, 3),
        (4, 3),
        (3, 2),
        (3, 2),
        (0, 0),
        (4, 2),
        (3, 4),
        (2, 1),
        (2, 2),
        (5, 5),
        (6, 7),
        (3, 4),
        (3, 3),
        (5, 4),
    ];
    if i == 13 {
        let mut boosted = 0;
        for v in o
            .objects
            .iter()
            .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.is_creature())
        {
            let base = if v.id == id {
                (2, 2)
            } else {
                let b = f.before.objects.iter().find(|b| b.id == v.id).unwrap();
                (b.power, b.toughness)
            };
            if (v.power, v.toughness) == (base.0 + 2, base.1 + 2) {
                boosted += 1;
            } else {
                assert_eq!((v.power, v.toughness), base);
            }
        }
        assert_eq!(boosted, 1, "one targeted Shepherd bonus");
    } else {
        assert_eq!((source.power, source.toughness), stats[i]);
    }
    let keyword = match i {
        1 | 11 | 14 | 17 => Some(Keywords::FLYING),
        5 | 18 => Some(Keywords::MENACE),
        6 => Some(Keywords::REACH),
        12 => Some(Keywords::FIRST_STRIKE),
        16 => Some(Keywords::TRAMPLE),
        _ => None,
    };
    if let Some(k) = keyword {
        assert!(source.keywords.contains(k));
    }
    match i {
        0 => assert_eq!(
            o.players[p.index()].cards_drawn_this_turn,
            f.before.players[p.index()].cards_drawn_this_turn + 1
        ),
        3 => assert_eq!(
            token_count(o, p, "Food"),
            token_count(&f.before, p, "Food") + 1
        ),
        8 => {
            for b in f
                .before
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.is_creature())
            {
                let a = o
                    .objects
                    .iter()
                    .find(|a| a.reference == b.reference)
                    .unwrap();
                assert_eq!((a.power, a.toughness), (b.power + 1, b.toughness));
                assert!(a.keywords.contains(Keywords::HASTE));
            }
            assert!(source.keywords.contains(Keywords::HASTE));
        }
        9 => {
            assert!(source.card.is_land());
            assert!(source.tapped);
        }
        _ => {}
    }
}

fn field(o: &Observation, owner: Player, card: Card) -> usize {
    o.objects
        .iter()
        .filter(|v| v.owner == owner && v.zone == Zone::Battlefield && v.card == card)
        .count()
}
fn score(o: &Observation, action: &Action, focal: Card, progress: &[Pilot; 2]) -> i32 {
    let actor = o.actor.unwrap();
    let state = &progress[actor.index()];
    let all_done = progress.iter().all(body_ready);
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    match action {
        Action::Concede | Action::Mulligan => -10000,
        Action::Keep => 5000,
        Action::PlayLand { card } => {
            let v = object(*card);
            if v.card == focal {
                return if v.zone == Zone::Exile && state.permission {
                    3000
                } else if all_done {
                    2000
                } else {
                    -1000
                };
            }
            let count = field(o, actor, v.card);
            let color = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .position(|name| v.card == Card::from_name(name).unwrap())
                .unwrap();
            let symbol = ["{W}", "{U}", "{B}", "{R}", "{G}"][color];
            let main = focal.info().mana_cost.matches(symbol).count();
            let inset = focal
                .adventure_spell()
                .unwrap()
                .info()
                .mana_cost
                .matches(symbol)
                .count();
            2000 - count as i32 * 20 + if count < main.max(inset) { 100 } else { 0 }
        }
        Action::AnnounceAdventure { .. } => {
            // Setup is historical: a resolved damage/sweeper may legitimately
            // kill an Elf. Requiring it to remain alive can stall the opponent
            // forever even though its Adventure has legal meaningful targets.
            let ready = progress.iter().all(|p| p.setup_ready)
                && (!matches!(focal, c if c == Card::from_name(NAMES[4]).unwrap()
                    || c == Card::from_name(NAMES[13]).unwrap())
                    || o.objects.iter().any(|v| {
                        v.owner == actor.other() && v.zone == Zone::Battlefield && v.is_creature()
                    }));
            if ((!state.permission && state.inset.is_none())
                || (state.main_verified && !state.second && state.second_id.is_none()))
                && ready
                && !(focal == Card::from_name(NAMES[13]).unwrap()
                    && state.normal
                    && progress[actor.other().index()].normal
                    && !progress[actor.other().index()].attacked)
                && o.stack.is_empty()
                && o.resolving.is_none()
            {
                1800
            } else {
                -1000
            }
        }
        Action::AnnounceExile { card } if object(*card).card == focal => {
            if !state.normal || complete(state) {
                2400
            } else {
                -1000
            }
        }
        Action::AnnounceCast { card } => {
            let c = object(*card).card;
            if c == focal {
                return if all_done && state.attacked {
                    900
                } else {
                    -1000
                };
            }
            if ["Bear Cub", "Chrome Companion", "Llanowar Elves"]
                .iter()
                .any(|n| c == Card::from_name(n).unwrap())
            {
                if field(o, actor, c) == 0 || all_done {
                    1700
                } else {
                    -1000
                }
            } else if c == Card::from_name("Bygone Colossus").unwrap() {
                if state.main_verified || all_done {
                    1000
                } else {
                    -1000
                }
            } else {
                -1000
            }
        }
        Action::Cast { .. } | Action::BeginCast { .. } | Action::AnnounceWarp { .. } => -1000,
        Action::AnnounceActivate { card, .. } => {
            if state.main_verified
                && state.second
                && !state.extra
                && o.stack.is_empty()
                && (Some(*card) == state.inset
                    || (focal == Card::from_name(NAMES[3]).unwrap()
                        && object(*card).card.has_subtype("Food")))
            {
                1500
            } else {
                -1000
            }
        }
        Action::Activate { .. } | Action::BeginActivate { .. } => -1000,
        Action::TapMana { card } => {
            if o.active == actor
                && (o.stack.is_empty() || o.announcing.is_some() || o.resolution_payment.is_some())
                && matches!(o.step, rustmage::Step::Main1 | rustmage::Step::Main2)
            {
                if object(*card).card == Card::from_name("Llanowar Elves").unwrap() {
                    2200
                } else {
                    20
                }
            } else {
                -1000
            }
        }
        Action::Bottom { card } | Action::Discard { card } => {
            let candidate = object(*card).card;
            let reserved_observer_spell = focal == Card::from_name(NAMES[16]).unwrap()
                && candidate == Card::from_name("Bygone Colossus").unwrap()
                && !state.high_paid
                && o.objects
                    .iter()
                    .filter(|v| {
                        v.owner == actor
                            && v.zone == Zone::Hand
                            && v.card == candidate
                            && !o.effect_discards.contains(&v.id)
                            && o.effect_choice.as_ref().is_none_or(|choice| {
                                !choice.selected[actor.index()].contains(&v.reference)
                            })
                    })
                    .count()
                    <= 1;
            if reserved_observer_spell {
                -1000
            } else if candidate == focal {
                -50
            } else {
                200
            }
        }
        Action::ChooseAdditionalCost { object: reference } => {
            if object(reference.card).card == focal {
                -50
            } else {
                200
            }
        }
        Action::Attack { card } | Action::AttackPlaneswalker { card, .. } => {
            if (state.main_verified && !state.attacked && Some(*card) == state.inset)
                || (all_done && progress.iter().all(|p| p.attacked))
            {
                100
            } else {
                -1000
            }
        }
        Action::Block { .. } => {
            if all_done && progress.iter().all(|p| p.attacked) {
                100
            } else {
                -1000
            }
        }
        Action::Pass | Action::FinishAttackers | Action::FinishBlockers => 0,
        Action::ChooseOptional { accept } => {
            if *accept {
                200
            } else {
                -100
            }
        }
        Action::PayCosts { .. } | Action::PayResolutionMana { .. } => 400,
        Action::DeclineResolutionMana => -100,
        Action::ChooseTarget {
            target: Target::Card(id),
            ..
        } if object(*id).owner != actor
            && Some(*id) == progress[actor.other().index()].inset
            && !complete(&progress[actor.other().index()]) =>
        {
            -1000
        }
        Action::ChooseTarget { .. } => 150,
        Action::FinishTargets => 0,
        _ => 100,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games: usize = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse()?;
    let start: usize = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "0".into())
        .parse()?;
    let mut casts = [[0u64; 2]; 19];
    let mut grants = [[0u64; 2]; 19];
    let mut entries = [[0u64; 2]; 19];
    let mut seconds = [[0u64; 2]; 19];
    let mut main_effects = [[0u64; 2]; 19];
    let mut residuals = [[0u64; 2]; 19];
    let mut attacks = [[0u64; 2]; 19];
    let (mut decisions, mut closed, mut denials, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0usize, 0usize);
    for seed in start..start + games {
        let i = seed % NAMES.len();
        let focal = Card::from_name(NAMES[i]).unwrap();
        let mut game = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: 70000 + seed as u64,
            starting_player: if seed % 2 == 0 {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        initial = game.state_bytes();
        maximum = maximum.max(initial);
        let mut progress: [Pilot; 2] = std::array::from_fn(|_| Pilot {
            extra: !matches!(i, 1 | 2 | 3 | 4 | 6 | 7 | 9 | 12 | 16),
            attacked: i == 9,
            ..Pilot::default()
        });
        for _ in 0..30000 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            let o = game.observe(actor);
            if [Player::P0, Player::P1].iter().all(|p| {
                ["Bear Cub", "Chrome Companion", "Llanowar Elves"]
                    .iter()
                    .all(|name| field(&o, *p, Card::from_name(name).unwrap()) > 0)
            }) {
                for state in &mut progress {
                    state.setup_ready = true;
                }
            }
            for p in [Player::P0, Player::P1] {
                let state = &mut progress[p.index()];
                if !state.permission
                    && state.inset.is_some_and(|id| {
                        o.adventures
                            .iter()
                            .any(|r| r.player == p && r.object.card == id)
                    })
                {
                    verify_adventure(i, p, state.first_effect.as_ref().unwrap(), &o);
                    state.first_effect = None;
                    state.permission = true;
                    grants[i][p.index()] += 1;
                }
                if state.permission
                    && !state.normal
                    && state.inset.is_some_and(|id| {
                        o.objects
                            .iter()
                            .any(|v| v.id == id && v.zone == Zone::Battlefield && v.card == focal)
                    })
                {
                    state.normal = true;
                    entries[i][p.index()] += 1;
                }
                if !state.second
                    && state.second_id.is_some_and(|id| {
                        o.adventures
                            .iter()
                            .any(|r| r.player == p && r.object.card == id)
                    })
                {
                    let flight = state.second_effect.take().unwrap();
                    verify_adventure(i, p, &flight, &o);
                    if i == 1 {
                        let before = flight
                            .before
                            .objects
                            .iter()
                            .find(|v| Some(v.id) == state.inset)
                            .unwrap();
                        let after = o
                            .objects
                            .iter()
                            .find(|v| Some(v.id) == state.inset)
                            .unwrap();
                        assert_eq!(
                            (after.power, after.toughness),
                            (before.power + 1, before.toughness + 1)
                        );
                        state.extra = true;
                    }
                    if i == 2 {
                        assert!(o.objects.iter().any(|a| {
                            a.owner == p.other()
                                && a.zone == Zone::Battlefield
                                && a.tapped
                                && flight.before.objects.iter().any(|b| {
                                    b.reference == a.reference
                                        && a.counters.stun == b.counters.stun + 1
                                })
                        }));
                        state.extra = true;
                    }
                    state.second = true;
                    seconds[i][p.index()] += 1;
                }
            }
            if quiet(&o) {
                for p in [Player::P0, Player::P1] {
                    let state = &mut progress[p.index()];
                    if state.normal && !state.main_verified {
                        let flight = state.main_effect.take().unwrap();
                        verify_main(i, p, state.inset.unwrap(), &flight, &o);
                        state.main_verified = true;
                    }
                    if let Some(flight) = state.activation.take() {
                        if i == 3 {
                            assert_eq!(
                                o.players[p.index()].life,
                                flight.before.players[p.index()].life + 3
                            );
                            assert_eq!(
                                token_count(&o, p, "Food") + 1,
                                token_count(&flight.before, p, "Food")
                            );
                        }
                        if i == 6 {
                            assert_eq!(
                                o.players[p.index()].cards_drawn_this_turn,
                                flight.before.players[p.index()].cards_drawn_this_turn + 1
                            );
                            let count = |v: &Observation| {
                                v.objects
                                    .iter()
                                    .filter(|a| {
                                        a.owner == p
                                            && a.zone == Zone::Battlefield
                                            && (a.card.is_artifact() || a.card.is_land())
                                    })
                                    .count()
                            };
                            assert!(count(&o) < count(&flight.before));
                        }
                        if i == 7 {
                            let b = flight
                                .before
                                .objects
                                .iter()
                                .find(|v| Some(v.id) == state.inset)
                                .unwrap();
                            let a = o
                                .objects
                                .iter()
                                .find(|v| Some(v.id) == state.inset)
                                .unwrap();
                            assert_eq!(
                                (a.power, a.toughness),
                                (b.power + 2, b.toughness + 2),
                                "Hollow before={b:?} after={a:?} before_turn={} before_step={:?} after_turn={} after_step={:?}",
                                flight.before.turn,
                                flight.before.step,
                                o.turn,
                                o.step
                            );
                            assert_eq!(
                                token_count(&o, p, "Food") + 1,
                                token_count(&flight.before, p, "Food")
                            );
                        }
                        state.extra = true;
                    }
                }
            }
            for p in [Player::P0, Player::P1] {
                let state = &mut progress[p.index()];
                if i == 4 && state.main_verified && !state.extra {
                    if let Some((id, red)) = state.mana_trigger {
                        if !o.stack.iter().any(|v| v.id == id) && o.resolving.is_none() {
                            assert_eq!(o.players[p.index()].mana[3], red + 2);
                            state.extra = true;
                        }
                    } else if let Some(packet) = o.stack.iter().find(|v| {
                        v.controller == p
                            && v.source_card == focal
                            && matches!(v.kind, StackKind::TriggeredAbility { .. })
                    }) {
                        assert_eq!(o.active, p);
                        assert_eq!(o.step, rustmage::Step::Main1);
                        state.mana_trigger = Some((packet.id, o.players[p.index()].mana[3]));
                    }
                }
                if state.main_verified
                    && i == 16
                    && state.high_paid
                    && o.objects.iter().any(|v| {
                        Some(v.id) == state.inset
                            && v.zone == Zone::Battlefield
                            && v.counters.plus_one > 0
                    })
                {
                    state.extra = true;
                }
                if i == 12 && state.main_verified {
                    let source = o
                        .objects
                        .iter()
                        .find(|v| Some(v.id) == state.inset)
                        .unwrap();
                    if o.active == p {
                        assert!(source.keywords.contains(Keywords::FIRST_STRIKE));
                        state.own_strike = true;
                    } else {
                        assert!(!source.keywords.contains(Keywords::FIRST_STRIKE));
                        state.other_strike = true;
                    }
                    state.extra = state.own_strike && state.other_strike;
                }
            }
            let other = game.observe(actor.other());
            if o.mana_payment.is_some() {
                assert!(other.mana_payment.is_none());
            }
            if o.library_choice.is_some() {
                assert!(other.library_choice.is_none());
            }
            if o.library_selection.is_some() {
                assert!(other.library_selection.is_none());
            }
            let actions = game.legal_actions();
            for right in &o.adventures {
                if right.player == actor {
                    assert_eq!(right.card, focal);
                    assert!(!actions.contains(&Action::AnnounceAdventure {
                        card: right.object.card
                    }));
                }
            }
            let best_score = actions
                .iter()
                .map(|a| score(&o, a, focal, &progress))
                .max()
                .unwrap();
            let candidates: Vec<_> = actions
                .iter()
                .enumerate()
                .filter(|(_, a)| score(&o, a, focal, &progress) == best_score)
                .map(|(j, a)| (j, a.clone()))
                .collect();
            // The smoke policy resolves beneficial/harmful target polarity without
            // choosing a different route, cast, or premature attack.
            let filtered: Vec<_> = candidates.iter().map(|(_, a)| a.clone()).collect();
            let pick = rustmage::policy::smoke_index(&game, &filtered);
            let (index, action) = candidates[pick].clone();
            if std::env::args().any(|v| v == "--trace") && i == 16 {
                let colossus = Card::from_name("Bygone Colossus").unwrap();
                if matches!(action, Action::Discard { card } | Action::AnnounceCast { card }
                    if o.objects.iter().any(|v| v.id == card && v.card == colossus))
                {
                    eprintln!(
                        "TRACE action={action:?} score={best_score} actor={actor:?} turn={} pilot={:?} cards={:?}",
                        o.turn,
                        progress[actor.index()],
                        o.objects
                            .iter()
                            .filter(|v| v.owner == actor && v.card == colossus)
                            .map(|v| (v.id, v.zone))
                            .collect::<Vec<_>>()
                    );
                }
            }
            if matches!(action, Action::ArrangeLibrary { .. }) {
                if let Some(f) = &mut progress[actor.index()].first_effect {
                    f.scry += 1;
                }
                if let Some(f) = &mut progress[actor.index()].second_effect {
                    f.scry += 1;
                }
            }
            if let Action::Attack { card } = action
                && Some(card) == progress[actor.index()].inset
            {
                progress[actor.index()].attacked = true;
            }
            if let Action::PayCosts { .. } = action
                && let Some(item) = &o.announcing
            {
                let flight = || Flight {
                    before: o.clone(),
                    targets: item.targets.clone(),
                    scry: 0,
                };
                if item.kind == StackKind::Spell
                    && item.source_card == focal
                    && progress[actor.index()].permission
                    && !progress[actor.index()].normal
                {
                    progress[actor.index()].main_effect = Some(flight());
                }
                if matches!(item.kind, StackKind::ActivatedAbility { .. })
                    && !progress[actor.index()].extra
                    && (item.source_card == focal
                        || (i == 3 && item.source_card.has_subtype("Food")))
                {
                    progress[actor.index()].activation = Some(flight());
                }
                if i == 16
                    && item.kind == StackKind::Spell
                    && item.source_card == Card::from_name("Bygone Colossus").unwrap()
                    && progress[actor.index()].main_verified
                {
                    assert!(item.mana_value() >= 5);
                    progress[actor.index()].high_paid = true;
                }
            }
            if let Action::PlayLand { card } = action
                && Some(card) == progress[actor.index()].inset
            {
                progress[actor.index()].main_effect = Some(Flight {
                    before: o.clone(),
                    targets: Vec::new(),
                    scry: 0,
                });
            }
            if let Action::PayCosts { .. } = action
                && let Some(item) = &o.announcing
                && item.kind == StackKind::Spell
                && item.source_card == focal.adventure_spell().unwrap()
            {
                let cast = item.cast.unwrap();
                assert_eq!(cast.face, rustmage::CastFace::Adventure);
                assert_eq!(cast.method, rustmage::CastMethod::Normal);
                assert_eq!(cast.physical_card, Some(focal));
                let traits = o
                    .spell_characteristics
                    .iter()
                    .find(|v| v.id == item.id)
                    .unwrap();
                assert_eq!(traits.physical_card, focal);
                assert_eq!(traits.face, rustmage::CastFace::Adventure);
                assert_eq!(
                    traits.mana_cost,
                    focal.adventure_spell().unwrap().info().mana_cost
                );
                assert!(!traits.type_line.contains("Creature"));
                assert!(!traits.type_line.contains("Land"));
                if progress[actor.index()].normal {
                    progress[actor.index()].second_id = Some(item.card);
                    progress[actor.index()].second_effect = Some(Flight {
                        before: o.clone(),
                        targets: item.targets.clone(),
                        scry: 0,
                    });
                } else {
                    progress[actor.index()].inset = Some(item.card);
                    progress[actor.index()].first_effect = Some(Flight {
                        before: o.clone(),
                        targets: item.targets.clone(),
                        scry: 0,
                    });
                    casts[i][actor.index()] += 1;
                }
            }
            let saved = game.snapshot();
            let revision = game.revision();
            let mut branch = game.clone();
            branch.apply(actor, action.clone())?;
            let parent = game.clone();
            let mut child = parent.clone();
            drop(parent);
            child.apply(actor, action.clone())?;
            closed += 1;
            assert_eq!(
                game.apply(actor.other(), action.clone()),
                Err(Error::WrongActor)
            );
            denials += 1;
            assert_eq!(game.snapshot(), saved);
            game.apply_index(actor, revision, index)?;
            let after = game.snapshot();
            assert_eq!(branch.snapshot(), after);
            assert_eq!(child.snapshot(), after);
            for p in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(game.observe(p))?,
                    serde_json::to_value(child.observe(p))?
                );
            }
            game.restore(&saved)?;
            assert_eq!(
                game.apply_index(actor, revision, index),
                Err(Error::StaleRevision)
            );
            denials += 1;
            game.apply(actor, action.clone())?;
            assert_eq!(game.snapshot(), after);
            if let Action::TapMana { card } = action
                && i == 9
                && Some(card) == progress[actor.index()].inset
            {
                let next = game.observe(actor);
                assert_eq!(
                    next.players[actor.index()].mana[2],
                    o.players[actor.index()].mana[2] + 1
                );
                assert!(next.objects.iter().find(|v| v.id == card).unwrap().tapped);
                progress[actor.index()].extra = true;
            }
            decisions += 1;
            maximum = maximum.max(game.state_bytes());
        }
        assert!(game.outcome().is_some(), "unfinished seed{seed}");
        if !progress.iter().all(complete) {
            eprintln!(
                "terminal P0={} P1={}",
                serde_json::to_string(&game.observe(Player::P0))?,
                serde_json::to_string(&game.observe(Player::P1))?
            );
        }
        assert!(
            progress.iter().all(complete),
            "incomplete seed{seed} {} {progress:?}",
            NAMES[i]
        );
        for p in [Player::P0, Player::P1] {
            assert!(progress[p.index()].main_verified);
            main_effects[i][p.index()] += 1;
            // A residual witness is required only for bodies with additional
            // activation, mana, cast-trigger, or turn-dependent keyword behavior.
            if matches!(i, 1 | 2 | 3 | 4 | 6 | 7 | 9 | 12 | 16) {
                assert!(progress[p.index()].extra);
                residuals[i][p.index()] += 1;
            }
            if i != 9 {
                assert!(progress[p.index()].attacked);
                attacks[i][p.index()] += 1;
            }
        }
        eprintln!(
            "completed seed={seed} card={} decisions={decisions}",
            NAMES[i]
        );
    }
    if games >= 100 {
        for matrix in [&casts, &grants, &entries, &seconds, &main_effects] {
            assert!(matrix.iter().flatten().all(|v| *v > 0));
        }
    }
    if games >= NAMES.len() {
        for i in 0..NAMES.len() {
            for p in 0..2 {
                assert_eq!(
                    residuals[i][p] > 0,
                    matches!(i, 1 | 2 | 3 | 4 | 6 | 7 | 9 | 12 | 16)
                );
                assert_eq!(attacks[i][p] > 0, i != 9);
            }
        }
    }
    println!(
        "games={games} decisions={decisions} parent_closed={closed} denied={denials} initial={initial} max={maximum} inset_paid={casts:?} granted={grants:?} normal_entry={entries:?} second_resolved={seconds:?} main_effects={main_effects:?} residuals={residuals:?} actual_attacks={attacks:?}"
    );
    Ok(())
}
