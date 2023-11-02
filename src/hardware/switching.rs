use std::str::FromStr;
#[derive(Debug, Clone)]
pub enum Switching {
    CutThrough,
    StoreAndForward,
}

impl FromStr for Switching {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cut_through" => Ok(Self::CutThrough),
            "store_and_forward" => Ok(Self::StoreAndForward),
            _ => Err(format!("{} is not a valid switching type", s)),
        }
    }
}
