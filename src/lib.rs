use bevy::{
    ecs::{
        component::ComponentId,
        query::{FilteredAccess, QueryData, QueryFilter, WorldQuery},
        system::{SystemParam, SystemParamItem, SystemState},
    },
    prelude::*,
    utils::intern::Interned,
};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct TemplatePlugin {
    template_fns: Vec<Box<dyn Fn(&mut App, &mut Vec<TemplateData>) + Send + Sync>>,
}

impl TemplatePlugin {
    pub fn with_template<T: Component, Marker>(
        mut self,
        label: T,
        template: impl IntoTemplate<Marker>,
    ) -> Self {
        let _ = label;
        let template = template.into_template();
        self.template_fns.push(Box::new(move |app, data| {
            template.build::<T>(app, data);
        }));
        self
    }
}

impl Plugin for TemplatePlugin {
    fn build(&self, app: &mut App) {
        let templates = self
            .template_fns
            .iter()
            .map(|f| {
                let mut data = Vec::new();
                f(app, &mut data);
                data
            })
            .flatten()
            .collect::<Vec<_>>();

        for template in &templates {
            template.system.init(app);
        }

        for template in templates.clone() {
            let mut after = Vec::new();
            for other in &templates {
                for read in &template.reads {
                    if app.world.components().get_id(other.output).unwrap() == *read {
                        after.push(other.system.system_set_any());
                    }
                }
            }

            template.system.add_any(app, after);
        }
    }
}

pub trait AnySystemParamFunction: Send + Sync + 'static {
    fn clone_any(&self) -> Box<dyn AnySystemParamFunction>;

    fn system_set_any(&self) -> Interned<dyn SystemSet>;

    fn init(&self, app: &mut App);

    fn add_any(&self, app: &mut App, after: Vec<Interned<dyn SystemSet>>);
}

struct SystemParamFunctionData<F, Marker> {
    f: F,
    _marker: PhantomData<Marker>,
}

impl<F: Clone, Marker> Clone for SystemParamFunctionData<F, Marker> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<Marker, F> AnySystemParamFunction for SystemParamFunctionData<F, Marker>
where
    F: SystemParamFunction<Marker, In = (), Out = ()> + Clone,
    Marker: Send + Sync + 'static,
{
    fn clone_any(&self) -> Box<dyn AnySystemParamFunction> {
        Box::new(self.clone())
    }

    fn system_set_any(&self) -> Interned<dyn SystemSet> {
        self.f.clone().into_system_set().intern()
    }

    fn init(&self, app: &mut App) {
        // TODO hack
        let state = SystemState::<F::Param>::new(&mut World::new());
        let meta = state.meta();
        F::Param::init_state(&mut app.world, &mut meta.clone());
    }

    fn add_any(&self, app: &mut App, after: Vec<Interned<dyn SystemSet>>) {
        let mut system = (apply_deferred, self.f.clone(), apply_deferred).chain();

        for set in after {
            system = system.after(set);
        }

        app.add_systems(Update, system);
    }
}

pub struct TemplateData {
    system: Box<dyn AnySystemParamFunction>,
    reads: Vec<ComponentId>,
    output: TypeId,
}

impl Clone for TemplateData {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone_any(),
            reads: self.reads.clone(),
            output: self.output.clone(),
        }
    }
}

pub trait Template: Send + Sync + 'static {
    fn build<T: Component>(&self, app: &mut App, data: &mut Vec<TemplateData>);
}

pub struct FunctionData<F, Marker> {
    f: Arc<Mutex<F>>,
    _marker: PhantomData<Marker>,
}

impl<F, C, Marker> Template for FunctionData<F, Marker>
where
    F: SystemParamFunction<Marker, In = Entity, Out = C>,
    for<'w, 's> SystemParamItem<'w, 's, F::Param>: IsChanged,
    C: Component,
    Marker: Send + Sync + 'static,
{
    fn build<T: Component>(&self, app: &mut App, data: &mut Vec<TemplateData>) {
        let f = self.f.clone();

        let system = move |mut params: ParamSet<(
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
        };

        data.push(TemplateData {
            system: Box::new(SystemParamFunctionData {
                f: system,
                _marker: PhantomData,
            }),
            reads: {
                let mut reads = Vec::new();
                SystemParamItem::<F::Param>::reads(app, &mut reads);
                reads
            },
            output: TypeId::of::<C>(),
        })
    }
}

impl<T1: Template, T2: Template> Template for (T1, T2) {
    fn build<T: Component>(&self, app: &mut App, data: &mut Vec<TemplateData>) {
        self.0.build::<T>(app, data);
        self.1.build::<T>(app, data);
    }
}

pub trait IntoTemplate<Marker> {
    type Data: Template;

    fn into_template(self) -> Self::Data;
}

impl<F, C, Marker> IntoTemplate<fn(Marker)> for F
where
    F: SystemParamFunction<Marker, In = Entity, Out = C>,
    for<'w, 's> SystemParamItem<'w, 's, F::Param>: IsChanged,
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
    fn build<T: Component>(&self, app: &mut App, data: &mut Vec<TemplateData>) {
        let f = self.f.clone();

        let system = move |mut params: ParamSet<(
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
        };

        data.push(TemplateData {
            system: Box::new(SystemParamFunctionData {
                f: system,
                _marker: PhantomData,
            }),
            reads: Vec::new(),
            output: TypeId::of::<C>(),
        })
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

    fn reads(app: &mut App, type_ids: &mut Vec<ComponentId>);

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

    fn reads(app: &mut App, type_ids: &mut Vec<ComponentId>) {
        let mut state = D::init_state(&mut app.world);
        let mut access = FilteredAccess::default();
        D::update_component_access(&mut state, &mut access);
        type_ids.extend(access.access().reads());
    }

    fn build<'w>(&'w self) -> Self::State<'w> {
        self.iter().map(|x| (*x).clone()).collect()
    }

    fn is_changed<'w>(&'w self, state: &'w mut Self::State<'w>) -> bool {
        // TODO
        let new_state = self.build();
        if new_state != *state {
            *state = new_state;
            true
        } else {
            false
        }
    }
}

impl<T: IsChanged> IsChanged for (T,) {
    type State<'w> = T::State<'w>where Self: 'w;

    fn reads(app: &mut App, type_ids: &mut Vec<ComponentId>) {
        T::reads(app, type_ids);
    }

    fn build<'w>(&'w self) -> Self::State<'w> {
        self.0.build()
    }

    fn is_changed<'w>(&'w self, state: &'w mut Self::State<'w>) -> bool {
        self.0.is_changed(state)
    }
}
