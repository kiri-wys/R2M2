pub mod app;
pub mod mods;

use std::{
    fs::File,
    io::{Read, Write},
    time::Duration,
};

use app::Model;
use crossterm::event;
use mods::{Mod, Tags, game::ModMetaData};
use ron::ser::PrettyConfig;

use color_eyre::Result;
use quick_xml::de::from_str;
use ratatui::{Terminal, prelude::CrosstermBackend};

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut model: Model,
) -> Result<app::Persistent> {
    while !model.should_close() {
        terminal.draw(|f| model.view(f))?;

        let mut msg = None;
        if event::poll(Duration::from_millis(200))? {
            let ev = event::read()?;
            msg = app::try_message(&model, ev);
        }

        while msg.is_some() {
            msg = model.update(msg.unwrap());
        }
    }
    Ok(model.result())
}
fn read_dir() -> Result<app::Persistent> {
    let path = std::env::args().nth(1).unwrap();
    let mut mods = vec![];
    for path in std::fs::read_dir(path)? {
        let mut path = path?.path();
        if !path.is_dir() {
            continue;
        }
        path.push("About/About.xml");
        let xml = std::fs::read_to_string(path).unwrap();
        let metadata: ModMetaData = from_str(&xml)?;
        mods.push(Mod {
            metadata,
            tags: Tags::default(),
        });
    }
    Ok(app::Persistent {
        mods,
        tags: Tags::default(),
    })
}
fn main() -> Result<()> {
    color_eyre::install()?;
    let persistent: app::Persistent = match File::open("./mod_info.ron") {
        Ok(mut f) => {
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            match ron::de::from_str(&buf) {
                Ok(v) => v,
                Err(_) => read_dir()?,
            }
        }
        Err(_) => read_dir()?,
    };
    let mut terminal = ratatui::init();
    let mut model = Model::default();
    model.set_persistent(persistent);
    let res = run_app(&mut terminal, model)?;
    ratatui::restore();
    let mut buff = String::new();
    ron::ser::to_writer_pretty(&mut buff, &res, PrettyConfig::default())?;
    {
        let mut file = File::create("./mod_info.ron")?;
        file.write_all(buff.as_bytes())?;
    }
    Ok(())
}
