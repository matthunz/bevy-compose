use bevy::prelude::*;
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
        template: impl IntoTemplateData<Marker>,
    ) -> Self {
        let _ = label;
        let data = template.into_template_data();
        self.fns.push(Box::new(move |app| {
            data.build::<T>(app);
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
    fn build<T: Component>(&self, app: &mut App);
}

pub struct FunctionData<F, Marker> {
    f: Arc<Mutex<F>>,
    _marker: PhantomData<Marker>,
}

impl<F, C, Marker> Template for FunctionData<F, Marker>
where
    F: SystemParamFunction<Marker, In = (), Out = C>,
    F::Param: 'static,
    C: Component,
    Marker: Send + Sync + 'static,
{
    fn build<T: Component>(&self, app: &mut App) {
        let f = self.f.clone();
        app.add_systems(
            Update,
            move |mut params: ParamSet<(
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
            },
        );
    }
}

pub trait IntoTemplateData<Marker> {
    type Data: Template;

    fn into_template_data(self) -> Self::Data;
}

impl<F, C, Marker> IntoTemplateData<Marker> for F
where
    F: SystemParamFunction<Marker, In = (), Out = C>,
    F::Param: 'static,
    C: Component,
    Marker: Send + Sync + 'static,
{
    type Data = FunctionData<F, Marker>;

    fn into_template_data(self) -> Self::Data {
        FunctionData {
            f: Arc::new(Mutex::new(self)),
            _marker: PhantomData,
        }
    }
}
