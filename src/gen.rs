use crate::utils;
use serde::Deserialize;
use std::error;
use std::path::PathBuf;

use self::network::{Network, *};
use self::network_info::{NetworkInfo, Topology};

pub(crate) mod network;
pub(crate) mod network_info;
pub(crate) mod node_info;
#[derive(Debug, Deserialize)]
pub struct Config {
    pub path: PathBuf,
    pub network_info: NetworkInfo,
    pub network: Network,
}

impl Config {
    pub fn new(input_path: PathBuf, output_path: PathBuf) -> Result<Self, Box<dyn error::Error>> {
        let network_info: NetworkInfo = utils::read_json(input_path)?;
        let network = Network::new(vec![]);
        Ok(Config {
            path: output_path,
            network_info,
            network,
        })
    }

    pub fn build(&mut self) {
        match self.network_info.topology {
            Topology::Random(ref random_topology) => {
                self.network = generate_random(random_topology)
                    .map_err(|e| {
                        panic!("Error: {}", e);
                    })
                    .unwrap();
            }
            Topology::FromFile(ref from_file_topology) => {
                self.network = generate_from_file(from_file_topology)
                    .map_err(|e| {
                        panic!("Error: {}", e);
                    })
                    .unwrap();
            }
            Topology::Mesh(ref mesh_topology) => {
                self.network = generate_mesh(mesh_topology)
                    .map_err(|e| {
                        panic!("Error: {}", e);
                    })
                    .unwrap();
            }
        }
    }

    pub fn generate(&self) -> Result<(), Box<dyn error::Error>> {
        let network_str = self.network.json();
        utils::write_json(self.path.clone(), &network_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_read_json() {
        let path = PathBuf::from("tests/gen/success1.json");
        let network_info = utils::read_json::<NetworkInfo>(path).unwrap();
        assert_eq!(
            network_info.topology,
            Topology::Random(network_info::RandomTopology {
                node_num: 10,
                random_seed: Some(42),
            })
        );
    }
}
