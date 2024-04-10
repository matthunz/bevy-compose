use bevy::prelude::*;

mod flex;
pub use self::flex::{flex, handler_system, Flex};

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
        }
    }
}

impl<C1: Compose, C2: Compose, C3: Compose> Compose for (C1, C2, C3) {
    type State = (C1::State, C2::State, C3::State);

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        (
            self.0.build(world, children),
            self.1.build(world, children),
            self.2.build(world, children),
        )
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        self.0.rebuild(&mut target.0, &mut state.0, world, children);
        self.1.rebuild(&mut target.1, &mut state.1, world, children);
        self.2.rebuild(&mut target.2, &mut state.2, world, children);
    }
}
