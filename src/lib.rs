use bevy::{ecs::schedule::SystemConfigs, prelude::*};
use std::{
    marker::PhantomData,
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
        (self.0.build::<T>(), apply_deferred, self.1.build::<T>()).chain()
    }
}

pub trait IntoTemplate<Marker> {
    type Data: Template;

    fn into_template(self) -> Self::Data;
}

impl<F, C, Marker> IntoTemplate<fn(Marker)> for F
where
    F: SystemParamFunction<Marker, In = Entity, Out = C>,
    F::Param: 'static,
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
