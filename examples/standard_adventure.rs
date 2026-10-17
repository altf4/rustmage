//! Legal Standard Adventure routes with actual payments and owned branch replay.
//! Additional full-body effect ledgers are added before final acceptance.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Observation, Player, StackKind, Zone,
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
#[derive(Default, Debug)]
struct Pilot {
    inset: Option<u32>,
    permission: bool,
    normal: bool,
    second: bool,
    second_id: Option<u32>,
}
fn complete(p: &Pilot) -> bool {
    p.permission && p.normal && p.second
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
    let all_done = progress.iter().all(complete);
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
            let ready = [actor, actor.other()].iter().all(|p| {
                ["Bear Cub", "Chrome Companion", "Llanowar Elves"]
                    .iter()
                    .all(|n| field(o, *p, Card::from_name(n).unwrap()) > 0)
            });
            if ((!state.permission && state.inset.is_none())
                || (state.normal && !state.second && state.second_id.is_none()))
                && ready
            {
                1800
            } else {
                -1000
            }
        }
        Action::AnnounceExile { card } if object(*card).card == focal => 2400,
        Action::AnnounceCast { card } => {
            let c = object(*card).card;
            if c == focal {
                return if all_done { 900 } else { -1000 };
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
                if state.normal || all_done {
                    1000
                } else {
                    -1000
                }
            } else {
                -1000
            }
        }
        Action::Cast { .. } | Action::BeginCast { .. } => -1000,
        Action::AnnounceActivate { .. }
        | Action::Activate { .. }
        | Action::BeginActivate { .. } => -1000,
        Action::TapMana { card } => {
            if o.active == actor && matches!(o.step, rustmage::Step::Main1 | rustmage::Step::Main2)
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
            if object(*card).card == focal {
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
        Action::Attack { .. } | Action::AttackPlaneswalker { .. } | Action::Block { .. } => {
            if all_done {
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
        Action::PayCosts { .. } => 400,
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
    let mut casts = [[0u64; 2]; 19];
    let mut grants = [[0u64; 2]; 19];
    let mut entries = [[0u64; 2]; 19];
    let mut seconds = [[0u64; 2]; 19];
    let (mut decisions, mut closed, mut denials, mut initial, mut maximum) =
        (0u64, 0u64, 0u64, 0usize, 0usize);
    for seed in 0..games {
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
        let mut progress: [Pilot; 2] = std::array::from_fn(|_| Pilot::default());
        for _ in 0..30000 {
            if game.outcome().is_some() {
                break;
            }
            let actor = game.actor().unwrap();
            let o = game.observe(actor);
            for p in [Player::P0, Player::P1] {
                let state = &mut progress[p.index()];
                if !state.permission
                    && state.inset.is_some_and(|id| {
                        o.adventures
                            .iter()
                            .any(|r| r.player == p && r.object.card == id)
                    })
                {
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
                    state.second = true;
                    seconds[i][p.index()] += 1;
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
                } else {
                    progress[actor.index()].inset = Some(item.card);
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
            game.apply(actor, action)?;
            assert_eq!(game.snapshot(), after);
            decisions += 1;
            maximum = maximum.max(game.state_bytes());
        }
        assert!(game.outcome().is_some(), "unfinished seed{seed}");
        assert!(
            progress.iter().all(complete),
            "incomplete seed{seed} {} {progress:?}",
            NAMES[i]
        );
        eprintln!(
            "completed seed={seed} card={} decisions={decisions}",
            NAMES[i]
        );
    }
    if games >= 100 {
        for matrix in [&casts, &grants, &entries, &seconds] {
            assert!(matrix.iter().flatten().all(|v| *v > 0));
        }
    }
    println!(
        "games={games} decisions={decisions} parent_closed={closed} denied={denials} initial={initial} max={maximum} inset_paid={casts:?} granted={grants:?} normal_entry={entries:?} second_resolved={seconds:?}"
    );
    Ok(())
}
