use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn resolve(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.state.stack.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.announcing.is_none()
        {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PutTrigger { .. }
                        | Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::Pass
                )
            })
            .expect("trigger continuation");
        act(g, action);
    }
    panic!("unsettled trigger")
}

fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    g.state.active = p;
    g.state.priority = p;
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, n) = g.state.objects[id as usize].card.cost();
    mana[5] += n;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    pass_pair(g);
    resolve(g);
    id
}
#[test]
fn job_select_complete_basic_bodies_create_actual_heroes_with_live_types() {
    for p in [Player::P0, Player::P1] {
        for (name, subtype, power, toughness, keywords) in [
            ("Bard's Bow", "Bard", 3, 3, Keywords::REACH),
            ("Dark Knight's Greatsword", "Knight", 4, 1, Keywords::NONE),
            ("Dragoon's Lance", "Knight", 2, 1, Keywords::FLYING),
            ("Machinist's Arsenal", "Artificer", 3, 3, Keywords::NONE),
            ("Monk's Fist", "Monk", 2, 1, Keywords::NONE),
            (
                "Samurai's Katana",
                "Samurai",
                3,
                3,
                Keywords::TRAMPLE.union(Keywords::HASTE),
            ),
            ("Warrior's Sword", "Warrior", 4, 3, Keywords::NONE),
        ] {
            let mut g = scenario();
            let equipment = paid(&mut g, p, name);
            let hero = g.state.objects[equipment as usize].attached_to.unwrap();
            assert!(g.state.objects[hero.card as usize].card.is_token());
            assert!(g.state.objects[hero.card as usize].card.has_subtype("Hero"));
            assert!(
                !g.state.objects[hero.card as usize]
                    .card
                    .has_subtype(subtype)
            );
            assert!(g.object_view(hero.card).has_subtype(subtype));
            assert_eq!(g.stats(hero.card), Stats { power, toughness });
            assert!(g.keywords(hero.card).contains(keywords));
            let before = g.snapshot();
            let mut fork = g.clone();
            for viewer in [Player::P0, Player::P1] {
                let observed = g.observe(viewer);
                assert!(
                    observed
                        .objects
                        .iter()
                        .find(|o| o.id == hero.card)
                        .unwrap()
                        .has_subtype(subtype)
                );
            }
            g.move_object(equipment, Zone::Hand);
            assert!(!g.has_subtype(hero.card, subtype));
            assert_eq!(
                g.stats(hero.card),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            g.restore(&before).unwrap();
            assert_eq!(g.snapshot(), fork.snapshot());
            fork.state.active = p.other();
            if name == "Dragoon's Lance" {
                assert!(!fork.keywords(hero.card).contains(Keywords::FLYING));
            }
        }
    }
}
#[test]
fn job_select_exact_source_departure_still_creates_but_does_not_reattach() {
    for reenter in [false, true] {
        let mut g = scenario();
        let equipment = named(&mut g, Player::P0, "Monk's Fist", Zone::Hand);
        g.state.players[0].mana[5] = 2;
        cast(&mut g, equipment, None);
        pass_pair(&mut g);
        let old = g.object_ref(equipment);
        g.move_object(equipment, Zone::Hand);
        if reenter {
            g.move_object(equipment, Zone::Battlefield);
        }
        resolve(&mut g);
        let heroes: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield && o.card.is_token() && o.card.has_subtype("Hero")
            })
            .collect();
        assert_eq!(heroes.len(), if reenter { 2 } else { 1 });
        let attached = g.object_view(equipment).attached_to;
        assert_eq!(
            heroes
                .iter()
                .filter(|(id, _)| Some(g.object_ref(*id as CardId)) != attached)
                .count(),
            1
        );
        assert_ne!(g.object_ref(equipment), old);
        if !reenter {
            assert_eq!(attached, None);
        }
        // Reentry has its own genuine Job Select trigger; the old trigger's Hero stays unattached.
        for (id, _) in heroes {
            if Some(g.object_ref(id as CardId)) != attached {
                assert_eq!(
                    g.stats(id as CardId),
                    Stats {
                        power: 1,
                        toughness: 1
                    }
                );
            }
        }
    }
}
#[test]
fn job_select_union_exact_links_and_captured_lki() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let sword = named(&mut g, Player::P0, "Warrior's Sword", Zone::Battlefield);
    let fist = named(&mut g, Player::P1, "Monk's Fist", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(sword), g.object_ref(bear)));
    assert!(g.attach_equipment(g.object_ref(fist), g.object_ref(bear)));
    let before = g.object_view(bear);
    assert!(
        before.has_subtype("Warrior") && before.has_subtype("Monk") && before.has_subtype("Bear")
    );
    let damage = DamageFacts::from_view(&before);
    assert!(damage.has_subtype("Monk"));
    g.move_object(bear, Zone::Hand);
    assert!(!g.object_view(bear).has_subtype("Warrior"));
    assert!(before.has_subtype("Warrior") && damage.has_subtype("Monk"));
    g.move_object(bear, Zone::Battlefield);
    assert!(!g.has_subtype(bear, "Monk"));
    assert!(!crate::SearchFilter::Subtype("Warrior").matches(g.state.objects[bear as usize].card));
}

