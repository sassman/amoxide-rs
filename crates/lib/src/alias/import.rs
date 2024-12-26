use log::info;
use std::io::{BufRead, BufReader, Read};

use crate::{
    alias::{add::add_alias, Alias},
    context::Context,
};

use super::ShellAlias;

pub fn import(_ctx: &Context) -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let mut all_aliases = Vec::new();

    import_aliases_from_file(stdin, &mut all_aliases)?;
    info!("found {} aliases to import", all_aliases.len());

    for alias in all_aliases {
        let alias_cmd = Alias::from(alias.value);
        add_alias(&alias.name, &alias_cmd, false, false)?;
    }

    Ok(())
}

// fn find_source_files<R: Read>(
//     file: R,
//     source_files: &mut Vec<BufReader<File>>,
// ) -> anyhow::Result<()> {
//     for line in BufReader::new(file).lines().flatten() {
//         let line = line.trim_start();
//         if line.starts_with("source ") {
//             let path = line.replacen("source ", "", 1);
//             let Ok(path) = expand_path_variables(&path) else {
//                 warn!("failed to expand variables in `{path}`, skipping...");
//                 continue;
//             };
//             let file = File::open(home()?.join(&path))?;
//             info!("found a source file to import from `{path}`");
//             source_files.push(BufReader::new(file));

//             // Recursively read source files
//             let file = File::open(home()?.join(&path))?;
//             find_source_files(file, source_files)?
//         }
//     }

//     Ok(())
// }

// fn expand_path_variables(path: &str) -> anyhow::Result<String> {
//     Ok(shellexpand::full(&path)?.to_string())
// }

fn import_aliases_from_file<R: Read>(file: R, aliases: &mut Vec<ShellAlias>) -> anyhow::Result<()> {
    // Consumes the iterator, returns an (Optional) String
    for line in BufReader::new(file).lines().flatten() {
        if line.starts_with("alias ") {
            info!("found an alias to import `{line}`");
            let alias = ShellAlias::try_from(line)?;
            aliases.push(alias);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use indoc::indoc;

    #[test]
    fn test_import_of_aliases_via_source_files() {
        let mock = indoc! { r#"
            alias foo="bar"
            alias baz="bak"
        "#};
        let mut reader = Cursor::new(mock);
        let mut aliases = Vec::new();
        // let mut tmp = tempfile().unwrap();
        // tmp.write_all(mock.as_bytes()).unwrap();
        // let ctx = Context::new(Box::new(Zsh));
        import_aliases_from_file(&mut reader, &mut aliases).unwrap();

        assert_eq!(aliases.len(), 2);
    }
}
