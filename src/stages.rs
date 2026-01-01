use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stage {
    pub name: String,
    #[serde(
        deserialize_with = "duration_from_secs",
        serialize_with = "duration_to_secs"
    )]
    pub duration: Duration,
}

fn duration_from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

fn duration_to_secs<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

impl Stage {
    pub fn new(name: &str, secs: u64) -> Self {
        Self {
            name: name.into(),
            duration: Duration::from_secs(secs),
        }
    }
}
impl Default for Stage {
    fn default() -> Self {
        Self {
            name: "stage".into(),
            duration: Duration::from_secs(0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stages {
    iter: Vec<Stage>,
    id: usize,
}
impl Stages {
    pub fn new(stages: Vec<Stage>) -> Stages {
        Stages {
            iter: stages,
            id: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.iter.len()
    }

    pub fn get_current(&self) -> Option<Stage> {
        match self.iter.get(self.id) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Stage> {
        match self.iter.get_mut(index) {
            Some(s) => Some(s),
            None => None,
        }
    }

    pub fn add(&mut self, stage: Stage) {
        self.iter.push(stage);
    }

    pub fn remove(&mut self, index: usize) {
        self.iter.remove(index);
    }

    pub fn reset(&mut self) {
        self.id = 0;
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Stage> {
        self.iter.iter_mut()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Stages, std::io::Error> {
        match std::fs::read_to_string(path) {
            Ok(c) => Ok(Stages {
                iter: from_str(&c).unwrap(),
                id: 0,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let new_json_string = serde_json::to_string(&self.iter).unwrap();

        let mut file = match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path.as_ref())
        {
            Ok(f) => f,
            Err(e) => match e.kind() {
                ErrorKind::NotFound => File::create(path).unwrap(),
                _ => return Err(e),
            },
        };
        file.write_all(new_json_string.as_bytes())
    }
}

impl Iterator for Stages {
    type Item = Stage;

    fn next(&mut self) -> Option<Self::Item> {
        self.id += 1;
        self.get_current()
    }
}