#[test]
fn job_select_life_equip_is_targeted_once_each_turn_and_scaling_tracks_controller_artifacts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let sword = paid(&mut g, p, "Dark Knight's Greatsword");
        let old = g.object_view(sword).attached_to.unwrap();
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..} if *card==sword&&*t==bear)).unwrap();
        let before = g.snapshot();
        let mut fork = g.clone();
        let life = g.state.players[p.index()].life;
        act(&mut g, action.clone());
        act(&mut fork, action.clone());
        assert_eq!(g.snapshot(), fork.snapshot());
        assert_eq!(g.state.players[p.index()].life, life - 3);
        assert_eq!(g.object_view(sword).attached_to, Some(old));
        resolve(&mut g);
        assert_eq!(g.object_view(sword).attached_to, Some(g.object_ref(bear)));
        assert!(g.has_subtype(bear, "Knight"));
        assert!(!g.has_subtype(old.card, "Knight"));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Activate{card,..} if *card==sword))
        );
        g.restore(&before).unwrap();
        act(&mut g, action);
        resolve(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 3);
        let arsenal = paid(&mut g, p, "Machinist's Arsenal");
        let hero = g.object_view(arsenal).attached_to.unwrap();
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        let foe = named(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        g.move_object(sword, Zone::Graveyard);
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        g.move_object(foe, Zone::Graveyard);
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}

