use bash_builtins::{builtin_metadata, Args, Builtin, BuiltinOptions, Result};
use std::{
    ffi::{c_char, CStr},
    io::{stdout, Write},
};

builtin_metadata!(
    name = "sm_exp1",
    create = SmExp1::default,
    short_doc = "sm_exp1 [-l]",
    long_doc = "
        sm_exp1: A simple builtin that demonstrates how to list all aliases.

        Options:
          -l\tList all aliases.
    ",
);

#[derive(BuiltinOptions)]
enum Opt {
    #[opt = 'l']
    List,
}

#[derive(Default)]
struct SmExp1;

impl Builtin for SmExp1 {
    fn call(&mut self, args: &mut Args) -> Result<()> {
        for opt in args.options() {
            match opt? {
                Opt::List => {
                    writeln!(stdout(), "Listing all alaises ..")?;
                    let aliases = AliasIterator::new(unsafe { all_aliases() });
                    writeln!(stdout(), "got aliases..")?;
                    for alias in aliases {
                        // cast as AlaisType
                        // CStr to str conversoin that is safe
                        let name = unsafe { CStr::from_ptr(alias.name) };
                        let value = unsafe { CStr::from_ptr(alias.value) };
                        writeln!(stdout(), "alias name={:?}, value={:?}", name, value)?;
                    }
                }
            }
        }

        // It is an error if we receive free arguments.
        args.finished()?;

        Ok(())
    }
}

extern "C" {
    pub fn all_aliases() -> *const *const AliasType;
}

#[repr(C)]
pub struct AliasType {
    pub name: *const c_char,
    pub value: *const c_char,
    pub flags: c_char,
}

pub struct AliasIterator {
    current: *const *const AliasType,
}

impl AliasIterator {
    pub fn new(start: *const *const AliasType) -> Self {
        AliasIterator { current: start }
    }
}

impl Iterator for AliasIterator {
    type Item = &'static AliasType;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if !self.current.is_null() && !(*self.current).is_null() {
                let alias = &**self.current;
                self.current = self.current.add(1);
                Some(alias)
            } else {
                None
            }
        }
    }
}
