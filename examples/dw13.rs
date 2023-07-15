
use settrade_dw::{
    self,
    instrument::dw,
    instrument::dw::DWPriceTable,
    dw13,
};

use tokio;

#[tokio::main]
async fn main ( ) {

    let dw_info = dw::DWInfo::from_str("SET5013C2306I").unwrap ( );
    println ! ( "{:?}", dw_info );
    let out = dw13::DW13::get_underlying_dw_price_table( &dw_info )
        .await;
    
    println ! ( "{:?}", out );
}