use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;

use crate::utils::enums::AddMode;
use crate::utils::index::{add_files_to_index, Index};
use crate::utils::ignore::is_ignored;
use crate::utils::dir::is_in_ink;

/// Main `add` command dispatcher
pub fn run(mode: AddMode) -> Result<()> {
    match mode {
        AddMode::All => {
            let files = Arc::new(Mutex::new(Vec::new()));

            WalkBuilder::new(".")
                .add_custom_ignore_filename(".inkignore")
                .standard_filters(false)
                .hidden(false)
                .build_parallel()
                .run(|| {
                    let files = Arc::clone(&files);
                    Box::new(move |res| {
                        if let Ok(entry) = res {
                            let path = entry.path();

                            if path.is_file() && !is_in_ink(path) {
                                files.lock().unwrap().push(path.to_path_buf());
                            }
                        }
                        ignore::WalkState::Continue
                    })
                });

            let files_to_add = Arc::try_unwrap(files)
                .map(|mutex| mutex.into_inner().unwrap())
                .unwrap_or_else(|arc| (*arc.lock().unwrap()).clone());

            add_files_to_index(&files_to_add)?;
        }

        AddMode::Update => {
            let mut index = Index::load()?;
            let tracked = index.tracked_files();

            let (existing, deleted): (Vec<_>, Vec<_>) = tracked
                .into_par_iter()
                .partition(|path| path.exists() && path.is_file());

            for path in deleted {
                index.remove(&path);
            }

            index.save()?; // save deletions
            add_files_to_index(&existing)?; // only add valid ones
        }

        AddMode::Files(files) => {
            let filtered: Vec<PathBuf> = files
                .into_par_iter()
                .filter(|f| f.is_file() && !is_in_ink(f) && !is_ignored(f))
                .collect();

            add_files_to_index(&filtered)?;
        }
    }

    Ok(())
}
