
/// # Settrade Underlying-DW Price Table Scraper
/// 
/// Scrape DW (derivative warrant) price table from official DW websites.
/// 
/// ## Supported DW
/// 
/// | DW # | Website |
/// | ---- | ---- |
/// | DW13 | https://www.thaiwarrant.com/ |
/// | DW28 | https://www.thaidw.com/ |
/// 

#[cfg(test)]
mod reqwest_mock;
#[cfg(test)]
mod testing;

use std::collections::HashMap;
use chrono::{
    Duration,
    Local,
    NaiveDateTime,
    Timelike,
    Weekday,
    Datelike,
};

use regex::{
    Regex,
    RegexBuilder,
};

use snafu::Snafu;

use lazy_static::lazy_static;

lazy_static ! {
    static ref RE_S50 : Regex = RegexBuilder::new ( r#"^\s*s50"# )
        .case_insensitive ( true )
        .build ( )
        .expect ( "Failed to create Regex pattern of the underlying type as SET50." );
}

pub const DEFAULT_PRICE_DIGIT: usize = 2;

// #[cfg(not(test))]
pub mod dw13;

//use dw13::DW13;

// #[cfg(test)]
// mod dw13 {
//     use super::*;
//     use async_trait::async_trait;
    
//     // mock target
//     use crate::instrument::dw::*;

//     pub struct DW13;
    
//     pub static mut LAST_DW_SYMBOL: String = String::new();
//     pub static mut COUNT: u32 = 0;

//     #[async_trait(?Send)]
//     impl DWPriceTable for DW13 {
//         type UnderlyingType = i32;
//         type DWType = f32;

//         // outdated
//         async fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Result<HashMap<i32, Vec<f32>>, Error> {
//             unsafe {
//                 COUNT += 1;
//                 LAST_DW_SYMBOL = dw_info.symbol.clone().to_string();
//             }
//             Err ( Error::Test )
//         }
//     }
// }

// #[cfg(not(test))]
pub mod dw28;

// #[cfg(test)]
// mod dw28 {
//     use super::*;
//     use async_trait::async_trait;
    
//     // mock target
//     use crate::instrument::dw::*;

//     pub struct DW28;
    
//     pub static mut LAST_DW_SYMBOL: String = String::new();
//     pub static mut COUNT: u32 = 0;

//     #[async_trait(?Send)]
//     impl DWPriceTable for DW28 {
//         type UnderlyingType = i32;
//         type DWType = f32;

//         // outdated
//         async fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Result<HashMap<i32, Vec<f32>>, Error> {
//             unsafe {
//                 COUNT += 1;
//                 LAST_DW_SYMBOL = dw_info.symbol.clone().to_string();
//             }
//             Err ( Error::Test )
//         }
//     }
// }

/// # Underlying-price-based underlying-DW price map
/// 
/// The underlying and DW price are in f32 type, from original data
#[derive(Debug, Clone)]
pub struct UnderlyingDWMap <T> {
    pub dw_symbol: Box<str>,
    pub date: Box<str>,
    pub u_based_price_map: HashMap<T, T>,
}

impl <T> UnderlyingDWMap <T> {
}

/// Returns the latest working date of settrade market
/// 
/// # See
/// 
/// get_working_date_time_from(DateTime)
pub fn get_latest_working_date_time ( ) -> NaiveDateTime {
    get_working_date_time_from ( Local::now ( ).naive_local ( ) )
}

/// Returns the working date/time nearest to given DateTime.get_latest_working_date_time()
/// 
/// If given DateTime base is in working day (Mon-Fri) and before 16:30, the given date/time
/// is returned.
/// 
/// If given DateTime base is in working day (Mon-Fri) and since 16:30, the next working day
/// is returned.get_latest_working_date_time()
/// 
/// Otherwise; If given DateTime base is in Sat-Sun, the next Mon is returned.
/// 
/// # Arguments
/// 
/// * `datetime` - A DateTime<Local> object as the base date/time.
pub fn get_working_date_time_from ( mut datetime: NaiveDateTime ) -> NaiveDateTime {

    let time = datetime.time ( );
    if time.hour() > 16 || ( time.hour() == 16 && time.minute() >= 30 ) {
        datetime = datetime + Duration::days ( 1 );
    }
    
    match datetime.date ( ).weekday ( ) {
        Weekday::Sat => {
            datetime = datetime + Duration::days ( 2 );
        },
        Weekday::Sun => {
            datetime = datetime + Duration::days ( 1 );
        }
        _ => ()
    }
    
    datetime
}

pub mod instrument {
    use super::*;
    
    /// Returns upper adjacent price.
    /// 
    /// The range is in following:
    /// 	0.00-1.99	0.01
    /// 	2.00-4.98	0.02
    /// 	5.00-9.95	0.05
    /// 	10.00-24.90	0.10
    /// 	25.00-99.75	0.25
    /// 	100.00-199.50	0.50
    /// 	200.00-399.00	1.00
    /// 	400.00-upper	2.00
    /// 
    /// # Arguments
    /// 
    /// * `price` - Price to be converted.
    pub fn to_lower_adjacent_price ( price: i32 ) -> i32 {
        price -
            if price <= 200 {
                1
            } else if price <= 500 {
                2
            } else if price <= 1000 {
                5
            } else if price <= 2500 {
                10
            } else if price <= 10000 {
                25
            } else if price <= 20000 {
                50
            } else if price <= 40000 {
                100
            } else {
                200
            }
    }

    /// Returns upper adjacent price.
    /// 
    /// The range is in following:
    /// 	000-199	1
    /// 	200-498	2
    /// 	500-995	5
    /// 	1000-2490	10
    /// 	2500-9975	25
    /// 	10000-19950	50
    /// 	20000-39900	100
    /// 	40000-upper	200
    /// 
    /// # Arguments
    /// 
    /// * `price` - Price to be converted.
    pub fn to_upper_adjacent_price ( price: i32 ) -> i32 {
        price +
            if price < 200 {
                1
            } else if price < 500 {
                2
            } else if price < 1000 {
                5
            } else if price < 2500 {
                10
            } else if price < 10000 {
                25
            } else if price < 20000 {
                50
            } else if price < 40000 {
                100
            } else {
                200
            }
    }
    
    /// Returns the i32-formatted price, based on given [price_digit]
    /// 
    /// # Arguments
    /// 
    /// * `price` - Price in f32
    /// * `price_digit` - 10 power digits of f32 to be converted to i32
    pub fn to_int_price ( price: f32, price_digit: usize ) -> i32 {
        ( price * ( 10.0f32.powi ( price_digit as i32 ) ) ).round ( ) as i32
    }
    
    #[cfg(test)]
    pub mod tests {
        use super::*;

        #[test]
        fn test_to_lower_adjacent_price ( ) {
            assert_eq ! ( to_lower_adjacent_price ( 190 ), 189 );
            assert_eq ! ( to_lower_adjacent_price ( 200 ), 199 );
            assert_eq ! ( to_lower_adjacent_price ( 202 ), 200 );
            assert_eq ! ( to_lower_adjacent_price ( 398 ), 396 );
            assert_eq ! ( to_lower_adjacent_price ( 400 ), 398 );
            assert_eq ! ( to_lower_adjacent_price ( 402 ), 400 );
            assert_eq ! ( to_lower_adjacent_price ( 498 ), 496 );
            assert_eq ! ( to_lower_adjacent_price ( 500 ), 498 );
            assert_eq ! ( to_lower_adjacent_price ( 505 ), 500 );
            assert_eq ! ( to_lower_adjacent_price ( 1000 ), 995 );
            assert_eq ! ( to_lower_adjacent_price ( 1010 ), 1000 );
            assert_eq ! ( to_lower_adjacent_price ( 2500 ), 2490 );
            assert_eq ! ( to_lower_adjacent_price ( 2525 ), 2500 );
            assert_eq ! ( to_lower_adjacent_price ( 9975 ), 9950 );
            assert_eq ! ( to_lower_adjacent_price ( 10000 ), 9975 );
            assert_eq ! ( to_lower_adjacent_price ( 10050 ), 10000 );
            assert_eq ! ( to_lower_adjacent_price ( 20000 ), 19950 );
            assert_eq ! ( to_lower_adjacent_price ( 20100 ), 20000 );
            assert_eq ! ( to_lower_adjacent_price ( 40000 ), 39900 );
            assert_eq ! ( to_lower_adjacent_price ( 40200 ), 40000 );
        }
        
        #[test]
        fn test_to_upper_adjacent_price ( ) {
            assert_eq ! ( to_upper_adjacent_price ( 190 ), 191 );
            assert_eq ! ( to_upper_adjacent_price ( 199 ), 200 );
            assert_eq ! ( to_upper_adjacent_price ( 200 ), 202 );
            assert_eq ! ( to_upper_adjacent_price ( 396 ), 398 );
            assert_eq ! ( to_upper_adjacent_price ( 398 ), 400 );
            assert_eq ! ( to_upper_adjacent_price ( 400 ), 402 );
            assert_eq ! ( to_upper_adjacent_price ( 496 ), 498 );
            assert_eq ! ( to_upper_adjacent_price ( 498 ), 500 );
            assert_eq ! ( to_upper_adjacent_price ( 500 ), 505 );
            assert_eq ! ( to_upper_adjacent_price ( 995 ), 1000 );
            assert_eq ! ( to_upper_adjacent_price ( 1000 ), 1010 );
            assert_eq ! ( to_upper_adjacent_price ( 2490 ), 2500 );
            assert_eq ! ( to_upper_adjacent_price ( 2500 ), 2525 );
            assert_eq ! ( to_upper_adjacent_price ( 9950 ), 9975 );
            assert_eq ! ( to_upper_adjacent_price ( 9975 ), 10000 );
            assert_eq ! ( to_upper_adjacent_price ( 10000 ), 10050 );
            assert_eq ! ( to_upper_adjacent_price ( 19950 ), 20000 );
            assert_eq ! ( to_upper_adjacent_price ( 20000 ), 20100 );
            assert_eq ! ( to_upper_adjacent_price ( 39900 ), 40000 );
            assert_eq ! ( to_upper_adjacent_price ( 40000 ), 40200 );
        }

        #[test]
        fn test_to_int_price ( ) {
            let f = 1.2345678f32;
            assert_eq ! ( to_int_price ( f, 0 ), 1 );
            assert_eq ! ( to_int_price ( f, 1 ), 12 );
            assert_eq ! ( to_int_price ( f, 2 ), 123 );
            assert_eq ! ( to_int_price ( f, 3 ), 1235 );
            assert_eq ! ( to_int_price ( f, 4 ), 12346 );
            assert_eq ! ( to_int_price ( f, 5 ), 123457 );
            assert_eq ! ( to_int_price ( f, 6 ), 1234568 );
            assert_eq ! ( to_int_price ( f, 7 ), 12345678 );
        }
    }

    pub mod dw {
        use async_trait::async_trait;
        use chrono::NaiveDate;
        use super::*;
        /*
        use std::pin::Pin;
        use futures::future::Future;
        */
        
        /// Trait of DW price table
        #[async_trait(?Send)]
        pub trait DWPriceTable {
            type UnderlyingType;
            type DWType;
            
            // // Returns next returning web content, pretending the connection to outside website.
            // #[cfg(test)]
            // fn mock_next_return(&self) -> String;
            
            // // Pushes queued returning web content.
            // #[cfg(test)]
            // fn mock_push_return(&self, retn: String);

            /*
            type TableResult = Result<HashMap<U, Vec<D>>, ( )>;
            type TableResult;
            */
            
            /// Returns the map to underlying-DW prices.get_latest_working_date_time()
            /// 
            /// # Arguments
            /// 
            /// * `underlying_symbol` - Underlying symbol
            //async fn get_underlying_dw_price_table ( dw_info: &dw::DWInfo ) -> Self::TableResult;
            async fn get_underlying_dw_price_table ( dw_info: &dw::DWInfo ) -> Result<HashMap<Self::UnderlyingType, Vec<Self::DWType>>, Error>;
            //async fn get_underlying_dw_price_table ( dw_info: &dw::DWInfo ) -> Option<HashMap<U, Vec<D>>>;
            //fn get_underlying_dw_price_table ( dw_info: &dw::DWInfo ) -> Pin<Box<dyn Future<Output = Result<HashMap<U, Vec<D>>, ()>> + Send>>;
            //fn get_underlying_dw_price_table ( dw_info: &dw::DWInfo ) -> dyn Future<Output = Option<HashMap<U, Vec<D>>>> + '_;
            //fn get_underlying_dw_price_table ( dw_info: &dw::DWInfo ) -> Option<HashMap<U, Vec<D>>>;
            
            // TODO
            //async fn get_underlying_dw_price_map <U, D> ( dw_info: &dw::DWInfo ) -> Result<Vec<UnderlyingDWPricePairList<U, D>>, Error>;
            
        }
        
        /*
        #[derive(fmt::Debug)]
        pub struct UnsupportedDWTableScraping {
            pub broker_id: u8,
        }
        
        fn f ( ) {
            UnsupportedDWTableScraping {
                broker_id: 1u8,
            };
        }
        
        impl std::error::Error for UnsupportedDWTableScraping{

         }
        
        impl fmt::Display for UnsupportedDWTableScraping {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write! ( f, "Unsupported DW table scraping: {}", self.broker_id )
            }
        }

        #[derive(fmt::Debug)]
        pub struct DataNotFound {
            pub symbol: String,
        }

        impl std::error::Error for DataNotFound {

        }
        
        impl fmt::Display for DataNotFound {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write! ( f, "Data not found: {}", self.symbol )
            }
        }

        #[derive(fmt::Debug)]
        pub struct FailedParsing {
            pub symbol: String,
        }
        
        impl std::error::Error for FailedParsing {

        }
        
        impl fmt::Display for FailedParsing {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write! ( f, "Failed to parse: {}", self.symbol )
            }
        }
        */

        #[derive(Debug, PartialEq, Snafu)]
        //#[derive(Debug)]
        pub enum Error {
            #[snafu(display("Data not found: {}", "symbol"))]
            DataNotFound{symbol: Box<str>, info: Option<String>},
            
            #[snafu(display("Failed to parse: {}", "symbol"))]
            FailedParsing{symbol: Box<str>, info: Option<String>},
            
            #[snafu(display("Unsupported DW table scraping: {}", "broker_id"))]
            UnsupportedDWTableScraping{broker_id: u8},
            
            #[snafu(display("Test"))]
            Test,
        }
            
        /// Underlying-DW price pair list, based on the date
        ///
        /// The key of each pair is the DW price that should be unique in the list.
        ///
        /// The pairs are sorted.
        // TODO
        struct UnderlyingDWPricePairList <U, D> {
            date: NaiveDate,
            pairs: Vec<(U, D)>,
        }

        /// DW Info from symbol
        /// 
        /// - underlying symbol. Up to 4 chars.
        /// - broker id. 2 chars.
        /// - side. 'C' or 'P'. See [DWSide]
        /// - expiration date. YYMM format.
        /// - series
        #[derive(PartialEq, Clone, Debug)]
        pub struct DWInfo {
            pub symbol: Box<str>,
            pub underlying_symbol: Box<str>,
            pub broker_id: u8,
            pub side: DWSide,
            pub expire_yymm: [u8; 4],
            pub series: char,
        }

        #[derive(PartialEq, Clone, Debug)]
        pub enum DWSide {
            C,
            P,
            Unknown,
        }

        impl DWInfo {

            /// Returns option of [DWInfo] by parsing given [symbol].
            /// 
            /// # Arguments
            /// 
            /// * `dw_symbol` - DW symbol to be parsed.
            pub fn from_str ( dw_symbol: &str ) -> Option<Self> {
                let regex = Regex::new ( r#"(\d{2})([CP])(\d{4})"# )
                    .expect ( "Failed to create Regex of DW symbol.");
                
                let captures = regex.captures ( dw_symbol );
                
                if let Some ( captures ) = captures {
                    let captured_broker_id = captures.get ( 0 ).unwrap ( );
                    if captured_broker_id.start ( ) <= 0 ||
                            captured_broker_id.end ( ) < dw_symbol.len ( ) - 1 {
                        return None;
                    }

                    //std::panic::catch_unwind ( || {
                        let mut expire = [0u8; 4];
                        expire.copy_from_slice(captures.get ( 3 ).unwrap ( ).as_str ( ).as_bytes() );

                        Some ( DWInfo {
                            symbol: dw_symbol.clone ( ).to_owned ( ).into_boxed_str ( ),
                            underlying_symbol: dw_symbol.get ( 0..captured_broker_id.start ( ) ).unwrap ( ).to_owned ( ).into_boxed_str ( ),
                            broker_id: captures.get ( 1 ).unwrap ( ).as_str ( ).parse::<u8> ( ).unwrap ( ),
                            side: match captures.get ( 2 ).unwrap ( ).as_str ( ) {
                                "C" => DWSide::C,
                                "P" => DWSide::P,
                                _ => DWSide::Unknown,
                            },
                            expire_yymm: expire,
                            series: dw_symbol.chars ( ).nth ( captured_broker_id.end ( ) ).unwrap ( ),
                        } )
                    //} ).unwrap_or ( None )
                } else {
                    None
                }
            }
        }
        
        #[async_trait(?Send)]
        impl DWPriceTable for DWInfo {
            type UnderlyingType = i32;
            type DWType = f32;

            async fn get_underlying_dw_price_table(dw_info: &Self) -> Result<HashMap<i32, Vec<f32>>, Error> {
                match dw_info.broker_id {
                    13  => dw13::DW13::get_underlying_dw_price_table(dw_info).await,
                    28  => dw28::DW28::get_underlying_dw_price_table(dw_info).await,
                    _   => Err ( Error::UnsupportedDWTableScraping { broker_id: dw_info.broker_id.into() } )
                }
            }
        }
        
        #[allow(non_snake_case)]
        #[cfg(test)]
        pub mod tests {
            use super::*;
            use crate::testing::*;
            use crate::reqwest_mock::HTML_MAP;
            use std::sync::Once;

            pub static BEFORE_ALL: Once = Once::new ( );

            pub fn setup ( ) {
                if ! BEFORE_ALL.is_completed() {
                    BEFORE_ALL.call_once( || {
                        let _ = env_logger::try_init ( );
                    } );
                }
            }

            macro_rules! to_fixed_u8_arr {
                ($s:expr, $sz:expr) => {
                    {
                        let mut out = [0u8; $sz];
                        out.copy_from_slice ( $s.as_bytes ( ) );
                        out
                    }
                };
            }
            
            #[tokio::test]
            async fn givenDW13Symbol_whenGetPriceTable_thenGotResultSameAsFromDW13Struct ( ) {
                setup ( );
                HTML_MAP.with ( |html_map| {
                    let mut result = html_map.borrow_mut ( );
                    // TODO: map the url to default value
                    // result.insert ( DW_LIST_URL!().to_string ( ).into_boxed_str(), target_list_html!().to_string ( ) );
                    result.insert ( "".to_string ( ).into_boxed_str(), "".to_string ( ) );
                } );

                assert_eq ! (
                    0u32,
                    test_count!(dw13)
                );
                assert_eq ! (
                    "".to_string(),
                    test_last_dw_symbol!(dw13)
                );
                test_count!(dw13, 0);
                    
                let symbol = "S5013P2109A";
                let dw_info = DWInfo::from_str ( symbol ).unwrap ( );
                
                let _ = DWInfo::get_underlying_dw_price_table(&dw_info).await;

                // assert!(result.is_err(), "{:?}", result.unwrap());
                
                assert_eq ! (
                    1u32,
                    test_count!(dw13)
                );
                assert_eq ! (
                    symbol.to_string(),
                    test_last_dw_symbol!(dw13)
                );
                test_count!(dw13, 0);
            }
            
            #[tokio::test]
            async fn givenDW28Symbol_whenGetPriceTable_thenGotResultSameAsFromDW28Struct ( ) {
                setup ( );
                HTML_MAP.with ( |html_map| {
                    let mut result = html_map.borrow_mut ( );
                    // TODO: map the url to default value
                    // use crate::dw28::{DW_LIST_URL, target_list_html};
                    // use crate::dw28::target_list_html;

                    // result.insert ( DW_LIST_URL!().to_string ( ).into_boxed_str(), target_list_html!().to_string ( ) );
                    result.insert ( "".to_string ( ).into_boxed_str(), "".to_string ( ) );
                } );

                assert_eq ! (
                    0u32,
                    test_count!(dw28)
                );
                assert_eq ! (
                    "".to_string(),
                    test_last_dw_symbol!(dw28)
                );
                test_count!(dw28, 0);
                    
                let symbol = "S5028P2109A";
                let dw_info = DWInfo::from_str ( symbol ).unwrap ( );
                
                let _ = DWInfo::get_underlying_dw_price_table(&dw_info).await;
                
                // assert!(result.is_ok());

                assert_eq ! (
                    1u32,
                    test_count!(dw28)
                );
                assert_eq ! (
                    symbol.to_string(),
                    test_last_dw_symbol!(dw28)
                );
                test_count!(dw28, 0);
            }
            
            #[tokio::test]
            async fn givenUnknownSymbol_whenGetPriceTable_thenGotErr ( ) {
                let dw_info = DWInfo {
                    symbol: "XX00C3333Z".to_owned ( ).into_boxed_str ( ),
                    underlying_symbol: "XX".to_owned ( ).into_boxed_str ( ),
                    broker_id: 0,
                    side: DWSide::C,
                    expire_yymm: to_fixed_u8_arr! ( "3333", 4 ),
                    series: 'Z',
                };
                
                let price_table = DWInfo::get_underlying_dw_price_table( &dw_info ).await;
                assert ! ( price_table.is_err() );
            }
            
            #[test]
            fn givenPutDWSymbol_whenFromStr_thenGotSomeDWInfo ( ) {
                assert_eq ! ( DWInfo::from_str ( "ABCD00P5678A" ),
                    Some ( DWInfo {
                        symbol: "ABCD00P5678A".to_owned ( ).into_boxed_str ( ),
                        underlying_symbol: "ABCD".to_owned ( ).into_boxed_str ( ),
                        broker_id: 0,
                        side: DWSide::P,
                        expire_yymm: to_fixed_u8_arr! ( "5678", 4 ),
                        series: 'A',
                    } )
                );
            }

            #[test]
            fn givenCallDWSymbol_whenFromStr_thenGotSomeDWInfo ( ) {
                assert_eq ! ( DWInfo::from_str ( "VVVV00C5678A" ),
                    Some ( DWInfo {
                        symbol: "VVVV00C5678A".to_owned ( ).into_boxed_str ( ),
                        underlying_symbol: "VVVV".to_owned ( ).into_boxed_str ( ),
                        broker_id: 0,
                        side: DWSide::C,
                        expire_yymm: to_fixed_u8_arr! ( "5678", 4 ),
                        series: 'A',
                    } )
                );
            }

            #[test]
            fn givenDWSymbolWithShortName_whenFromStr_thenGotSomeDWInfo ( ) {
                assert_eq ! ( DWInfo::from_str ( "CC00C2020A" ),
                    Some ( DWInfo {
                        symbol: "CC00C2020A".to_owned ( ).into_boxed_str ( ),
                        underlying_symbol: "CC".to_owned ( ).into_boxed_str ( ),
                        broker_id: 0,
                        side: DWSide::C,
                        expire_yymm: to_fixed_u8_arr! ( "2020", 4 ),
                        series: 'A',
                    } )
                );

                assert_eq ! ( DWInfo::from_str ( "XX00C3333Z" ),
                    Some ( DWInfo {
                        symbol: "XX00C3333Z".to_owned ( ).into_boxed_str ( ),
                        underlying_symbol: "XX".to_owned ( ).into_boxed_str ( ),
                        broker_id: 0,
                        side: DWSide::C,
                        expire_yymm: to_fixed_u8_arr! ( "3333", 4 ),
                        series: 'Z',
                    } )
                );
            }

            #[test]
            fn givenUnknownDWType_whenFromStr_thenNone ( ) {
                assert_eq ! ( DWInfo::from_str ( "AA00X5555Y" ),
                    None
                    // currently, no support for unknown type
                    /*
                    Some ( DWInfo {
                        symbol: "AA".to_owned ( ).into_boxed_str ( ),
                        broker_id: 0,
                        side: DWSide::Unknown,
                        expire_yymm: to_fixed_u8_arr! ( "5555", 4 ),
                        series: 'Y',
                    } )
                    */
                );
            }

            #[test]
            fn givenTooLongBrokerIdSize_whenFromStr_thenDWInfoAsExceedingIdIsAPartOfUnderlyingSymbol ( ) {
                // broker sz=1
                assert_eq ! ( DWInfo::from_str ( "WW333C0000Z" ),
                    Some ( DWInfo {
                        symbol: "WW333C0000Z".to_owned ( ).into_boxed_str ( ),
                        underlying_symbol: "WW3".to_owned ( ).into_boxed_str ( ),
                        broker_id: 33,
                        side: DWSide::C,
                        expire_yymm: to_fixed_u8_arr! ( "0000", 4 ),
                        series: 'Z',
                    } )
                );
            }

            #[test]
            fn givenTooShortBrokerIdSize_whenFromStr_thenNone ( ) {
                // broker sz=1
                assert_eq ! ( DWInfo::from_str ( "WW1C0000Z" ),
                    None
                );
            }

            #[test]
            fn givenNoBrokerIdSize_whenFromStr_thenNone ( ) {
                // broker sz=0
                assert_eq ! ( DWInfo::from_str ( "QQC0000Z" ),
                    None
                );
            }

            #[test]
            fn givenNoUnderlyingPart_whenFromStr_thenNone ( ) {
                // no underlying part
                assert_eq ! ( DWInfo::from_str ( "00C0000Z" ),
                    None
                );
            }

            #[test]
            fn givenTooShortExpiryDateSize_whenFromStr_thenNone ( ) {
                // expiry date size < 4
                assert_eq ! ( DWInfo::from_str ( "EE00C123Z" ),
                    None
                );
            }

            #[test]
            fn givenTooLongExpiryDateSize_whenFromStr_thenNone ( ) {
                // expiry date size > 4
                assert_eq ! ( DWInfo::from_str ( "EE00C12345Z" ),
                    None
                );
            }
        } // tests
    } // mod: dw

} // mod: instrument

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use chrono::NaiveDate;

    #[test]
    /// Test: get current date time
    fn test_get_latest_working_date_time ( ) {
        let now = Local::now ( ).naive_local ( );

        let result = get_latest_working_date_time();
        
        let gap = now - result;

        assert! ( gap < Duration::seconds ( 2 ) );
    }
    
    /// Returns generated current working datetime
    fn gen_working_day ( ) -> NaiveDate {
        let mut date = Local::now ( ).date_naive ( );

        // make sure it's working day
        match date.weekday ( ) {
            Weekday::Sat => date = date + Duration::days ( 2 ),
            Weekday::Sun => date = date + Duration::days ( 1 ),
            _ =>  ( )
        }
        
        date
    }

    #[test]
    /// Test: in current working day, Fri-Sun, after 16:30, get next Mon
    fn test_get_working_date_time_from_mon_to_thu_after_1630 () {
        let mut rand = rand::thread_rng();

        let mut datetime = gen_working_day().and_hms_opt ( 16, rand.gen_range (30..60), rand.gen_range (0..60) );
        assert!(datetime.is_some());
        let mut datetime = datetime.unwrap();

        match datetime.date().weekday() {
            Weekday::Fri => {
                datetime = datetime.with_day((datetime.date().day() + rand.gen_range(3..7)) % 7).unwrap ( );
            },
            Weekday::Sat => {
                datetime = datetime.with_day((datetime.date().day() + rand.gen_range(2..6)) % 7).unwrap ( );
            },
            Weekday::Sun => {
                datetime = datetime.with_day((datetime.date().day() + rand.gen_range(1..5)) % 7).unwrap ( );
            },
            _ => ( ),
        };

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_ne ! ( datetime, new_datetime );
        assert_eq ! ( datetime.date()+Duration::days(1), new_datetime.date() );
    }

    #[test]
    /// Test: in current working day, Mon-Thu, after 16:30, get current next date
    fn test_get_working_date_time_from_fri_to_sun_after_1630 () {
        let _ = env_logger::try_init();
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms_opt ( 16, rand.gen_range (30..60), rand.gen_range (0..60) );
        assert!(datetime.is_some());
        let mut datetime = datetime.unwrap();

        match datetime.date().weekday() {
            Weekday::Mon => {
                datetime = datetime.with_day((datetime.date().day() + rand.gen_range(4..7)) % 7 + 7).unwrap ( );
            },
            Weekday::Tue => {
                datetime = datetime.with_day((datetime.date().day() + rand.gen_range(3..6)) % 7 + 7).unwrap ( );
            },
            Weekday::Wed => {
                datetime = datetime.with_day((datetime.date().day() + rand.gen_range(2..5)) % 7 + 7).unwrap ( );
            },
            Weekday::Thu => {
                datetime = datetime.with_day((datetime.date().day() + rand.gen_range(2..4)) % 7 + 7).unwrap ( );
            },
            _ => ( ),
        };

        // find next Mon
        let days_to_mon = match datetime.date().weekday() {
            Weekday::Fri => 3,
            Weekday::Sat => 2,
            Weekday::Sun => 1,
            _ => 0,
        };

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_ne ! ( datetime, new_datetime );
        assert_eq ! ( datetime.date()+Duration::days(days_to_mon), new_datetime.date() );
    }

    #[test]
    /// Test: in current working day, 16:00 - 16:29, get current date time
    fn test_get_working_date_time_from_working_day_1600_to_1629 () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms_opt ( 16, rand.gen_range (0..30), rand.gen_range (0..60) );
        assert!(datetime.is_some());
        let datetime = datetime.unwrap();

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_eq ! ( datetime, new_datetime );
    }

    #[test]
    /// Test: in current working day, 00:00 - 16:00, get current date time
    fn test_get_working_date_time_from_working_day_before_1600 () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms_opt ( rand.gen_range ( 0..16 ), rand.gen_range (0..60), rand.gen_range (0..60) );
        assert!(datetime.is_some());
        let datetime = datetime.unwrap();

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_eq ! ( datetime, new_datetime );
    }

    #[test]
    /// Test: in current working day, after 16:30, get current next date
    fn test_get_working_date_time_from_saturday () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms_opt ( rand.gen_range(0..24), rand.gen_range (0..60), rand.gen_range (0..60) );
        assert!(datetime.is_some());
        let datetime = datetime.unwrap();
        let date = datetime.date();
        let offset = match date.weekday() {
            Weekday::Mon => 5,
            Weekday::Tue => 4,
            Weekday::Wed => 3,
            Weekday::Thu => 2,
            Weekday::Fri => 1,
            Weekday::Sun => -1,
            _ => 0,
        };
        let datetime = datetime + Duration::days ( offset );

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_ne ! ( datetime, new_datetime );
        assert_eq ! ( Weekday::Mon, new_datetime.date().weekday() );
    }

    #[test]
    /// Test: in current working day, 16:00 - 16:29, get current date time
    fn test_get_working_date_time_from_sunday () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms_opt ( rand.gen_range(0..24), rand.gen_range (0..60), rand.gen_range (0..60) );
        assert!(datetime.is_some());
        let datetime = datetime.unwrap();
        let date = datetime.date();
        let offset = match date.weekday() {
            Weekday::Mon => 6,
            Weekday::Tue => 5,
            Weekday::Wed => 4,
            Weekday::Thu => 3,
            Weekday::Fri => 2,
            Weekday::Sat => 1,
            _ => 0,
        };
        let datetime = datetime + Duration::days ( offset );

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_ne ! ( datetime, new_datetime );
        assert_eq ! ( Weekday::Mon, new_datetime.date().weekday() );
    }
}
