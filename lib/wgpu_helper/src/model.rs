use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;
use std::{error::Error, fs::File, io::BufReader, path::Path};

fn from_string_vec_f64<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
where
  D: Deserializer<'de>,
{
  from_string_vec(deserializer)
}

fn from_string_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr,
  T::Err: Display,
{
  let str_vec: Vec<String> = Vec::deserialize(deserializer)?;
  str_vec
    .into_iter()
    .map(|s| T::from_str(&s).map_err(serde::de::Error::custom))
    .collect()
}

#[derive(Deserialize, Debug)]
pub struct Model {
  pub vertex_count: u64,
  pub face_count: usize,
  #[serde(deserialize_with = "from_string_vec_f64")]
  pub position: Vec<f64>,
  #[serde(deserialize_with = "from_string_vec_f64")]
  pub normal: Vec<f64>,
  #[serde(deserialize_with = "from_string_vec_f64")]
  pub uv: Vec<f64>,
  pub indices: Vec<u32>,
}

pub fn load_model_json<P: AsRef<Path>>(
  path: P,
) -> Result<Model, Box<dyn Error>> {
  let file = File::open(path)?;
  let reader = BufReader::new(file);

  let model = serde_json::from_reader(reader)?;

  Ok(model)
}
