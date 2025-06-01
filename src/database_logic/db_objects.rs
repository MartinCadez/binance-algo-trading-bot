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