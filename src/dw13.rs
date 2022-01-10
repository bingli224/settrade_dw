
use crate::{
    instrument::{
        to_lower_adjacent_price,
        to_int_price,
        dw::{
            DWInfo,
            DWSide,
            DWPriceTable,
            Error,
        },
    },
    RE_S50,
    DEFAULT_PRICE_DIGIT,
};
use async_trait::async_trait;

#[cfg(test)]
#[allow(unused_imports)]
use log::{
    debug,
};

#[cfg(test)]
use env_logger;

#[cfg(not(test))]
use crate::get_latest_working_date_time;

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
fn get_latest_working_date_time ( ) -> DateTime<Local> {
    DateTime::from_str ("2020-12-15T12:00:00-00:00").unwrap ( )
}

#[cfg(test)]
use chrono::{
    DateTime,
    Local,
};

#[cfg(test)]
macro_rules! target_html {
    () => {
        std::fs::read_to_string( "tests/dw13/dw13_result.html" )
            .expect ( "Failed to open file" )
    };
}

#[cfg(not(test))]
//use reqwest::blocking::Client;
use reqwest::Client;

#[cfg(test)]
use crate::reqwest_mock::HTML_MAP;

#[cfg(test)]
use crate::reqwest_mock::Client;

use std::{
    collections::HashMap,
};

use regex::{
    Regex,
    RegexBuilder,
};

use lazy_static::lazy_static;

