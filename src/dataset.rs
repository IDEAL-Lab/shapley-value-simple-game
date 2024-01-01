use crate::{OwnerSet, Table};
use anyhow::Result;
use glob::glob;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Debug, Clone)]
pub struct DataSet {
    pub name: String,
    pub tables: HashMap<String, Table>,
    pub owner_set: OwnerSet,
}

impl DataSet {
    pub fn load(
        name: impl Into<String>,
        csv_dir: impl AsRef<Path>,
        meta_dir: &Option<PathBuf>,
    ) -> Result<Self> {
        match meta_dir {
            Some(assignment_dir) => Self::load_with_assignment(name, csv_dir, assignment_dir),
            None => Self::load_without_assignment(name, csv_dir),
        }
    }

    fn load_with_assignment(
        name: impl Into<String>,
        csv_dir: impl AsRef<Path>,
        meta_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let begin = Instant::now();
        let csv_dir = csv_dir.as_ref();
        let meta_dir = meta_dir.as_ref();
        info!("load source data from {}...", csv_dir.display());
        info!("load assignment data from {}...", meta_dir.display());

        let mut tables = HashMap::new();
        for csv_f in glob(&csv_dir.join("*.csv").to_string_lossy())? {
            let csv_f = csv_f?;
            let name = csv_f.file_stem().unwrap().to_string_lossy().to_string();
            let row_id_f = meta_dir.join(format!("{name}-index.json"));
            let owner_f = meta_dir.join(format!("{name}-owner.json"));
            let table = Table::load(name.clone(), csv_f, row_id_f, owner_f)?;
            tables.insert(name, table);
        }
        // TODO: store owner list directly
        let owner_set = {
            let mut owners = HashSet::new();
            for t in tables.values() {
                for s in t.owner_map.values() {
                    owners.extend(s.iter().copied());
                }
            }
            OwnerSet::new(owners.into_iter().collect())
        };

        info!("done in {:?}", Instant::now() - begin);
        Ok(Self {
            name: name.into(),
            tables,
            owner_set,
        })
    }

    fn load_without_assignment(name: impl Into<String>, csv_dir: impl AsRef<Path>) -> Result<Self> {
        let begin = Instant::now();
        let csv_dir = csv_dir.as_ref();
        info!("load source data from {}...", csv_dir.display());

        let mut tables = HashMap::new();
        for csv_f in glob(&csv_dir.join("*.csv").to_string_lossy())? {
            let csv_f = csv_f?;
            let name = csv_f.file_stem().unwrap().to_string_lossy().to_string();
            let table = Table::load_without_assignment(name.clone(), csv_f)?;
            tables.insert(name, table);
        }

        info!("done in {:?}", Instant::now() - begin);
        Ok(Self {
            name: name.into(),
            tables,
            owner_set: OwnerSet::default(),
        })
    }
}
