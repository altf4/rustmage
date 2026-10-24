//! Independent printed-outcome scenarios for SOS multicolor spells and graveyard events.
use super::*;
use super::big_helpers::checked;
fn pause(g: &mut Game) {
    for _ in 0..24 {
        if g.state.pending != Pending::Priority { return; }
        act(g,Action::Pass);
    }
    panic!("expected choice");
}
fn mode(g: &mut Game, card: CardId, mode: u16, targets: &[Target]) {
    act(g,Action::AnnounceCast {card}); act(g,Action::ChooseMode{mode});
    for &target in targets { act(g,Action::ChooseTarget{target}); }
    if g.legal_actions().contains(&Action::FinishTargets) { act(g,Action::FinishTargets); }
    pay(g);
}
#[test]
fn sos_growth_curve_control_replacement_and_exact_target() {
    for p in [Player::P0,Player::P1] {
        let mut g=scenario(p);
        let own=put(&mut g,p,"Bear Cub",Zone::Battlefield);
        let enemy=put(&mut g,p.other(),"Bear Cub",Zone::Battlefield);
        let spell=put(&mut g,p,"Growth Curve",Zone::Hand);
        put(&mut g,p,"Doubling Season",Zone::Battlefield);
        g.state.objects[own as usize].counters.stun=2;
        checked(&mut g,Action::AnnounceCast{card:spell});
        reject(&mut g,Action::ChooseTarget{target:Target::Card(enemy)});
        checked(&mut g,Action::ChooseTarget{target:Target::Card(own)});
        pay(&mut g); settle(&mut g);
        assert_eq!(g.counter_count(own,CounterKind::PlusOne),6);
        assert_eq!(g.counter_count(own,CounterKind::Stun),2);
        assert_eq!(g.counter_count(enemy,CounterKind::PlusOne),0);
        let spell=put(&mut g,p,"Growth Curve",Zone::Hand);
        announce(&mut g,spell,&[Target::Card(own)]);
        g.move_object(own,Zone::Hand); g.move_object(own,Zone::Battlefield);
        settle(&mut g); assert_eq!(g.counter_count(own,CounterKind::PlusOne),0);
    }
}
#[test]
fn sos_silverquill_charm_all_modes_power_legality_and_life() {
    for p in [Player::P0,Player::P1] {
        let mut g=scenario(p);
        let bear=put(&mut g,p.other(),"Bear Cub",Zone::Battlefield);
        let giant=put(&mut g,p,"Hill Giant",Zone::Battlefield);
        let small=put(&mut g,p,"Savannah Lions",Zone::Battlefield);
        let spell=put(&mut g,p,"Silverquill Charm",Zone::Hand);
        mode(&mut g,spell,0,&[Target::Card(bear)]); settle(&mut g);
        assert_eq!(g.counter_count(bear,CounterKind::PlusOne),2);
        let spell=put(&mut g,p,"Silverquill Charm",Zone::Hand);
        act(&mut g,Action::AnnounceCast{card:spell}); act(&mut g,Action::ChooseMode{mode:1});
        reject(&mut g,Action::ChooseTarget{target:Target::Card(giant)});
        reject(&mut g,Action::ChooseTarget{target:Target::Card(bear)});
        act(&mut g,Action::ChooseTarget{target:Target::Card(small)}); pay(&mut g); settle(&mut g);
        assert_eq!(g.object_view(small).zone,Zone::Exile);
        let spell=put(&mut g,p,"Silverquill Charm",Zone::Hand);
        mode(&mut g,spell,2,&[]); settle(&mut g);
        assert_eq!(g.state.players[p.index()].life,23); assert_eq!(g.state.players[p.other().index()].life,17);
    }
}
#[test]
fn sos_prismari_charm_all_modes_and_distinct_damage_targets() {
    for p in [Player::P0,Player::P1] {
        let mut g=scenario(p);
        let top=put(&mut g,p,"Island",Zone::Library);
        let spell=put(&mut g,p,"Prismari Charm",Zone::Hand);
        let hand=g.hand_count(p); mode(&mut g,spell,0,&[]); pause(&mut g);
        assert!(g.observe(p.other()).library_choice.is_none());
        act(&mut g,Action::ArrangeLibrary{card:top,destination:LibraryDestination::Graveyard});
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ArrangeLibrary{destination:LibraryDestination::Top,..})).unwrap();act(&mut g,a);settle(&mut g);
        assert_eq!(g.object_view(top).zone,Zone::Graveyard);assert_eq!(g.hand_count(p),hand);
        let bear=put(&mut g,p.other(),"Bear Cub",Zone::Battlefield);
        let spell=put(&mut g,p,"Prismari Charm",Zone::Hand);
        act(&mut g,Action::AnnounceCast{card:spell});act(&mut g,Action::ChooseMode{mode:1});
        act(&mut g,Action::ChooseTarget{target:Target::Card(bear)});
        reject(&mut g,Action::ChooseTarget{target:Target::Card(bear)});
        act(&mut g,Action::ChooseTarget{target:Target::Player(p.other())});pay(&mut g);settle(&mut g);
        assert_eq!(g.object_view(bear).damage,1);assert_eq!(g.state.players[p.other().index()].life,19);
        let spell=put(&mut g,p,"Prismari Charm",Zone::Hand);
        mode(&mut g,spell,2,&[Target::Card(bear)]);settle(&mut g);assert_eq!(g.object_view(bear).zone,Zone::Hand);
    }
}
#[test]
fn sos_witherbloom_charm_sacrifice_choice_life_and_mana_value() {
    for p in [Player::P0,Player::P1] {
        for accept in [false,true] {
            let mut g=scenario(p);let land=put(&mut g,p,"Forest",Zone::Battlefield);
            let spell=put(&mut g,p,"Witherbloom Charm",Zone::Hand);let hand=g.hand_count(p);
            mode(&mut g,spell,0,&[]);pause(&mut g);
            if accept { let object=g.object_ref(land);act(&mut g,Action::Sacrifice{object}); } else { act(&mut g,Action::DeclineEffectPayment); }
            settle(&mut g);assert_eq!(g.hand_count(p),hand-1+if accept{2}else{0});
            assert_eq!(g.object_view(land).zone,if accept{Zone::Graveyard}else{Zone::Battlefield});
        }
        let mut g=scenario(p);let spell=put(&mut g,p,"Witherbloom Charm",Zone::Hand);mode(&mut g,spell,1,&[]);settle(&mut g);assert_eq!(g.state.players[p.index()].life,25);
        let land=put(&mut g,p.other(),"Forest",Zone::Battlefield);let big=put(&mut g,p.other(),"Hill Giant",Zone::Battlefield);let small=put(&mut g,p.other(),"Bear Cub",Zone::Battlefield);
        let spell=put(&mut g,p,"Witherbloom Charm",Zone::Hand);act(&mut g,Action::AnnounceCast{card:spell});act(&mut g,Action::ChooseMode{mode:2});
        for id in [land,big] {reject(&mut g,Action::ChooseTarget{target:Target::Card(id)});}
        act(&mut g,Action::ChooseTarget{target:Target::Card(small)});pay(&mut g);settle(&mut g);assert_eq!(g.object_view(small).zone,Zone::Graveyard);
    }
}
#[test]
fn sos_fractal_mascot_trigger_is_opponent_only_and_survives_departure() {
    for p in [Player::P0,Player::P1] {
        let mut g=scenario(p);let own=put(&mut g,p,"Bear Cub",Zone::Battlefield);let foe=put(&mut g,p.other(),"Hill Giant",Zone::Battlefield);
        let card=put(&mut g,p,"Fractal Mascot",Zone::Hand);cast(&mut g,card,None);pause(&mut g);
        reject(&mut g,Action::ChooseTarget{target:Target::Card(own)});
        act(&mut g,Action::ChooseTarget{target:Target::Card(foe)});
        g.move_object(card,Zone::Graveyard);settle(&mut g);
        assert!(g.object_view(foe).tapped);assert_eq!(g.counter_count(foe,CounterKind::Stun),1);
    }
}
#[test]
fn sos_stress_dream_optional_target_private_selection_and_fizzle() {
    for p in [Player::P0,Player::P1] {
        for targeted in [false,true] {
            let mut g=scenario(p);let foe=put(&mut g,p.other(),"Hill Giant",Zone::Battlefield);
            let top=put(&mut g,p,"Island",Zone::Library);let spell=put(&mut g,p,"Stress Dream",Zone::Hand);
            announce(&mut g,spell,&if targeted{vec![Target::Card(foe)]}else{vec![]});pause(&mut g);
            act(&mut g,Action::SelectLibrary{card:top});act(&mut g,Action::FinishLibrarySelection);
            if let Some(action)=g.legal_actions().into_iter().find(|a|matches!(a,Action::OrderLibraryBottom{..})) {act(&mut g,action);}
            settle(&mut g);
            assert_eq!(g.object_view(top).zone,Zone::Hand);assert_eq!(g.object_view(foe).zone,if targeted{Zone::Graveyard}else{Zone::Battlefield});
        }
        let mut g=scenario(p);let foe=put(&mut g,p.other(),"Hill Giant",Zone::Battlefield);let spell=put(&mut g,p,"Stress Dream",Zone::Hand);
        let count=g.state.players[p.index()].library.len();announce(&mut g,spell,&[Target::Card(foe)]);g.move_object(foe,Zone::Hand);settle(&mut g);assert_eq!(g.state.players[p.index()].library.len(),count);
    }
}
#[test]
fn sos_stirring_honormancer_counts_on_resolution_and_mills_remainder() {
    for p in [Player::P0,Player::P1] {
        let mut g=scenario(p);let other=put(&mut g,p,"Bear Cub",Zone::Battlefield);
        let top=put(&mut g,p,"Island",Zone::Library);let card=put(&mut g,p,"Stirring Honormancer",Zone::Hand);cast(&mut g,card,None);
        pause(&mut g);act(&mut g,Action::SelectLibrary{card:top});act(&mut g,Action::FinishLibrarySelection);settle(&mut g);
        assert_eq!(g.object_view(top).zone,Zone::Hand);assert_eq!(g.state.players[p.index()].library.len(),7);assert_eq!(g.object_view(other).zone,Zone::Battlefield);
    }
}
#[test]
fn sos_graveyard_departures_batch_ownership_tokens_and_owned_restore() {
    for p in [Player::P0,Player::P1] {
        let mut g=scenario(p);let mascot=put(&mut g,p,"Spirit Mascot",Zone::Battlefield);
        let a=put(&mut g,p,"Shock",Zone::Graveyard);let b=put(&mut g,p,"Island",Zone::Graveyard);
        let events=g.move_objects_raw(&[a,b],Zone::Exile);g.finish_zone_events(&events,&[]);g.prepare_triggers();
        assert_eq!(g.state.stack.len()+g.state.pending_triggers.len()+g.state.ordering_triggers.len(),1);
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Pass|Action::PutTrigger{..})).unwrap();checked(&mut g,action);settle(&mut g);
        assert_eq!(g.counter_count(mascot,CounterKind::PlusOne),1);assert!(g.state.players[p.index()].card_left_graveyard_this_turn);
        let foreign=put(&mut g,p.other(),"Island",Zone::Graveyard);g.move_object(foreign,Zone::Hand);settle(&mut g);assert_eq!(g.counter_count(mascot,CounterKind::PlusOne),1);
        let new=put(&mut g,p,"Spirit Mascot",Zone::Graveyard);g.move_object(new,Zone::Battlefield);g.prepare_triggers();settle(&mut g);
        assert_eq!(g.counter_count(new,CounterKind::PlusOne),0);assert_eq!(g.counter_count(mascot,CounterKind::PlusOne),2);
        g.finish_cleanup();assert!(!g.state.players[p.index()].card_left_graveyard_this_turn);
        let token=put(&mut g,p,"@token/sos-pest",Zone::Battlefield);g.move_object(token,Zone::Graveyard);g.move_object(token,Zone::Exile);settle(&mut g);
        assert!(!g.state.players[p.index()].card_left_graveyard_this_turn);
    }
}
