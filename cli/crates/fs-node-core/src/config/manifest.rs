// Module plugin manifest — re-exported from fs-plugin-sdk.
//
// All plugin protocol types live in fs-plugin-sdk; this module provides
// a stable import path for the rest of fs-*.

/// Re-export all plugin protocol types from `fs-plugin-sdk`.
pub use fs_plugin_sdk::{
    ModuleManifest,
    ManifestInputs,
    ManifestOutputFile,
    PluginContext,
    InstanceInfo,
    PeerService,
    PeerRoute,
    PluginResponse,
    OutputFile,
    ShellCommand,
    LogLine,
    LogLevel,
};
