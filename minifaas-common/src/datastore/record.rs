pub use crate::types::*;
use crate::{runtime::FunctionCode, triggers::Trigger};
use serde::{Deserialize, Serialize};
use std::fmt;

use uuid::Uuid;
///
/// A DB record to store a user function and the corresponding trigger/env id.
///
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct UserFunctionRecord {
    func: UserFunctionDeclaration,
    pub environment_id: Uuid,
}

impl UserFunctionRecord {
    pub fn new(func: UserFunctionDeclaration, env_id: Uuid) -> Self {
        UserFunctionRecord {
            func,
            environment_id: env_id,
        }
    }

    pub fn language(&self) -> &ProgrammingLanguage {
        &self.func.code.language
    }

    pub fn code(&self) -> &FunctionCode {
        &self.func.code
    }

    pub fn name(&self) -> &String {
        &self.func.name
    }

    pub fn trigger(&self) -> &Trigger {
        &self.func.trigger
    }

    pub fn update_function(
        &mut self,
        new_func: UserFunctionDeclaration,
    ) -> UserFunctionDeclaration {
        std::mem::replace(&mut self.func, new_func)
    }
}

impl fmt::Display for UserFunctionRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Function(Name: {}, Language: {}, Trigger: {}, Environment: {})",
            self.name(),
            self.language(),
            self.trigger(),
            self.environment_id
        )
    }
}

impl From<UserFunctionDeclaration> for UserFunctionRecord {
    fn from(f: UserFunctionDeclaration) -> Self {
        UserFunctionRecord::new(f, Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use minifaas_test::get_empty_tmp_dir;
}
