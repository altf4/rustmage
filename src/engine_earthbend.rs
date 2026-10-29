//! Source-independent delayed return for each exact earthbent land.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EarthbendReturnView {
    pub subject: ObjectRef,
    pub controller: Player,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EarthbendReturn {
    subject: ObjectRef,
    pub(super) item: SpellView,
}
impl Game {
    pub(super) fn animate_earthbend_targets(&mut self, item: &SpellView) {
        for id in self.keyword_subjects(item, Subject::Targets) {
            let spec = crate::cards::AnimationSpec {
                artifact: false,
                creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                base: Some((0, 0)),
                keywords: Keywords::HASTE,
                duration: crate::cards::AnimationDuration::Permanent,
            };
            self.change_object_form(
                self.object_ref(id),
                spec,
                characteristic_layers::AnimationTypeMode::AddCreatureAndLand,
            );
        }
    }
    pub(super) fn register_earthbend_return(&mut self, item: &SpellView) {
        let subjects = self.keyword_subjects(item, Subject::Targets);
        for id in subjects {
            let row = EarthbendReturn {
                subject: self.object_ref(id),
                item: item.clone(),
            };
            self.state
                .earthbend_returns
                .get_or_insert_default()
                .push(row);
        }
    }
    pub(super) fn queue_earthbend_returns(&mut self, events: &[ZoneChange]) {
        let Some(rows) = self.state.earthbend_returns.take() else {
            return;
        };
        let mut retained = Vec::new();
        for row in *rows {
            let Some(event) = events
                .iter()
                .find(|e| e.moved && e.before.reference == row.subject)
            else {
                retained.push(row);
                continue;
            };
            if event.before.zone != Zone::Battlefield
                || !matches!(event.destination, Zone::Graveyard | Zone::Exile)
            {
                continue;
            }
            let mut item = row.item;
            item.id = self.next_stack_id();
            item.kind = StackKind::DelayedReturn;
            item.return_link = Some(Box::new(ReturnLinkView {
                exile_after: false,
                additional_returning: None,
                object: event.after,
                zone: event.destination,
                controller: event.before.owner,
                counter: None,
                tapped: true,
            }));
            item.targets.clear();
            item.mode = None;
            self.state.pending_triggers.push(item);
        }
        if !retained.is_empty() {
            self.state.earthbend_returns = Some(Box::new(retained));
        }
    }
    pub(super) fn earthbend_views(&self) -> Vec<EarthbendReturnView> {
        self.state
            .earthbend_returns
            .as_ref()
            .map_or_else(Vec::new, |rows| {
                rows.iter()
                    .map(|r| EarthbendReturnView {
                        subject: r.subject,
                        controller: r.item.controller,
                    })
                    .collect()
            })
    }
    pub(super) fn earthbend_bytes(&self) -> usize {
        self.state.earthbend_returns.as_ref().map_or(0, |rows| {
            std::mem::size_of::<Vec<EarthbendReturn>>()
                + rows
                    .iter()
                    .map(|row| std::mem::size_of::<EarthbendReturn>() + row.item.owned_bytes())
                    .sum::<usize>()
        })
    }
}