#[test]
fn job_select_all_sixteen_whole_bodies_create_and_attach() {
    for p in [Player::P0, Player::P1] {
        for (name, subtype) in [
            ("Astrologian's Planisphere", "Wizard"),
            ("Bard's Bow", "Bard"),
            ("Black Mage's Rod", "Wizard"),
            ("Dark Knight's Greatsword", "Knight"),
            ("Dragoon's Lance", "Knight"),
            ("Machinist's Arsenal", "Artificer"),
            ("Monk's Fist", "Monk"),
            ("Ninja's Blades", "Ninja"),
            ("Paladin's Arms", "Knight"),
            ("Red Mage's Rapier", "Wizard"),
            ("Sage's Nouliths", "Cleric"),
            ("Samurai's Katana", "Samurai"),
            ("Summoner's Grimoire", "Shaman"),
            ("Thief's Knife", "Rogue"),
            ("Warrior's Sword", "Warrior"),
            ("White Mage's Staff", "Cleric"),
        ] {
            let mut g = scenario();
            let equipment = paid(&mut g, p, name);
            let hero = g.object_view(equipment).attached_to.unwrap();
            assert!(g.object_view(hero.card).has_subtype(subtype), "{name}");
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
                    .count(),
                1
            );
        }
    }
}
#[test]
fn job_granted_cast_draw_controller_provenance_and_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let orb = named(
            &mut g,
            p.other(),
            "Astrologian's Planisphere",
            Zone::Battlefield,
        );
        let rod = named(&mut g, p.other(), "Black Mage's Rod", Zone::Battlefield);
        let rapier = named(&mut g, p.other(), "Red Mage's Rapier", Zone::Battlefield);
        for equipment in [orb, rod, rapier] {
            assert!(g.attach_equipment(g.object_ref(equipment), g.object_ref(bear)));
        }
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana[3] = 1;
        cast(&mut g, shock, Some(Target::Player(p.other())));
        let packets: Vec<_> = g
            .state
            .stack
            .iter()
            .chain(g.state.ordering_triggers.iter())
            .chain(g.state.pending_triggers.iter())
            .filter(|s| s.attachment_trigger.is_some())
            .collect();
        assert_eq!(packets.len(), 3);
        for s in packets {
            assert_eq!(s.object, g.object_ref(bear));
            assert_eq!(s.source_card, Card::from_name("Bear Cub").unwrap());
            assert_eq!(s.controller, p);
            let ctx = s.attachment_trigger.as_ref().unwrap();
            assert_eq!(ctx.recipient, s.object);
            assert_eq!(ctx.defender, None);
            assert!(ctx.granted);
        }
        g.move_object(orb, Zone::Graveyard);
        g.move_object(rod, Zone::Graveyard);
        g.move_object(rapier, Zone::Graveyard);
        resolve(&mut g);
        assert_eq!(g.object_view(bear).counters.plus_one, 1);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 5,
                toughness: 3
            }
        );
        assert_eq!(g.state.players[p.other().index()].life, 17);
        let second = named(&mut g, p, "Astrologian's Planisphere", Zone::Battlefield);
        g.attach_equipment(g.object_ref(second), g.object_ref(bear));
        for _ in 0..2 {
            g.draw(p);
            resolve(&mut g);
        }
        assert_eq!(g.object_view(bear).counters.plus_one, 1);
        g.draw(p);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let old = g.object_ref(bear);
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        assert_ne!(old, g.object_ref(bear));
        resolve(&mut g);
        assert_eq!(g.object_view(bear).counters.plus_one, 0);
    }
}
#[test]
fn job_granted_ninja_actual_private_discard_causes_life_loss_not_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let blades = paid(&mut g, p, "Ninja's Blades");
        let hero = g.object_view(blades).attached_to.unwrap();
        g.state.objects[hero.card as usize].gained_keywords = Keywords::LIFELINK;
        let discard = named(&mut g, p, "Warrior's Sword", Zone::Hand);
        let event = DamageEvent {
            source: g.damage_source(hero.card),
            target: Target::Player(p.other()),
            amount: 2,
            combat: true,
        };
        g.player_damage_triggers(&[event]);
        g.prepare_triggers();
        pass_pair(&mut g);
        assert!(matches!(g.state.pending,Pending::EffectChoice{player} if player==p));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(g.observe(p).effect_choice.is_some());
        assert!(g.observe(p.other()).effect_choice.is_none());
        let snap = g.snapshot();
        let mut fork = g.clone();
        act(&mut g, Action::Discard { card: discard });
        act(&mut fork, Action::Discard { card: discard });
        assert_eq!(g.snapshot(), fork.snapshot());
        assert_eq!(g.state.objects[discard as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].life, 20);
        g.restore(&snap).unwrap();
        act(&mut g, Action::Discard { card: discard });
        assert_eq!(g.snapshot(), fork.snapshot());
    }
}

