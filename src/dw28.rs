
pub struct DW28;

use crate::{DEFAULT_PRICE_DIGIT, instrument::{
        to_int_price,
        dw::{
            DWInfo,
            DWSide,
            DWPriceTable,
            Error,
        },
    }};
use async_trait::async_trait;

use serde_json;
use log::debug;

#[cfg(test)]
use env_logger;

// #[cfg(test)]
// use mockall::predicate::*;

#[cfg(not(test))]
use crate::get_latest_working_date_time;

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
fn get_latest_working_date_time ( ) -> DateTime<Local> {
    DateTime::from_str ("2020-12-23T12:00:00-00:00").unwrap ( )
}

use chrono::NaiveDate;

#[cfg(test)]
use chrono::{
    DateTime,
    Local,
};

#[cfg(not(test))]
use reqwest::Client;

#[cfg(test)]
use super::reqwest_mock::HTML_MAP;

#[cfg(test)]
use super::reqwest_mock::Client;

use std::collections::HashMap;

use regex::{
    Regex,
    RegexBuilder,
};
            
use lazy_static::lazy_static;

lazy_static ! {
    //static ref DW_INFO_REGEX_FORMAT : &'static str = r#"{([^}]*"security_code":"{dw_symbol}"[^}]*)}"#;
    /*
    static ref RE_DW_INFO : Regex = Regex::new ( r#"{([^}]*"security_code":"$"[^}]*)}"# )
        .expect ( "Failed to create Regex pattern of the security_code data." );
        */
    static ref RE_DW_RIC : Regex = RegexBuilder::new ( r#""ric":"([^"]+)"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the ric data." );
    static ref RE_COMPRESSED_TYPE : Regex = RegexBuilder::new ( r#""is_compressed":"true""# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the is_compression data." );
    static ref RE_DATE_LIST : Regex = RegexBuilder::new ( r#""dates":\[([^\]]+)"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the dates data." );
    static ref RE_DAILY_PRICE_LIST : Regex = RegexBuilder::new ( r#""(\d{4}-\d{2}-\d{2})":\[([^\]]+)"# )
        .multi_line ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the daily price data." );
    static ref RE_DW_DATA : Regex = RegexBuilder::new ( r#"\{([^\}]+)"# )
        .multi_line ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the DW data." );
    static ref RE_DW_BID_PRICE : Regex = RegexBuilder::new ( r#""bid":"?([\d\.]+)"?"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the DW bid price data." );
    static ref RE_UNDERLYING_BID_PRICE : Regex = RegexBuilder::new ( r#""underlying_bid":"?([\d\.]+)"?"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the underlying bid price data." );
    static ref RE_DATE_KEYS : Regex = RegexBuilder::new ( r#""date_keys":(\[[^\[]+])"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the date_keys data." );
    static ref RE_NONCOMPRESSED_PRICE_TABLE : Regex = RegexBuilder::new ( r#""livematrix":\[([^\]]+)"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the non-compessed price table." );
    static ref RE_PRICE_COLUMN : Regex = RegexBuilder::new ( r#"\{"([\d\.]+)":\{([^\}]+)"# )
        .multi_line ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the price column data." );
    static ref RE_DW_DATE_PRICE : Regex = RegexBuilder::new ( r#""([^"]+)":"([^"]+)"# )
        .multi_line ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the DW date data." );
}

macro_rules! DW_INFO_RE {
    ($dw_symbol:expr) => {
        RegexBuilder::new ( format ! ( 
                r#"\{{([^\}}]*"security_code":"{dw_symbol}"[^\}}]*)\}}"#,
                dw_symbol=$dw_symbol ).as_str ( )
            )
            .case_insensitive ( true )
            .build ( )
            .expect ( "Failed to create Regex pattern of the security_code data." )
    };
}

macro_rules! DW_LIST_URL {
    () => {
        if cfg!(not(feature = "stub-server")) {
            "https://www.thaidw.com/apimqth/LiveMatrixJSON?mode=1"
        } else {
            "http://localhost:54040/mock/dw28/dwList"
        }
    };
}

macro_rules! DW_PRICE_TABLE_URL {
    ($ric:expr) => {
        if cfg!(not(feature = "stub-server")) {
            format ! ( "https://www.thaidw.com/apimqth/LiveMatrixJSON?mode=1&ric={ric}", ric=$ric )
        } else {
            format ! ( "http://localhost:54040/mock/dw28/priceTable/{ric}", ric=$ric )
        }
    };
}

#[cfg(test)]
macro_rules! target_list_html {
    () => {
        std::fs::read_to_string( "tests/dw28/dw28_list_20201223.html" )
            .expect ( "Failed to open file [dw28_list_20201223]" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_s50_call_url {
    () => {
        DW_PRICE_TABLE_URL ! ( "S5028C012D.BK" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_s50_call {
    () => {
        std::fs::read_to_string( "tests/dw28/dw28_S5028C2012D_20201223.html" )
            .expect ( "Failed to open file [dw28_S5028C2012D]" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_hsi_call_url {
    () => {
        DW_PRICE_TABLE_URL ! ( "HSI28C012L.BK" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_hsi_call {
    () => {
        std::fs::read_to_string( "tests/dw28/dw28_HSI28C2012C_20201223.html" )
            .expect ( "Failed to open file [dw28_HSI28C2012C]" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_hsi_put_url {
    () => {
        DW_PRICE_TABLE_URL ! ( "HSI28P101C.BK" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_hsi_put {
    () => {
        std::fs::read_to_string( "tests/dw28/dw28_HSI28P2101C_20201223.html" )
            .expect ( "Failed to open file [dw28_HSI28P2101C]" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_advanc_call_url {
    () => {
        DW_PRICE_TABLE_URL ! ( "ADVA28C102L.BK" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_advanc_call {
    () => {
        std::fs::read_to_string( "tests/dw28/dw28_ADVA28C2102L_20201223.html" )
            .expect ( "Failed to open file [dw28_ADVA28C2102L]" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_spx_put_url {
    () => {
        DW_PRICE_TABLE_URL ! ( "SPX28P103A.BK" )
    };
}

#[cfg(test)]
macro_rules! target_html_compressed_spx_put {
    () => {
        std::fs::read_to_string( "tests/dw28/dw28_SPX28P2103A_20201223.html" )
            .expect ( "Failed to open file [dw28_SPX28P2103A]" )
    };
}

impl DW28 {
    pub fn get_predicted_dw_ric ( dw_info: &DWInfo ) -> String {
        format ! (
            "{underlying_part}{broker_id}{dw_type}{expiration_ymm}.BK",
            underlying_part=dw_info.underlying_symbol,
            broker_id=dw_info.broker_id,
            dw_type=match dw_info.side {
                DWSide::C => "C",
                DWSide::P => "P",
                _ => ".",
            },
            expiration_ymm=std::str::from_utf8( &dw_info.expire_yymm [ 1..4 ] ).expect ( "Failed to convert expiration_yymm from [u8] to string" ),
        )
    }
}

#[async_trait(?Send)]
impl DWPriceTable for DW28 {
    type UnderlyingType = i32;
    type DWType = f32;

    // outdated
    async fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Result<HashMap<Self::UnderlyingType, Vec<Self::DWType>>, Error> {
    //async fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Result<HashMap<i32, Vec<f32>>, ()> {
        #[cfg(test)]
        {
            let mut states = TEST_STATE
                .lock()
                .unwrap();

            match states.get_mut(&thread::current().id()) {
                None => {
                    states
                        .insert(
                            thread::current().id(),
                            TestState {
                                count: 1,
                                last_dw_symbol: dw_info.symbol.clone().to_string(),
                            }
                        );
                },
                Some(s) => {
                    s.count += 1;
                    s.last_dw_symbol = dw_info.symbol.clone().to_string();
                }
            }
        }

        let now = get_latest_working_date_time ( );

        let content =
            Client::new ( )
                .get (
                    DW_LIST_URL!()
                )
                .send ( )
                .await
                .expect ( format! ( "Failed to connect to {}", DW_LIST_URL!() ).as_str ( ) )
                .text ( )
                .await
                .expect ( "Failed to get data from thaidw.com in text format" )
            ;
            
        // debug ! ( "DW List: {}\n", content.as_str ( ) );
            
        let mut dw_ric = String::new ( );
        if let Some ( dw_ric_matches ) = DW_INFO_RE ! ( dw_info.symbol ).captures_iter ( content.as_str ( ) ).next ( ) {
            if let Some ( dw_ric_capture ) = dw_ric_matches.get ( 1 ) {
                if let Some ( dw_ric_matches ) = RE_DW_RIC.captures_iter ( dw_ric_capture.as_str ( ) ).next ( ) {
                    if let Some ( found_dw_ric ) = dw_ric_matches.get ( 1 ) {
                        dw_ric = found_dw_ric.as_str ( ).to_string ( );
                    }
                }
            }
        }

        // debug ! ( "DW List dw_ric: {}", dw_ric );

        if dw_ric.is_empty() {
            dw_ric = DW28::get_predicted_dw_ric ( &dw_info );
            debug ! ( "dw_ric is not found, so be predicted instead: {}", dw_ric );
        }

        let content = Client::new ( )
            .get (
                DW_PRICE_TABLE_URL ! ( dw_ric )
                    .as_str ( )
            )
            .send ( )
            .await
            .expect ( "Failed to get ajax data from thaidw.com" )
            .text ( )
            .await
            .expect ( "Failed to get data from thaidw.com in text format" )
            ;
            
        let content = content.as_str ( );
            
        let mut dw_price_table = HashMap::<Self::UnderlyingType, Vec<f32>>::new ( );
        
        if RE_COMPRESSED_TYPE.is_match ( content ) {
            let mut dw_underlying_map = HashMap::<Box<str>, f32>::new ( );

            let current_date = now.format ( "%Y-%m-%d" ).to_string ( );
            let current_date = current_date.as_str ( );

            RE_DAILY_PRICE_LIST.captures_iter ( content )
                .filter_map ( |daily_price_list_captures|
                    if let Some ( date_match ) = daily_price_list_captures.get ( 1 ) {
                        Some ( ( date_match.as_str ( ), daily_price_list_captures ) )
                    } else {
                        None
                    }
                )
                .filter_map ( |(found_date, daily_price_list_captures)|
                    if found_date == current_date {
                        Some ( daily_price_list_captures )
                    } else {
                        None
                    } )
                .for_each ( |daily_price_list_captures| {
                    if let Some ( daily_data_match ) = daily_price_list_captures.get ( 2 ) {
                        RE_DW_DATA.captures_iter ( daily_data_match.as_str ( ) )
                            .map ( |captures| captures.get ( 1 ) )
                            .filter_map ( |daily_data_option_match| daily_data_option_match )
                            .map ( |daily_data_match| daily_data_match.as_str ( ) )
                            .filter_map ( |daily_data| {
                                if let Some ( underlying_bid_captures ) = RE_UNDERLYING_BID_PRICE.captures_iter ( daily_data ).next ( ) {
                                    if let Some ( underlying_bid ) = underlying_bid_captures.get ( 1 ) {
                                        if let Ok ( underlying_bid_f32 ) = underlying_bid.as_str ( ).parse::<f32> ( ) {
                                            Some ( ( underlying_bid_f32, daily_data ) )
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } )
                            .for_each ( |(underlying_bid, daily_data)| {
                                if let Some ( dw_bid_captures ) = RE_DW_BID_PRICE.captures_iter ( daily_data ).next ( ) {
                                    if let Some ( dw_bid ) = dw_bid_captures.get ( 1 ) {
                                        let dw_bid = dw_bid.as_str ( );
                                        let last_underlying_price = dw_underlying_map.get ( dw_bid );
                                        if last_underlying_price.is_none ( ) || *last_underlying_price.unwrap() > underlying_bid {
                                            dw_underlying_map.insert ( dw_bid.to_string().into_boxed_str(), underlying_bid );
                                        }
                                    }
                                }
                            } )
                    }
                } );
                
                dw_underlying_map.into_iter()
                    .for_each ( |(dw, u)| {
                        dw_price_table.insert (to_int_price ( u, DEFAULT_PRICE_DIGIT ), vec ! [dw.parse::<f32>().unwrap ( )]);
                    } );
        } else {
            // noncompressed data
            
            let mut date_index = HashMap::<NaiveDate, usize>::new ( );
            
            if let Some ( found_date_captures ) = RE_DATE_KEYS.captures_iter ( content ).next ( ) {
                if let Some ( found_date_match ) = found_date_captures.get ( 1 ) {
                    if let Ok ( dates ) = serde_json::from_str::<Vec<String>> ( found_date_match.as_str ( ) ) {
                        
                        let mut dates :Vec<NaiveDate> = dates.into_iter ( )
                            //.map ( |s| format ! ( "{} {}", now.date().year(), s ) )
                            .map ( |s| format ! ( "20 {}", s ) )
                            .map ( |s| NaiveDate::parse_from_str ( &s, "%y %d %b").unwrap ( ) )
                            .collect ( );

                        dates.sort ( );
                        
                        dates.into_iter ( )
                            .enumerate ( )
                            .for_each ( |(idx, date)| {
                                date_index.insert ( date, idx );
                             } );
                    }
                }
            }
            
            if let Some ( found_date_captures ) = RE_NONCOMPRESSED_PRICE_TABLE.captures_iter ( content ).next ( ) {
                let current_date = now.format ( "%d %b" ).to_string ( );
                let current_date = current_date.as_str ( );
                
                if let Some ( price_column_match ) = found_date_captures.get ( 1 ) {
                    RE_PRICE_COLUMN.captures_iter ( price_column_match.as_str ( ) )
                        .filter_map ( |c| {
                            if let Some ( column ) = c.get ( 2 ) {
                                Some ( (
                                    c.get ( 1 ).unwrap ( ).as_str ( ).parse::<f32> ( ).unwrap ( ),  // underlying price
                                    column
                                ) )
                            } else {
                                None
                            }
                        } )
                        .for_each ( |(underlying, price_column)| {
                            RE_DW_DATE_PRICE.captures_iter ( price_column.as_str ( ) )
                                .filter_map ( |c| {
                                    if let Some ( date ) = c.get ( 1 ) {
                                        if date.as_str ( ) == current_date {
                                            c.get ( 2 )
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                 } )
                                .for_each ( |d| {
                                    dw_price_table.insert (
                                        //to_int_price ( price_column.as_str ( ).parse ( ).unwrap ( ), DEFAULT_PRICE_DIGIT ),
                                        to_int_price ( underlying, DEFAULT_PRICE_DIGIT ),
                                        vec ! [ d.as_str ( ).parse::<f32> ( ).expect ( "Failed to parse from str to f32" ) ]
                                    );
                                } );
                        } );

                } else {
                    return Err ( Error::DataNotFound { symbol: dw_info.symbol.clone(), info: Some("Not found date in RE_NONCOMPRESSED_PRICE_TABLE.".to_owned()) } );
                }
            }
        }

        Ok ( dw_price_table )
    }
}

#[cfg(test)]
use crate::testing::{gen_mock, test_count, test_last_dw_symbol};
#[cfg(test)]
gen_mock!(dw28);
#[cfg(test)]
pub mod dw28_tests {
    use super::*;
    use super::DW28;

    use std::sync::Once;
    
    pub static BEFORE_ALL: Once = Once::new ( );

    pub fn setup ( ) {
        if ! BEFORE_ALL.is_completed() {
            BEFORE_ALL.call_once( || {
                let _ = env_logger::try_init ( );
            } );
        }
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            result.insert ( DW_LIST_URL!().to_string ( ).into_boxed_str(), target_list_html!().to_string ( ) );
        } );
    }

    // PROBLEM: cannot find this test
    //#[cfg(feature = "stub-server")]
    // #[tokio::test]
    // pub async fn test_get_underlying_dw_price_table_with_stub_server ( ) {
    //     setup ( );
    //     let out = DW28::get_underlying_dw_price_table(& DWInfo::from_str ( "HSI28C2012L" ).unwrap ( ) )
    //         .await;
            
    //     debug!("{:?}", out);
    // }
    
    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table_compressed_s50_call ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( target_html_compressed_s50_call_url!().into_boxed_str ( ), target_html_compressed_s50_call!().to_string ( ) );
        } );
        
        let out = DW28::get_underlying_dw_price_table(& DWInfo::from_str ( "S5028C2012D" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );
        
        // check details
        assert_eq ! ( table.keys ( ).len ( ), 25 );
        
        assert ! ( table.contains_key ( &90680 ) && table.get ( &90680 ) == Some ( & vec ! [ 0.69 ] ) );
        assert ! ( table.contains_key ( &90630 ) && table.get ( &90630 ) == Some ( & vec ! [ 0.68 ] ) );
        assert ! ( table.contains_key ( &90590 ) && table.get ( &90590 ) == Some ( & vec ! [ 0.67 ] ) );
        assert ! ( table.contains_key ( &90550 ) && table.get ( &90550 ) == Some ( & vec ! [ 0.66 ] ) );
        assert ! ( table.contains_key ( &90500 ) && table.get ( &90500 ) == Some ( & vec ! [ 0.65 ] ) );
        assert ! ( table.contains_key ( &90460 ) && table.get ( &90460 ) == Some ( & vec ! [ 0.64 ] ) );
        assert ! ( table.contains_key ( &90410 ) && table.get ( &90410 ) == Some ( & vec ! [ 0.63 ] ) );
        assert ! ( table.contains_key ( &90370 ) && table.get ( &90370 ) == Some ( & vec ! [ 0.62 ] ) );
        assert ! ( table.contains_key ( &90320 ) && table.get ( &90320 ) == Some ( & vec ! [ 0.61 ] ) );
        assert ! ( table.contains_key ( &90280 ) && table.get ( &90280 ) == Some ( & vec ! [ 0.60 ] ) );
        assert ! ( table.contains_key ( &90230 ) && table.get ( &90230 ) == Some ( & vec ! [ 0.59 ] ) );
        assert ! ( table.contains_key ( &90180 ) && table.get ( &90180 ) == Some ( & vec ! [ 0.58 ] ) );
        assert ! ( table.contains_key ( &90140 ) && table.get ( &90140 ) == Some ( & vec ! [ 0.57 ] ) );
        assert ! ( table.contains_key ( &90090 ) && table.get ( &90090 ) == Some ( & vec ! [ 0.56 ] ) );
        assert ! ( table.contains_key ( &90040 ) && table.get ( &90040 ) == Some ( & vec ! [ 0.55 ] ) );
        assert ! ( table.contains_key ( &89990 ) && table.get ( &89990 ) == Some ( & vec ! [ 0.54 ] ) );
        assert ! ( table.contains_key ( &89940 ) && table.get ( &89940 ) == Some ( & vec ! [ 0.53 ] ) );
        assert ! ( table.contains_key ( &89890 ) && table.get ( &89890 ) == Some ( & vec ! [ 0.52 ] ) );
        assert ! ( table.contains_key ( &89840 ) && table.get ( &89840 ) == Some ( & vec ! [ 0.51 ] ) );
        assert ! ( table.contains_key ( &89790 ) && table.get ( &89790 ) == Some ( & vec ! [ 0.50 ] ) );
        assert ! ( table.contains_key ( &89730 ) && table.get ( &89730 ) == Some ( & vec ! [ 0.49 ] ) );
        assert ! ( table.contains_key ( &89680 ) && table.get ( &89680 ) == Some ( & vec ! [ 0.48 ] ) );
        assert ! ( table.contains_key ( &89630 ) && table.get ( &89630 ) == Some ( & vec ! [ 0.47 ] ) );
        assert ! ( table.contains_key ( &89570 ) && table.get ( &89570 ) == Some ( & vec ! [ 0.46 ] ) );
        assert ! ( table.contains_key ( &89520 ) && table.get ( &89520 ) == Some ( & vec ! [ 0.45 ] ) );
    }
    
    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table_compressed_hsi_call ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( target_html_compressed_hsi_call_url!().into_boxed_str ( ), target_html_compressed_hsi_call!().to_string ( ) );
        } );
        
        let out = DW28::get_underlying_dw_price_table(& DWInfo::from_str ( "HSI28C2012L" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );
        
        // check details
        assert_eq ! ( table.keys ( ).len ( ), 29 );
        
        assert ! ( table.contains_key ( &2678100 ) && table.get ( &2678100 ) == Some ( & vec ! [ 0.32 ] ) );
        assert ! ( table.contains_key ( &2676200 ) && table.get ( &2676200 ) == Some ( & vec ! [ 0.31 ] ) );
        assert ! ( table.contains_key ( &2674100 ) && table.get ( &2674100 ) == Some ( & vec ! [ 0.30 ] ) );
        assert ! ( table.contains_key ( &2672000 ) && table.get ( &2672000 ) == Some ( & vec ! [ 0.29 ] ) );
        assert ! ( table.contains_key ( &2669900 ) && table.get ( &2669900 ) == Some ( & vec ! [ 0.28 ] ) );
        assert ! ( table.contains_key ( &2667700 ) && table.get ( &2667700 ) == Some ( & vec ! [ 0.27 ] ) );
        assert ! ( table.contains_key ( &2665500 ) && table.get ( &2665500 ) == Some ( & vec ! [ 0.26 ] ) );
        assert ! ( table.contains_key ( &2663200 ) && table.get ( &2663200 ) == Some ( & vec ! [ 0.25 ] ) );
        assert ! ( table.contains_key ( &2660800 ) && table.get ( &2660800 ) == Some ( & vec ! [ 0.24 ] ) );
        assert ! ( table.contains_key ( &2658400 ) && table.get ( &2658400 ) == Some ( & vec ! [ 0.23 ] ) );
        assert ! ( table.contains_key ( &2655800 ) && table.get ( &2655800 ) == Some ( & vec ! [ 0.22 ] ) );
        assert ! ( table.contains_key ( &2653200 ) && table.get ( &2653200 ) == Some ( & vec ! [ 0.21 ] ) );
        assert ! ( table.contains_key ( &2650500 ) && table.get ( &2650500 ) == Some ( & vec ! [ 0.20 ] ) );
        assert ! ( table.contains_key ( &2647700 ) && table.get ( &2647700 ) == Some ( & vec ! [ 0.19 ] ) );
        assert ! ( table.contains_key ( &2644800 ) && table.get ( &2644800 ) == Some ( & vec ! [ 0.18 ] ) );
        assert ! ( table.contains_key ( &2641800 ) && table.get ( &2641800 ) == Some ( & vec ! [ 0.17 ] ) );
        assert ! ( table.contains_key ( &2638600 ) && table.get ( &2638600 ) == Some ( & vec ! [ 0.16 ] ) );
        assert ! ( table.contains_key ( &2635200 ) && table.get ( &2635200 ) == Some ( & vec ! [ 0.15 ] ) );
        assert ! ( table.contains_key ( &2631700 ) && table.get ( &2631700 ) == Some ( & vec ! [ 0.14 ] ) );
        assert ! ( table.contains_key ( &2628000 ) && table.get ( &2628000 ) == Some ( & vec ! [ 0.13 ] ) );
        assert ! ( table.contains_key ( &2624000 ) && table.get ( &2624000 ) == Some ( & vec ! [ 0.12 ] ) );
        assert ! ( table.contains_key ( &2619800 ) && table.get ( &2619800 ) == Some ( & vec ! [ 0.11 ] ) );
        assert ! ( table.contains_key ( &2615200 ) && table.get ( &2615200 ) == Some ( & vec ! [ 0.10 ] ) );
        assert ! ( table.contains_key ( &2610300 ) && table.get ( &2610300 ) == Some ( & vec ! [ 0.09 ] ) );
        assert ! ( table.contains_key ( &2604900 ) && table.get ( &2604900 ) == Some ( & vec ! [ 0.08 ] ) );
        assert ! ( table.contains_key ( &2598800 ) && table.get ( &2598800 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &2591900 ) && table.get ( &2591900 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &2584000 ) && table.get ( &2584000 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &2574400 ) && table.get ( &2574400 ) == Some ( & vec ! [ 0.04 ] ) );
    }
    
    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table_compressed_hsi_put ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( target_html_compressed_hsi_put_url!().into_boxed_str ( ), target_html_compressed_hsi_put!().to_string ( ) );
        } );
        
        let out = DW28::get_underlying_dw_price_table(& DWInfo::from_str ( "HSI28P2101C" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );
        
        // check details
        assert_eq ! ( table.keys ( ).len ( ), 41 );
        
        assert ! ( table.contains_key ( &2739400 ) && table.get ( &2739400 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &2720300 ) && table.get ( &2720300 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &2703600 ) && table.get ( &2703600 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &2688800 ) && table.get ( &2688800 ) == Some ( & vec ! [ 0.08 ] ) );
        assert ! ( table.contains_key ( &2675500 ) && table.get ( &2675500 ) == Some ( & vec ! [ 0.09 ] ) );
        assert ! ( table.contains_key ( &2663400 ) && table.get ( &2663400 ) == Some ( & vec ! [ 0.10 ] ) );
        assert ! ( table.contains_key ( &2652300 ) && table.get ( &2652300 ) == Some ( & vec ! [ 0.11 ] ) );
        assert ! ( table.contains_key ( &2642000 ) && table.get ( &2642000 ) == Some ( & vec ! [ 0.12 ] ) );
        assert ! ( table.contains_key ( &2632400 ) && table.get ( &2632400 ) == Some ( & vec ! [ 0.13 ] ) );
        assert ! ( table.contains_key ( &2623400 ) && table.get ( &2623400 ) == Some ( & vec ! [ 0.14 ] ) );
        assert ! ( table.contains_key ( &2615000 ) && table.get ( &2615000 ) == Some ( & vec ! [ 0.15 ] ) );
        assert ! ( table.contains_key ( &2606900 ) && table.get ( &2606900 ) == Some ( & vec ! [ 0.16 ] ) );
        assert ! ( table.contains_key ( &2599400 ) && table.get ( &2599400 ) == Some ( & vec ! [ 0.17 ] ) );
        assert ! ( table.contains_key ( &2592100 ) && table.get ( &2592100 ) == Some ( & vec ! [ 0.18 ] ) );
        assert ! ( table.contains_key ( &2585200 ) && table.get ( &2585200 ) == Some ( & vec ! [ 0.19 ] ) );
        assert ! ( table.contains_key ( &2578600 ) && table.get ( &2578600 ) == Some ( & vec ! [ 0.20 ] ) );
        assert ! ( table.contains_key ( &2572300 ) && table.get ( &2572300 ) == Some ( & vec ! [ 0.21 ] ) );
        assert ! ( table.contains_key ( &2566200 ) && table.get ( &2566200 ) == Some ( & vec ! [ 0.22 ] ) );
        assert ! ( table.contains_key ( &2560400 ) && table.get ( &2560400 ) == Some ( & vec ! [ 0.23 ] ) );
        assert ! ( table.contains_key ( &2554700 ) && table.get ( &2554700 ) == Some ( & vec ! [ 0.24 ] ) );
        assert ! ( table.contains_key ( &2549300 ) && table.get ( &2549300 ) == Some ( & vec ! [ 0.25 ] ) );
        assert ! ( table.contains_key ( &2544000 ) && table.get ( &2544000 ) == Some ( & vec ! [ 0.26 ] ) );
        assert ! ( table.contains_key ( &2538900 ) && table.get ( &2538900 ) == Some ( & vec ! [ 0.27 ] ) );
        assert ! ( table.contains_key ( &2533900 ) && table.get ( &2533900 ) == Some ( & vec ! [ 0.28 ] ) );
        assert ! ( table.contains_key ( &2529100 ) && table.get ( &2529100 ) == Some ( & vec ! [ 0.29 ] ) );
        assert ! ( table.contains_key ( &2524400 ) && table.get ( &2524400 ) == Some ( & vec ! [ 0.30 ] ) );
        assert ! ( table.contains_key ( &2519900 ) && table.get ( &2519900 ) == Some ( & vec ! [ 0.31 ] ) );
        assert ! ( table.contains_key ( &2515500 ) && table.get ( &2515500 ) == Some ( & vec ! [ 0.32 ] ) );
        assert ! ( table.contains_key ( &2511100 ) && table.get ( &2511100 ) == Some ( & vec ! [ 0.33 ] ) );
        assert ! ( table.contains_key ( &2506900 ) && table.get ( &2506900 ) == Some ( & vec ! [ 0.34 ] ) );
        assert ! ( table.contains_key ( &2502800 ) && table.get ( &2502800 ) == Some ( & vec ! [ 0.35 ] ) );
        assert ! ( table.contains_key ( &2498800 ) && table.get ( &2498800 ) == Some ( & vec ! [ 0.36 ] ) );
        assert ! ( table.contains_key ( &2494900 ) && table.get ( &2494900 ) == Some ( & vec ! [ 0.37 ] ) );
        assert ! ( table.contains_key ( &2491000 ) && table.get ( &2491000 ) == Some ( & vec ! [ 0.38 ] ) );
        assert ! ( table.contains_key ( &2487300 ) && table.get ( &2487300 ) == Some ( & vec ! [ 0.39 ] ) );
        assert ! ( table.contains_key ( &2483600 ) && table.get ( &2483600 ) == Some ( & vec ! [ 0.40 ] ) );
        assert ! ( table.contains_key ( &2480000 ) && table.get ( &2480000 ) == Some ( & vec ! [ 0.41 ] ) );
        assert ! ( table.contains_key ( &2476400 ) && table.get ( &2476400 ) == Some ( & vec ! [ 0.42 ] ) );
        assert ! ( table.contains_key ( &2473000 ) && table.get ( &2473000 ) == Some ( & vec ! [ 0.43 ] ) );
        assert ! ( table.contains_key ( &2469500 ) && table.get ( &2469500 ) == Some ( & vec ! [ 0.44 ] ) );
        assert ! ( table.contains_key ( &2466200 ) && table.get ( &2466200 ) == Some ( & vec ! [ 0.45 ] ) );
    }
    
    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table_compressed_spx_put ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( target_html_compressed_spx_put_url!().into_boxed_str ( ), target_html_compressed_spx_put!().to_string ( ) );
        } );
        
        let out = DW28::get_underlying_dw_price_table(& DWInfo::from_str ( "SPX28P2103A" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );
        
        // check details
        assert_eq ! ( table.keys ( ).len ( ), 41 );
        
        assert ! ( table.contains_key ( &375900 ) && table.get ( &375900 ) == Some ( & vec ! [ 0.65 ] ) );
        assert ! ( table.contains_key ( &375300 ) && table.get ( &375300 ) == Some ( & vec ! [ 0.66 ] ) );
        assert ! ( table.contains_key ( &374800 ) && table.get ( &374800 ) == Some ( & vec ! [ 0.67 ] ) );
        assert ! ( table.contains_key ( &374300 ) && table.get ( &374300 ) == Some ( & vec ! [ 0.68 ] ) );
        assert ! ( table.contains_key ( &373700 ) && table.get ( &373700 ) == Some ( & vec ! [ 0.69 ] ) );
        assert ! ( table.contains_key ( &373200 ) && table.get ( &373200 ) == Some ( & vec ! [ 0.70 ] ) );
        assert ! ( table.contains_key ( &372700 ) && table.get ( &372700 ) == Some ( & vec ! [ 0.71 ] ) );
        assert ! ( table.contains_key ( &372200 ) && table.get ( &372200 ) == Some ( & vec ! [ 0.72 ] ) );
        assert ! ( table.contains_key ( &371700 ) && table.get ( &371700 ) == Some ( & vec ! [ 0.73 ] ) );
        assert ! ( table.contains_key ( &371200 ) && table.get ( &371200 ) == Some ( & vec ! [ 0.74 ] ) );
        assert ! ( table.contains_key ( &370700 ) && table.get ( &370700 ) == Some ( & vec ! [ 0.75 ] ) );
        assert ! ( table.contains_key ( &370200 ) && table.get ( &370200 ) == Some ( & vec ! [ 0.76 ] ) );
        assert ! ( table.contains_key ( &369700 ) && table.get ( &369700 ) == Some ( & vec ! [ 0.77 ] ) );
        assert ! ( table.contains_key ( &369200 ) && table.get ( &369200 ) == Some ( & vec ! [ 0.78 ] ) );
        assert ! ( table.contains_key ( &368700 ) && table.get ( &368700 ) == Some ( & vec ! [ 0.79 ] ) );
        assert ! ( table.contains_key ( &368300 ) && table.get ( &368300 ) == Some ( & vec ! [ 0.80 ] ) );
        assert ! ( table.contains_key ( &367800 ) && table.get ( &367800 ) == Some ( & vec ! [ 0.81 ] ) );
        assert ! ( table.contains_key ( &367400 ) && table.get ( &367400 ) == Some ( & vec ! [ 0.82 ] ) );
        assert ! ( table.contains_key ( &366900 ) && table.get ( &366900 ) == Some ( & vec ! [ 0.83 ] ) );
        assert ! ( table.contains_key ( &366400 ) && table.get ( &366400 ) == Some ( & vec ! [ 0.84 ] ) );
        assert ! ( table.contains_key ( &366000 ) && table.get ( &366000 ) == Some ( & vec ! [ 0.85 ] ) );
        assert ! ( table.contains_key ( &365600 ) && table.get ( &365600 ) == Some ( & vec ! [ 0.86 ] ) );
        assert ! ( table.contains_key ( &365100 ) && table.get ( &365100 ) == Some ( & vec ! [ 0.87 ] ) );
        assert ! ( table.contains_key ( &364700 ) && table.get ( &364700 ) == Some ( & vec ! [ 0.88 ] ) );
        assert ! ( table.contains_key ( &364300 ) && table.get ( &364300 ) == Some ( & vec ! [ 0.89 ] ) );
        assert ! ( table.contains_key ( &363800 ) && table.get ( &363800 ) == Some ( & vec ! [ 0.90 ] ) );
        assert ! ( table.contains_key ( &363400 ) && table.get ( &363400 ) == Some ( & vec ! [ 0.91 ] ) );
        assert ! ( table.contains_key ( &363000 ) && table.get ( &363000 ) == Some ( & vec ! [ 0.92 ] ) );
        assert ! ( table.contains_key ( &362600 ) && table.get ( &362600 ) == Some ( & vec ! [ 0.93 ] ) );
        assert ! ( table.contains_key ( &362200 ) && table.get ( &362200 ) == Some ( & vec ! [ 0.94 ] ) );
        assert ! ( table.contains_key ( &361800 ) && table.get ( &361800 ) == Some ( & vec ! [ 0.95 ] ) );
        assert ! ( table.contains_key ( &361400 ) && table.get ( &361400 ) == Some ( & vec ! [ 0.96 ] ) );
        assert ! ( table.contains_key ( &361000 ) && table.get ( &361000 ) == Some ( & vec ! [ 0.97 ] ) );
        assert ! ( table.contains_key ( &360600 ) && table.get ( &360600 ) == Some ( & vec ! [ 0.98 ] ) );
        assert ! ( table.contains_key ( &360200 ) && table.get ( &360200 ) == Some ( & vec ! [ 0.99 ] ) );
        assert ! ( table.contains_key ( &359800 ) && table.get ( &359800 ) == Some ( & vec ! [ 1.00 ] ) );
        assert ! ( table.contains_key ( &359400 ) && table.get ( &359400 ) == Some ( & vec ! [ 1.01 ] ) );
        assert ! ( table.contains_key ( &359000 ) && table.get ( &359000 ) == Some ( & vec ! [ 1.02 ] ) );
        assert ! ( table.contains_key ( &358600 ) && table.get ( &358600 ) == Some ( & vec ! [ 1.03 ] ) );
        assert ! ( table.contains_key ( &358300 ) && table.get ( &358300 ) == Some ( & vec ! [ 1.04 ] ) );
        assert ! ( table.contains_key ( &357900 ) && table.get ( &357900 ) == Some ( & vec ! [ 1.05 ] ) );
    }
    
    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table_noncompressed_advanc_call ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( target_html_compressed_advanc_call_url!().into_boxed_str ( ), target_html_compressed_advanc_call!().to_string ( ) );
        } );
        
        let out = DW28::get_underlying_dw_price_table(& DWInfo::from_str ( "ADVA28C2102L" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );
        
        // check details
        assert_eq ! ( table.keys ( ).len ( ), 41 );
        
        assert ! ( table.contains_key ( &16850 ) && table.get ( &16850 ) == Some ( & vec ! [ 0.03 ] ) );
        assert ! ( table.contains_key ( &16900 ) && table.get ( &16900 ) == Some ( & vec ! [ 0.03 ] ) );
        assert ! ( table.contains_key ( &16950 ) && table.get ( &16950 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17000 ) && table.get ( &17000 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17050 ) && table.get ( &17050 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17100 ) && table.get ( &17100 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17150 ) && table.get ( &17150 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17200 ) && table.get ( &17200 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17250 ) && table.get ( &17250 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17300 ) && table.get ( &17300 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17350 ) && table.get ( &17350 ) == Some ( & vec ! [ 0.04 ] ) );
        assert ! ( table.contains_key ( &17400 ) && table.get ( &17400 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &17450 ) && table.get ( &17450 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &17500 ) && table.get ( &17500 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &17550 ) && table.get ( &17550 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &17600 ) && table.get ( &17600 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &17650 ) && table.get ( &17650 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &17700 ) && table.get ( &17700 ) == Some ( & vec ! [ 0.05 ] ) );
        assert ! ( table.contains_key ( &17750 ) && table.get ( &17750 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &17800 ) && table.get ( &17800 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &17850 ) && table.get ( &17850 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &17900 ) && table.get ( &17900 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &17950 ) && table.get ( &17950 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &18000 ) && table.get ( &18000 ) == Some ( & vec ! [ 0.06 ] ) );
        assert ! ( table.contains_key ( &18050 ) && table.get ( &18050 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &18100 ) && table.get ( &18100 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &18150 ) && table.get ( &18150 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &18200 ) && table.get ( &18200 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &18250 ) && table.get ( &18250 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &18300 ) && table.get ( &18300 ) == Some ( & vec ! [ 0.07 ] ) );
        assert ! ( table.contains_key ( &18350 ) && table.get ( &18350 ) == Some ( & vec ! [ 0.08 ] ) );
        assert ! ( table.contains_key ( &18400 ) && table.get ( &18400 ) == Some ( & vec ! [ 0.08 ] ) );
        assert ! ( table.contains_key ( &18450 ) && table.get ( &18450 ) == Some ( & vec ! [ 0.08 ] ) );
        assert ! ( table.contains_key ( &18500 ) && table.get ( &18500 ) == Some ( & vec ! [ 0.08 ] ) );
        assert ! ( table.contains_key ( &18550 ) && table.get ( &18550 ) == Some ( & vec ! [ 0.08 ] ) );
        assert ! ( table.contains_key ( &18600 ) && table.get ( &18600 ) == Some ( & vec ! [ 0.09 ] ) );
        assert ! ( table.contains_key ( &18650 ) && table.get ( &18650 ) == Some ( & vec ! [ 0.09 ] ) );
        assert ! ( table.contains_key ( &18700 ) && table.get ( &18700 ) == Some ( & vec ! [ 0.09 ] ) );
        assert ! ( table.contains_key ( &18750 ) && table.get ( &18750 ) == Some ( & vec ! [ 0.09 ] ) );
        assert ! ( table.contains_key ( &18800 ) && table.get ( &18800 ) == Some ( & vec ! [ 0.10 ] ) );

    }
    
    #[test]
    pub fn test_get_predicted_dw_ric ( ) {
        setup ( );

        // random DWInfo
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::C,
            expire_yymm: [ 0x32, 0x33, 0x34, 0x35 ],
            series: 'A',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11C345.BK" );

        // underlying_symbol based
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "XYZ".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::C,
            expire_yymm: [ 0x32, 0x33, 0x34, 0x35 ],
            series: 'A',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "XYZ11C345.BK" );

        // broker_id
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 99,
            side: DWSide::C,
            expire_yymm: [ 0x32, 0x33, 0x34, 0x35 ],
            series: 'A',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC99C345.BK" );

        // C type
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::C,
            expire_yymm: [ 0x32, 0x33, 0x34, 0x35 ],
            series: 'A',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11C345.BK" );

        // P type
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::P,
            expire_yymm: [ 0x32, 0x33, 0x34, 0x35 ],
            series: 'B',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11P345.BK" );

        // unknown type
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::Unknown,
            expire_yymm: [ 0x32, 0x33, 0x34, 0x35 ],
            series: 'A',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11.345.BK" );

        // changed expiration Y_: no consequence
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::C,
            expire_yymm: [ 0x50, 0x33, 0x34, 0x35 ],
            series: 'B',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11C345.BK" );

        // changed expiration _Y
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::C,
            expire_yymm: [ 0x32, 0x39, 0x34, 0x35 ],
            series: 'B',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11C945.BK" );

        // changed expiration MM
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::C,
            expire_yymm: [ 0x32, 0x33, 0x30, 0x30 ],
            series: 'B',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11C300.BK" );

        // changed series: no consequence
        let dw_info = DWInfo {
            symbol: "ABC11C2345A".to_owned ( ).into_boxed_str(),
            underlying_symbol: "ABC".to_owned ( ).into_boxed_str(),
            broker_id: 11,
            side: DWSide::C,
            expire_yymm: [ 0x32, 0x33, 0x34, 0x35 ],
            series: 'B',
        };
        let ric = DW28::get_predicted_dw_ric ( &dw_info );
        assert_eq! ( ric, "ABC11C345.BK" );
    }
}
