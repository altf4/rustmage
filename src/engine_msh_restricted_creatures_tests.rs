//! Individual paid restricted-mana cards and activation-only haste rules.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
const RONIN: &str = "Ronin, Shadow Stalker";
const SHANG: &str = "Shang-Chi, Master of Kung Fu";
fn seed(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, zone);
    }
    assert!([RONIN, SHANG].contains(&name));
    assert_eq!(zone, Zone::Hand);
    let id = named(g, p, "Forest", zone);
    g.state.objects[id as usize].card = card;
    id
}
pub(super) fn game(p: Player) -> Game {
    let mut g = setup(p);
    for s in &mut g.state.players {
        s.mana = [0; 6]
    }
    g
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass)
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = seed(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
pub(super) fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    settle(g);
    id
}
fn announce(id: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability,
        counter_payment: Counters::default(),
    }
}
fn mana(id: CardId, production: Mana) -> Action {
    Action::AnnounceMana {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
        production,
    }
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let b = g.state_bytes();
    let v = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.state_bytes(), b);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        v
    );
}
fn finish(g: &mut Game) {
    for _ in 0..35 {
        if g.state.announcing.is_none() && !matches!(g.state.pending, Pending::ManaLotSpend { .. })
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .unwrap_or_else(|| panic!("unfinished {:?}", g.state.pending));
        checked(g, a)
    }
    panic!("unfinished")
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..250 {
        if stop(g) {
            return;
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::AssignDamage { .. }
                        | Action::ChooseKicker { pay: false }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a)
    }
    panic!("stalled")
}
fn main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.step == Step::Main1 && g.state.active == p && g.state.pending == Pending::Priority
    });
    priority(g, p)
}
fn equip(g: &mut Game, p: Player, e: CardId, who: CardId) {
    priority(g, p);
    checked(g, announce(e, 0));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(who),
        },
    );
    finish(g);
    settle(g);
    assert_eq!(
        g.state.objects[e as usize].attached_to,
        Some(g.object_ref(who))
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|x| *x == a).unwrap();
    owned_checked_step(g, p, i, a)
}
fn removal(g: &mut Game, p: Player, r: CardId, e: CardId, target: CardId) {
    priority(g, p);
    checked(g, announce(r, 1));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    assert_eq!(g.state.pending, Pending::AdditionalCost { player: p });
    checked(
        g,
        Action::ChooseAdditionalCost {
            object: g.object_ref(e),
        },
    );
    finish(g);
}
#[test]
fn msh_restricted_creatures_ronin_paid_body_life_one_color_and_immediate_mana() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = game(p);
            let r = paid(&mut g, p, RONIN, &[]);
            assert_eq!(
                g.stats(r),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            assert!(g.object_view(r).summoning_sick);
            reject(&mut g, mana(r, [1, 1, 0, 0, 0, 0]));
            reject(&mut g, mana(r, [0, 0, 0, 0, 0, 2]));
            let mut n = [0; 6];
            n[color] = 2;
            let stack = g.state.stack.len();
            checked(&mut g, mana(r, n));
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].life, 18);
            assert_eq!(g.state.players[p.index()].mana, n);
            assert_eq!(g.state.stack.len(), stack);
            assert!(!g.object_view(r).tapped);
            assert_eq!(
                g.observe(p).mana_lots[0].restriction,
                ManaRestrictionView::EquipmentSpellOrEquip
            );
        }
    }
}
#[test]
fn msh_restricted_creatures_ronin_once_each_turn_life_affordability_and_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        g.state.players[p.index()].life = 1;
        reject(&mut g, mana(r, [2, 0, 0, 0, 0, 0]));
        g.state.players[p.index()].life = 20;
        checked(&mut g, mana(r, [2, 0, 0, 0, 0, 0]));
        finish(&mut g);
        reject(&mut g, mana(r, [2, 0, 0, 0, 0, 0]));
        next_turn(&mut g);
        priority(&mut g, p);
        checked(&mut g, mana(r, [0, 2, 0, 0, 0, 0]));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 16);
        reject(&mut g, mana(r, [0, 2, 0, 0, 0, 0]));
    }
}
#[test]
fn msh_restricted_creatures_ronin_mana_pays_real_equipment_spell_and_equip() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        checked(&mut g, mana(r, [2, 0, 0, 0, 0, 0]));
        finish(&mut g);
        let e = seed(&mut g, p, "S.H.I.E.L.D. Spy Kit", Zone::Hand);
        cast(&mut g, e, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        equip(&mut g, p, e, r);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.stats(r),
            Stats {
                power: 4,
                toughness: 4
            }
        );
    }
}
#[test]
fn msh_restricted_creatures_ronin_rejects_non_equipment_spells_and_channel() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        checked(&mut g, mana(r, [0, 0, 0, 2, 0, 0]));
        finish(&mut g);
        for name in ["Grizzly Bears", "Arc Reactor", "Shock"] {
            let c = seed(&mut g, p, name, Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: c });
        }
        let m = seed(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Hand);
        g.state.players[p.index()].mana[3] += 1;
        reject(&mut g, announce(m, 1));
        assert_eq!(g.state.players[p.index()].life, 18);
    }
}
#[test]
fn msh_restricted_creatures_ronin_restriction_survives_producer_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        checked(&mut g, mana(r, [2, 0, 0, 0, 0, 0]));
        finish(&mut g);
        paid(&mut g, p.other(), "Murder", &[Target::Card(r)]);
        priority(&mut g, p);
        assert_eq!(g.object_view(r).zone, Zone::Graveyard);
        let e = seed(&mut g, p, "S.H.I.E.L.D. Spy Kit", Zone::Hand);
        cast(&mut g, e, None, &[]);
        settle(&mut g);
        let c = seed(&mut g, p, "Shock", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: c });
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
    }
}
#[test]
fn msh_restricted_creatures_ronin_paid_attached_sacrifice_gives_minus_four_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        let e = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, p, e, r);
        let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        next_turn(&mut g);
        next_turn(&mut g);
        main(&mut g, p);
        removal(&mut g, p, r, e, victim);
        assert_eq!(g.object_view(e).zone, Zone::Graveyard);
        assert!(g.object_view(r).tapped);
        settle(&mut g);
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert_eq!(
            g.stats(r),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        next_turn(&mut g);
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 10,
                toughness: 10
            }
        );
    }
}
#[test]
fn msh_restricted_creatures_ronin_cost_only_current_controlled_attached_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        let e = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        let other = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let wrong = named(&mut g, p, "S.H.I.E.L.D. Spy Kit", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "S.H.I.E.L.D. Spy Kit", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(wrong), g.object_ref(other)));
        assert!(g.attach_equipment(g.object_ref(foreign), g.object_ref(r)));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, p, e, r);
        let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        next_turn(&mut g);
        next_turn(&mut g);
        main(&mut g, p);
        checked(&mut g, announce(r, 1));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let v = g.observe(p).additional_cost.unwrap();
        assert_eq!(v.kind, EffectChoiceKind::SacrificeAttachedEquipment);
        assert_eq!(v.candidates, vec![g.object_ref(e)]);
        for c in [wrong, foreign, other] {
            let object = g.object_ref(c);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(e);
        g = orphan(g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 6,
                toughness: 6
            }
        );
    }
}
#[test]
fn msh_restricted_creatures_ronin_tap_sorcery_and_equipment_cost_are_real_starting_requirements() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        reject(&mut g, announce(r, 1));
        let e = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, p, e, r);
        reject(&mut g, announce(r, 1));
        let s = paid(&mut g, p, SHANG, &[]);
        assert!(!g.keywords(r).contains(Keywords::HASTE));
        assert!(g.legal_actions().contains(&announce(r, 1)));
        g.state.objects[r as usize].tapped = true;
        reject(&mut g, announce(r, 1));
        g.state.objects[r as usize].tapped = false;
        next_turn(&mut g);
        priority(&mut g, p);
        reject(&mut g, announce(r, 1));
        assert_eq!(g.object_view(s).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_restricted_creatures_shang_paid_self_mana_immediate_without_haste_or_attack() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = game(p);
            let s = paid(&mut g, p, SHANG, &[]);
            assert_eq!(
                g.stats(s),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert!(!g.keywords(s).contains(Keywords::HASTE));
            assert!(g.object_view(s).summoning_sick);
            let mut production = [0; 6];
            production[color] = 2;
            reject(&mut g, mana(s, [1, 1, 0, 0, 0, 0]));
            reject(&mut g, mana(s, [0, 0, 0, 0, 0, 2]));
            checked(&mut g, mana(s, production));
            finish(&mut g);
            assert!(g.object_view(s).tapped);
            assert_eq!(g.state.players[p.index()].mana, production);
            assert_eq!(
                g.observe(p).mana_lots[0].restriction,
                ManaRestrictionView::CreatureActivation
            );
        }
    }
}
#[test]
fn msh_restricted_creatures_shang_allows_new_intrinsic_and_printed_mana_without_granting_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let elf = paid(&mut g, p, "Llanowar Elves", &[]);
        let helper = paid(&mut g, p, "Hydraulic Helper", &[]);
        assert!(g.object_view(elf).summoning_sick);
        assert!(g.object_view(helper).summoning_sick);
        assert!(!g.keywords(elf).contains(Keywords::HASTE));
        checked(&mut g, Action::TapMana { card: elf });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        checked(&mut g, mana(helper, [0, 1, 0, 0, 0, 0]));
        finish(&mut g);
        assert!(g.object_view(helper).tapped);
        assert!(!g.object_view(s).tapped);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Attackers { .. })
        });
        reject(&mut g, Action::Attack { card: s });
    }
}
#[test]
fn msh_restricted_creatures_shang_suppression_and_restoration_control_live_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let elf = paid(&mut g, p, "Llanowar Elves", &[]);
        let witness = paid(&mut g, p.other(), "Witness Protection", &[Target::Card(s)]);
        priority(&mut g, p);
        reject(&mut g, Action::TapMana { card: elf });
        reject(&mut g, mana(s, [0, 0, 0, 0, 2, 0]));
        paid(&mut g, p, "Disenchant", &[Target::Card(witness)]);
        checked(&mut g, Action::TapMana { card: elf });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
    }
}
#[test]
fn msh_restricted_creatures_shang_restricted_mana_pays_creature_hand_cycling() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let scientist = seed(&mut g, p, "A.I.M. Scientists", Zone::Hand);
        checked(&mut g, mana(s, [0, 0, 0, 0, 2, 0]));
        finish(&mut g);
        let count = g.hand_count(p);
        checked(&mut g, announce(scientist, 0));
        finish(&mut g);
        assert_eq!(g.object_view(scientist).zone, Zone::Graveyard);
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseSearch { .. }))
        });
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSearch { .. }))
            .unwrap();
        checked(&mut g, a);
        settle(&mut g);
        assert_eq!(g.hand_count(p), count);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_restricted_creatures_shang_restricted_mana_pays_graveyard_creature_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let w = named(&mut g, p, "Project Deathlok Soldier", Zone::Graveyard);
        let old = g.object_ref(w);
        g.state.players[p.index()].mana[5] += 1;
        checked(&mut g, mana(s, [0, 0, 2, 0, 0, 0]));
        finish(&mut g);
        checked(&mut g, announce(w, 0));
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(w).zone, Zone::Hand);
        assert_ne!(g.object_ref(w), old);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_restricted_creatures_shang_rejects_spells_noncreature_equip_and_channel() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let e = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        checked(&mut g, mana(s, [0, 0, 0, 2, 0, 0]));
        finish(&mut g);
        reject(&mut g, announce(e, 0));
        for name in ["Grizzly Bears", "Arc Reactor", "Shock"] {
            let c = seed(&mut g, p, name, Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: c });
        }
        let m = seed(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Hand);
        g.state.players[p.index()].mana[3] += 1;
        reject(&mut g, announce(m, 1));
    }
}
#[test]
fn msh_restricted_creatures_ronin_attached_cost_and_queue_survive_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        paid(&mut g, p, SHANG, &[]);
        let e = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, p, e, r);
        let victim = named(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        removal(&mut g, p, r, e, victim);
        assert_eq!(g.object_view(e).zone, Zone::Graveyard);
        paid(&mut g, p.other(), "Murder", &[Target::Card(r)]);
        assert_eq!(g.object_view(r).zone, Zone::Graveyard);
        assert_eq!(
            g.stats(victim),
            Stats {
                power: 6,
                toughness: 6
            }
        );
    }
}
#[test]
fn msh_restricted_creatures_ronin_target_departure_does_not_refund_tap_or_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        paid(&mut g, p, SHANG, &[]);
        let e = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, p, e, r);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        removal(&mut g, p, r, e, victim);
        paid(&mut g, p.other(), "Unsummon", &[Target::Card(victim)]);
        assert_eq!(g.object_view(victim).zone, Zone::Hand);
        assert_eq!(g.object_view(e).zone, Zone::Graveyard);
        assert!(g.object_view(r).tapped);
    }
}
#[test]
fn msh_restricted_creatures_ronin_self_target_loses_equipment_before_minus_four() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let r = paid(&mut g, p, RONIN, &[]);
        paid(&mut g, p, SHANG, &[]);
        let e = paid(&mut g, p, "S.H.I.E.L.D. Spy Kit", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        equip(&mut g, p, e, r);
        removal(&mut g, p, r, e, r);
        assert_eq!(g.object_view(e).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(r).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_restricted_creatures_ronin_equipment_mana_funds_equip_inherited_by_non_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let e = named(&mut g, p.other(), "S.H.I.E.L.D. Spy Kit", Zone::Battlefield);
        let f = packet(&mut g, p, "Territory Forge", &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(e),
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(e),
            },
        );
        settle(&mut g);
        let b = g
            .borrowed_ability_views()
            .into_iter()
            .find(|b| {
                b.source.card == f
                    && b.definition == Card::from_name("S.H.I.E.L.D. Spy Kit").unwrap()
            })
            .unwrap();
        let r = paid(&mut g, p, RONIN, &[]);
        checked(&mut g, mana(r, [2, 0, 0, 0, 0, 0]));
        finish(&mut g);
        checked(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: f,
                exiled: b.exiled,
                ability: b.ability,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(r),
            },
        );
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.object_view(f).attached_to, None);
    }
}
#[test]
fn msh_restricted_creatures_shang_permission_moves_to_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let own = paid(&mut g, p, "Llanowar Elves", &[]);
        let foreign = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.state.objects[foreign as usize].summoning_sick = true;
        paid(
            &mut g,
            p,
            "Harmless Offering",
            &[Target::Player(p.other()), Target::Card(s)],
        );
        assert_eq!(g.object_view(s).controller, p.other());
        reject(&mut g, Action::TapMana { card: own });
        priority(&mut g, p.other());
        checked(&mut g, Action::TapMana { card: foreign });
        assert_eq!(g.state.players[p.other().index()].mana, [0, 0, 0, 0, 1, 0]);
    }
}
#[test]
fn msh_restricted_creatures_shang_departure_removes_permission_but_not_produced_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let elf = paid(&mut g, p, "Llanowar Elves", &[]);
        g.state.players[p.index()].mana[5] += 1;
        checked(&mut g, mana(s, [0, 0, 2, 0, 0, 0]));
        finish(&mut g);
        paid(&mut g, p.other(), "Murder", &[Target::Card(s)]);
        priority(&mut g, p);
        reject(&mut g, Action::TapMana { card: elf });
        let w = named(&mut g, p, "Project Deathlok Soldier", Zone::Graveyard);
        checked(&mut g, announce(w, 0));
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(w).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_restricted_creatures_shang_real_land_and_restricted_mana_fund_creature_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        paid(&mut g, p, SHANG, &[]);
        let r = paid(&mut g, p, "Reptil, Dinomorpher", &[]);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let before = g.snapshot();
        let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
        assert!(g.legal_actions().contains(&announce(r, 0)));
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
            views
        );
        checked(&mut g, announce(r, 0));
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let shang = (0..g.state.objects.len())
            .find(|&i| g.state.objects[i].card == Card::from_name(SHANG).unwrap())
            .unwrap() as CardId;
        checked(&mut g, mana(shang, [0, 0, 0, 0, 2, 0]));
        checked(&mut g, Action::TapMana { card: land });
        finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(land).tapped);
        assert_eq!(g.stats(r).power, 3);
    }
}
#[test]
fn msh_restricted_creatures_shang_allows_attached_granted_tap_activation_and_its_mana_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = game(p);
        let s = paid(&mut g, p, SHANG, &[]);
        let bear = paid(&mut g, p, "Grizzly Bears", &[]);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        equip(&mut g, p, pole, bear);
        assert!(g.object_view(bear).summoning_sick);
        checked(&mut g, mana(s, [0, 0, 0, 0, 2, 0]));
        finish(&mut g);
        let action = Action::AnnounceAttachedAbility {
            grantor: g.object_ref(pole),
            ability: 1,
        };
        assert!(g.legal_actions().contains(&action));
        g = orphan(g, action);
        finish(&mut g);
        settle(&mut g);
        assert!(g.object_view(bear).tapped);
        assert!(g.object_view(pole).tapped);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 1);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(!g.keywords(bear).contains(Keywords::HASTE));
    }
}
