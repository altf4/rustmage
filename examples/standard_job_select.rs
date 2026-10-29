//! Legal Standard self-play witnesses for all sixteen complete Job Select bodies.
use rustmage::{
    Action, Card, CombatDefender, Config, DeckRules, EffectChoiceKind, Error, Game, Keywords,
    ObjectRef, Observation, Player, StackKind, Target, Zone,
};
const NAMES: [&str; 16] = [
    "Astrologian's Planisphere",
    "Bard's Bow",
    "Black Mage's Rod",
    "Dark Knight's Greatsword",
    "Dragoon's Lance",
    "Machinist's Arsenal",
    "Monk's Fist",
    "Ninja's Blades",
    "Paladin's Arms",
    "Red Mage's Rapier",
    "Sage's Nouliths",
    "Samurai's Katana",
    "Summoner's Grimoire",
    "Thief's Knife",
    "Warrior's Sword",
    "White Mage's Staff",
];
const TYPES: [&str; 16] = [
    "Wizard",
    "Bard",
    "Wizard",
    "Knight",
    "Knight",
    "Artificer",
    "Monk",
    "Ninja",
    "Knight",
    "Wizard",
    "Cleric",
    "Samurai",
    "Shaman",
    "Rogue",
    "Warrior",
    "Cleric",
];
const COLORS: [&str; 16] = [
    "Island", "Forest", "Swamp", "Swamp", "Plains", "Plains", "Plains", "Swamp", "Plains",
    "Mountain", "Island", "Mountain", "Forest", "Island", "Mountain", "Plains",
];
const BONUS: [(i128, i128); 16] = [
    (0, 0),
    (2, 2),
    (1, 0),
    (3, 0),
    (1, 0),
    (2, 2),
    (1, 0),
    (1, 1),
    (2, 1),
    (0, 0),
    (1, 0),
    (2, 2),
    (0, 0),
    (1, 1),
    (3, 2),
    (1, 1),
];
fn deck(i: usize, focal: bool) -> Vec<Card> {
    let mut d = Vec::new();
    for (n, k) in [
        ("Forest", 8),
        ("Plains", 8),
        ("Mountain", 8),
        (COLORS[i], 4),
    ] {
        d.extend(vec![Card::from_name(n).unwrap(); k]);
    }
    for n in [
        if focal {
            NAMES[i]
        } else if i == 12 {
            "Ajani, Caller of the Pride"
        } else {
            "Chrome Companion"
        },
        "Bear Cub",
        "Disenchant",
        "Llanowar Elves",
        "Cartographer's Companion",
        if i == 12 {
            "Appendage Amalgam"
        } else {
            "Ambush Wolf"
        },
        "Shock",
        "Thrill of Possibility",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
#[derive(Default)]
struct Pilot {
    source: Option<u32>,
    hero: Option<ObjectRef>,
    bear: Option<ObjectRef>,
    paid: bool,
    equipped: bool,
    phase: u8,
    other_turn: bool,
    life_before: i128,
    started: bool,
    effect_turn: u32,
    enemy_life: i128,
    own_life: i128,
    counters: u64,
    draws: u64,
    shock: bool,
    thrill: bool,
    attacked: bool,
    discard: Option<u64>,
    summon: Option<(ObjectRef, bool)>,
    declined: bool,
    defender: Option<CombatDefender>,
    enemy_shock: Option<u32>,
    ward_choice: bool,
    rapier: Vec<(u64, u32)>,
    seen: Vec<u64>,
    removal_turn: u32,
}
fn hand(o: &Observation, p: Player, n: &str) -> bool {
    o.objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Hand && v.card.name() == n)
}
fn walker(o: &Observation, p: Player) -> Option<ObjectRef> {
    o.objects
        .iter()
        .find(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_planeswalker())
        .map(|v| v.reference)
}
fn enchantment(card: Card) -> bool {
    card.info()
        .type_line
        .split(" — ")
        .next()
        .unwrap()
        .split_whitespace()
        .any(|k| k == "Enchantment")
}
// Printed off-stack mana value, independent of the cost actually paid.
fn mana_value(card: Card) -> u64 {
    card.info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|part| {
            let symbol = part.split('}').next().unwrap();
            symbol.parse::<u64>().unwrap_or_else(|_| {
                if symbol == "X" {
                    0
                } else if symbol.starts_with("2/") {
                    2
                } else {
                    1
                }
            })
        })
        .sum()
}
fn suitable(v: &rustmage::ObjectView, route: usize) -> bool {
    v.card.is_creature()
        && if route == 1 {
            enchantment(v.card)
        } else {
            !enchantment(v.card)
        }
}
fn ready(o: &Observation, p: Player, i: usize) -> bool {
    o.objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == "Bear Cub")
        && (i != 5
            || o.objects.iter().any(|v| {
                v.owner == p
                    && v.zone == Zone::Battlefield
                    && v.card.name() == "Cartographer's Companion"
            }))
}
fn protected(
    o: &Observation,
    p: Player,
    i: usize,
    s: &Pilot,
    route: usize,
    v: &rustmage::ObjectView,
) -> bool {
    if s.phase >= 5 {
        return false;
    }
    let n = v.card.name();
    let copies = o
        .objects
        .iter()
        .filter(|x| x.owner == p && x.zone == Zone::Hand && x.card == v.card)
        .count();
    copies == 1
        && ((n == NAMES[i] && !s.paid)
            || n == "Disenchant"
            || (i == 0 && n == "Thrill of Possibility" && !s.thrill)
            || (matches!(i, 0 | 2 | 9) && n == "Shock" && !s.shock)
            || (n == "Bear Cub" && !ready(o, p, i))
            || (i == 5 && n == "Cartographer's Companion" && !ready(o, p, i)))
        || i == 12
            && v.owner == p
            && v.zone == Zone::Hand
            && suitable(v, route)
            && o.objects
                .iter()
                .filter(|x| x.owner == p && x.zone == Zone::Hand && suitable(x, route))
                .count()
                == 1
}
fn score(
    o: &Observation,
    a: &Action,
    p: Player,
    i: usize,
    s: &Pilot,
    route: usize,
    ward_pay: bool,
) -> i32 {
    let actor = o.actor.unwrap();
    let obj = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    match a {
        Action::Keep => 3000,
        Action::Mulligan | Action::Concede => -10000,
        Action::AnnounceCast { card } => {
            let v = obj(*card);
            let n = v.card.name();
            if s.phase == 5 {
                return if v.card.is_creature() {
                    1200
                } else if n == "Shock" {
                    1100
                } else {
                    -1000
                };
            }
            if actor != p {
                if i == 8 && s.phase == 2 && s.started && s.enemy_shock.is_none() && n == "Shock" {
                    return 2100;
                }
                if i == 12 && n == "Ajani, Caller of the Pride" && walker(o, actor).is_none() {
                    return 1800;
                }
                return -1000;
            }
            if n == NAMES[i] {
                return if !s.paid && ready(o, p, i) {
                    1900
                } else {
                    -1000
                };
            }
            if n == "Bear Cub"
                && !o
                    .objects
                    .iter()
                    .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
            {
                return 1750;
            }
            if i == 5
                && n == "Cartographer's Companion"
                && !o
                    .objects
                    .iter()
                    .any(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.name() == n)
            {
                return 1750;
            }
            if n == "Disenchant"
                && s.phase == 3
                && (i != 4 || s.other_turn)
                && (i != 8 || !ward_pay || o.turn > s.effect_turn)
            {
                return 2000;
            }
            if s.phase == 2 && s.started {
                if n == "Shock" && matches!(i, 0 | 2 | 9) && !s.shock {
                    return 2000;
                }
                if n == "Thrill of Possibility" && i == 0 && s.shock && !s.thrill {
                    return 2000;
                }
            }
            if n == "Thrill of Possibility"
                && i != 0
                && (s.phase == 0
                    || s.phase == 2 && !s.started
                    || s.phase == 3 && !hand(o, p, "Disenchant"))
                && o.objects.iter().any(|v| {
                    v.owner == p
                        && v.zone == Zone::Hand
                        && v.card.name() != n
                        && !protected(o, p, i, s, route, v)
                })
            {
                return 1400;
            }
            -1000
        }
        Action::AnnounceActivate { card, .. } => {
            if actor == p && Some(*card) == s.source && s.phase == 1 && !s.equipped {
                2000
            } else {
                -1000
            }
        }
        Action::Cast { .. }
        | Action::BeginCast { .. }
        | Action::Activate { .. }
        | Action::AnnounceExile { .. }
        | Action::AnnounceFlashback { .. } => -1000,
        Action::ChooseTarget {
            target: Target::Card(id),
        } => {
            let v = obj(*id);
            if s.phase < 5 {
                if i == 8 && actor != p && s.phase == 2 && Some(v.reference) == s.bear {
                    return 3000;
                }
                if actor == p
                    && s.phase == 1
                    && v.owner == p
                    && v.zone == Zone::Battlefield
                    && v.card.name() == "Bear Cub"
                {
                    return 3000;
                }
                if actor == p && s.phase == 3 && Some(*id) == s.source {
                    return 3000;
                }
                if actor == p && i == 10 && s.phase == 2 && Some(v.reference) == s.bear {
                    return 3000;
                }
                return -1000;
            }
            if v.owner != actor { 1800 } else { 700 }
        }
        Action::ChooseTarget {
            target: Target::Player(who),
        } => {
            if *who != actor {
                1800
            } else {
                -1000
            }
        }
        Action::ChooseAdditionalCost { object } => {
            if protected(o, p, i, s, route, obj(object.card)) {
                -100
            } else {
                2500
            }
        }
        Action::PayCosts { .. } | Action::ChooseSpellCost { .. } => 2400,
        Action::PayResolutionMana { .. } => {
            if ward_pay {
                2600
            } else {
                -1000
            }
        }
        Action::DeclineResolutionMana => {
            if ward_pay {
                -1000
            } else {
                2600
            }
        }
        Action::TapMana { .. } | Action::ActivateMana { .. } | Action::AnnounceMana { .. } => {
            if matches!(o.decision, "pay_costs" | "resolution_mana") {
                2100
            } else {
                -1000
            }
        }
        Action::PutHandCreature { object } => {
            if i == 12 && route != 2 && suitable(obj(object.card), route) {
                2600
            } else {
                -1000
            }
        }
        Action::DeclineHandEntry => {
            if route == 2 {
                2700
            } else {
                -1000
            }
        }
        Action::ChooseHandDefender { defender } => {
            if matches!(defender, CombatDefender::Planeswalker(_)) {
                2700
            } else {
                2600
            }
        }
        Action::PlayLand { card } => {
            let n = obj(*card).card.name();
            let count = o
                .objects
                .iter()
                .filter(|v| v.owner == actor && v.zone == Zone::Battlefield && v.card.name() == n)
                .count();
            if n == COLORS[i] && count < 2 {
                1650
            } else if n == "Forest" && count < 2 {
                1620
            } else {
                1550 - 20 * count as i32
            }
        }
        Action::Attack { card } => {
            if s.phase == 5
                || actor == p
                    && s.phase == 2
                    && s.started
                    && !s.attacked
                    && s.bear.is_some_and(|r| r.card == *card)
            {
                1700
            } else {
                -1000
            }
        }
        Action::Block { .. } => {
            if s.phase == 5 {
                1600
            } else {
                -1000
            }
        }
        Action::AssignDamage { amount, .. } => 1800 + (*amount).min(100) as i32,
        Action::Discard { card } | Action::Bottom { card } => {
            let v = obj(*card);
            if i == 7
                && o.effect_choice
                    .as_ref()
                    .is_some_and(|c| c.kind == EffectChoiceKind::Discard)
            {
                return 2600 + mana_value(v.card).min(100) as i32;
            }
            if actor == p && protected(o, p, i, s, route, v) {
                -100
            } else {
                2500
            }
        }
        Action::Pass => 0,
        Action::FinishAttackers | Action::FinishBlockers => 1000,
        Action::PutTrigger { .. } | Action::FinishTargets | Action::ArrangeLibrary { .. } => 2200,
        _ => 100,
    }
}
fn boosted(
    o: &Observation,
    p: Player,
    i: usize,
    v: &rustmage::ObjectView,
    base: i128,
    growth: i128,
) {
    let (mut power, mut toughness) = BONUS[i];
    if i == 5 {
        let n = o
            .objects
            .iter()
            .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_artifact())
            .count() as i128;
        assert!(n >= 2);
        power *= n;
        toughness *= n;
    }
    let counters = i128::from(v.counters.plus_one) - i128::from(v.counters.minus_one);
    assert_eq!(
        (v.power, v.toughness),
        (
            base + power + counters + growth,
            base + toughness + counters
        )
    );
    assert!(v.has_subtype(TYPES[i]));
    assert!(!v.card.has_subtype(TYPES[i]));
    let expected = match i {
        1 => Keywords::REACH,
        4 if o.active == p => Keywords::FLYING,
        11 => Keywords::TRAMPLE.union(Keywords::HASTE),
        _ => Keywords::NONE,
    };
    assert!(v.keywords.contains(expected));
    if i == 4 && o.active != p {
        assert!(!v.keywords.contains(Keywords::FLYING));
    }
}
fn rapier_growth(s: &Pilot, turn: u32) -> i128 {
    2 * s.rapier.iter().filter(|(_, t)| *t == turn).count() as i128
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let first_seed = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "0".into())
        .parse::<usize>()?;
    let (mut paid, mut entered, mut equipped, mut effects, mut removed, mut cleaned) = (
        [[0u64; 2]; 16],
        [[0u64; 2]; 16],
        [[0u64; 2]; 16],
        [[0u64; 2]; 16],
        [[0u64; 2]; 16],
        [[0u64; 2]; 16],
    );
    let (mut summoner, mut ward) = ([[0u64; 2]; 3], [[0u64; 2]; 2]);
    let (mut decisions, mut replayed, mut closed, mut rejects, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0u64, 0usize, 0usize);
    for seed in first_seed..first_seed + games {
        let i = seed % 16;
        let p = if (seed / 16) % 2 == 0 {
            Player::P0
        } else {
            Player::P1
        };
        let route = (seed / 32) % 3;
        let ward_pay = (seed / 32) % 2 != 0;
        let (a, b) = (deck(i, true), deck(i, false));
        let mut g = Game::new(Config {
            decks: if p == Player::P0 { [a, b] } else { [b, a] },
            seed: seed as u64,
            starting_player: p,
            deck_rules: DeckRules::Standard,
        })?;
        if seed == first_seed {
            initial = g.state_bytes();
        }
        let mut s = Pilot::default();
        for step in 0..20000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let o = g.observe(actor);
            assert_eq!(o.schema, rustmage::OBSERVATION_SCHEMA);
            for packet in o
                .stack
                .iter()
                .chain(o.waiting_triggers.iter())
                .chain(o.announcing.iter())
                .chain(o.resolving.iter())
            {
                if let Some(ctx) = &packet.attachment_trigger
                    && ctx.granting_card.name() == NAMES[i]
                {
                    assert!(ctx.granted);
                    assert_eq!(Some(ctx.recipient), s.bear);
                    assert_eq!(packet.object, ctx.recipient);
                    assert_eq!(packet.controller, p);
                    assert_eq!(packet.source().expect("example expects an object-backed packet").card.name(), "Bear Cub");
                    if !s.seen.contains(&packet.id) {
                        s.seen.push(packet.id);
                        if i == 9 {
                            s.rapier.push((packet.id, o.turn));
                        }
                    }
                    if matches!(i, 10 | 12 | 15) {
                        assert_eq!(ctx.defender, Some(p.other()));
                    } else {
                        assert_eq!(ctx.defender, None);
                    }
                }
            }
            if o.effect_choice.as_ref().is_some_and(|c| {
                c.kind == EffectChoiceKind::HandCreature
                    || i == 7 && c.kind == EffectChoiceKind::Discard
            }) {
                assert!(g.observe(actor.other()).effect_choice.is_none());
            }
            let quiet = o.decision == "priority"
                && o.stack.is_empty()
                && o.announcing.is_none()
                && o.resolving.is_none();
            if quiet && s.paid {
                let source = o.objects.iter().find(|v| Some(v.id) == s.source).unwrap();
                if s.phase == 0 {
                    let reference = source.attached_to.expect("Job Select attached actual Hero");
                    let hero = o.objects.iter().find(|v| v.reference == reference).unwrap();
                    assert_eq!(hero.zone, Zone::Battlefield);
                    assert!(hero.card.is_token() && hero.card.has_subtype("Hero"));
                    assert_eq!(hero.token.unwrap().colors, 0);
                    boosted(&o, p, i, hero, 1, 0);
                    s.hero = Some(reference);
                    s.phase = 1;
                    entered[i][p.index()] += 1;
                } else if s.phase == 1 && s.equipped {
                    assert_eq!(source.attached_to, s.bear);
                    let bear = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.bear)
                        .unwrap_or_else(|| {
                            panic!(
                                "missing Bear seed{seed} {} phase{} turn{} source{:?} objects{:?}",
                                NAMES[i], s.phase, o.turn, source.zone, o.objects
                            )
                        });
                    boosted(&o, p, i, bear, 2, 0);
                    let hero = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.hero)
                        .unwrap();
                    assert!(!hero.has_subtype(TYPES[i]));
                    let n = i128::from(hero.counters.plus_one);
                    assert_eq!((hero.power, hero.toughness), (1 + n, 1 + n));
                    s.phase = 2;
                    equipped[i][p.index()] += 1;
                } else if s.phase == 2 {
                    let bear = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.bear)
                        .unwrap_or_else(|| {
                            panic!(
                                "missing Bear seed{seed} {} phase{} turn{} source{:?} objects{:?}",
                                NAMES[i], s.phase, o.turn, source.zone, o.objects
                            )
                        });
                    boosted(
                        &o,
                        p,
                        i,
                        bear,
                        2,
                        if i == 9 { rapier_growth(&s, o.turn) } else { 0 },
                    );
                    if o.active != p {
                        s.other_turn = true;
                    }
                    if !s.started && actor == p {
                        let cards: Vec<_> = o
                            .objects
                            .iter()
                            .filter(|v| v.owner == p && v.zone == Zone::Hand)
                            .collect();
                        let lands: Vec<_> = o
                            .objects
                            .iter()
                            .filter(|v| {
                                v.owner == p
                                    && v.zone == Zone::Battlefield
                                    && v.card.is_land()
                                    && !v.tapped
                            })
                            .collect();
                        let attack_ready = !bear.summoning_sick
                            && !bear.tapped
                            && o.active == p
                            && o.step == rustmage::Step::Main1;
                        let can_start = match i {
                            0 => {
                                o.active == p
                                    && o.players[p.index()].cards_drawn_this_turn == 1
                                    && hand(&o, p, "Shock")
                                    && hand(&o, p, "Thrill of Possibility")
                                    && cards.len() >= 4
                                    && lands.len() >= 3
                                    && lands.iter().filter(|v| v.card.name() == "Mountain").count()
                                        >= 2
                            }
                            2 | 9 => {
                                hand(&o, p, "Shock")
                                    && lands.len() >= 3
                                    && lands.iter().any(|v| v.card.name() == "Mountain")
                                    && lands.iter().any(|v| v.card.name() == "Plains")
                            }
                            7 => {
                                attack_ready
                                    && cards.iter().any(|v| {
                                        mana_value(v.card) > 0 && v.card.name() != "Disenchant"
                                    })
                            }
                            10 | 13 | 15 => attack_ready,
                            12 => {
                                attack_ready
                                    && cards.iter().any(|v| suitable(v, route))
                                    && (route != 1 || walker(&o, p.other()).is_some())
                            }
                            _ => true,
                        };
                        if can_start {
                            s.started = true;
                            s.effect_turn = o.turn;
                            s.enemy_life = o.players[p.other().index()].life;
                            s.own_life = o.players[p.index()].life;
                            s.counters = bear.counters.plus_one;
                            s.draws = o.players[p.index()].cards_drawn_this_turn;
                        }
                    }
                    if s.started {
                        let complete = match i {
                            0 => {
                                s.shock
                                    && s.thrill
                                    && o.turn == s.effect_turn
                                    && bear.counters.plus_one == s.counters + 3
                                    && o.players[p.index()].cards_drawn_this_turn == 3
                            }
                            2 => s.shock && o.players[p.other().index()].life == s.enemy_life - 3,
                            9 => {
                                s.shock
                                    && rapier_growth(&s, o.turn) >= 2
                                    && bear.power == 2 + rapier_growth(&s, o.turn)
                            }
                            7 => {
                                s.discard.is_some_and(|mv| {
                                    o.players[p.other().index()].life
                                        == s.enemy_life - 3 - i128::from(mv)
                                }) && o.players[p.index()].cards_drawn_this_turn > s.draws
                            }
                            8 => {
                                s.ward_choice
                                    && s.enemy_shock.is_some_and(|id| {
                                        o.objects
                                            .iter()
                                            .any(|v| v.id == id && v.zone == Zone::Graveyard)
                                    })
                                    && bear.damage == if ward_pay { 2 } else { 0 }
                            }
                            10 => s.attacked && !bear.tapped,
                            13 => {
                                s.attacked && o.players[p.index()].cards_drawn_this_turn > s.draws
                            }
                            15 => s.attacked && o.players[p.index()].life == s.own_life + 1,
                            12 => {
                                if route == 2 {
                                    s.declined
                                } else {
                                    s.summon.is_some_and(|(r, enchantment)| {
                                        o.objects.iter().any(|v| {
                                            v.id == r.card
                                                && v.reference != r
                                                && v.zone == Zone::Battlefield
                                                && v.tapped == enchantment
                                        }) && o.attack_declarations.iter().any(|a| a.card == r.card)
                                            == enchantment
                                    })
                                }
                            }
                            _ => true,
                        };
                        if complete {
                            if matches!(i, 0 | 2 | 7 | 8 | 9 | 10 | 12 | 13 | 15) {
                                assert!(!s.seen.is_empty(), "missing real recipient packet");
                            }
                            if i == 12 {
                                if let Some((r, enchantment)) = s.summon {
                                    if enchantment {
                                        let d = o
                                            .attack_declarations
                                            .iter()
                                            .find(|a| a.card == r.card)
                                            .unwrap()
                                            .defender;
                                        assert_eq!(d, s.defender);
                                        assert!(matches!(d, Some(CombatDefender::Planeswalker(_))));
                                        assert!(o.library_choice.is_none());
                                    } else {
                                        assert!(!o.attackers.contains(&r.card));
                                    }
                                }
                                summoner[route][p.index()] += 1;
                            }
                            if i == 8 {
                                // The enemy may wait a later turn to draw Shock. Cleanup
                                // must follow the actual damage, not initial readiness.
                                s.effect_turn = o.turn;
                                ward[usize::from(ward_pay)][p.index()] += 1;
                            }
                            effects[i][p.index()] += 1;
                            s.phase = 3;
                        }
                    }
                } else if s.phase == 3 {
                    if o.active != p {
                        s.other_turn = true;
                    }
                    let bear = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.bear)
                        .unwrap_or_else(|| {
                            panic!(
                                "missing Bear seed{seed} {} phase{} turn{} source{:?} objects{:?}",
                                NAMES[i], s.phase, o.turn, source.zone, o.objects
                            )
                        });
                    if source.zone == Zone::Battlefield {
                        boosted(
                            &o,
                            p,
                            i,
                            bear,
                            2,
                            if i == 9 { rapier_growth(&s, o.turn) } else { 0 },
                        );
                    } else {
                        assert_eq!(source.zone, Zone::Graveyard);
                        assert!(!bear.has_subtype(TYPES[i]));
                        let n = i128::from(bear.counters.plus_one);
                        assert_eq!(
                            (bear.power, bear.toughness),
                            (
                                2 + n + if i == 9 { rapier_growth(&s, o.turn) } else { 0 },
                                2 + n
                            )
                        );
                        s.phase = 4;
                        s.removal_turn = o.turn;
                        removed[i][p.index()] += 1;
                    }
                } else if s.phase == 4 && o.turn > s.removal_turn {
                    let bear = o
                        .objects
                        .iter()
                        .find(|v| Some(v.reference) == s.bear)
                        .unwrap_or_else(|| {
                            panic!(
                                "missing Bear seed{seed} {} phase{} turn{} source{:?} objects{:?}",
                                NAMES[i], s.phase, o.turn, source.zone, o.objects
                            )
                        });
                    let n = i128::from(bear.counters.plus_one);
                    assert_eq!((bear.power, bear.toughness), (2 + n, 2 + n));
                    assert!(!bear.has_subtype(TYPES[i]));
                    assert_eq!(bear.damage, 0);
                    cleaned[i][p.index()] += 1;
                    s.phase = 5;
                }
            }
            let actions = g.legal_actions();
            let (index, a) = actions
                .iter()
                .enumerate()
                .max_by_key(|(_, a)| score(&o, a, p, i, &s, route, ward_pay))
                .unwrap();
            assert!(
                score(&o, a, p, i, &s, route, ward_pay) >= 0,
                "unhandled seed{seed} step{step} phase{} {} {actions:?}",
                s.phase,
                o.decision
            );
            let a = a.clone();
            if let Action::AnnounceCast { card } = a
                && actor == p
                && !s.paid
                && o.objects
                    .iter()
                    .any(|v| v.id == card && v.card.name() == NAMES[i])
            {
                s.source = Some(card);
            }
            if let Action::ChooseTarget {
                target: Target::Card(id),
            } = a
                && actor == p
                && s.phase == 1
                && o.objects
                    .iter()
                    .any(|v| v.id == id && v.owner == p && v.card.name() == "Bear Cub")
            {
                s.bear = Some(o.objects.iter().find(|v| v.id == id).unwrap().reference);
            }
            if matches!(a, Action::PayCosts { .. })
                && let Some(packet) = &o.announcing
            {
                if actor == p
                    && packet.kind == StackKind::Spell
                    && packet.source().is_some_and(|source| source.card.name() == NAMES[i])
                    && !s.paid
                {
                    s.paid = true;
                    paid[i][p.index()] += 1;
                }
                if actor == p
                    && matches!(packet.kind, StackKind::ActivatedAbility { .. })
                    && Some(packet.card) == s.source
                    && s.phase == 1
                {
                    s.equipped = true;
                    s.life_before = o.players[p.index()].life;
                }
                if s.phase == 2 && s.started && packet.kind == StackKind::Spell {
                    if actor == p && packet.source().is_some_and(|source| source.card.name() == "Shock") {
                        s.shock = true;
                    }
                    if actor == p && packet.source().is_some_and(|source| source.card.name() == "Thrill of Possibility") {
                        s.thrill = true;
                    }
                    if actor != p && i == 8 && packet.source().is_some_and(|source| source.card.name() == "Shock") {
                        s.enemy_shock = Some(packet.card);
                    }
                }
            }
            if matches!(a, Action::Attack { .. }) && s.phase == 2 {
                s.attacked = true;
            }
            if let Action::Discard { card } = a
                && i == 7
                && o.effect_choice
                    .as_ref()
                    .is_some_and(|c| c.kind == EffectChoiceKind::Discard)
            {
                s.discard = Some(mana_value(
                    o.objects.iter().find(|v| v.id == card).unwrap().card,
                ));
            }
            if let Action::PutHandCreature { object } = a {
                let card = o
                    .objects
                    .iter()
                    .find(|v| v.reference == object)
                    .unwrap()
                    .card;
                s.summon = Some((object, enchantment(card)));
            }
            if matches!(a, Action::DeclineHandEntry) {
                s.declined = true;
            }
            if let Action::ChooseHandDefender { defender } = a {
                s.defender = Some(defender);
            }
            if i == 8
                && matches!(
                    a,
                    Action::PayResolutionMana { .. } | Action::DeclineResolutionMana
                )
            {
                assert_eq!(actor, p.other());
                s.ward_choice = true;
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            let expected = fork.snapshot();
            let child = if (1..=4).contains(&s.phase) {
                let parent = g.clone();
                let child = parent.clone();
                drop(parent);
                Some(child)
            } else {
                None
            };
            assert!(g.apply(actor.other(), a.clone()).is_err());
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), revision);
            rejects += 1;
            g.apply_index(actor, revision, index)?;
            assert_eq!(g.snapshot(), expected);
            if let Some(mut child) = child {
                child.apply(actor, a.clone())?;
                assert_eq!(child.snapshot(), expected);
                closed += 1;
            }
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    serde_json::to_value(g.observe(viewer))?,
                    serde_json::to_value(fork.observe(viewer))?
                );
            }
            if s.equipped && s.phase == 1 && matches!(a, Action::PayCosts { .. }) {
                assert_eq!(
                    g.observe(p).players[p.index()].life,
                    s.life_before - if i == 3 { 3 } else { 0 }
                );
            }
            g.restore(&saved)?;
            assert_eq!(
                g.apply_index(actor, revision, index),
                Err(Error::StaleRevision)
            );
            rejects += 1;
            g.apply(actor, a)?;
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            replayed += 1;
            maximum = maximum.max(g.state_bytes());
        }
        assert!(
            s.phase == 5 && g.outcome().is_some(),
            "incomplete {} seed{seed} phase{} turn{} started{} shock{} thrill{} attacked{} draws{} discard{:?} summon{:?} ward{}",
            NAMES[i],
            s.phase,
            g.observe(p).turn,
            s.started,
            s.shock,
            s.thrill,
            s.attacked,
            s.draws,
            s.discard,
            s.summon,
            s.ward_choice
        );
    }
    if games >= 100 {
        for ledger in [&paid, &entered, &equipped, &effects, &removed, &cleaned] {
            assert!(ledger.iter().flatten().all(|n| *n > 0));
        }
        assert!(summoner.iter().flatten().all(|n| *n > 0));
        assert!(ward.iter().flatten().all(|n| *n > 0));
        assert!(closed > 0);
    }
    println!(
        "games={games} decisions={decisions} replays={replayed} parent_closed={closed} rejects={rejects} initial_bytes={initial} max_bytes={maximum} paid={paid:?} entered={entered:?} equipped={equipped:?} effects={effects:?} removed={removed:?} cleanup={cleaned:?} summoner={summoner:?} ward={ward:?}"
    );
    Ok(())
}
