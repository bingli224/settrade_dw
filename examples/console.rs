
use settrade_dw::{
    self,
    instrument::dw,
    instrument::dw::DWPriceTable,
};

use tokio;
use std::io::stdin;
use env_logger;

#[tokio::main]
async fn main ( ) {
    let _ = env_logger::try_init();

    let mut input = String::new();
    
    print ! ("Type in DW symbol: ");
    stdin().read_line(&mut input).unwrap();
    
    let dw_info = dw::DWInfo::from_str(input.as_str().trim()).unwrap ( );
    println ! ( "{:?}", dw_info );
    let out = dw::DWInfo::get_underlying_dw_price_table ( &dw_info )
        .await;
    
    println ! ( "{:?}", out );
}