fn attack_job(g: &mut Game, p: Player, id: CardId) {
    g.state.objects[id as usize].summoning_sick = false;
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}
#[test]
fn job_summoner_private_optional_entry_chooses_real_defender_without_attack_events() {
    for p in [Player::P0, Player::P1] {
        for enchantment in [false, true] {
            for choose_walker in [false, true] {
                let mut g = scenario();
                let book = paid(&mut g, p, "Summoner's Grimoire");
                assert!(g.state.objects[book as usize].card.has_subtype("Book"));
                let hero = g.object_view(book).attached_to.unwrap();
                let card = named(
                    &mut g,
                    p,
                    if enchantment {
                        "Appendage Amalgam"
                    } else {
                        "Bear Cub"
                    },
                    Zone::Hand,
                );
                let old = g.object_ref(card);
                let walker_def = Card::catalogue()
                    .find(|c| c.is_supported() && c.is_planeswalker())
                    .unwrap();
                let walker = spawn(&mut g, p.other(), walker_def, Zone::Battlefield);
                g.state.objects[walker as usize].counters.loyalty = 5;
                attack_job(&mut g, p, hero.card);
                g.move_object(book, Zone::Graveyard);
                pass_pair(&mut g);
                let view = g.observe(p);
                assert_eq!(
                    view.effect_choice.as_ref().unwrap().kind,
                    EffectChoiceKind::HandCreature
                );
                assert!(g.observe(p.other()).effect_choice.is_none());
                assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
                let saved = g.snapshot();
                let mut fork = g.clone();
                let a = Action::PutHandCreature { object: old };
                act(&mut g, a.clone());
                act(&mut fork, a);
                assert_eq!(g.snapshot(), fork.snapshot());
                if enchantment {
                    assert!(matches!(g.state.pending,Pending::HandDefender{player} if player==p));
                    assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
                    assert!(g.observe(p).effect_choice.as_ref().unwrap().defender_choice);
                    assert!(g.observe(p.other()).effect_choice.is_none());
                    let defender = if choose_walker {
                        CombatDefender::Planeswalker(g.object_ref(walker))
                    } else {
                        CombatDefender::Player(p.other())
                    };
                    let action = Action::ChooseHandDefender { defender };
                    act(&mut g, action.clone());
                    act(&mut fork, action);
                    assert_eq!(g.snapshot(), fork.snapshot());
                    assert!(g.object_view(card).tapped);
                    assert_eq!(
                        g.state
                            .attackers
                            .iter()
                            .find(|a| a.card == card)
                            .unwrap()
                            .defender,
                        Some(defender)
                    );
                    // Amalgam's attack-surveil ability does not trigger merely from entering attacking.
                    assert!(g.state.pending_triggers.is_empty());
                    assert!(g.state.ordering_triggers.is_empty());
                    assert!(g.state.stack.is_empty());
                } else {
                    assert!(!g.object_view(card).tapped);
                    assert!(!g.state.attackers.iter().any(|a| a.card == card));
                }
                assert_eq!(g.state.objects[card as usize].zone, Zone::Battlefield);
                assert_ne!(g.object_ref(card), old);
                g.restore(&saved).unwrap();
                act(&mut g, Action::DeclineHandEntry);
                assert_eq!(g.object_ref(card), old);
                assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
            }
        }
    }
}
#[test]
fn job_granted_ward_is_recipient_owned_independent_and_payer_is_targeting_controller() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = scenario();
            let arms = paid(&mut g, p, "Paladin's Arms");
            let hero = g.object_view(arms).attached_to.unwrap();
            let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.priority = p.other();
            g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 1];
            cast(&mut g, shock, Some(Target::Card(hero.card)));
            let ward = g.state.stack.last().unwrap();
            assert_eq!(ward.object, hero);
            assert_eq!(ward.controller, p);
            assert_eq!(ward.related_stack.unwrap().controller, p.other());
            assert_eq!(
                ward.attachment_trigger.as_ref().unwrap().granting_card,
                Card::from_name("Paladin's Arms").unwrap()
            );
            g.move_object(arms, Zone::Graveyard);
            pass_pair(&mut g);
            assert_eq!(g.actor(), Some(p.other()));
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    if pay {
                        matches!(a, Action::PayResolutionMana { .. })
                    } else {
                        matches!(a, Action::DeclineResolutionMana)
                    }
                })
                .unwrap();
            act(&mut g, a);
            resolve(&mut g);
            assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
            assert_eq!(
                g.state.objects[hero.card as usize].zone,
                if pay {
                    Zone::Outside
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn job_sage_white_and_thief_grants_are_real_recipient_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        for name in ["Sage's Nouliths", "White Mage's Staff"] {
            let e = named(&mut g, p.other(), name, Zone::Battlefield);
            g.attach_equipment(g.object_ref(e), g.object_ref(bear));
        }
        attack_job(&mut g, p, bear);
        assert!(g.object_view(bear).tapped);
        resolve(&mut g);
        assert!(!g.object_view(bear).tapped);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let knife = named(&mut g, p.other(), "Thief's Knife", Zone::Battlefield);
        g.attach_equipment(g.object_ref(knife), g.object_ref(bear));
        let hit = DamageEvent {
            source: g.damage_source(bear),
            target: Target::Player(p.other()),
            amount: 1,
            combat: true,
        };
        g.player_damage_triggers(&[hit]);
        g.prepare_triggers();
        g.move_object(knife, Zone::Graveyard);
        g.move_object(bear, Zone::Graveyard);
        resolve(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn job_summoner_obeys_opponent_entry_tapped_replacement_for_ordinary_creature() {
    let mut g = scenario();
    let book = paid(&mut g, Player::P0, "Summoner's Grimoire");
    let hero = g.object_view(book).attached_to.unwrap();
    named(
        &mut g,
        Player::P1,
        "Authority of the Consuls",
        Zone::Battlefield,
    );
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    let before = g.object_ref(bear);
    attack_job(&mut g, Player::P0, hero.card);
    pass_pair(&mut g);
    act(&mut g, Action::PutHandCreature { object: before });
    assert!(g.object_view(bear).tapped);
    assert!(!g.state.attackers.iter().any(|a| a.card == bear));
    resolve(&mut g);
    assert_eq!(g.state.players[1].life, 21);
}

#[test]
fn job_summoner_exact_private_defender_and_late_capacity_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let book = paid(&mut g, p, "Summoner's Grimoire");
        let hero = g.object_view(book).attached_to.unwrap();
        let creature = named(&mut g, p, "Fear of Immobility", Zone::Hand);
        let old = g.object_ref(creature);
        let walkers: Vec<_> = Card::catalogue()
            .filter(|c| c.is_supported() && c.is_planeswalker())
            .take(2)
            .collect();
        let mut refs = Vec::new();
        for card in walkers {
            let id = spawn(&mut g, p.other(), card, Zone::Battlefield);
            g.state.objects[id as usize].counters.loyalty = 5;
            refs.push(g.object_ref(id));
        }
        attack_job(&mut g, p, hero.card);
        pass_pair(&mut g);
        let saved = g.snapshot();
        let actor = g.actor().unwrap();
        let revision = g.revision;
        assert_eq!(
            g.apply(actor.other(), Action::PutHandCreature { object: old }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), saved);
        let mut stale = old;
        stale.generation += 1;
        assert_eq!(
            g.apply(actor, Action::PutHandCreature { object: stale }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), saved);
        act(&mut g, Action::PutHandCreature { object: old });
        assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
        let actions = g.legal_actions();
        assert_eq!(
            actions
                .iter()
                .filter(|a| matches!(a, Action::ChooseHandDefender { .. }))
                .count(),
            3
        );
        let current = refs[1];
        let mut stale = current;
        stale.generation += 1;
        let snap = g.snapshot();
        assert_eq!(
            g.apply(
                actor,
                Action::ChooseHandDefender {
                    defender: CombatDefender::Planeswalker(stale)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), snap);
        g.state.next_stack_id = u64::MAX;
        let failure = g.snapshot();
        let revision = g.revision;
        let action = Action::ChooseHandDefender {
            defender: CombatDefender::Planeswalker(current),
        };
        assert!(matches!(
            g.apply(actor, action.clone()),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), failure);
        assert_eq!(g.revision, revision);
        assert_eq!(g.object_ref(creature), old);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
        g.restore(&snap).unwrap();
        act(&mut g, action);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
        assert_eq!(
            g.state
                .attackers
                .iter()
                .find(|a| a.card == creature)
                .unwrap()
                .defender,
            Some(CombatDefender::Planeswalker(current))
        );
    }
}
#[test]
fn job_ninja_discarded_x_is_zero_off_stack_and_late_life_overflow_rolls_back_only_choice() {
    for (name, expected) in [("Spectral Denial", 1), ("Forest", 0)] {
        let mut g = scenario();
        let blades = paid(&mut g, Player::P0, "Ninja's Blades");
        let hero = g.object_view(blades).attached_to.unwrap();
        let discard = named(&mut g, Player::P0, name, Zone::Hand);
        let old = g.object_ref(discard);
        g.player_damage_triggers(&[DamageEvent {
            source: g.damage_source(hero.card),
            target: Target::Player(Player::P1),
            amount: 2,
            combat: true,
        }]);
        g.prepare_triggers();
        pass_pair(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        if expected > 0 {
            g.state.players[1].life = i128::MIN;
            let saved = g.snapshot();
            let rev = g.revision;
            assert!(matches!(
                g.apply(Player::P0, Action::Discard { card: discard }),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision, rev);
            assert_eq!(g.object_ref(discard), old);
            assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
            g.state.players[1].life = 20;
        }
        act(&mut g, Action::Discard { card: discard });
        assert_eq!(g.state.players[1].life, 20 - expected);
        assert_eq!(g.state.objects[discard as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn job_independent_duplicate_grants_do_not_merge_and_third_draw_uses_global_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.draw(p);
        g.draw(p);
        let mut equipment = Vec::new();
        for owner in [p, p.other()] {
            let id = named(
                &mut g,
                owner,
                "Astrologian's Planisphere",
                Zone::Battlefield,
            );
            g.attach_equipment(g.object_ref(id), g.object_ref(bear));
            equipment.push(id);
        }
        assert_eq!(
            serde_json::to_value(g.object_view(bear).granted_subtypes).unwrap(),
            serde_json::json!(["Wizard"])
        );
        g.draw(p);
        assert_eq!(g.state.pending_triggers.len(), 2);
        let grants: Vec<_> = g
            .state
            .pending_triggers
            .iter()
            .map(|s| s.attachment_trigger.as_ref().unwrap().grantor.card)
            .collect();
        assert!(equipment.iter().all(|id| grants.contains(id)));
        g.move_object(equipment[0], Zone::Graveyard);
        g.move_object(equipment[1], Zone::Graveyard);
        resolve(&mut g);
        assert_eq!(g.object_view(bear).counters.plus_one, 2);
        g.draw(p);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn job_hero_entry_precedes_attachment_and_suppression_is_not_retroactive() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        named(&mut g, p, "Serra Redeemer", Zone::Battlefield);
        let sword = paid(&mut g, p, "Warrior's Sword");
        let hero = g.object_view(sword).attached_to.unwrap();
        assert_eq!(g.object_view(hero.card).counters.plus_one, 2);
        assert_eq!(
            g.stats(hero.card),
            Stats {
                power: 6,
                toughness: 5
            }
        );
        let mut g = scenario();
        let thrull = named(&mut g, p.other(), "Doorkeeper Thrull", Zone::Battlefield);
        let sword = paid(&mut g, p, "Warrior's Sword");
        assert_eq!(g.object_view(sword).attached_to, None);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card.is_token())
        );
        g.move_object(thrull, Zone::Graveyard);
        resolve(&mut g);
        assert_eq!(g.object_view(sword).attached_to, None);
        g.move_object(sword, Zone::Hand);
        g.move_object(sword, Zone::Battlefield);
        resolve(&mut g);
        assert!(g.object_view(sword).attached_to.is_some());
    }
}
#[test]
fn job_ninja_empty_library_and_no_discarded_card_loses_zero_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario();
        let blades = paid(&mut g, p, "Ninja's Blades");
        let hero = g.object_view(blades).attached_to.unwrap();
        for id in g.state.players[p.index()].library.clone() {
            g.move_object(id, Zone::Graveyard);
        }
        assert_eq!(g.hand_count(p), 0);
        g.player_damage_triggers(&[DamageEvent {
            source: g.damage_source(hero.card),
            target: Target::Player(p.other()),
            amount: 2,
            combat: true,
        }]);
        g.prepare_triggers();
        pass_pair(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(g.state.resolving.is_none());
        assert!(g.state.players[p.index()].attempted_empty_draw);
    }
}
