use bevy::{
    ecs::{
        query::{QueryData, QueryFilter, WorldQuery},
        schedule::SystemConfigs,
        system::SystemParamItem,
    },
    prelude::*,
};
use std::{
    any::Any,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct TemplatePlugin {
    fns: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl TemplatePlugin {
    pub fn with_template<T: Component, Marker>(
        mut self,
        label: T,
        template: impl IntoTemplate<Marker>,
    ) -> Self {
        let _ = label;
        let data = template.into_template();
        self.fns.push(Box::new(move |app| {
            app.add_systems(Update, data.build::<T>());
        }));
        self
    }
}

impl Plugin for TemplatePlugin {
    fn build(&self, app: &mut App) {
        for f in &self.fns {
            f(app);
        }
    }
}

pub trait Template: Send + Sync + 'static {
    fn build<T: Component>(&self) -> SystemConfigs;
}

pub struct FunctionData<F, Marker> {
    f: Arc<Mutex<F>>,
    _marker: PhantomData<Marker>,
}

impl<F, C, Marker> Template for FunctionData<F, Marker>
where
    F: SystemParamFunction<Marker, In = Entity, Out = C>,
    for<'w, 's> SystemParamItem<'w, 's, F::Param>: IsChanged ,
    C: Component,
    Marker: Send + Sync + 'static,
{
    fn build<T: Component>(&self) -> SystemConfigs {
        let f = self.f.clone();

        (move |mut params: ParamSet<(
            Commands,
            Query<(Entity, Option<&mut C>), With<T>>,
            F::Param,
        )>,
               mut cell: Local<Option<Box<dyn Any + Send>>>| {
            if let Some(state) = &mut *cell {
                if params.p2().is_changed((**state).downcast_mut().unwrap()) {
                    *cell = Some(Box::new(params.p2().build()));
                }
            } else {
                *cell = Some(Box::new(params.p2().build()));
            }

            let entities: Vec<_> = params.p1().iter().map(|(entity, _)| entity).collect();
            for entity in entities {
                let out = f.lock().unwrap().run(entity, params.p2());
                if let Some(mut x) = params.p1().get_mut(entity).unwrap().1 {
                    *x = out;
                } else {
                    params.p0().entity(entity).insert(out);
                }
            }
        })
        .into_configs()
    }
}

impl<T1: Template, T2: Template> Template for (T1, T2) {
    fn build<T: Component>(&self) -> SystemConfigs {
        (self.0.build::<T>(), apply_deferred, self.1.build::<T>()).into_configs()
    }
}

pub trait IntoTemplate<Marker> {
    type Data: Template;

    fn into_template(self) -> Self::Data;
}

impl<F, C, Marker> IntoTemplate<fn(Marker)> for F
where
    F: SystemParamFunction<Marker, In = Entity, Out = C>,
    for<'w, 's> SystemParamItem<'w, 's, F::Param>: IsChanged ,
    C: Component,
    Marker: Send + Sync + 'static,
{
    type Data = FunctionData<F, Marker>;

    fn into_template(self) -> Self::Data {
        FunctionData {
            f: Arc::new(Mutex::new(self)),
            _marker: PhantomData,
        }
    }
}

pub struct Empty<Marker>(Marker);

pub struct EmptyFunctionData<F, Marker> {
    f: Arc<Mutex<F>>,
    _marker: PhantomData<Marker>,
}

impl<F, C, Marker> Template for EmptyFunctionData<F, Marker>
where
    F: SystemParamFunction<Marker, In = (), Out = C>,
    F::Param: 'static,
    C: Component,
    Marker: Send + Sync + 'static,
{
    fn build<T: Component>(&self) -> SystemConfigs {
        let f = self.f.clone();

        (move |mut params: ParamSet<(
            Commands,
            Query<(Entity, Option<&mut C>), With<T>>,
            F::Param,
        )>| {
            let entities: Vec<_> = params.p1().iter().map(|(entity, _)| entity).collect();
            for entity in entities {
                let out = f.lock().unwrap().run((), params.p2());
                if let Some(mut x) = params.p1().get_mut(entity).unwrap().1 {
                    *x = out;
                } else {
                    params.p0().entity(entity).insert(out);
                }
            }
        })
        .into_configs()
    }
}

impl<F, C, Marker> IntoTemplate<Empty<Marker>> for F
where
    F: SystemParamFunction<Marker, In = (), Out = C>,
    F::Param: 'static,
    C: Component,
    Marker: Send + Sync + 'static,
{
    type Data = EmptyFunctionData<F, Marker>;

    fn into_template(self) -> Self::Data {
        EmptyFunctionData {
            f: Arc::new(Mutex::new(self)),
            _marker: PhantomData,
        }
    }
}

impl<T1, T2, Marker1, Marker2> IntoTemplate<(Marker1, Marker2)> for (T1, T2)
where
    T1: IntoTemplate<Marker1>,
    T2: IntoTemplate<Marker2>,
{
    type Data = (T1::Data, T2::Data);

    fn into_template(self) -> Self::Data {
        (self.0.into_template(), self.1.into_template())
    }
}

pub trait IsChanged {
    type State<'w>: Send + 'static
    where
        Self: 'w;

    fn build<'w>(&'w self) -> Self::State<'w>;

    fn is_changed<'w>(&'w self, state: &'w mut Self::State<'w>) -> bool;
}

impl<D, F> IsChanged for Query<'_, '_, D, F>
where
    D: QueryData,
    F: QueryFilter,
    for<'w> <<D as QueryData>::ReadOnly as WorldQuery>::Item<'w>: Deref,
    for<'w> <<<D as QueryData>::ReadOnly as WorldQuery>::Item<'w> as Deref>::Target:
        Clone + PartialEq + Send + 'static,
{
    type State<'w> =  Vec<<<<D as QueryData>::ReadOnly as WorldQuery>::Item<'w> as Deref>::Target>where Self: 'w;

    fn build<'w>(&'w self) -> Self::State<'w> {
        self.iter().map(|x| (*x).clone()).collect()
    }

    fn is_changed<'w>(&'w self, state: &'w mut Self::State<'w>) -> bool {
        // TODO
        let new_state = self.build();
        dbg!(if new_state != *state {
            *state = new_state;
            true
        } else {
            false
        })
    }
}

impl<T: IsChanged> IsChanged for (T,) {
    type State<'w> = T::State<'w>where Self: 'w;

    fn build<'w>(&'w self) -> Self::State<'w> {
        self.0.build()
    }

    fn is_changed<'w>(&'w self, state: &'w mut Self::State<'w>) -> bool {
        self.0.is_changed(state)
    }
}

fn react<Marker, S>(
    mut system: S,
) -> impl FnMut(ParamSet<(S::Param,)>, Local<Option<Box<dyn Any + Send>>>)
where
    S: SystemParamFunction<Marker, In = ()>,
    for<'w, 's> SystemParamItem<'w, 's, S::Param>: IsChanged,
{
    move |mut params, mut cell| {
        if let Some(state) = &mut *cell {
            if params.p0().is_changed((**state).downcast_mut().unwrap()) {
                *cell = Some(Box::new(params.p0().build()));
                system.run((), params.p0());
            }
        } else {
            *cell = Some(Box::new(params.p0().build()));
            system.run((), params.p0());
        }
    }
}
