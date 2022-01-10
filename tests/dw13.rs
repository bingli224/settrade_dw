
use settrade_dw::{
    instrument::dw::DWInfo,
    instrument::dw::DWPriceTable,
    dw13::DW13,
};

use tokio;

// ISSUE: separate the code from `src`?
// difference between `tests` and `examples`?
#[tokio::main]
pub async fn test_get_underlying_dw_price_table_intrg_real_dw ( ) {
    let out = DW13::get_underlying_dw_price_table(& DWInfo::from_str ( "DW13C0000A" ).unwrap ( ) )
        .await;
    
    assert ! ( out.is_ok ( ) );
    
    // TODO: check details
    
    println! ( "{:?}", out );
}

#[tokio::main]
pub async fn test_get_underlying_dw_price_table_intrg_unexisting_dw ( ) {
    let out = DW13::get_underlying_dw_price_table(& DWInfo::from_str ( "DW13C0000A" ).unwrap ( ) )
        .await;
    
    assert ! ( out.is_ok ( ) );
    
    // TODO: check details
    
    println! ( "{:?}", out );
}