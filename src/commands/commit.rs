use anyhow::Result;

use crate::commands;
use crate::utils::enums::AddMode;

pub fn run(message: String, a: bool) -> Result<()> {
    if a {
        commands::add::run(AddMode::Update)?
    }

    

    Ok(())
}
