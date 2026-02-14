use crate::{BoxResult, plugins, repo::PluginDefinitionProvider};

/// PluginOverlays is a collection of PluginDefinitionProviders, prioritized by order of insertion,
/// last to first..
pub struct PluginOverlays<'a> {
    overlays: Vec<Box<dyn PluginDefinitionProvider<'a>>>,
}

impl<'a> Default for PluginOverlays<'a> {
    fn default() -> Self {
        PluginOverlays {
            overlays: Vec::new(),
        }
    }
}

impl<'a> PluginOverlays<'a> {
    pub fn add_overlay(&mut self, overlay: Box<dyn PluginDefinitionProvider<'a>>) {
        // Inserted at start so that its given first over any previous overlays.
        self.overlays.insert(0, overlay);
    }
}

impl<'a> PluginDefinitionProvider<'a> for PluginOverlays<'a> {
    fn find_plugin_definitions(
        &self,
        _plugins: &Vec<String>,
    ) -> BoxResult<Vec<plugins::Definition>> {
        let mut definitions = Vec::new();
        for overlay in &self.overlays {
            definitions.extend(overlay.find_plugin_definitions(_plugins)?);
        }
        Ok(definitions)
    }

    fn find_plugin_definition(&self, plugin: &String) -> BoxResult<plugins::Definition> {
        for overlay in &self.overlays {
            if let Ok(def) = overlay.find_plugin_definition(plugin) {
                return Ok(def);
            }
        }
        Err(format!("Plugin not found: {}", plugin).into())
    }
}
