//! Real Improvise payments, exact reservations and existing typed observers.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const IRON: &str = "Ironheart, Clever Champion";
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [30; 6];
    cast(g, id, None, targets);
    settle(g);
    id
}
fn fuel(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n)
        .map(|_| named(g, p, "Hedron Archive", Zone::Battlefield))
        .collect()
}
fn select(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    checked(g, Action::ChooseImproviser { object });
}
fn reject_improviser(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    reject(g, Action::ChooseImproviser { object });
}
fn convoker(g: &mut Game, id: CardId, contribution: ConvokeContribution) {
    let object = g.object_ref(id);
    checked(
        g,
        Action::ChooseConvoker {
            object,
            contribution,
        },
    );
}
fn clause(g: &mut Game, kind: CastCostClause) {
    let clause = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == kind)
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause });
}
fn animate(g: &mut Game, p: Player, id: CardId, artifact: bool) {
    let item = g.cast_preview(p, id, Zone::Battlefield, g.object_ref(id), 0, [0; 6]);
    g.animate_source(
        &item,
        crate::cards::AnimationSpec {
            artifact,
            creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: crate::cards::AnimationDuration::Permanent,
        },
    );
}
#[test]
fn msh_improvise_arc_all_optional_counts_paid_tapped_entry_real_untap_and_ccc() {
    for p in [Player::P0, Player::P1] {
        for n in 0..=5 {
            let mut g = setup(p);
            let fs = fuel(&mut g, p, n);
            let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5 - n as u32];
            checked(&mut g, Action::AnnounceCast { card: arc });
            checked(&mut g, Action::BeginImprovise);
            for id in fs.iter().copied() {
                select(&mut g, id);
                assert!(!g.object_view(id).tapped);
                assert!(!g.first_tap(g.object_ref(id)));
            }
            pay(&mut g);
            assert!(fs.iter().all(|&id| g.object_view(id).tapped));
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(arc).zone, Zone::Battlefield);
            assert!(g.object_view(arc).tapped);
            assert!(!g.first_tap(g.object_ref(arc)));
            assert_eq!(g.object_view(arc).card.fixed_mana_value(), 5);
            let mana = Action::ActivateMana {
                card: arc,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 3],
            };
            reject(&mut g, mana.clone());
            next_turn(&mut g);
            next_turn(&mut g);
            assert!(!g.object_view(arc).tapped);
            checked(&mut g, mana);
            assert!(g.state.stack.is_empty());
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 3]);
        }
    }
}
#[test]
fn msh_improvise_ironheart_real_colored_payment_body_and_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fs = fuel(&mut g, p, 4);
        let iron = named(&mut g, p, IRON, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: iron });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: iron });
        checked(&mut g, Action::BeginImprovise);
        for id in fs {
            select(&mut g, id);
        }
        pay(&mut g);
        settle(&mut g);
        let v = g.object_view(iron);
        assert_eq!((v.power, v.toughness), (3, 4));
        assert!(v.keywords.contains(Keywords::FLYING));
        assert!(v.card.is_legendary());
        assert!(v.has_subtype("Human") && v.has_subtype("Hero"));
        assert!(v.summoning_sick);
        let extra = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::BeginImprovise);
        select(&mut g, iron);
        select(&mut g, extra);
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Hand)
            .count();
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count(),
            hand + 2
        );
        assert!(g.object_view(iron).tapped);
    }
}
#[test]
fn msh_improvise_membership_control_exact_types_and_duplicate_rejection() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = paid(&mut g, p, IRON, &[]);
        let fs = fuel(&mut g, p, 3);
        let foreign = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
        g.change_controller(g.object_ref(foreign), p);
        let enemy = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        animate(&mut g, p, land, true);
        g.state.objects[land as usize].summoning_sick = true;
        let stale = g.object_ref(fs[0]);
        g.move_object(fs[0], Zone::Exile);
        g.move_object(fs[0], Zone::Battlefield);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        for object in [
            stale,
            g.object_ref(enemy),
            g.object_ref(bear),
            g.object_ref(arc),
        ] {
            reject(&mut g, Action::ChooseImproviser { object });
        }
        select(&mut g, foreign);
        reject_improviser(&mut g, foreign);
        for id in [land, iron, fs[1], fs[2]] {
            select(&mut g, id);
        }
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(foreign).tapped && g.object_view(land).tapped);
        assert!(!g.object_view(enemy).tapped && !g.object_view(bear).tapped);
    }
}
#[test]
fn msh_improvise_mana_first_and_closed_window_never_double_uses_artifact() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fs = fuel(&mut g, p, 4);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: arc });
        let mana = Action::ActivateMana {
            card: fs[0],
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 2],
        };
        checked(&mut g, mana);
        checked(&mut g, Action::BeginImprovise);
        reject_improviser(&mut g, fs[0]);
        let late = Action::ActivateMana {
            card: fs[1],
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 2],
        };
        reject(&mut g, late);
        for id in fs.into_iter().skip(1) {
            select(&mut g, id);
        }
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
    }
}
#[test]
fn msh_improvise_any_mana_substitution_does_not_turn_artifacts_into_blue_or_colorless() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Vizier of the Menagerie", &[]);
        let fs: Vec<_> = (0..5)
            .map(|_| named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield))
            .collect();
        let iron = named(&mut g, p, IRON, Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: iron });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: iron });
        checked(&mut g, Action::BeginImprovise);
        for id in fs.iter().take(4) {
            select(&mut g, *id);
        }
        reject_improviser(&mut g, fs[4]);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(iron).zone, Zone::Battlefield);
        // Direct typed cost query below preserves explicit {C}; Improvise never produces mana.
        let purpose = g.spell_mana_purpose_for_card(p, iron, Card::from_name(IRON).unwrap());
        assert!(!g.can_fund_contributions_for(
            FundingGoal {
                actor: p,
                required: [0, 0, 0, 0, 0, 9],
                generic: 0,
                source: None,
                reserve: None
            },
            purpose,
            false
        ));
    }
}
#[test]
fn msh_improvise_noncreature_grant_current_controller_suppression_and_departure() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let iron = paid(&mut g, p, IRON, &[]);
            let fs = fuel(&mut g, p, 2);
            if mode == 0 {
                g.change_controller(g.object_ref(iron), p.other());
            }
            if mode == 1 {
                paid(&mut g, p, "Witness Protection", &[Target::Card(iron)]);
            }
            if mode == 2 {
                paid(&mut g, p, "Unsummon", &[Target::Card(iron)]);
            }
            if mode == 3 {
                g.change_controller(g.object_ref(iron), p.other());
                g.change_controller(g.object_ref(iron), p);
            }
            let spell = named(&mut g, p, "Divination", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            if mode < 3 {
                // Other artifacts can make real colorless mana; announcement remains fundable.
                checked(&mut g, Action::AnnounceCast { card: spell });
                reject(&mut g, Action::BeginImprovise);
            } else {
                checked(&mut g, Action::AnnounceCast { card: spell });
                checked(&mut g, Action::BeginImprovise);
                for id in fs {
                    select(&mut g, id);
                }
                pay(&mut g);
                settle(&mut g);
            }
        }
    }
}
#[test]
fn msh_improvise_creature_spell_does_not_gain_grant_and_intrinsic_is_redundant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, IRON, &[]);
        fuel(&mut g, p, 7);
        let giant = named(&mut g, p, "Hill Giant", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: giant });
        reject(&mut g, Action::BeginImprovise);
        // Arc has intrinsic and granted Improvise; one tap still reduces only one generic.
        let mut g = setup(p);
        paid(&mut g, p, IRON, &[]);
        let fs = fuel(&mut g, p, 5);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        for id in fs {
            select(&mut g, id);
        }
        assert_eq!(g.improvise_choice().unwrap().remaining.generic, 0);
        pay(&mut g);
        settle(&mut g);
    }
}
#[test]
fn msh_improvise_convoke_union_counts_each_artifact_creature_once() {
    for p in [Player::P0, Player::P1] {
        for begin in [Action::BeginConvoke, Action::BeginImprovise] {
            let mut g = setup(p);
            let iron = paid(&mut g, p, IRON, &[]);
            let fs = fuel(&mut g, p, 2);
            let black = named(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
            let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let vote = named(&mut g, p, "Vote Out", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            checked(&mut g, Action::AnnounceCast { card: vote });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            checked(&mut g, begin);
            convoker(&mut g, black, ConvokeContribution::Black);
            select(&mut g, iron);
            {
                let object = g.object_ref(iron);
                reject(
                    &mut g,
                    Action::ChooseConvoker {
                        object,
                        contribution: ConvokeContribution::Generic,
                    },
                );
            }
            for id in fs {
                select(&mut g, id);
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_improvise_equipment_separate_taps_bow_and_one_simultaneous_plan_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = paid(&mut g, p, IRON, &[]);
        let plan = paid(&mut g, p, "Rewrite History", &[]);
        let bow = paid(&mut g, p, "Hawkeye's Bow", &[]);
        let equip=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..} if *card==bow&&*t==iron)).unwrap();
        checked(&mut g, equip);
        settle(&mut g);
        let other = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        animate(&mut g, p, other, true);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        for id in [iron, bow, other] {
            select(&mut g, id);
        }
        assert!(!g.object_view(iron).tapped && !g.object_view(bow).tapped);
        assert!(g.state.pending_triggers.is_empty());
        let life = g.state.players[p.other().index()].life;
        pay(&mut g);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .chain(g.state.ordering_triggers.iter())
                .filter(|s| s.card == plan)
                .count(),
            1
        );
        settle(&mut g);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 1);
        assert_eq!(g.state.players[p.other().index()].life, life - 1);
    }
}
#[test]
fn msh_improvise_first_tap_trigger_waits_until_commit_then_untaps_without_refunding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Captain America, Living Legend", &[]);
        let iron = paid(&mut g, p, IRON, &[]);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        select(&mut g, iron);
        assert!(!g.first_tap(g.object_ref(iron)));
        assert!(!g.object_view(iron).tapped);
        pay(&mut g);
        assert!(g.object_view(iron).tapped);
        settle(&mut g);
        assert!(!g.object_view(iron).tapped);
        assert!(g.first_tap(g.object_ref(iron)));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_improvise_orphan_raw_index_restore_and_actor_owned_choice_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fs = fuel(&mut g, p, 5);
        let secret = named(&mut g, p, "Divination", Zone::Hand);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        select(&mut g, fs[0]);
        assert!(g.observe(p).improvise_choice.is_some());
        assert!(g.observe(p.other()).improvise_choice.is_none());
        assert!(g.observe(p).convoke_choice.is_none());
        let mut hidden = g.clone();
        hidden.state.objects[secret as usize].card = Card::from_name("Murder").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(hidden.observe(p.other())).unwrap()
        );
        let boundary = g.snapshot();
        let mut orphan = {
            let parent = g.clone();
            parent.clone()
        };
        for id in fs.iter().skip(1) {
            select(&mut orphan, *id);
        }
        pay(&mut orphan);
        let done = orphan.snapshot();
        assert_eq!(g.snapshot(), boundary);
        orphan.restore(&boundary).unwrap();
        for id in fs.into_iter().skip(1) {
            select(&mut orphan, id);
        }
        pay(&mut orphan);
        assert_eq!(orphan.snapshot(), done);
        settle(&mut orphan);
    }
}
#[test]
fn msh_improvise_ordered_casualty_can_use_artifact_before_sacrifice_not_after() {
    for p in [Player::P0, Player::P1] {
        for mana_first in [false, true] {
            let mut g = setup(p);
            let iron = paid(&mut g, p, IRON, &[]);
            named(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
            let a = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
            animate(&mut g, p, a, true);
            let b = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
            let spell = named(&mut g, p, "Divination", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, u32::from(!mana_first)];
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseCasualty { pay: true });
            checked(&mut g, Action::BeginImprovise);
            if !mana_first {
                clause(&mut g, CastCostClause::Casualty(0));
                {
                    let object = g.object_ref(a);
                    checked(&mut g, Action::PayCasualty { object });
                }
            }
            clause(&mut g, CastCostClause::Mana);
            if mana_first {
                select(&mut g, a);
            } else {
                reject_improviser(&mut g, a);
            }
            select(&mut g, b);
            pay(&mut g);
            if mana_first {
                assert!(g.object_view(a).tapped);
                clause(&mut g, CastCostClause::Casualty(0));
                {
                    let object = g.object_ref(a);
                    checked(&mut g, Action::PayCasualty { object });
                }
            }
            settle(&mut g);
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
            assert_eq!(g.object_view(iron).zone, Zone::Battlefield);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_improvise_source_sacrificed_for_mana_removes_grant_before_helpers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = paid(&mut g, p, IRON, &[]);
        let ring = paid(&mut g, p, "Lotus Ring", &[]);
        let equip=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..} if *card==ring&&*t==iron)).unwrap();
        checked(&mut g, equip);
        settle(&mut g);
        g.state.objects[iron as usize].summoning_sick = false;
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: iron,
                grantor,
                production: [0, 3, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(iron).zone, Zone::Graveyard);
        reject(&mut g, Action::BeginImprovise);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_improvise_casualty_of_grant_provider_uses_current_cost_payment_order() {
    for p in [Player::P0, Player::P1] {
        for mana_first in [false, true] {
            let mut g = setup(p);
            let iron = paid(&mut g, p, IRON, &[]);
            named(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
            let other = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
            let spell = named(&mut g, p, "Divination", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, if mana_first { 0 } else { 2 }];
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseCasualty { pay: true });
            checked(&mut g, Action::BeginImprovise);
            if !mana_first {
                clause(&mut g, CastCostClause::Casualty(0));
                let object = g.object_ref(iron);
                checked(&mut g, Action::PayCasualty { object });
            }
            clause(&mut g, CastCostClause::Mana);
            if mana_first {
                select(&mut g, iron);
                select(&mut g, other);
            } else {
                reject_improviser(&mut g, other);
            }
            pay(&mut g);
            if mana_first {
                clause(&mut g, CastCostClause::Casualty(0));
                let object = g.object_ref(iron);
                checked(&mut g, Action::PayCasualty { object });
            }
            settle(&mut g);
            assert_eq!(g.object_view(iron).zone, Zone::Graveyard);
            assert_eq!(g.object_view(other).tapped, mana_first);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_improvise_effective_artifact_loss_and_flying_ground_block_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = paid(&mut g, p, IRON, &[]);
        paid(&mut g, p, "Witness Protection", &[Target::Card(iron)]);
        let helper = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        reject_improviser(&mut g, iron);
        select(&mut g, helper);
        pay(&mut g);
        settle(&mut g);
        let mut g = setup(p);
        let iron = paid(&mut g, p, IRON, &[]);
        let ground = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[iron as usize].summoning_sick = false;
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: iron });
        checked(&mut g, Action::FinishAttackers);
        for _ in 0..10 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: iron,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        let life = g.state.players[p.other().index()].life;
        for _ in 0..20 {
            if g.state.step == Step::Main2 {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass))
                .unwrap();
            checked(&mut g, a);
        }
        assert_eq!(g.state.players[p.other().index()].life, life - 3);
        assert!(g.object_view(iron).tapped);
    }
}
#[test]
fn msh_improvise_legend_current_incarnation_retains_grant_and_old_object_is_ineligible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = paid(&mut g, p, IRON, &[]);
        let old = g.object_ref(first);
        let second = named(&mut g, p, IRON, Zone::Hand);
        g.state.players[p.index()].mana = [30; 6];
        cast(&mut g, second, None, &[]);
        for _ in 0..12 {
            if matches!(g.state.pending, Pending::Legend { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let object = g.object_ref(second);
        checked(&mut g, Action::KeepLegend { object });
        settle(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        let artifact = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::BeginImprovise);
        reject(&mut g, Action::ChooseImproviser { object: old });
        select(&mut g, second);
        select(&mut g, artifact);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_improvise_does_not_pay_ward_or_activation_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, IRON, &[]);
        let archives = fuel(&mut g, p, 2);
        let victim = named(&mut g, p.other(), "Hulking Raptor", Zone::Battlefield);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, bounce, None, &[Target::Card(victim)]);
        for _ in 0..20 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                })
                .unwrap();
            checked(&mut g, a);
        }
        assert!(matches!(g.state.pending, Pending::ResolutionMana { .. }));
        reject(&mut g, Action::BeginImprovise);
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bounce).zone, Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: archives[0],
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(&mut g, Action::BeginImprovise);
    }
}
#[test]
fn msh_improvise_bargain_restores_suppressed_grant_before_its_mana_clause() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = paid(&mut g, p, IRON, &[]);
        let blank = paid(&mut g, p, "Witness Protection", &[Target::Card(iron)]);
        let helper = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = named(&mut g, p, "Candy Grapple", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        let sacrifice = Some(g.object_ref(blank));
        checked(&mut g, Action::ChooseBargain { sacrifice });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        checked(&mut g, Action::BeginImprovise);
        let mana = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Mana)
            .unwrap() as u16;
        reject(&mut g, Action::ChooseCastCost { clause: mana });
        clause(&mut g, CastCostClause::Additional);
        assert_eq!(g.object_view(blank).zone, Zone::Graveyard);
        assert!(improvise::available(
            &g,
            p,
            Card::from_name("Candy Grapple").unwrap()
        ));
        clause(&mut g, CastCostClause::Mana);
        select(&mut g, helper);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_improvise_qualified_mana_cannot_pay_wrong_spell_but_artifact_can_contribute() {
    for p in [Player::P0, Player::P1] {
        for real in [0, 1] {
            let mut g = setup(p);
            let iron = paid(&mut g, p, IRON, &[]);
            let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            checked(&mut g, Action::AnnounceCast { card: arc });
            checked(&mut g, Action::BeginImprovise);
            select(&mut g, iron);
            pay(&mut g);
            settle(&mut g);
            let castle = named(&mut g, p, "Castle Doom", Zone::Battlefield);
            let helper = named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
            let spell = named(&mut g, p, "Divination", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, real];
            checked(
                &mut g,
                Action::ActivateMana {
                    card: castle,
                    ability: 1,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 1, 0, 0, 0, 0],
                },
            );
            if real == 0 {
                reject(&mut g, Action::AnnounceCast { card: spell });
            } else {
                checked(&mut g, Action::AnnounceCast { card: spell });
                checked(&mut g, Action::BeginImprovise);
                select(&mut g, helper);
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
                assert_eq!(g.mana_lots(p).len(), 1);
            }
        }
    }
}
