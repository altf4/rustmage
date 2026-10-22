use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 13] = [
    "Beastbond Outcaster",
    "Blacksnag Buzzard",
    "Cunning Coyote",
    "Djinn of Fool's Fall",
    "Irascible Wolverine",
    "Loan Shark",
    "Slickshot Show-Off",
    "Spinewoods Paladin",
    "Stagecoach Security",
    "Stingerback Terror",
    "Demonic Ruckus",
    "Rictus Robber",
    "Rise of the Varmints",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [40; 6];
    g
}
fn replay(g: &mut Game, a: Action) {
    assert!(
        g.legal_actions().contains(&a),
        "{a:?} {:?}",
        g.state.pending
    );
    let old = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(f.snapshot(), after);
    g.restore(&old).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn pick(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .min_by_key(|a| match a {
            Action::ChooseTarget {
                target: Target::Card(_),
            } => 0,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Graveyard,
                ..
            } => 0,
            Action::ChooseSpellCost { .. } => 0,
            Action::PayCosts { .. } | Action::PayPlot { .. } => 1,
            Action::PutTrigger { .. } => 1,
            Action::ChooseMobilizeDefender {
                defender: CombatDefender::Player(_),
            } => 0,
            Action::FinishTargets => 2,
            Action::Pass => 3,
            Action::Concede => 10000,
            _ => 100,
        })
        .unwrap();
    replay(g, a);
}
fn idle(g: &Game) -> bool {
    g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.pending == Pending::Priority
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if idle(g) {
            return;
        }
        pick(g);
    }
    panic!("unsettled {:?}", g.state.pending)
}

