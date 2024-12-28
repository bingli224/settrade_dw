
use settrade_dw::{
    self,
    instrument::dw,
    instrument::dw::DWPriceTable,
    dw13,
};

use tokio;

#[tokio::main]
async fn main ( ) {
    let mut symbols: Vec<_> = std::env::args().skip(1).collect();
    
    if symbols.is_empty() {
        symbols = vec!["SET5013C2412A".to_string()];
    }

    for symbol in symbols.iter() {
        let dw_info = dw::DWInfo::from_str(&symbol).unwrap ( );
        println ! ( "{:?}", dw_info );
        let out = dw13::DW13::get_underlying_dw_price_table( &dw_info )
            .await;
        
        println ! ( "{:?}", out );
    }
}