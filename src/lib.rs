use std::any::Any;
use std::error::Error as StdError;
use clap::ArgMatches;

pub mod xoshiro256starstar;
use libloading::{Library, Symbol};


pub type Error = Box<dyn StdError + Send + Sync + 'static>;


pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
}

/**
 Plugin Manager class - allows inserting your own hashers
 Inspired by https://michael-f-bryan.github.io/rust-ffi-guide/dynamic_loading.html
*/
impl PluginManager {
    pub fn new() -> Self {
        Self{
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
        }
    }

    pub(crate) unsafe fn load_single_plugin<'help>(&mut self, app: clap::App<'help>, path: &str) -> Result<clap::App<'help>,Error> {
        type PluginCreate<'help> = unsafe fn(*const clap::App<'help>) -> (*mut clap::App<'help>, *mut dyn Plugin);

        let lib = Library::new(path).expect("Unable to load the plugin");
        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();

        let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create")
            .expect("The `_plugin_create` symbol wasn't found.");
        let app = Box::into_raw(Box::new(app));
        let (app, boxed_raw) = constructor(app);

        let plugin = Box::from_raw(boxed_raw);
        self.plugins.push(plugin);

        //Ok(Box::from_raw(app))
        Ok(*Box::from_raw(app))
    }

    pub fn build(&self) -> Result<Vec<Box<dyn WorkerSpec + 'static>>, Error> {
        Ok(self.plugins.last().unwrap().get_worker_specs())
    }

    pub fn process_options(&mut self, matchs: &ArgMatches) -> Result<(), Error>{
        self.plugins.iter_mut().for_each(|plugin| {
            plugin.process_option(matchs).expect(
                format!("Could not process option for plugin {}", plugin.name()).as_str()
            )
        });
        Ok(())
    }
}

pub trait Plugin: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn get_worker_specs(&self) -> Vec<Box<dyn WorkerSpec>>;
    fn process_option(&mut self, matchs: &ArgMatches) -> Result<(), Error>;
}

pub trait WorkerSpec: Any + Send + Sync {
    /*type_: GPUWorkType,
    opencl_platform: u16,
    device_id: u32,
    workload: f32,
    is_absolute: bool*/
    fn build (&self) -> Box<dyn Worker>;
}

pub trait Worker {
    //fn new(device_id: u32, workload: f32, is_absolute: bool) -> Result<Self, Error>;
    fn id(&self) -> String;
    fn load_block_constants(&mut self, hash_header: &[u8; 72], matrix: &[[u16; 64]; 64], target: &[u64; 4]);

    fn calculate_hash(&mut self, nonces: Option<&Vec<u64>>);
    fn sync(&self) -> Result<(), Error>;

    fn get_workload(&self) -> usize;
    fn copy_output_to(&mut self, nonces: &mut Vec<u64>) -> Result<(), Error>;
}

pub fn load_plugins<'help>(app: clap::App<'help>, paths: &[String]) -> Result<(clap::App<'help>, PluginManager),Error> {
    let mut factory = PluginManager::new();
    let mut app = app;
    for path in paths {
        app = unsafe { factory.load_single_plugin(app, path.as_str())? };
    }
    Ok((app, factory))
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path, $args:ty) => {
        use clap::Args;
        #[no_mangle]
        pub extern "C" fn _plugin_create(app: *mut clap::App) -> (*mut clap::App, *mut dyn $crate::Plugin) {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<dyn $crate::Plugin> = Box::new(object);

            let boxed_app = Box::new(<$args>::augment_args(unsafe{*Box::from_raw(app)}));
            (Box::into_raw(boxed_app), Box::into_raw(boxed))
        }
    };
}