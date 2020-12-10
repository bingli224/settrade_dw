
use settrade_dw::{
    self,
    instrument::dw,
    instrument::dw::DWPriceTable,
    dw13,
};

fn main ( ) {
    let out = dw13::DW13::get_underlying_dw_price_table(& dw::DWInfo::from_str("S5013C2101A").unwrap ( ) );
    
    println ! ( "{:?}", out );
}