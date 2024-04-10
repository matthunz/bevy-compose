use crate::Compose;
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn remember<C: Compose>(input: impl Hash, content: C) -> Remember<C> {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    Remember {
        hash: hasher.finish(),
        content,
    }
}

pub struct Remember<C> {
    hash: u64,
    content: C,
}

impl<C> Compose for Remember<C>
where
    C: Compose,
{
    type State = (u64, C::State);

    fn build(
        &mut self,
        world: &mut bevy::prelude::World,
        children: &mut Vec<bevy::prelude::Entity>,
    ) -> Self::State {
        let content_state = self.content.build(world, children);
        (self.hash, content_state)
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut bevy::prelude::World,
        children: &mut Vec<bevy::prelude::Entity>,
    ) {
        if self.hash != state.0 {
            self.content
                .rebuild(&mut target.content, &mut state.1, world, children);

            state.0 = self.hash;
        }
    }
}
