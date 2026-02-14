use std::path::Path;

use crate::{
    BoxResult, PLUGIN_DEFINITION_FILE,
    plugins::{self},
    repo::PluginDefinitionProvider,
};
pub struct PathOverlay<'a> {
    root_path: &'a Path,
}

impl<'a> PathOverlay<'a> {
    pub fn new(root_path: &'a Path) -> Self {
        PathOverlay { root_path }
    }
}

impl<'a> PluginDefinitionProvider<'a> for PathOverlay<'a> {
    fn find_plugin_definition(&self, plugin: &String) -> BoxResult<plugins::Definition> {
        let definition_path = self.root_path.join(plugin).join(PLUGIN_DEFINITION_FILE);
        if !definition_path.exists() {
            return Err(format!(
                "Plugin definition file not found at {}",
                definition_path.display()
            )
            .into());
        }

        Err("...".into())
    }

    fn find_plugin_definitions(
        &self,
        _plugins: &Vec<String>,
    ) -> BoxResult<Vec<plugins::Definition>> {
        todo!()
    }
}
