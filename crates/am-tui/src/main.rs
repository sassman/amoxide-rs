mod app;
mod input;
mod model;
mod tree;
mod update;
mod view;

fn main() -> anyhow::Result<()> {
    app::run()
}
