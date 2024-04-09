use bevy::ecs::system::{ParamSet, SystemParam};

pub trait Compose {
    fn compose(&mut self);

    fn recompose(&mut self, target: &mut Self);
}

impl Compose for String {
    fn compose(&mut self) {
        dbg!(self);
    }

    fn recompose(&mut self, target: &mut Self) {
        dbg!(self);
    }
}

pub fn composer<I, C>(
    mut compose_fn: impl FnMut(I) -> C + FnMut(I::Item<'_, '_>) -> C,
) -> impl FnMut(ParamSet<(I,)>)
where
    I: SystemParam,
    C: Compose + 'static,
{
    let mut compose_cell = None;
    move |mut params| {
        let param = params.p0();
        let mut compose = compose_fn(param);
        if let Some(ref mut target) = compose_cell {
            compose.recompose(target)
        } else {
            compose.compose();
            compose_cell = Some(compose);
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
