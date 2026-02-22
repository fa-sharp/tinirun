//! Application state

use std::{any::type_name, ops::Deref, sync::Arc};

use type_map::concurrent::TypeMap;

use crate::{config::AppConfig, runner::DockerRunner};

/// App state stored in the Axum router
#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub runner: DockerRunner,
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<TypeMap> for AppState {
    type Error = anyhow::Error;

    fn try_from(mut map: TypeMap) -> Result<Self, Self::Error> {
        Ok(Self(Arc::new(AppStateInner {
            config: extract(&mut map)?,
            runner: extract(&mut map)?,
        })))
    }
}

fn extract<T: 'static>(type_map: &mut TypeMap) -> anyhow::Result<T> {
    type_map
        .remove()
        .ok_or_else(|| anyhow::anyhow!("Type not found in state: {}", type_name::<T>()))
}
