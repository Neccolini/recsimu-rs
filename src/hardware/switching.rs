use std::str::FromStr;
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Switching {
    CutThrough,
    #[default]
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
