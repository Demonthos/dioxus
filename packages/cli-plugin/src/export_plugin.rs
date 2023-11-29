#[macro_export]
macro_rules! export_plugin {
    ($name:ident) => {
        ::wit_bindgen::generate!({
            inline: "package plugins:main;

interface definitions {

  record config {
    // DioxusConfig?
  }

  // Initialize the plugin
  register: func(conf: config) -> bool;

  // Before the app is built
  before-build: func() -> bool;

  // After the application is built, before serve
  before-serve: func() -> bool;

  // Reload on serve with no hot-reloading(?)
  on-rebuild: func() -> bool;

  // Reload on serve with hot-reloading
  on-hot-reload: func();

  on-watched-paths-change: func(path: list<string>);
}

interface imports {
  enum platform {
    web,
    desktop,
  }

  get-platform: func() -> platform;
  
  output-directory: func() -> string;

  reload-browser: func();
  refresh-asset: func(old-url: string, new-url: string);

  // Add path to list of watched paths
  watch-path: func(path: string);

}

world plugin-world {
  import imports;

  export definitions;
}
",
            world: "plugin-world",
            exports: {
                world: $name,
                "plugins:main/definitions": $name
            },
        });
    };
}