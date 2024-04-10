use bevy::prelude::*;

mod flex;
pub use self::flex::{flex, handler_system, Flex};

mod lazy;
pub use self::lazy::{lazy, Lazy, LazyState};

mod remember;
pub use self::remember::{remember, Remember};

pub trait Compose: Send + Sync + 'static {
    type State: Send + Sync + 'static;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State;

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    );

    fn remove(&mut self, _state: &mut Self::State) {}
}

impl Compose for () {
    type State = ();

    fn build(&mut self, _world: &mut World, _children: &mut Vec<Entity>) -> Self::State {}

    fn rebuild(
        &mut self,
        _target: &mut Self,
        _state: &mut Self::State,
        _world: &mut World,
        _children: &mut Vec<Entity>,
    ) {
    }
}

impl Compose for &'static str {
    type State = Entity;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        let entity = world.spawn(TextBundle::from_section(
            self.to_owned(),
            Default::default(),
        ));
        let id = entity.id();
        children.push(id);
        id
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        children.push(*state);

        if self != target {
            world.get_mut::<Text>(*state).unwrap().sections[0] =
                TextSection::new(self.to_owned(), TextStyle::default());
            *target = self;
        }
    }
}

impl Compose for String {
    type State = Entity;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        let entity = world.spawn(TextBundle::from_section(self.clone(), Default::default()));
        let id = entity.id();
        children.push(id);
        id
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        children.push(*state);

        if self != target {
            world.get_mut::<Text>(*state).unwrap().sections[0] =
                TextSection::new(self.clone(), TextStyle::default());
            *target = self.clone();
        }
    }
}

impl<C: Compose> Compose for Option<C> {
    type State = Option<C::State>;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        self.as_mut().map(|compose| compose.build(world, children))
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        if let Some(mut compose) = self.take() {
            if let Some(target) = target {
                let state = state.as_mut().unwrap();
                compose.rebuild(target, state, world, children);
            } else {
                let new_state = compose.build(world, children);
                *state = Some(new_state);
                *target = Some(compose);
            }
        } else if let Some(mut target) = target.take() {
            let state = state.as_mut().unwrap();
            target.remove(state);
        }
    }
}

macro_rules! impl_compose_for_tuple {
    ($($t:tt: $idx:tt),*) => {
        impl<$($t:Compose),*> Compose for ($($t),*) {
            type State = ($($t::State),*);

            fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
                ( $(self.$idx.build(world, children)),* )
            }

            fn rebuild(
                &mut self,
                target: &mut Self,
                state: &mut Self::State,
                world: &mut World,
                children: &mut Vec<Entity>,
            ) {
                $( self.$idx.rebuild(&mut target.$idx, &mut state.$idx, world, children) );*
            }
        }
    };
}

impl_compose_for_tuple!(C1: 0, C2: 1);
impl_compose_for_tuple!(C1: 0, C2: 1, C3: 2);
impl_compose_for_tuple!(C1: 0, C2: 1, C3: 2, C4: 3);
impl_compose_for_tuple!(C1: 0, C2: 1, C3: 2, C4: 3, C5: 4);
impl_compose_for_tuple!(C1: 0, C2: 1, C3: 2, C4: 3, C5: 4, C6: 5);
impl_compose_for_tuple!(C1: 0, C2: 1, C3: 2, C4: 3, C5: 4, C6: 5, C7: 6);
impl_compose_for_tuple!(C1: 0, C2: 1, C3: 2, C4: 3, C5: 4, C6: 5, C7: 6, C8: 7);
