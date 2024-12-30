pub fn add_path(name: &str, value: &str, directory: bool) {
    if directory {
        println!(
            "Adding directory-specific path '{}' with value '{}'",
            name, value
        );
    } else {
        println!("Adding path '{}' with value '{}'", name, value);
    }
}

pub fn add_secret(name: &str, value: &str, directory: bool) {
    if directory {
        println!(
            "Adding directory-specific secret '{}' with value '{}'",
            name, value
        );
    } else {
        println!("Adding secret '{}' with value '{}'", name, value);
    }
}