lazy_static ! {
    //static ref RE_TABLE : Regex = RegexBuilder::new ( r#"\sid\s*=\s*"tablecenter"(.+?)</table"# )
    static ref RE_TABLE : Regex = RegexBuilder::new ( r#"\sclass\s*=\s*"dw_table2"(.+?)</table"# )
        .case_insensitive ( true )
        .dot_matches_new_line ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the underlying-DW price table." );
    static ref RE_COLUMN : Regex = RegexBuilder::new ( r#"<tr"# )
        .case_insensitive ( true )
        .multi_line ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the underlying-DW price tr." );
    //static ref RE_DATE : Regex = RegexBuilder::new ( r#">\s*(\d+-\w+)\s*<"# )
    static ref RE_DATE : Regex = RegexBuilder::new ( r#">\s*(\d+-\w+-\d+)\s*<"# )
        .case_insensitive ( true )
        .multi_line ( true )
        .dot_matches_new_line ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the date cell." );
    static ref RE_UNDERLYING_PRICE : Regex = RegexBuilder::new ( r#">\s*([\d,]+(\.\d+)?)\s*<"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the underlying price cell." );
}

/*
// not necessary
macro_rules! MAIN_URL {
    () => {
        "http://www.thaiwarrant.com/en/index.asp"
    };
}
*/

use mockall::automock;

pub struct DW13;

macro_rules! DW_PRICE_TABLE_URL {
    ($symbol:expr) => {
        format ! ( "https://www.thaiwarrant.com/en/kgi-dw/print_dw_indicative.asp?dn={symbol}", symbol=$symbol )
    };
}

#[automock]
#[async_trait(?Send)]
impl DWPriceTable for DW13 {
    type UnderlyingType = i32;
    type DWType = f32;

    //type TableResult = Result<HashMap<i32, Vec<f32>>, ( )>;

    // outdated
    async fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Result<HashMap<i32, Vec<f32>>, Error> {
        let now = get_latest_working_date_time ( );

        let table =
            Client::new ( )
                    .get (
                        DW_PRICE_TABLE_URL ! ( dw_info.symbol ).as_str ( )
                    )
                    .header ( "Cookie", "lang=E" )
                    .send ( )
                    .await
                    .expect ( "Failed to connect to thaiwarrant.com" )
                    .text ( )
                    .await
                    .expect ( "Failed to get data from thaiwarrant.com in text format" )
            ;
            
        if let Some ( table_match ) = RE_TABLE.find ( table.as_str ( ) ) {
            let mut u_dw_price_map = HashMap::<i32,Vec<f32>>::new ( );
            let columns = RE_COLUMN.split ( table_match.as_str ( ) )
                .collect::<Vec<&str>> ( );

            let mut column_offset = 0;
            
            let date = now.format ( "%d-%b-%y" ).to_string ( );
            
            if let Some ( &date_column ) = columns.get ( 2 ) {
                column_offset = RE_DATE.captures_iter ( date_column )
                    .position ( |c| {
                        if let Some ( found_date ) = c.get ( 1 ) {
                            found_date.as_str ( ) == date
                        } else {
                            false
                        }
                    } )
                    .unwrap_or ( 0 );
            }
            
            for & column in & columns [3..] {
                let mut idx_column_offset = 0;
                let mut found_underlying_price = false;
                let mut underlying_price = 0i32;
                
                let mut dw_price_list = Vec::<f32>::new ( );
                
                for price_capture in RE_UNDERLYING_PRICE.captures_iter ( column ) {
                    if found_underlying_price {
                        if idx_column_offset < column_offset {
                            idx_column_offset += 1;
                        } else {
                            if let Some ( price_match ) = price_capture.get ( 1 ) {
                                if let Ok ( price ) = price_match.as_str ( ).parse ( ) {
                                    dw_price_list.push ( price );
                                }
                            }
                        }
                    } else {
                        if let Some ( price_match ) = price_capture.get ( 1 ) {
                            if let Ok ( price ) = price_match.as_str ( ).parse::<f32> ( ) {
                                found_underlying_price = true;

                                if dw_info.side == DWSide::C && RE_S50.is_match ( &*dw_info.symbol ) {
                                    underlying_price = to_lower_adjacent_price (
                                        to_int_price ( price, DEFAULT_PRICE_DIGIT )
                                    );
                                } else {
                                    underlying_price = to_int_price ( price, DEFAULT_PRICE_DIGIT );
                                }
                            }
                        }
                    }
                }
                
                if found_underlying_price {
                    u_dw_price_map.insert ( underlying_price, dw_price_list );
                }
            }
            
            if u_dw_price_map.len() <= 0 {
                Err ( Error::DataNotFound { symbol: dw_info.symbol.clone() } )
            } else {
                Ok ( u_dw_price_map )
            }
        } else {
            Err ( Error::DataNotFound { symbol: dw_info.symbol.clone() } )
        }
    }

    /*
    // This case of result is found from Chrome inspect
    fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Option<HashMap<i32, Vec<f32>>> {
        let now = get_latest_working_date_time ( );

        /*
        // reqwest currently requires tokio 0.2, so downgrade to that
        let table = Runtime::new ( )
            .unwrap ( )
            .block_on ( async {
                Client::new ( )
                    .get (
                        DW_PRICE_TABLE_URL ! ( dw_info.symbol ).as_str ( )
                    )
                    .header ( "Cookie", "lang=E" )
                    .send ( )
                    .await
                    .expect ( "Failed to get ajax data from blswarrant.com" )
                    .text ( )
                    .await
                    .expect ( "Failed to get data from blswarrant.com in text format" )
            } );
            */
            
        let table = Client::new ( )
            .get (
                format ! (
                    DW_PRICE_TABLE_URL ! (),
                    symbol = dw_info.symbol,
                ).as_str ( )
            )
            .header ( "Cookie", "lang=E" )
            .send ( )
            .expect ( "Failed to get ajax data from blswarrant.com" )
            .text ( )
            .expect ( "Failed to get data from blswarrant.com in text format" )
            ;

            
        if let Some ( table_match ) = RE_TABLE.find ( table.as_str ( ) ) {
            let mut u_dw_price_map = HashMap::<i32,Vec<f32>>::new ( );
            let columns = RE_COLUMN.split ( table_match.as_str ( ) )
                .collect::<Vec<&str>> ( );

            let mut column_offset = 0;
            
            //let date = now.format ( "%d-%b-%y" ).to_owned ( );
            let date = now.format ( "%d-%b" ).to_owned ( );
            
            if let Some ( &date_column ) = columns.get ( 4 ) {
                column_offset = RE_DATE.captures_iter ( date_column )
                    .position ( |c| {
                        if let Some ( found_date ) = c.get ( 1 ) {
                            found_date.as_str ( ) == date
                        } else {
                            false
                        }
                    } )
                    .unwrap_or ( 0 );
            }
            
            for & column in & columns [5..] {
                let mut idx_column_offset = 0;
                let mut found_underlying_price = false;
                let mut underlying_price = 0i32;
                
                let mut dw_price_list = Vec::<f32>::new ( );
                
                for price_capture in RE_UNDERLYING_PRICE.captures_iter ( column ) {
                    if found_underlying_price {
                        if idx_column_offset < column_offset {
                            idx_column_offset += 1;
                        } else {
                            if let Some ( price_match ) = price_capture.get ( 1 ) {
                                if let Ok ( price ) = price_match.as_str ( ).parse ( ) {
                                    dw_price_list.push ( price );
                                }
                            }
                        }
                    } else {
                        if let Some ( price_match ) = price_capture.get ( 1 ) {
                            if let Ok ( price ) = price_match.as_str ( ).parse::<f64> ( ) {
                                found_underlying_price = true;

                                if dw_info.side == DWSide::C && RE_S50.is_match ( &*dw_info.underlying_symbol ) {
                                    underlying_price = to_lower_adjacent_price (
                                        to_int_price ( price, DEFAULT_PRICE_DIGIT )
                                    );
                                } else {
                                    underlying_price = to_int_price ( price, DEFAULT_PRICE_DIGIT );
                                }
                            }
                        }
                    }
                }
                
                if found_underlying_price {
                    u_dw_price_map.insert ( underlying_price, dw_price_list );
                }
            }
            
            Some ( u_dw_price_map )
        } else {
            None
        }
    }
    */
}
/*
use mockall::mock;
mock! {
    pub DW13{}
    
    use futures::future::Future;

    #[async_trait]
    impl DWPriceTable <i32, f32> for DW13 {
        async fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Future<Output=Result<HashMap<i32, Vec<f32>>, ()>> {
            let now = get_latest_working_date_time ( );
            Err(())
        }
    }
}
*/


#[cfg(test)]
pub mod tests {
    use super::*;
    
    use std::sync::Once;
    
    pub static BEFORE_ALL: Once = Once::new ( );

    pub fn setup ( ) {
        if ! BEFORE_ALL.is_completed() {
            BEFORE_ALL.call_once( || {
                let _ = env_logger::try_init ( );
            } );
        }
    }
    
    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table ( ) {
        setup ( );
        {
            let mut result = HTML_MAP.lock ( ).unwrap ( );
            result.insert ( "".to_owned ( ).into_boxed_str ( ), target_html!().to_owned ( ) );
        }
        
        let out = DW13::get_underlying_dw_price_table(& DWInfo::from_str ( "DW13C0000A" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );

        // check details
        assert_eq ! ( table.keys ( ).len ( ), 161 );
        for underlying_key in ( 92000i32..=99950i32 ).step_by ( 50 ) {
            assert ! ( table.contains_key ( &underlying_key ) );
            
            let dw_list = table.get ( &underlying_key );
            assert ! ( dw_list.is_some ( ) );
            let dw_list = dw_list.unwrap ( );
            assert_eq ! ( dw_list.len ( ), 7usize );

            match dw_list [ 0 ] {
                v if v == 0.14 => assert ! ( underlying_key >= 99500 && underlying_key <= 100000 ),
                v if v == 0.15 => assert ! ( underlying_key >= 98650 && underlying_key <= 99450 ),
                v if v == 0.16 => assert ! ( underlying_key >= 97850 && underlying_key <= 98600 ),
                v if v == 0.17 => assert ! ( underlying_key >= 97100 && underlying_key <= 97800 ),
                v if v == 0.18 => assert ! ( underlying_key >= 96350 && underlying_key <= 97050 ),
                v if v == 0.19 => assert ! ( underlying_key >= 95650 && underlying_key <= 96300 ),
                v if v == 0.20 => assert ! ( underlying_key >= 95000 && underlying_key <= 95600 ),
                v if v == 0.21 => assert ! ( underlying_key >= 94350 && underlying_key <= 94950 ),
                v if v == 0.22 => assert ! ( underlying_key >= 93700 && underlying_key <= 94300 ),
                v if v == 0.23 => assert ! ( underlying_key >= 93100 && underlying_key <= 93650 ),
                v if v == 0.24 => assert ! ( underlying_key >= 92550 && underlying_key <= 93050 ),
                v if v == 0.25 => assert ! ( underlying_key >= 92000 && underlying_key <= 92500 ),
                _ => panic ! ( )
            }

            match dw_list [ 1 ] {
                v if v == 0.14 => assert ! ( underlying_key >= 99200 && underlying_key <= 100000 ),
                v if v == 0.15 => assert ! ( underlying_key >= 98400 && underlying_key <= 99150 ),
                v if v == 0.16 => assert ! ( underlying_key >= 97600 && underlying_key <= 98350 ),
                v if v == 0.17 => assert ! ( underlying_key >= 96850 && underlying_key <= 97550 ),
                v if v == 0.18 => assert ! ( underlying_key >= 96150 && underlying_key <= 96800 ),
                v if v == 0.19 => assert ! ( underlying_key >= 95450 && underlying_key <= 96100 ),
                v if v == 0.20 => assert ! ( underlying_key >= 94800 && underlying_key <= 95400 ),
                v if v == 0.21 => assert ! ( underlying_key >= 94150 && underlying_key <= 94750 ),
                v if v == 0.22 => assert ! ( underlying_key >= 93500 && underlying_key <= 94100 ),
                v if v == 0.23 => assert ! ( underlying_key >= 92950 && underlying_key <= 93450 ),
                v if v == 0.24 => assert ! ( underlying_key >= 92350 && underlying_key <= 92900 ),
                v if v == 0.25 => assert ! ( underlying_key >= 92000 && underlying_key <= 92300 ),
                _ => panic ! ( )
            }

            match dw_list [ 2 ] {
                v if v == 0.13 => assert ! ( underlying_key >= 99800 && underlying_key <= 100000 ),
                v if v == 0.14 => assert ! ( underlying_key >= 98950 && underlying_key <= 99750 ),
                v if v == 0.15 => assert ! ( underlying_key >= 98150 && underlying_key <= 98900 ),
                v if v == 0.16 => assert ! ( underlying_key >= 97350 && underlying_key <= 98100 ),
                v if v == 0.17 => assert ! ( underlying_key >= 96600 && underlying_key <= 97300 ),
                v if v == 0.18 => assert ! ( underlying_key >= 95900 && underlying_key <= 96550 ),
                v if v == 0.19 => assert ! ( underlying_key >= 95200 && underlying_key <= 95850 ),
                v if v == 0.20 => assert ! ( underlying_key >= 94550 && underlying_key <= 95150 ),
                v if v == 0.21 => assert ! ( underlying_key >= 93950 && underlying_key <= 94500 ),
                v if v == 0.22 => assert ! ( underlying_key >= 93350 && underlying_key <= 93900 ),
                v if v == 0.23 => assert ! ( underlying_key >= 92750 && underlying_key <= 93300 ),
                v if v == 0.24 => assert ! ( underlying_key >= 92150 && underlying_key <= 92700 ),
                v if v == 0.25 => assert ! ( underlying_key >= 92000 && underlying_key <= 92100 ),
                _ => panic ! ( )
            }

            match dw_list [ 3 ] {
                v if v == 0.13 => assert ! ( underlying_key >= 99550 && underlying_key <= 100000 ),
                v if v == 0.14 => assert ! ( underlying_key >= 98700 && underlying_key <= 99500 ),
                v if v == 0.15 => assert ! ( underlying_key >= 97900 && underlying_key <= 98650 ),
                v if v == 0.16 => assert ! ( underlying_key >= 97100 && underlying_key <= 97850 ),
                v if v == 0.17 => assert ! ( underlying_key >= 96400 && underlying_key <= 97050 ),
                v if v == 0.18 => assert ! ( underlying_key >= 95700 && underlying_key <= 96350 ),
                v if v == 0.19 => assert ! ( underlying_key >= 95000 && underlying_key <= 95650 ),
                v if v == 0.20 => assert ! ( underlying_key >= 94350 && underlying_key <= 94950 ),
                v if v == 0.21 => assert ! ( underlying_key >= 93750 && underlying_key <= 94300 ),
                v if v == 0.22 => assert ! ( underlying_key >= 93150 && underlying_key <= 93700 ),
                v if v == 0.23 => assert ! ( underlying_key >= 92550 && underlying_key <= 93100 ),
                v if v == 0.24 => assert ! ( underlying_key >= 92000 && underlying_key <= 92500 ),
                _ => panic ! ( )
            }
            
            // TODO: compare with #4, #5, #6
        }
        
        println! ( "{:?}", table );
    }
}
/*
use futures::future::{
    self,
    Future,
    FutureExt,
    TryFutureExt,
};
use std::iter::FromIterator;

impl DWPriceTable <i32, f32> for DW13 {
    fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Pin<Box<dyn Future<Output = Result<HashMap<i32, Vec<f32>>, ()>> + Send>> {
    //fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Pin<Box<dyn Future<Output = Option<HashMap<i32, Vec<f32>>>> + Send>> {
            let now = get_latest_working_date_time ( );

            let underlying_dw_price_map = Client::new ( )
                .get (
                    DW_PRICE_TABLE_URL ! ( dw_info.symbol ).as_str ( )
                )
                .header ( "Cookie", "lang=E" )
                .send ( )
                .and_then ( |r| {
                    r.text ( )
                } )
                .and_then ( |table| async move {
                    //let table = table.as_str ( );
                    //future::ok ( RE_TABLE.find ( table ) )
                    let underlying_dw_price_map = RE_TABLE.find ( &table )
                        .and_then ( |table_match| {
                            let columns = RE_COLUMN.split ( table_match.as_str ( ) )
                                .collect::<Vec<&str>> ( );

                            let mut column_offset = 0;
                            
                            let date = now.format ( "%d-%b-%y" ).to_owned ( );
                            
                            if let Some ( &date_column ) = columns.get ( 2 ) {
                                column_offset = RE_DATE.captures_iter ( date_column )
                                    .position ( |c| {
                                        if let Some ( found_date ) = c.get ( 1 ) {
                                            found_date.as_str ( ) == date
                                        } else {
                                            false
                                        }
                                    } )
                                    .unwrap_or ( 0 );
                            }
                            
                            
                            let underlying_dw_price_map = columns.into_iter ( )
                                .skip ( 3 )
                                .filter_map ( |column| {
                                    let mut idx_column_offset = 0;
                                    let mut found_underlying_price = false;
                                    let mut underlying_price = 0i32;
                                    
                                    let mut dw_price_list = Vec::<f32>::new ( );
                                    
                                    for price_capture in RE_UNDERLYING_PRICE.captures_iter ( column ) {
                                        if found_underlying_price {
                                            if idx_column_offset < column_offset {
                                                idx_column_offset += 1;
                                            } else {
                                                if let Some ( price_match ) = price_capture.get ( 1 ) {
                                                    if let Ok ( price ) = price_match.as_str ( ).parse ( ) {
                                                        dw_price_list.push ( price );
                                                    }
                                                }
                                            }
                                        } else {
                                            if let Some ( price_match ) = price_capture.get ( 1 ) {
                                                if let Ok ( price ) = price_match.as_str ( ).parse::<f32> ( ) {
                                                    found_underlying_price = true;

                                                    if dw_info.side == DWSide::C && RE_S50.is_match ( &*dw_info.symbol ) {
                                                        underlying_price = to_lower_adjacent_price (
                                                            to_int_price ( price, DEFAULT_PRICE_DIGIT )
                                                        );
                                                    } else {
                                                        underlying_price = to_int_price ( price, DEFAULT_PRICE_DIGIT );
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    
                                    if found_underlying_price {
                                        Some ( ( underlying_price, dw_price_list ) )
                                    } else {
                                        None
                                    }
                                } )
                                ;

                            Some (
                                HashMap::<i32,Vec<f32>>::from_iter (
                                    underlying_dw_price_map
                                )
                            )
                        } )
                    ;
                    
                    Ok ( underlying_dw_price_map )
                } )
                .map_ok ( |r| r )
                .map_err ( |_| () )
                ;
        //Pin<Box<dyn Future<Output = Option<HashMap<i32, Vec<f32>>>> + Send>> {
        //Pin<Box<dyn Future<Output = Result<HashMap<i32, Vec<f32>>, ()>> + Send>> {
        Box::pin (
            future::ok (
                underlying_dw_price_map
                //None
            )
        )
    }
}
*/
