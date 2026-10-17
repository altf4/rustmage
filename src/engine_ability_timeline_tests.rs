use super::tests::{scenario, spawn};
use super::*;

pub(super) fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
pub(super) fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap_or_else(|| panic!("illegal independent {action:?} {:?}", g.state.pending));
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    let evaluated = ability_evaluation::evaluate_abilities(g);
    for (i, object) in g.state.objects.iter().enumerate() {
        if object.zone == Zone::Battlefield {
            assert_eq!(
                evaluated.result.objects[i].keywords,
                g.keywords(i as CardId),
                "actual ordered keyword projection at {i}"
            );
        }
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn settle_on(g: &mut Game, chosen: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if let Some(target) =
            chosen.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
pub(super) fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
    g
}

pub(super) fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
pub(super) fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: 3 });
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}

pub(super) fn next_turn(g: &mut Game) {
    let turn = g.state.turn;
    for _ in 0..100 {
        if g.state.turn != turn {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, action);
    }
    panic!("turn failed to advance");
}
fn source_time(g: &Game, id: CardId) -> u64 {
    g.state
        .roles
        .as_ref()
        .unwrap()
        .ability_timestamps
        .iter()
        .find(|r| r.source == g.object_ref(id))
        .unwrap()
        .timestamp
}
#[test]
fn ability_timeline_paid_boros_and_royal_preserve_one_effect_time_and_expire_naturally() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let giant = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let boros = named(&mut g, p, "Boros Charm", Zone::Hand);
        cast(&mut g, boros, Some(1), &[]);
        settle(&mut g);
        let effects = g.observe(p).keyword_effects;
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].timestamp, effects[1].timestamp);
        assert!(
            effects
                .iter()
                .all(|row| row.keywords == Keywords::INDESTRUCTIBLE
                    && row.expires_turn == Some(g.state.turn))
        );
        assert!(effects.iter().all(|row| row.recipient.card != enemy));
        let royal = named(&mut g, p, "Royal Treatment", Zone::Hand);
        cast(&mut g, royal, None, &[Target::Card(bear)]);
        settle(&mut g);
        let effects = g.observe(p).keyword_effects;
        let hex = effects
            .iter()
            .find(|r| r.keywords == Keywords::HEXPROOF)
            .unwrap();
        assert!(hex.timestamp > effects[0].timestamp);
        assert_eq!(hex.recipient, g.object_ref(bear));
        assert!(g.keywords(giant).contains(Keywords::INDESTRUCTIBLE));
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
        assert_eq!(
            g.observe(p).keyword_effects,
            g.observe(p.other()).keyword_effects
        );
        next_turn(&mut g);
        assert!(g.observe(p).keyword_effects.is_empty());
        assert!(!g.keywords(giant).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn ability_timeline_paid_repeated_suspect_does_not_refresh_and_survives_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let person = named(&mut g, p, "Person of Interest", Zone::Hand);
        cast(&mut g, person, None, &[]);
        settle(&mut g);
        let effects = g.observe(p).keyword_effects;
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].origin, KeywordEffectOrigin::Suspected);
        assert_eq!(effects[0].expires_turn, None);
        for _ in 0..2 {
            let aura = named(&mut g, p, "Convenient Target", Zone::Hand);
            cast(&mut g, aura, None, &[Target::Card(person)]);
            settle(&mut g);
            assert_eq!(g.observe(p).keyword_effects, effects);
        }
        next_turn(&mut g);
        assert!(g.is_suspected(person));
        assert_eq!(g.observe(p).keyword_effects, effects);
        let bounce = named(&mut g, p.other(), "Unsummon", Zone::Hand);
        // Setup mana after the actual phase transition; the real cost is paid.
        g.state.players[p.other().index()].mana = [10; 6];
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        cast(&mut g, bounce, None, &[Target::Card(person)]);
        settle(&mut g);
        assert!(!g.is_suspected(person));
        assert!(g.observe(p).keyword_effects.is_empty());
    }
}
#[test]
fn ability_timeline_paid_job_equipment_retimes_only_on_an_actual_new_attachment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let sword = named(&mut g, p, "Samurai's Katana", Zone::Hand);
        cast(&mut g, sword, None, &[]);
        settle(&mut g);
        let initial = source_time(&g, sword);
        let ability = g.state.objects[sword as usize]
            .card
            .activations()
            .iter()
            .position(|a| {
                a.program
                    .iter()
                    .any(|p| matches!(p, Instruction::AttachSource))
            })
            .unwrap() as u16;
        let mut attached = None;
        for index in 0..2 {
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: sword,
                    ability,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            for _ in 0..10 {
                if g.state.announcing.is_none() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets))
                    .or_else(|| {
                        g.legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::PayCosts { .. }))
                    })
                    .unwrap();
                checked(&mut g, a);
            }
            settle(&mut g);
            if index == 0 {
                assert!(source_time(&g, sword) > initial);
            } else {
                assert_eq!(source_time(&g, sword), attached.unwrap());
            }
            attached = Some(source_time(&g, sword));
        }
    }
}

#[test]
fn ability_timeline_paid_blocking_rule_effects_survive_hypothetical_recipient_ability_loss() {
    use super::ability_order::{LayerSixAction, LayerSixEffect, apply_layer_six};
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Enter the Enigma", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(ally)]);
        settle(&mut g);
        let verge = named(&mut g, p, "Sandstorm Verge", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: verge,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        for _ in 0..10 {
            if g.state.announcing.is_none() {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets))
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. }))
                })
                .unwrap();
            checked(&mut g, action);
        }
        settle(&mut g);
        let rows = &g.state.roles.as_ref().unwrap().keyword_effects;
        for (id, restriction) in [
            (ally, Keywords::CANT_BE_BLOCKED),
            (enemy, Keywords::CANT_BLOCK),
        ] {
            let row = rows
                .iter()
                .find(|row| row.recipient == g.object_ref(id))
                .unwrap();
            assert_eq!(row.rules, restriction);
            assert_eq!(row.keywords, Keywords::NONE);
        }
        // This is a hypothetical layer6 boundary fixture, not a claimed cast of
        // an unsupported removal Aura. Both independent rule effects above were
        // created by genuine paid actions and keep their actual exact recipients.
        let mut evaluated = ability_evaluation::evaluate_abilities(&g);
        let removals: Vec<_> = [ally, enemy]
            .into_iter()
            .map(|id| LayerSixEffect {
                provider: None,
                started_earlier: false,
                timestamp: u64::MAX,
                recipients: vec![id],
                action: LayerSixAction::Remove,
            })
            .collect();
        evaluated.result = apply_layer_six(evaluated.result.objects, &removals);
        assert_eq!(
            evaluated.result.objects[ally as usize].keywords,
            Keywords::NONE
        );
        assert_eq!(
            evaluated.result.objects[enemy as usize].keywords,
            Keywords::NONE
        );
        ability_evaluation::apply_rule_effects(&g, &mut evaluated);
        assert_eq!(
            evaluated.result.objects[ally as usize].keywords,
            Keywords::CANT_BE_BLOCKED
        );
        assert_eq!(
            evaluated.result.objects[enemy as usize].keywords,
            Keywords::CANT_BLOCK
        );
        assert!(!evaluated.result.objects[ally as usize].printed);
        assert!(!evaluated.result.objects[enemy as usize].printed);
    }
}
