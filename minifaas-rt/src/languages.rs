use crate::ext::toolchain::ToolchainSetup;
use crate::ActiveToolchain;
use async_std::sync::Arc;
use minifaas_common::ProgrammingLanguage;
use std::collections::HashMap;

///
/// A mapping of a toolchains to a languages in two separate stores.
/// Keeps build toolchains and exec toolchains separate.
///
#[derive(Debug, Default, Clone)]
pub struct ToolchainMap<T: ToolchainSetup + Default + Clone> {
    builders: HashMap<ProgrammingLanguage, T>,
    execs: HashMap<ProgrammingLanguage, Arc<ActiveToolchain>>,
}

impl<T: ToolchainSetup + Default + Clone> ToolchainMap<T> {
    pub fn new(
        toolchains: Vec<(ProgrammingLanguage, T)>,
        executors: Vec<(ProgrammingLanguage, Arc<ActiveToolchain>)>,
    ) -> Self {
        ToolchainMap {
            builders: toolchains.into_iter().collect(),
            execs: executors.into_iter().collect(),
        }
    }

    pub fn select_for(&self, lang: &ProgrammingLanguage) -> Option<&T> {
        self.builders.get(lang)
    }

    pub fn select_for_mut(&mut self, lang: &ProgrammingLanguage) -> Option<&mut T> {
        self.builders.get_mut(lang)
    }

    pub fn select_executor(&self, lang: &ProgrammingLanguage) -> Option<&Arc<ActiveToolchain>> {
        self.execs.get(lang)
    }

    pub fn len_toolchain_setups(&self) -> usize {
        self.builders.len()
    }

    pub fn len_toolchain_executors(&self) -> usize {
        self.execs.len()
    }
}
