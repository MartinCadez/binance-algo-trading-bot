/// UNUSED: 
/// tables management utilities are currently not used since, whole db and
/// all db-schemas are created initially with docker, basically there is no 
/// need for dynamic interaction with db, kept here in case for potential need

pub struct Column {
    pub name: String,
    pub col_type: String,
    pub constraints: Option<String>,
}

pub enum Tables{
    Trades,
    Prices
}

impl Tables{
    pub fn as_str(&self) -> &str {
        match self {
            Tables::Trades => "trades",
            Tables::Prices => "prices"
        }
    }
}