fn plot(g: &mut Game, p: Player, id: CardId) {
    let old = g.object_ref(id);
    let card = g.state.objects[id as usize].card;
    let spells = g.state.players[p.index()].spells_cast_this_turn;
    let entries = g.state.players[p.index()].nonland_permanents_entered_this_turn;
    let next = g.state.next_stack_id;
    replay(g, Action::AnnouncePlot { card: id });
    assert!(g.observe(p).plot_payment.is_some());
    assert!(g.observe(p.other()).plot_payment.is_none());
    assert_eq!(g.object_ref(id), old);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
    assert!(g.state.stack.is_empty());
    assert!(g.state.announcing.is_none());
    assert_eq!(g.apply(p.other(), Action::Pass), Err(Error::WrongActor));
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayPlot { .. }))
        .unwrap();
    let before = g.state.players[p.index()].mana;
    replay(g, a);
    assert_eq!(g.state.priority, p);
    assert_eq!(g.state.players[p.index()].spells_cast_this_turn, spells);
    assert_eq!(
        g.state.players[p.index()].nonland_permanents_entered_this_turn,
        entries
    );
    assert_eq!(g.state.next_stack_id, next);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
    assert_eq!(g.object_ref(id).generation, old.generation + 1);
    let perms = g.observe(p).plotted_cards;
    assert_eq!(perms, g.observe(p.other()).plotted_cards);
    assert!(
        perms.iter().any(|x| x.object == g.object_ref(id)
            && x.owner == p
            && x.plotted_turn == g.state.turn)
    );
    assert_eq!(
        before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
        card.plot_cost().unwrap().colored.iter().sum::<u32>() + card.plot_cost().unwrap().generic
    );
    reject(g, Action::AnnouncePlotted { card: id });
    reject(g, Action::AnnounceCast { card: id });
}
fn cast(g: &mut Game, id: CardId, plotted: bool) {
    if plotted {
        g.state.turn += 1;
    }
    replay(
        g,
        if plotted {
            Action::AnnouncePlotted { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    while g.state.announcing.is_some() {
        pick(g);
    }
    let spell = g
        .state
        .stack
        .iter()
        .find(|s| s.card == id && s.kind == StackKind::Spell)
        .unwrap();
    assert_eq!(
        spell.cast.unwrap().method,
        if plotted {
            CastMethod::Plot
        } else {
            CastMethod::Normal
        }
    );
    assert_eq!(
        spell.cast.unwrap().origin,
        if plotted { Zone::Exile } else { Zone::Hand }
    );
    assert_eq!(spell.x, None);
    assert!(!g.state.objects[id as usize].flashback);
    settle(g);
}
#[test]
fn all_thirteen_full_bodies_hardcast_and_plotted_both_seats() {
    for p in [Player::P0, Player::P1] {
        for plotted in [false, true] {
            for (i, n) in NAMES.iter().enumerate() {
                let mut g = setup(p);
                let ally = named(&mut g, p, "Gigantosaurus", Zone::Battlefield);
                named(&mut g, p, "Bear Cub", Zone::Graveyard);
                g.state.creatures_died_this_turn = 1;
                g.state.players[p.index()].spells_cast_this_turn = 1;
                let id = named(&mut g, p, n, Zone::Hand);
                let library = g.state.players[p.index()].library.len();
                if plotted {
                    plot(&mut g, p, id);
                }
                cast(&mut g, id, plotted);
                assert_eq!(
                    g.state.objects[id as usize].zone,
                    if i == 12 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    },
                    "{n}"
                );
                assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2, "{n}");
                assert!(g.observe(p).plotted_cards.is_empty());
                match i {
                    0 | 5 => {
                        assert_eq!(g.state.players[p.index()].library.len(), library - 1, "{n}")
                    }
                    1 => assert_eq!((g.stats(id).power, g.stats(id).toughness), (3, 2)),
                    2 => assert_eq!((g.stats(ally).power, g.stats(ally).toughness), (11, 11)),
                    4 => assert_eq!(g.state.exile_permissions.len(), 1),
                    7 => assert_eq!(g.state.players[p.index()].life, 23),
                    8 => assert_eq!((g.stats(ally).power, g.stats(ally).toughness), (11, 11)),
                    9 => assert_eq!((g.stats(id).power, g.stats(id).toughness), (7, 7)),
                    10 => {
                        assert_eq!((g.stats(ally).power, g.stats(ally).toughness), (11, 11));
                        assert!(g.keywords(ally).contains(Keywords::MENACE));
                        assert!(g.keywords(ally).contains(Keywords::TRAMPLE));
                    }
                    11 | 12 => {
                        let tokens: Vec<_> = g
                            .state
                            .objects
                            .iter()
                            .filter(|o| {
                                o.owner == p && o.zone == Zone::Battlefield && o.card.is_token()
                            })
                            .collect();
                        assert_eq!(tokens.len(), 1, "{n}");
                        assert_eq!(tokens[0].card.colors(), if i == 11 { 6 } else { 16 });
                    }
                    _ => {}
                }
            }
        }
    }
}
#[test]
fn strict_turn_main_empty_stack_actor_and_exact_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Djinn of Fool's Fall", Zone::Hand);
        for step in [Step::Upkeep, Step::End, Step::CombatDamage] {
            g.state.step = step;
            reject(&mut g, Action::AnnouncePlot { card: id });
        }
        g.state.step = Step::Main1;
        plot(&mut g, p, id);
        g.state.turn += 1;
        g.state.active = p.other();
        reject(&mut g, Action::AnnouncePlotted { card: id });
        g.state.active = p;
        for step in [Step::Upkeep, Step::End, Step::CombatDamage] {
            g.state.step = step;
            reject(&mut g, Action::AnnouncePlotted { card: id });
        }
        g.state.step = Step::Main2;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnouncePlotted { card: id })
        );
        let old = g.object_ref(id);
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Exile);
        assert_ne!(old, g.object_ref(id));
        assert!(g.observe(p).plotted_cards.is_empty());
        reject(&mut g, Action::AnnouncePlotted { card: id });
    }
}
#[test]
fn unfunded_wrong_color_and_late_transition_capacity_restore_every_owned_field() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Spinewoods Paladin", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnouncePlot { card: id });
        g.state.players[p.index()].mana = [100, 0, 0, 0, 0, 0];
        reject(&mut g, Action::AnnouncePlot { card: id });
        g.state.players[p.index()].mana = [40; 6];
        replay(&mut g, Action::AnnouncePlot { card: id });
        g.state.objects[id as usize].generation = u64::MAX;
        g.state
            .plot
            .as_mut()
            .unwrap()
            .payment
            .as_mut()
            .unwrap()
            .object = g.object_ref(id);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(g.apply(p, a), Err(Error::CapacityExceeded(_))));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        g.state.objects[id as usize].generation = 0;
        g.state
            .plot
            .as_mut()
            .unwrap()
            .payment
            .as_mut()
            .unwrap()
            .object = g.object_ref(id);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        replay(&mut g, a);
        g.state.turn += 1;
        g.state.next_stack_id = u64::MAX;
        let before = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::AnnouncePlotted { card: id }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.observe(p).plotted_cards.len(), 1);
    }
}
#[test]
fn real_land_mana_funds_special_action_without_consuming_hand_source_or_casting() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let mountain = named(&mut g, p, "Mountain", Zone::Battlefield);
        let id = named(&mut g, p, "Slickshot Show-Off", Zone::Hand);
        let before = g.object_ref(id);
        replay(&mut g, Action::AnnouncePlot { card: id });
        reject(&mut g, Action::Pass);
        for land in [forest, mountain] {
            replay(&mut g, Action::TapMana { card: land });
            assert_eq!(g.object_ref(id), before);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            assert!(g.state.stack.is_empty());
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.priority, p);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
    }
}
fn grant(g: &mut Game, p: Player, id: CardId) {
    let object = g.object_ref(id);
    g.state
        .plot
        .get_or_insert_with(Default::default)
        .cards
        .push(PlottedCard {
            object,
            owner: p,
            plotted_turn: g.state.turn - 1,
        });
    g.state
        .plot
        .as_mut()
        .unwrap()
        .cards
        .sort_by_key(|x| x.object.card);
}
#[test]
fn plot_is_not_noncreature_cast_but_later_free_aura_cast_is_actual_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let observer = named(&mut g, p, "Slickshot Show-Off", Zone::Battlefield);
        let aura = named(&mut g, p, "Demonic Ruckus", Zone::Hand);
        plot(&mut g, p, aura);
        assert_eq!(g.stats(observer).power, 1);
        assert!(g.state.pending_triggers.is_empty());
        cast(&mut g, aura, true);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
        assert_eq!(g.stats(observer).power, 4); // genuine cast +2, attached aura +1
        g.move_object(observer, Zone::Graveyard);
        g.state_based_actions();
        settle(&mut g);
        assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn stinger_scaling_is_battlefield_only_live_and_uses_shared_virtual_hand_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Stingerback Terror", Zone::Hand);
        for _ in 0..5 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (7, 7));
        plot(&mut g, p, id);
        assert_eq!(g.stats(id).power, 7);
        cast(&mut g, id, true);
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (2, 2));
        let hand = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.owner == p && o.zone == Zone::Hand)
            .unwrap()
            .0 as CardId;
        g.move_object(hand, Zone::Graveyard);
        assert_eq!(g.stats(id).power, 3);
        let value = StatValue::ControllerHandSize;
        assert_eq!(g.stat_value(value, g.object_ref(id), p), 4);
        named(&mut g, p.other(), "Forest", Zone::Hand);
        assert_eq!(g.stats(id).power, 3);
        for _ in 0..3 {
            g.draw(p);
        }
        g.state_based_actions();
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(g.stats(id).power, 7);
    }
}
#[test]
fn free_cast_timing_overrides_intrinsic_flash_and_free_x_is_zero_without_alternative_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let flash = named(&mut g, p, "Cogwork Wrestler", Zone::Exile);
        grant(&mut g, p, flash);
        g.state.active = p.other();
        reject(&mut g, Action::AnnouncePlotted { card: flash });
        g.state.active = p;
        g.state.step = Step::BeginCombat;
        reject(&mut g, Action::AnnouncePlotted { card: flash });
        g.state.step = Step::Main1;
        let x = named(&mut g, p, "Mind Spring", Zone::Exile);
        grant(&mut g, p, x);
        g.state.players[p.index()].mana = [0; 6];
        replay(&mut g, Action::AnnouncePlotted { card: x });
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseX { .. } | Action::ChooseSpellCost { .. }))
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.stack[0].x, Some(0));
        assert_eq!(g.state.stack[0].mana_value(), 2);
        settle(&mut g);
        assert_eq!(g.state.objects[x as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn free_cast_preserves_actual_mandatory_discard_cost_and_exact_private_reservation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = named(&mut g, p, "Thrill of Possibility", Zone::Exile);
        grant(&mut g, p, spell);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnouncePlotted { card: spell });
        let fodder = named(&mut g, p, "Forest", Zone::Hand);
        let old = g.object_ref(fodder);
        replay(&mut g, Action::AnnouncePlotted { card: spell });
        assert!(g.observe(p.other()).additional_cost.is_none());
        replay(&mut g, Action::ChooseAdditionalCost { object: old });
        assert_eq!(g.state.objects[fodder as usize].zone, Zone::Hand);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.objects[fodder as usize].zone, Zone::Graveyard);
        assert_eq!(g.object_ref(fodder).generation, old.generation + 1);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn plot_permissions_and_ordinary_paid_exile_rights_are_independent_and_one_shot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Spinewoods Paladin", Zone::Hand);
        plot(&mut g, p, id);
        g.state.exile_permissions.push(ExilePlayPermission {
            cast_only: false,
            without_mana: false,
            object: g.object_ref(id),
            player: p,
            expires_turn: g.state.turn,
        });
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: id })
        );
        reject(&mut g, Action::AnnouncePlotted { card: id });
        replay(&mut g, Action::AnnounceExile { card: id });
        assert!(g.observe(p).plotted_cards.is_empty());
        assert!(g.state.exile_permissions.is_empty());
        while g.state.announcing.is_some() {
            pick(&mut g);
        }
        assert_eq!(g.state.stack[0].cast.unwrap().method, CastMethod::Normal);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 23);
    }
}
#[test]
fn real_mana_sacrifice_is_allowed_during_plot_and_never_sacrifices_reserved_hand_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let id = named(&mut g, p, "Demonic Ruckus", Zone::Hand);
        replay(&mut g, Action::AnnouncePlot { card: id });
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==pizza&&production[3]==1)).unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Hand);
        assert!(!g.state.objects[forest as usize].tapped);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
    }
}
#[test]
fn spectral_denial_domain_after_actual_hand_departure_sees_live_stinger_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stinger = named(&mut g, p, "Stingerback Terror", Zone::Battlefield);
        let denial = named(&mut g, p, "Spectral Denial", Zone::Hand);
        for _ in 0..3 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        g.state.players[p.other().index()].mana = [40; 6];
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let item = g.cast_item(p.other(), foe, [0; 6]);
        g.state.stack.push(item);
        assert_eq!(g.stats(stinger).power, 3);
        replay(&mut g, Action::AnnounceCast { card: denial });
        assert_eq!(g.stats(stinger).power, 4);
        assert!(
            g.x_choice()
                .unwrap()
                .intervals
                .iter()
                .any(|i| i.minimum <= 1 && i.maximum >= 1)
        );
        replay(&mut g, Action::ChooseX { value: 1 });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(foe),
            },
        );
        let total = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
        assert_eq!(total.generic, 0);
        assert_eq!(total.colored, [0, 1, 0, 0, 0, 0]);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.stack.last().unwrap().x, Some(1));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hand_spell_preflight_targets_see_stinger_after_the_spell_leaves_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stinger = named(&mut g, p, "Stingerback Terror", Zone::Battlefield);
        let spell = named(&mut g, p, "Stand Up for Yourself", Zone::Hand);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        assert_eq!(g.stats(stinger).power, 2);
        replay(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(g.stats(stinger).power, 3);
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(stinger),
            },
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        settle(&mut g);
        assert_eq!(g.state.objects[stinger as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn fixed_power_reduction_preflight_locks_after_real_hand_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stinger = named(&mut g, p, "Stingerback Terror", Zone::Battlefield);
        let giant = named(&mut g, p, "Ghalta the Unstoppable", Zone::Hand);
        for _ in 0..5 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 6];
        assert_eq!(g.stats(stinger).power, 1);
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Cast{card,..}if *card==giant))
        );
        replay(&mut g, Action::AnnounceCast { card: giant });
        assert_eq!(g.stats(stinger).power, 2);
        let total = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
        assert_eq!(total.generic, 6);
        assert_eq!(total.colored, [0, 0, 0, 0, 1, 0]);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.objects[giant as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn direct_cast_recursive_power_filter_and_plot_payment_have_distinct_hand_departures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stinger = named(&mut g, p, "Stingerback Terror", Zone::Battlefield);
        let spell = named(&mut g, p, "Repel Calamity", Zone::Hand);
        for _ in 0..3 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        assert_eq!(g.stats(stinger).power, 3);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Cast{card,target:Some(Target::Card(t)),..}if *card==spell && *t==stinger)).unwrap();
        replay(&mut g, action);
        assert_eq!(g.stats(stinger).power, 4);
        settle(&mut g);
        assert_eq!(g.state.objects[stinger as usize].zone, Zone::Graveyard);
        let mut g = setup(p);
        let stinger = named(&mut g, p, "Stingerback Terror", Zone::Battlefield);
        let plot = named(&mut g, p, "Demonic Ruckus", Zone::Hand);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        assert_eq!(g.stats(stinger).power, 2);
        replay(&mut g, Action::AnnouncePlot { card: plot });
        assert_eq!(g.stats(stinger).power, 2);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.stats(stinger).power, 3);
    }
}
#[test]
fn sparse_plot_storage_is_owned_and_frees_after_exact_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Djinn of Fool's Fall", Zone::Hand);
        let baseline = g.state_bytes();
        replay(&mut g, Action::AnnouncePlot { card: id });
        let window = g.state_bytes();
        assert_eq!(window - baseline, std::mem::size_of::<plot::PlotState>());
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayPlot { .. }))
            .unwrap();
        replay(&mut g, a);
        let active = g.state_bytes();
        assert_eq!(active - window, std::mem::size_of::<PlottedCard>());
        let snap = g.snapshot();
        let mut branch = g.clone();
        g.move_object(id, Zone::Hand);
        assert!(g.state.plot.is_none());
        // The public Exile-to-Hand move separately retains one known-hand fact.
        assert_eq!(
            g.state_bytes(),
            baseline + g.state.known_hand.len() * std::mem::size_of::<ObjectRef>()
        );
        assert_eq!(branch.observe(p).plotted_cards.len(), 1);
        branch.restore(&snap).unwrap();
        assert_eq!(branch.state_bytes(), active);
        println!(
            "plot owner={p:?} baseline={baseline} window={window} active={active} box={} entry={}",
            std::mem::size_of::<plot::PlotState>(),
            std::mem::size_of::<PlottedCard>()
        );
    }
}
#[test]
fn exact_plot_permission_survives_real_cleanup_and_both_turns() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Djinn of Fool's Fall", Zone::Hand);
        plot(&mut g, p, id);
        let permission = g.observe(p).plotted_cards[0];
        let turn = g.state.turn;
        for _ in 0..100 {
            if g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1 {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                    )
                })
                .unwrap();
            replay(&mut g, a);
        }
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.step, Step::Main1);
        assert!(g.state.turn > turn);
        assert_eq!(g.observe(p).plotted_cards, vec![permission]);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnouncePlotted { card: id })
        );
    }
}
#[test]
fn hand_discard_activation_retains_actual_hand_characteristics_until_cost_commit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stinger = named(&mut g, p, "Stingerback Terror", Zone::Battlefield);
        let arm = named(&mut g, p, "Spinewoods Armadillo", Zone::Hand);
        for _ in 0..4 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        assert_eq!(g.stats(stinger).power, 2);
        let old = g.object_ref(arm);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: arm,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.object_ref(arm), old);
        assert_eq!(g.state.objects[arm as usize].zone, Zone::Hand);
        assert_eq!(g.stats(stinger).power, 2);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.objects[arm as usize].zone, Zone::Graveyard);
        assert_eq!(g.stats(stinger).power, 3);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
    }
}
#[test]
fn conditional_reduction_guard_uses_prospective_total_toughness_and_current_payment_locks() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stinger = named(&mut g, p, "Stingerback Terror", Zone::Battlefield);
        named(&mut g, p, "Hamlet Glutton", Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let orysa = named(&mut g, p, "Orysa, Tide Choreographer", Zone::Hand);
        for _ in 0..5 {
            named(&mut g, p, "Forest", Zone::Hand);
        }
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        assert_eq!(g.stats(stinger).toughness, 1);
        assert!(!g.step_guard(
            StepGuard::Threshold {
                value: StepValue::TotalCreatureToughness,
                minimum: Some(10),
                maximum: None
            },
            p,
            g.object_ref(orysa),
            false
        ));
        replay(&mut g, Action::AnnounceCast { card: orysa });
        assert_eq!(g.stats(stinger).toughness, 2);
        assert_eq!(
            g.state.announcing.as_ref().unwrap().total_cost.unwrap(),
            ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 1
            }
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        settle(&mut g);
        assert_eq!(g.state.objects[orysa as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
