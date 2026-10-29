//! Complete bargain bodies in deterministic legal Standard self-play.
use rustmage::{
    Action, Card, Config, DeckRules, Error, Game, Keywords, ObjectRef, Observation, Player,
    SpellView, StackKind, Target, Zone,
};
use std::collections::BTreeMap;
const NAMES: [&str; 10] = [
    "Archon's Glory",
    "Candy Grapple",
    "Farsight Ritual",
    "Hamlet Glutton",
    "High Fae Negotiator",
    "Ice Out",
    "Johann's Stopgap",
    "Kellan's Lightblades",
    "Tenacious Tomeseeker",
    "Troublemaker Ouphe",
];
const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
fn index(c: Card) -> Option<usize> {
    NAMES.iter().position(|n| *n == c.name())
}
fn cost(c: Card, pay: bool) -> ([u32; 6], u32) {
    let (mut r, mut n) = ([0; 6], 0u32);
    for s in c
        .info()
        .mana_cost
        .split('{')
        .skip(1)
        .map(|s| s.trim_end_matches('}'))
    {
        if let Ok(v) = s.parse::<u32>() {
            n += v;
        } else {
            r["WUBRGC".find(s).unwrap()] += 1;
        }
    }
    if pay {
        n = n.saturating_sub(match c.name() {
            "Hamlet Glutton" | "Johann's Stopgap" => 2,
            "Ice Out" => 1,
            _ => 0,
        });
    }
    (r, n)
}
fn deck(i: usize) -> Vec<Card> {
    let mut d = vec![];
    for (n, k) in [
        ("Plains", 6),
        ("Island", 8),
        ("Swamp", 8),
        ("Mountain", 6),
        ("Forest", 8),
    ] {
        d.extend(std::iter::repeat_n(Card::from_name(n).unwrap(), k));
    }
    for n in [
        NAMES[i],
        "Bear Cub",
        "Candy Trail",
        "Shock",
        "Thrill of Possibility",
        "Hop to It",
    ] {
        d.extend([Card::from_name(n).unwrap(); 4]);
    }
    assert_eq!(d.len(), 60);
    d
}
fn packets(o: &Observation) -> impl Iterator<Item = &SpellView> {
    o.stack
        .iter()
        .chain(&o.waiting_triggers)
        .chain(o.announcing.iter())
        .chain(o.resolving.iter())
}
fn score(
    o: &Observation,
    a: &Action,
    i: usize,
    want: bool,
    done: [bool; 2],
    fallback: bool,
) -> i32 {
    let p = o.actor.unwrap();
    let object = |id| o.objects.iter().find(|v| v.id == id).unwrap();
    let friends = o
        .objects
        .iter()
        .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card.is_creature())
        .count();
    let grave = o
        .objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Graveyard && v.card.name() == "Shock");
    let handshock = o
        .objects
        .iter()
        .any(|v| v.owner == p && v.zone == Zone::Hand && v.card.name() == "Shock");
    let resources = o.objects.iter().any(|v| {
        v.owner == p
            && v.zone == Zone::Battlefield
            && (v.card.info().type_line.contains("Artifact")
                || v.card.info().type_line.contains("Enchantment")
                || v.token.is_some())
    });
    let required = cost(Card::from_name(NAMES[i]).unwrap(), want);
    let mut potential = o.players[p.index()].mana;
    for v in &o.objects {
        if v.owner == p
            && v.zone == Zone::Battlefield
            && !v.tapped
            && let Some(k) = BASICS.iter().position(|n| *n == v.card.name())
        {
            potential[k] += 1;
        }
    }
    let fundable = potential.iter().zip(required.0).all(|(a, b)| *a >= b)
        && potential.iter().sum::<u32>() >= required.0.iter().sum::<u32>() + required.1;
    match a {
        Action::AnnounceCast { card } | Action::Cast { card, .. } => {
            let n = object(*card).card.name();
            if n == NAMES[i] {
                if fundable
                    && (!want || resources)
                    && (i != 8 || grave)
                    && matches!(a, Action::AnnounceCast { .. })
                {
                    1300
                } else {
                    -1000
                }
            } else if n == "Candy Trail" {
                if !resources || i == 9 { 900 } else { -1000 }
            } else if n == "Thrill of Possibility" {
                if i == 8 && !grave && handshock && matches!(a, Action::AnnounceCast { .. }) {
                    1100
                } else {
                    -1000
                }
            } else if n == "Shock" {
                if i == 5 && fundable { 1000 } else { -1000 }
            } else if n == "Bear Cub" {
                if friends < 3 || done.iter().all(|x| *x) {
                    750
                } else {
                    -1000
                }
            } else if n == "Hop to It" {
                if want && !resources || done.iter().all(|x| *x) {
                    700
                } else {
                    -1000
                }
            } else {
                -1000
            }
        }
        Action::ChooseBargain { sacrifice } => {
            if sacrifice.is_some() == want {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseAdditionalCost { object: r } => {
            if object(r.card).card.name() == "Shock" {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Player(t),
        } => {
            if *t == p.other() {
                1500
            } else {
                -1000
            }
        }
        Action::ChooseTarget {
            target: Target::Spell(t),
        } => {
            if object(*t).owner == p.other() {
                1500
            } else {
                100
            }
        }
        Action::ChooseTarget {
            target: Target::Card(t),
        } => {
            let v = object(*t);
            if i == 8 && v.owner == p && v.zone == Zone::Graveyard && v.card.name() == "Shock" {
                1500
            } else if v.zone == Zone::Battlefield
                && (i == 9 && v.owner == p.other() || i == 0 && v.owner == p)
            {
                1400
            } else if i != 0 && v.owner == p.other() && v.zone == Zone::Battlefield {
                1300
            } else {
                100
            }
        }
        Action::PlayLand { card } => {
            let c = object(*card).card;
            let have = o
                .objects
                .iter()
                .filter(|v| v.owner == p && v.zone == Zone::Battlefield && v.card == c)
                .count();
            let k = BASICS.iter().position(|n| *n == c.name()).unwrap();
            1200 - have as i32 * 10 + if required.0[k] > have as u32 { 100 } else { 0 }
        }
        Action::TapMana { .. } => {
            if o.decision == "pay_costs" {
                800
            } else {
                -1000
            }
        }
        Action::PayCosts { .. }
        | Action::PutTrigger { .. }
        | Action::SelectLibrary { .. }
        | Action::FinishLibrarySelection => 1500,
        Action::ArrangeLibrary { destination, .. } => {
            if *destination == rustmage::LibraryDestination::Top {
                1000
            } else {
                900
            }
        }
        Action::FinishTargets => 1300,
        Action::Discard { card } => {
            let n = object(*card).card.name();
            if i == 8 && !grave && n == "Shock" {
                1500
            } else if n == NAMES[i] {
                100
            } else if object(*card).card.is_land() {
                1200
            } else {
                900
            }
        }
        Action::Attack { .. } => {
            if done.iter().all(|x| *x) || i == 7 && fundable {
                800
            } else {
                -1000
            }
        }
        Action::FinishAttackers | Action::FinishBlockers => 500,
        Action::Concede
        | Action::Mulligan
        | Action::Block { .. }
        | Action::AnnounceActivate { .. }
        | Action::BeginActivate { .. }
        | Action::Activate { .. } => -1000,
        Action::Keep => 1700,
        _ => {
            if fallback {
                100
            } else {
                0
            }
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let games = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "100".into())
        .parse::<usize>()?;
    let (mut casts, mut effects) = ([[[0u64; 2]; 2]; 10], [[[0u64; 2]; 2]; 10]);
    let (
        mut decisions,
        mut replayed,
        mut invalid,
        mut private,
        mut sacrifices,
        mut fizzles,
        mut countered,
    ) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut initial, mut maximum) = (0, 0);
    let (mut selected_cards, mut returned, mut exiled, mut drawn) = (0u64, 0u64, 0u64, 0u64);
    for seed in 0..games {
        let i = seed % 10;
        let round = seed / 10;
        let mut g = Game::new(Config {
            decks: [deck(i), deck(i)],
            seed: seed as u64,
            starting_player: if round.is_multiple_of(2) {
                Player::P0
            } else {
                Player::P1
            },
            deck_rules: DeckRules::Standard,
        })?;
        initial = g.state_bytes();
        let mut done = [false; 2];
        let mut entries = BTreeMap::<ObjectRef, bool>::new();
        let mut active = BTreeMap::<u64, (SpellView, Observation)>::new();
        let mut selected = BTreeMap::<u64, Vec<u32>>::new();
        for _ in 0..100000 {
            if g.outcome().is_some() {
                break;
            }
            let actor = g.actor().unwrap();
            let before = g.observe(actor);
            let want = (round + actor.index()).is_multiple_of(2);
            if before.decision == "bargain" {
                let ann = before.announcing.as_ref().unwrap();
                assert!(ann.targets.is_empty());
                assert_eq!(ann.cast.unwrap().bargain, None);
                assert!(before.payment_request.is_none());
                assert!(g.observe(actor.other()).actions.is_empty());
                let saved = g.snapshot();
                assert_eq!(
                    g.apply(actor.other(), Action::ChooseBargain { sacrifice: None }),
                    Err(Error::WrongActor)
                );
                assert_eq!(g.snapshot(), saved);
                invalid += 1;
                let bad = Action::ChooseBargain {
                    sacrifice: Some(ObjectRef {
                        card: u32::MAX,
                        generation: 0,
                    }),
                };
                assert_eq!(g.apply(actor, bad), Err(Error::IllegalAction));
                assert_eq!(g.snapshot(), saved);
                invalid += 1;
            }
            if let Some(f) = &before.library_selection {
                assert_eq!(i, 2);
                let packet = before.resolving.as_ref().unwrap();
                let pay = packet.cast.unwrap().bargain.unwrap();
                assert_eq!(f.remainder, "bottom_random");
                assert!(!f.reveal);
                assert!(f.remaining.len() + f.selected.len() <= if pay { 8 } else { 4 });
                assert!(g.observe(actor.other()).library_selection.is_none());
                assert!(g.observe(actor.other()).actions.is_empty());
                selected.entry(packet.id).or_default();
                private += 1;
            }
            let fallback = rustmage::policy::smoke_index(&g, &before.actions);
            let j = before
                .actions
                .iter()
                .enumerate()
                .max_by_key(|(j, a)| score(&before, a, i, want, done, *j == fallback))
                .unwrap()
                .0;
            let a = before.actions[j].clone();
            if let Action::PayCosts { payment } = a
                && let Some(ann) = &before.announcing
                && let Some(k) = ann.source().and_then(|source| index(source.card))
            {
                let cast = ann.cast.unwrap();
                assert_eq!(cast.origin, Zone::Hand);
                assert_eq!(cast.method, rustmage::CastMethod::Normal);
                assert_eq!(cast.kicker, None);
                assert_eq!(cast.offspring, None);
                let pay = cast.bargain.unwrap();
                let request = before.payment_request.as_ref().unwrap();
                let expected = cost(ann.source_card, pay);
                assert_eq!((request.required, request.generic), expected);
                assert_eq!(
                    payment.iter().sum::<u32>(),
                    expected.0.iter().sum::<u32>() + expected.1
                );
                casts[k][usize::from(pay)][actor.index()] += 1;
                done[actor.index()] = true;
            }
            if let Action::SelectLibrary { card } = a {
                selected
                    .get_mut(&before.resolving.as_ref().unwrap().id)
                    .unwrap()
                    .push(card);
            }
            if matches!(a, Action::Pass)
                && let Some(packet) = before.stack.last()
                && packet.source().and_then(|source| index(source.card)).is_some()
            {
                active.insert(packet.id, (packet.clone(), g.observe(packet.controller)));
            }
            let saved = g.snapshot();
            let revision = g.revision();
            let mut fork = g.clone();
            fork.apply(actor, a.clone())?;
            g.apply_index(actor, revision, j)?;
            let expected = g.snapshot();
            assert_eq!(fork.snapshot(), expected);
            g.restore(&saved)?;
            assert_eq!(g.apply_index(actor, revision, j), Err(Error::StaleRevision));
            g.apply(actor, a.clone())?;
            assert_eq!(g.snapshot(), expected);
            let after = g.observe(actor);
            assert_eq!(after.schema, rustmage::OBSERVATION_SCHEMA);
            decisions += 1;
            replayed += 1;
            maximum = maximum.max(g.state_bytes());
            if let Action::ChooseBargain { sacrifice: Some(r) } = a {
                let v = after.objects.iter().find(|v| v.reference == r).unwrap();
                assert_eq!(v.zone, Zone::Battlefield);
            }
            if matches!(a, Action::PayCosts { .. })
                && let Some(ann) = before.announcing.as_ref()
                && ann.cast.is_some_and(|c| c.bargain == Some(true))
            {
                let r = before.additional_cost.as_ref().unwrap().selected.unwrap();
                let view = g.observe(ann.controller);
                let v = view.objects.iter().find(|v| v.id == r.card);
                assert!(v.is_none_or(|v| v.zone == Zone::Graveyard || v.zone == Zone::Outside));
                sacrifices += 1;
            }
            if before.library_selection.is_some() && after.library_selection.is_some() {
                assert_eq!(
                    after.players[actor.index()].library_count,
                    before.players[actor.index()].library_count
                );
            }
            let complete = active
                .keys()
                .copied()
                .filter(|id| !packets(&after).any(|s| s.id == *id))
                .collect::<Vec<_>>();
            for id in complete {
                let (packet, base) = active.remove(&id).unwrap();
                let k = packet.source().and_then(|source| index(source.card)).unwrap();
                let p = packet.controller.index();
                let view = g.observe(packet.controller);
                let resolved = before.resolving.as_ref().is_some_and(|s| s.id == id)
                    || matches!(a, Action::Pass) && before.stack.last().is_some_and(|s| s.id == id);
                if !resolved {
                    countered += 1;
                    continue;
                }
                if !packet.targets.is_empty()
                    && packet.targets.iter().all(|t| {
                        t.object
                            .is_some_and(|r| !base.objects.iter().any(|v| v.reference == r))
                    })
                {
                    fizzles += 1;
                    continue;
                }
                if view.outcome.is_some() {
                    continue;
                }
                let pay = if packet.kind == StackKind::Spell {
                    packet.cast.unwrap().bargain.unwrap()
                } else {
                    *entries.get(&packet.object).unwrap()
                };
                if packet.kind == StackKind::Spell && packet.source().is_some_and(|source| source.card.is_creature()) {
                    let v = view.objects.iter().find(|v| v.id == packet.card).unwrap();
                    assert_eq!(v.zone, Zone::Battlefield);
                    assert_eq!(v.reference.generation, packet.object.generation + 1);
                    entries.insert(v.reference, pay);
                    if k == 3 {
                        assert!(v.keywords.contains(Keywords::TRAMPLE));
                        continue;
                    }
                    if k == 4 {
                        assert!(v.keywords.contains(Keywords::FLYING));
                    }
                    if pay {
                        continue;
                    }
                    assert!(!packets(&view).any(|s|s.source_card==packet.source_card&&s.object==v.reference));
                    effects[k][0][p] += 1;
                    continue;
                }
                match k {
                    0 => {
                        for t in &packet.targets {
                            let old = base
                                .objects
                                .iter()
                                .find(|v| v.reference == t.object.unwrap())
                                .unwrap();
                            let new = view
                                .objects
                                .iter()
                                .find(|v| v.reference == old.reference)
                                .unwrap();
                            assert_eq!(
                                (new.power, new.toughness),
                                (old.power + 2, old.toughness + 2)
                            );
                            assert_eq!(
                                new.keywords.contains(Keywords::FLYING),
                                pay || old.keywords.contains(Keywords::FLYING)
                            );
                            assert_eq!(
                                new.keywords.contains(Keywords::LIFELINK),
                                pay || old.keywords.contains(Keywords::LIFELINK)
                            );
                        }
                    }
                    1 | 7 => {
                        for t in &packet.targets {
                            let v = view.objects.iter().find(|v| v.id == t.object.unwrap().card);
                            assert!(
                                v.is_none_or(|v| v.zone == Zone::Graveyard
                                    || k == 7 && !pay && v.damage >= 3)
                            );
                        }
                    }
                    2 => {
                        let cards = selected.remove(&id).unwrap_or_default();
                        let count = base.players[p].library_count.min(2);
                        assert_eq!(
                            view.players[p].library_count,
                            base.players[p].library_count - count
                        );
                        for c in &cards {
                            assert_eq!(
                                view.objects.iter().find(|v| v.id == *c).unwrap().zone,
                                Zone::Hand
                            );
                        }
                        assert_eq!(
                            view.players[p].cards_drawn_this_turn,
                            base.players[p].cards_drawn_this_turn
                        );
                        selected_cards += count as u64;
                    }
                    3 => assert_eq!(view.players[p].life, base.players[p].life + 3),
                    4 => {
                        assert!(pay);
                        assert_eq!(view.players[p].life, base.players[p].life + 3);
                        assert_eq!(
                            view.players[packet.controller.other().index()].life,
                            base.players[packet.controller.other().index()].life - 3
                        );
                    }
                    5 => {
                        for t in &packet.targets {
                            let v = view.objects.iter().find(|v| v.id == t.object.unwrap().card);
                            assert!(v.is_none_or(|v| v.zone == Zone::Graveyard));
                        }
                    }
                    6 | 8 | 9 => {
                        for t in &packet.targets {
                            let v = view.objects.iter().find(|v| v.id == t.object.unwrap().card);
                            assert!(v.is_none_or(
                                |v| v.zone == if k == 9 { Zone::Exile } else { Zone::Hand }
                            ));
                            if k == 9 {
                                exiled += 1;
                            } else {
                                returned += 1;
                            }
                        }
                        if k == 6 {
                            assert_eq!(
                                view.players[p].cards_drawn_this_turn,
                                base.players[p].cards_drawn_this_turn + 1
                            );
                            drawn += 1;
                        }
                    }
                    _ => unreachable!(),
                }
                effects[k][usize::from(pay)][p] += 1;
            }
        }
        assert!(g.outcome().is_some(), "unfinished seed {seed}");
    }
    if games >= 100 {
        assert!(
            casts.iter().flatten().flatten().all(|n| *n > 0),
            "casts {casts:?}"
        );
        assert!(
            effects.iter().flatten().flatten().all(|n| *n > 0),
            "effects {effects:?}"
        );
        assert!(
            private > 0
                && sacrifices > 0
                && selected_cards > 0
                && returned > 0
                && exiled > 0
                && drawn > 0
        );
    }
    println!(
        "Completed {games} bargain Standard games, {decisions} decisions/{replayed} replay boundaries; cast[card][bargained][controller] {casts:?}; effect[card][bargained][controller] {effects:?}; {sacrifices} real sacrifices, {private} private frames/{selected_cards} selected cards, {returned} returns/{drawn} draws/{exiled} exiles, {fizzles} all-invalid fizzles/{countered} countered packets, {invalid} rejected actions; initial {initial}/max {maximum} logical bytes"
    );
    Ok(())
}
