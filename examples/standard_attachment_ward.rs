//! Legal Standard paid attachment-Ward/exile/equip/Crew/cycling acceptance.
use rustmage::{Card, Config, DeckRules, Error, Game, Player};
use serde_json::Value;
const NAMES: [&str; 10] = [
    "Dusk Rose Reliquary",
    "Trapped in the Screen",
    "Dwarven Mattock",
    "Falcon's Wing Harness",
    "Lavaspur Boots",
    "Hardlight Containment",
    "Sheltered by Ghosts",
    "Super Strength",
    "Detention Chariot",
    "Static Snare",
];
const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn number(v: &Value) -> u64 {
    v.as_u64().unwrap_or(0)
}
fn field<'a>(o: &'a Value, owner: &str, name: &str) -> Option<&'a Value> {
    o["objects"].as_array().unwrap().iter().find(|v| {
        text(v, "owner") == owner && text(v, "zone") == "Battlefield" && text(v, "card") == name
    })
}
fn object(o: &Value, id: u64) -> &Value {
    o["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| number(&v["id"]) == id)
        .unwrap()
}
fn count(o: &Value, owner: &str, name: &str, zone: &str) -> usize {
    o["objects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| text(v, "owner") == owner && text(v, "card") == name && text(v, "zone") == zone)
        .count()
}
#[derive(Default, Debug)]
struct Progress {
    source: Option<u64>,
    target: Value,
    enemy: Option<u64>,
    paid: bool,
    entry: bool,
    equipped: bool,
    ward_cast: bool,
    ward_done: bool,
    removed: bool,
    cycle_paid: bool,
    cycled: bool,
    crew_paid: bool,
    crewed: bool,
    cycle_draws: u64,
    victim: Option<u64>,
    done: bool,
}
fn funds(o: &Value, owner: &str, name: &str, index: usize) -> bool {
    let cost = if name == NAMES[index] {
        [
            (1, 0, 1),
            (3, 0, 1),
            (2, 0, 0),
            (2, 1, 1),
            (1, 0, 0),
            (1, 0, 1),
            (2, 0, 1),
            (5, 4, 1),
            (6, 0, 2),
            (4, 0, 1),
        ][index]
    } else {
        match name {
            "Bear Cub" => (2, 4, 1),
            "Axgard Cavalry" => (2, 3, 1),
            "Omni-Cheese Pizza" => (2, 0, 0),
            "Disenchant" => (2, 0, 1),
            _ => (0, 0, 0),
        }
    };
    let pi = usize::from(owner == "P1");
    let mut available = [0u64; 6];
    for (i, value) in available.iter_mut().enumerate() {
        *value = number(&o["players"][pi]["mana"][i]);
        if i < 5 {
            *value += o["objects"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| {
                    text(v, "owner") == owner
                        && text(v, "zone") == "Battlefield"
                        && text(v, "card") == BASICS[i]
                        && !v["tapped"].as_bool().unwrap()
                })
                .count() as u64;
        }
    }
    available.iter().sum::<u64>() >= cost.0 && available[cost.1] >= cost.2
}
fn score(
    o: &Value,
    own: &Value,
    s: &Progress,
    i: usize,
    p: &str,
    ward: Option<bool>,
    a: &Value,
) -> i32 {
    let q = if p == "P0" { "P1" } else { "P0" };
    let actor = text(o, "actor");
    let name = NAMES[i];
    let k = text(a, "kind");
    let bear = field(o, p, "Bear Cub");
    let dwarf = field(o, p, "Axgard Cavalry");
    let pizza = field(o, p, "Omni-Cheese Pizza");
    let enemy = field(o, q, "Bear Cub");
    let quiet = text(o, "decision") == "priority"
        && o["stack"].as_array().unwrap().is_empty()
        && o["announcing"].is_null()
        && o["resolving"].is_null();
    let ready = quiet
        && s.entry
        && (!(2..=4).contains(&i) || s.equipped)
        && (i != 8 || s.cycled && s.crewed);
    if s.done {
        return match k {
            "Keep" => 3000,
            "PlayLand" => 1000,
            "Discard" => 1000,
            "Pass" => 0,
            "FinishAttackers" | "FinishBlockers" => 1000,
            _ => -1000,
        };
    }
    match k {
        "Cast" | "BeginCast" | "Activate" | "BeginActivate" | "Mulligan" | "Concede" => -1000,
        "AnnounceCast" => {
            let n = text(object(o, number(&a["card"])), "card");
            if actor == p {
                if !funds(o, p, n, i) {
                    return -1000;
                }
                if ["Axgard Cavalry", "Bear Cub", "Omni-Cheese Pizza"].contains(&n) {
                    return if !s.paid && field(o, p, n).is_none() {
                        1800
                    } else {
                        -1000
                    };
                }
                if n == name {
                    return if !s.paid
                        && bear.is_some()
                        && dwarf.is_some()
                        && pizza.is_some()
                        && enemy.is_some()
                        && (i != 9 || !o["attackers"].as_array().unwrap().is_empty())
                    {
                        2000
                    } else {
                        -1000
                    };
                }
                if n == "Disenchant" {
                    return if ward.is_none() && ready && !s.removed {
                        2100
                    } else {
                        -1000
                    };
                }
            } else {
                if n == "Bear Cub" {
                    return if enemy.is_none() && !s.paid {
                        1700
                    } else {
                        -1000
                    };
                }
                let wanted = if [0, 1, 5].contains(&i) {
                    "Disenchant"
                } else {
                    "Shock"
                };
                if n == wanted {
                    return if ward.is_some() && ready && !s.ward_cast {
                        2200
                    } else {
                        -1000
                    };
                }
            }
            -1000
        }
        "AnnounceActivate" => {
            let obj = object(o, number(&a["card"]));
            if actor == p && (2..=4).contains(&i) && Some(number(&a["card"])) == s.source {
                let minimum = [4, 3, 1][i - 2];
                let pi = usize::from(p == "P1");
                let mana = o["players"][pi]["mana"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(number)
                    .sum::<u64>();
                let lands = o["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|v| {
                        text(v, "owner") == p
                            && text(v, "zone") == "Battlefield"
                            && BASICS.contains(&text(v, "card"))
                            && !v["tapped"].as_bool().unwrap()
                    })
                    .count() as u64;
                if mana + lands < minimum || i == 3 && !funds(o, p, NAMES[i], i) {
                    return -1000;
                }
                return if s.entry && !s.equipped { 2200 } else { -1000 };
            }
            if actor == p && i == 8 && text(obj, "card") == name {
                if text(obj, "zone") == "Hand" {
                    return if s.entry && !s.cycle_paid && funds(o, p, "Disenchant", i) {
                        2100
                    } else {
                        -1000
                    };
                }
                return if s.cycled && !s.crew_paid {
                    2100
                } else {
                    -1000
                };
            }
            -1000
        }
        "ChooseTarget" => {
            let t = a["target"]["Card"].as_u64();
            let ann = &o["announcing"];
            let n = text(ann, "source_card");
            let wanted = if n == "Disenchant" {
                if actor == p || [0, 1].contains(&i) {
                    s.source
                } else {
                    Some(number(&s.target["card"]))
                }
            } else if n == "Shock" {
                Some(number(&s.target["card"]))
            } else if n == name && text(ann, "kind") == "Spell" && [5, 6, 7].contains(&i) {
                if i == 5 {
                    pizza.map(|v| number(&v["id"]))
                } else {
                    bear.map(|v| number(&v["id"]))
                }
            } else if n == name
                && (text(ann, "kind") == "ActivatedAbility"
                    || ann["kind"].get("ActivatedAbility").is_some())
            {
                bear.map(|v| number(&v["id"]))
            } else if n == name && [2, 3].contains(&i) {
                dwarf.map(|v| number(&v["id"]))
            } else if n == name && [0, 1, 5, 6, 8, 9].contains(&i) {
                enemy.map(|v| number(&v["id"]))
            } else {
                bear.map(|v| number(&v["id"]))
            };
            if t.is_some() && t == wanted {
                2600
            } else {
                -1000
            }
        }
        "ChooseAdditionalCost" => {
            if text(object(o, number(&a["object"]["card"])), "card") == "Omni-Cheese Pizza" {
                2400
            } else {
                -1000
            }
        }
        "ChooseCrewer" => {
            if ["Bear Cub", "Axgard Cavalry"]
                .contains(&text(object(o, number(&a["object"]["card"])), "card"))
            {
                2000
            } else {
                -1000
            }
        }
        "FinishCrew" => 2600,
        "PayResolutionMana" => {
            if ward == Some(true) {
                2600
            } else {
                -1000
            }
        }
        "DeclineResolutionMana" => {
            if ward == Some(false) {
                2600
            } else {
                -1000
            }
        }
        "TapMana" => {
            if !(text(o, "decision") == "pay_costs"
                || !o["resolution_payment"].is_null() && ward == Some(true))
            {
                return -1000;
            }
            let n = text(object(o, number(&a["card"])), "card");
            let z = BASICS.iter().position(|&v| v == n).unwrap();
            let domain = &o["mana_payment"];
            if !domain.is_null() && number(&domain["available"][z]) < number(&domain["required"][z])
            {
                2450
            } else {
                2300
            }
        }
        "Attack" => {
            if i == 9
                && actor == q
                && !s.paid
                && funds(o, p, name, i)
                && bear.is_some()
                && dwarf.is_some()
                && pizza.is_some()
                && own["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| text(v, "card") == name && text(v, "zone") == "Hand")
            {
                2000
            } else {
                -1000
            }
        }
        "PlayLand" => {
            1500 - 20
                * count(
                    o,
                    actor,
                    text(object(o, number(&a["card"])), "card"),
                    "Battlefield",
                ) as i32
        }
        "Discard" => {
            let n = text(object(o, number(&a["card"])), "card");
            let copies = count(o, actor, n, "Hand");
            if n == name {
                if copies > 2 { 1900 } else { -100 }
            } else if n == "Disenchant" {
                if copies > 1 { 2100 } else { 100 }
            } else if ["Axgard Cavalry", "Bear Cub", "Omni-Cheese Pizza"].contains(&n) {
                if field(o, actor, n).is_some() || copies > 1 {
                    2200
                } else {
                    -300
                }
            } else if BASICS.contains(&n) {
                1800
            } else {
                1000
            }
        }
        "Keep" => 3000,
        "Pass" => 0,
        "PutTrigger" => 2500,
        "FinishTargets" => 1600,
        "PayCosts" => 2600,
        "ChooseSpellCost" => 1800,
        "FinishAttackers" | "FinishBlockers" => 1300,
        _ => -1000,
    }
}
fn decks(i: usize) -> [Vec<Card>; 2] {
    let mut own = Vec::new();
    let mut enemy = Vec::new();
    for n in BASICS {
        for _ in 0..8 {
            own.push(Card::from_name(n).unwrap());
            enemy.push(Card::from_name(n).unwrap());
        }
    }
    for n in [
        NAMES[i],
        "Omni-Cheese Pizza",
        "Axgard Cavalry",
        "Bear Cub",
        "Disenchant",
    ] {
        own.extend([Card::from_name(n).unwrap(); 4]);
    }
    for n in ["Bear Cub", "Shock", "Disenchant"] {
        enemy.extend([Card::from_name(n).unwrap(); 4]);
    }
    enemy.extend([Card::from_name("Forest").unwrap(); 8]);
    assert_eq!(own.len(), 60);
    assert_eq!(enemy.len(), 60);
    [own, enemy]
}
fn main() {
    let games = std::env::args()
        .nth(1)
        .map(|v| v.parse::<usize>().unwrap())
        .unwrap_or(100);
    let mut effects = [[0usize; 2]; 10];
    let mut payments = [[0usize; 2]; 8];
    let mut declines = [[0usize; 2]; 8];
    let mut decisions = 0u64;
    let mut denials = 0u64;
    let mut private = 0u64;
    let mut max_bytes = 0;
    let mut initial = 0;
    let start = std::env::args()
        .nth(2)
        .map(|v| v.parse::<usize>().unwrap())
        .unwrap_or(0);
    for game in start..start + games {
        let i = game % 10;
        let seat = (game / 10) % 2;
        let p = if seat == 0 { Player::P0 } else { Player::P1 };
        let q = if seat == 0 { Player::P1 } else { Player::P0 };
        let ps = if seat == 0 { "P0" } else { "P1" };
        let qs = if seat == 0 { "P1" } else { "P0" };
        let ward = if i < 8 {
            match (game / 20) % 3 {
                1 => Some(false),
                2 => Some(true),
                _ => None,
            }
        } else {
            None
        };
        let [own, enemy] = decks(i);
        let mut g = Game::new(Config {
            decks: if seat == 0 {
                [own, enemy]
            } else {
                [enemy, own]
            },
            seed: 257 + game as u64,
            starting_player: p,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut s = Progress::default();
        initial = g.state_bytes();
        for _ in 0..16000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let opponent = if actor == Player::P0 {
                Player::P1
            } else {
                Player::P0
            };
            let observation = g.observe(actor);
            let o = serde_json::to_value(&observation).unwrap();
            let own = serde_json::to_value(g.observe(p)).unwrap();
            assert_eq!(
                number(&o["schema"]),
                u64::from(rustmage::OBSERVATION_SCHEMA)
            );
            let quiet = text(&o, "decision") == "priority"
                && o["stack"].as_array().unwrap().is_empty()
                && o["announcing"].is_null()
                && o["resolving"].is_null();
            if !s.done {
                if s.paid && quiet && !s.entry {
                    let body = object(&o, s.source.unwrap());
                    assert_eq!(text(body, "zone"), "Battlefield");
                    if [0, 1, 5, 6, 8, 9].contains(&i) {
                        assert_eq!(text(object(&o, s.enemy.unwrap()), "zone"), "Exile");
                    }
                    if [2, 3].contains(&i) {
                        assert_eq!(
                            body["attached_to"],
                            field(&o, ps, "Axgard Cavalry").unwrap()["reference"]
                        );
                    }
                    if [5, 6, 7].contains(&i) {
                        assert_eq!(body["attached_to"], s.target);
                    }
                    if i == 5 {
                        assert_eq!(
                            number(&object(&o, number(&s.target["card"]))["kinds"]) & 1,
                            0
                        );
                    }
                    s.entry = true;
                    effects[i][seat] += 1;
                }
                if s.equipped && quiet && !s.removed && !s.ward_done {
                    let body = object(&o, s.source.unwrap());
                    assert_eq!(body["attached_to"], s.target);
                    let target = object(&o, number(&s.target["card"]));
                    let (a, b, k) = [(2, 2, 0), (1, 1, 1), (1, 0, 4)][i - 2];
                    assert_eq!(number(&target["power"]), 2 + a);
                    assert_eq!(number(&target["toughness"]), 2 + b);
                    assert_eq!(number(&target["keywords"]) & k, k);
                }
                if s.entry && [6, 7].contains(&i) && !s.removed && !s.ward_done {
                    let target = object(&o, number(&s.target["card"]));
                    let (a, b, k) = if i == 6 { (1, 0, 64) } else { (4, 4, 256) };
                    assert_eq!(number(&target["power"]), 2 + a);
                    assert_eq!(number(&target["toughness"]), 2 + b);
                    assert_eq!(number(&target["keywords"]) & k, k);
                }
                if s.cycle_paid && quiet && !s.cycled {
                    assert_eq!(
                        number(&own["players"][seat]["cards_drawn_this_turn"]),
                        s.cycle_draws + 1
                    );
                    s.cycled = true;
                }
                if s.crew_paid && quiet && !s.crewed {
                    let body = object(&o, s.source.unwrap());
                    assert_eq!(number(&body["kinds"]) & 1, 1);
                    assert_eq!(number(&body["power"]), 6);
                    assert_eq!(number(&body["toughness"]), 6);
                    s.crewed = true;
                }
                if s.ward_done && quiet {
                    assert_eq!(text(object(&o, s.victim.unwrap()), "zone"), "Graveyard");
                    let id = if [0, 1].contains(&i) {
                        s.source.unwrap()
                    } else {
                        number(&s.target["card"])
                    };
                    let victim = object(&o, id);
                    if ward == Some(false) {
                        assert_eq!(text(victim, "zone"), "Battlefield");
                    } else if [0, 1, 5].contains(&i) {
                        assert_eq!(text(victim, "zone"), "Graveyard");
                    } else if text(victim, "zone") == "Battlefield" {
                        assert_eq!(number(&victim["damage"]), 2);
                    }
                    s.done = true;
                }
                if s.removed && quiet {
                    assert_eq!(text(object(&o, s.source.unwrap()), "zone"), "Graveyard");
                    if [0, 1, 5, 6, 8, 9].contains(&i) {
                        assert_eq!(text(object(&o, s.enemy.unwrap()), "zone"), "Battlefield");
                    }
                    s.done = true;
                }
            }
            let actions = g.legal_actions();
            let values: Vec<_> = actions
                .iter()
                .map(|a| serde_json::to_value(a).unwrap())
                .collect();
            let (index, best) = values
                .iter()
                .enumerate()
                .map(|(j, a)| (j, score(&o, &own, &s, i, ps, ward, a)))
                .max_by_key(|&(j, v)| (v, std::cmp::Reverse(j)))
                .unwrap();
            assert!(
                best >= 0,
                "{} {} {} {:?}",
                NAMES[i],
                ps,
                text(&o, "decision"),
                values
            );
            let a = &values[index];
            let k = text(a, "kind");
            let ann = &o["announcing"];
            if !s.done {
                if k == "PayCosts" && !ann.is_null() {
                    let n = text(ann, "source_card");
                    if n == NAMES[i] && text(ann, "kind") == "Spell" {
                        s.paid = true;
                        s.source = Some(number(&ann["card"]));
                        s.enemy = field(&o, qs, "Bear Cub").map(|v| number(&v["id"]));
                        if [5, 6, 7].contains(&i) {
                            s.target = field(
                                &o,
                                ps,
                                if i == 5 {
                                    "Omni-Cheese Pizza"
                                } else {
                                    "Bear Cub"
                                },
                            )
                            .unwrap()["reference"]
                                .clone();
                        }
                        if i == 9 {
                            assert_eq!(
                                a["payment"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(number)
                                    .sum::<u64>(),
                                4
                            );
                        }
                    } else if n == NAMES[i]
                        && text(object(&o, number(&ann["card"])), "zone") == "Hand"
                    {
                        s.cycle_paid = true;
                        s.cycle_draws = number(&own["players"][seat]["cards_drawn_this_turn"]);
                    } else if n == NAMES[i] && (2..=4).contains(&i) {
                        s.equipped = true;
                        s.target = field(&o, ps, "Bear Cub").unwrap()["reference"].clone();
                    } else if n == "Disenchant" && actor == p {
                        s.removed = true;
                    } else if ["Shock", "Disenchant"].contains(&n) && actor == q {
                        s.ward_cast = true;
                        s.victim = Some(number(&ann["card"]));
                    }
                }
                if k == "FinishCrew" {
                    s.crew_paid = true;
                }
                if k == "PayResolutionMana" {
                    s.ward_done = true;
                    payments[i][seat] += 1;
                }
                if k == "DeclineResolutionMana" {
                    s.ward_done = true;
                    declines[i][seat] += 1;
                }
            }
            if !o["resolution_payment"].is_null() || !o["announcing"].is_null() {
                private += 1;
            }
            let saved = g.snapshot();
            let revision = g.revision();
            assert_eq!(
                g.apply_index(opponent, revision, index),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), saved);
            denials += 1;
            let parent = g.clone();
            let mut child = parent.clone();
            drop(parent);
            child.apply(actor, actions[index].clone()).unwrap();
            g.apply_index(actor, revision, index).unwrap();
            assert_eq!(g.snapshot(), child.snapshot());
            let expected = g.snapshot();
            g.restore(&saved).unwrap();
            assert_eq!(
                g.apply_index(actor, revision, index),
                Err(Error::StaleRevision)
            );
            denials += 1;
            g.apply_index(actor, g.revision(), index).unwrap();
            assert_eq!(g.snapshot(), expected);
            decisions += 1;
            max_bytes = max_bytes.max(g.state_bytes());
        }
        assert!(g.outcome().is_some(), "no terminal {game}");
        assert!(
            s.done,
            "incomplete game={game} {} {ps} {ward:?} {s:?} outcome={:?}",
            NAMES[i],
            g.outcome()
        );
        eprintln!(
            "completed game={game} card={} seat={ps} ward={ward:?}",
            NAMES[i]
        );
    }
    if games >= 60 {
        assert!(effects.iter().flatten().all(|&v| v > 0));
        assert!(payments.iter().flatten().all(|&v| v > 0));
        assert!(declines.iter().flatten().all(|&v| v > 0));
    }
    println!(
        "attachment_ward Standard games={games} decisions={decisions} replays={decisions} denials={denials} private_frames={private} initial={initial} max={max_bytes} effects={effects:?} pay={payments:?} decline={declines:?}"
    );
}
