//! A linked exile and a separately owned, source-incarnation-bound copy effect.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LinkedCopyCapture {
    pub source: ObjectRef,
    pub recipe: CopyRecipe,
}
impl Game {
    pub(super) fn exile_and_capture_attached_copy(&mut self, item: &SpellView) {
        self.state.resolving.as_mut().unwrap().linked_copy = None;
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.object.card)
            || self.object_ref(source.object.card) != source.object
        {
            return;
        }
        if self.state.objects[source.object.card as usize]
            .attached_to
            .is_none()
        {
            return;
        }
        let Some(target) = item.targets.iter().find_map(|bound| {
            (self.same_target_object(*bound))
                .then_some(bound)
                .and_then(|bound| bound.object)
        }) else {
            return;
        };
        let recipe = self.copiable_recipe(target);
        let events = self.move_objects_raw(&[target.card], Zone::Exile);
        if let Some(event) = events
            .iter()
            .find(|e| e.moved && e.destination == Zone::Exile)
        {
            self.state.exile_links.push(ExileLink {
                source: source.object,
                exiled: event.after,
                return_zone: event.before.zone,
            });
            self.state.resolving.as_mut().unwrap().linked_copy =
                Some(Box::new(LinkedCopyCapture {
                    source: source.object,
                    recipe,
                }));
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn apply_captured_attached_copy(&mut self) {
        let Some(capture) = self.state.resolving.as_mut().unwrap().linked_copy.take() else {
            return;
        };
        if !self.on_battlefield(capture.source.card)
            || self.object_ref(capture.source.card) != capture.source
        {
            return;
        }
        let Some(recipient) = self.state.objects[capture.source.card as usize].attached_to else {
            return;
        };
        self.apply_temporary_copy(
            recipient,
            capture.recipe,
            CopyDeadline::WhileSource {
                source: capture.source,
            },
        );
    }
}
