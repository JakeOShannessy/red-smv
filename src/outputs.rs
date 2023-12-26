use crate::{CsvDataBlock, SmvFile, SmvValue};
use data_vector::DataVector;
use std::path::{Path, PathBuf};

pub struct Outputs {
    pub smv_path: PathBuf,
    pub smv: SmvFile,
}

impl Outputs {
    pub fn new(smv_path: PathBuf) -> Self {
        let smv = SmvFile::from_file(&smv_path).expect("Could not read smv file");
        Self { smv_path, smv }
    }

    pub fn from_file<P: AsRef<Path>>(smv_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let smv_path = PathBuf::from(smv_path.as_ref());
        let smv = SmvFile::from_file(&smv_path)
            .map_err(|err| format!("Could not read smv file: {err}"))?;
        Ok(Self { smv_path, smv })
    }

    pub fn get_csv_vec(
        &mut self,
        csv_type: String,
        vec_name: String,
    ) -> Result<Option<DataVector<f64, SmvValue>>, Box<dyn std::error::Error>> {
        // TODO: add caching
        let csvf = if let Some(f) = self
            .smv
            .csvfs
            .iter()
            .find(|csvf| csvf.type_ == csv_type.as_str())
        {
            f
        } else {
            return Ok(None);
        };
        let smv_dir = PathBuf::from(self.smv_path.parent().unwrap());
        let mut csv_file_path = PathBuf::new();
        csv_file_path.push(smv_dir);
        csv_file_path.push(csvf.filename.clone());
        let data_block = CsvDataBlock::from_file(&csv_file_path)?;
        let vec = data_block.make_data_vector("Time", &vec_name);
        Ok(vec)
    }

    pub fn get_csv_vec_f64(
        &mut self,
        csv_type: String,
        vec_name: String,
    ) -> Result<Option<DataVector<f64, f64>>, Box<dyn std::error::Error>> {
        self.get_csv_vec(csv_type, vec_name)?
            .map(take_f64_vec)
            .transpose()
    }
}

fn take_f64_vec(
    vec: DataVector<f64, SmvValue>,
) -> Result<DataVector<f64, f64>, Box<dyn std::error::Error>> {
    let n = vec.values().len();
    let values = vec.values();
    let mut new_dv = DataVector::new(
        vec.name.clone(),
        vec.x_name.clone(),
        vec.y_name.clone(),
        vec.x_units.clone(),
        vec.y_units.clone(),
        Vec::with_capacity(n),
    );
    for value in values.iter() {
        let x = value.x;
        let y = match value.y {
            SmvValue::Float(y) => y,
            _ => return Err("not float".into()),
        };
        new_dv.insert(data_vector::Point { x, y });
    }
    Ok(new_dv)
}
