mod app;
mod delegation;
mod input;
mod model;
mod tree;
mod update;
mod view;

fn main() -> anyhow::Result<()> {
    app::run()
}
