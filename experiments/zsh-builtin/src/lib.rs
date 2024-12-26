use std::ffi::CStr;

use log::info;
use zsh_module::{Builtin, CStrArray, MaybeZError, Module, ModuleBuilder, Opts};

// Notice how this module gets installed as `zsh_sm`
zsh_module::export_module!(zsm, setup);

struct ShellManager;

impl ShellManager {
    fn zsm_cmd(&mut self, name: &CStr, args: CStrArray, opts: Opts) -> MaybeZError {
        info!("name: {:?}", name);
        info!("args: {:?}", args);
        // println!("opts: {:?}", opts);

        info!("Hello, world!");
        Ok(())
    }
    // fn get_cmd(&mut self, _name: &CStr, args: CStrArray, _opts: Opts) -> MaybeZError {
    //     if args.len() == 0 {
    //         return Err(ZError::Conversion("Expected at least 1 element".into()));
    //     }
    //     for arg in args.iter() {
    //         if let Some(mut param) = zsh_module::zsh::get(arg) {
    //             println!("{}={:?}", arg.to_string_lossy(), param.get_value());
    //         }
    //     }
    //     Ok(())
    // }
}

fn setup() -> Result<Module, Box<dyn std::error::Error>> {
    let module = ModuleBuilder::new(ShellManager)
        .builtin(ShellManager::zsm_cmd, Builtin::new("zsm"))
        // .builtin(ShellManager::get_cmd, Builtin::new("rget"))
        .build();

    env_logger::init();

    Ok(module)
}
