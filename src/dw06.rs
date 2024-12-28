
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

use serde::{Deserializer, Deserialize};
use serde_json;

#[cfg(test)]
#[allow(unused_imports)]
use log::debug;

#[cfg(test)]
use env_logger;

#[cfg(not(test))]
use crate::get_latest_working_date_time;

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
fn get_latest_working_date_time ( ) -> DateTime<Local> {
    DateTime::from_str ("2024-07-04T12:00:00-00:00").unwrap ( )
}

#[cfg(test)]
use chrono::{
    DateTime,
    Local,
};

#[cfg(test)]
macro_rules! target_json {
    ($name: expr) => {
        match $name {
            "PUT" => std::fs::read_to_string( "tests/dw06/dw06_HSI06P2408A_20240704_GetCalculator.json" ).expect ( "Failed to open file" ),
            "FAIL" => std::fs::read_to_string( "tests/dw06/dw06_404_20240704_GetCalculator.json" ).expect ( "Failed to open file" ),
            "CALL" => std::fs::read_to_string( "tests/dw06/dw06_HSI06C2408F_20240704_GetCalculator.json" ).expect ( "Failed to open file" ),
            _ => panic ! ("Unknown targeted file"),
        }
    };
}

#[cfg(test)]
use crate::reqwest_mock::HTML_MAP;
#[cfg(test)]
async fn client_get(url: &str) -> JsonData {
        use crate::reqwest_mock::Client;
        let t = Client::new ( )
            .get(url)
            .send()
            .await
            .expect ( "Failed to connect to dw06.kkpfg.com" )
            // .json::<JsonData> ( )
            // .await

            .text()
            .await
            .unwrap()
            ;
            
        if t.contains("<html") {
            panic!("DEBUG JSON: wrong data? url={}\n\t{}[..]", &url, &t[..64])
        } else {
            debug!("DEBUG JSON: CORRECT JSON: url={}", &url)
        }

        match serde_json::from_str(&t)
            {
                Err(e) => {
                    eprintln!("ERRORRRRR: data={}", t);
                    Err(e)
                },
                Ok(d) => Ok(d)
            }
            .unwrap()
            // .expect ( format ! ( "Failed to get data from dw06.kkpfg.com in json format: url={}",
            //     url
            // ).as_str ( ) )
            // .unwrap()
}
#[cfg(test)]
macro_rules ! client_get {
    ($url:expr) => { {
        use crate::reqwest_mock::Client;
        Client::new ( )
            .get($url)
            .send()
            .await
            .expect ( "Failed to connect to dw06.kkpfg.com" )
            .json::<JsonData> ( )
            .await
            .expect ( format ! ( "Failed to get data from dw06.kkpfg.com in json format: url={}",
                $url
                // {
                //     let res = Client::new().get(DW_PRICE_TABLE_URL!(dw_info.symbol).as_str()).send().await.unwrap().text().await.unwrap();
                //     if res.len() > 500 {
                //         (res[..500]).to_string()
                //     } else {
                //         res
                //     }
                // }
            ).as_str ( ) )
    } }
}
#[cfg(not(test))]
async fn client_get (url: &str) -> JsonData {
        use reqwest::{
            header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, DNT, CONNECTION, UPGRADE_INSECURE_REQUESTS},
            redirect,
            Client,
        };
        use log::debug;
        use brotli::Decompressor;
        use std::io::Read;
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
        headers.insert(DNT, HeaderValue::from_static("1"));
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));

        let resp = Client::builder()
            .default_headers(headers)
            .use_rustls_tls()
            .redirect(redirect::Policy::limited(10))
            .cookie_store(true)
            .build()
            .expect("Failed to build reqwest from reqwest::Client::builder()")
            .get(url)
            .send()
            .await
            .unwrap()
            ;
        debug!("RES: {:?}", resp);
        debug!("RES.status(): {:?}", resp.status());
        debug!("RES.headers(): {:?}", resp.headers());
        // debug!("{:?}", resp.text().await.unwrap());
        
        let content_encoding = resp.headers().get("content-encoding")
            .and_then(|h| Some(Some(h.to_str().unwrap().to_owned())))
            .unwrap_or(None)
            ;
            
        let bytes = resp.bytes().await.unwrap();
        
        // let text = if let Some("br") = content_encoding {
        let text = if content_encoding.is_some() && content_encoding == Some("br".to_owned()) {
            let mut decompressor = Decompressor::new(&bytes[..], bytes.len());
            let mut dec = Vec::new();
            decompressor.read_to_end(&mut dec)
                .unwrap()
                ;
            String::from_utf8(dec)
                .unwrap()
        } else {
            String::from_utf8(bytes.to_vec())
                .unwrap()
        };
        
        use serde_json;
        serde_json::from_str(&text)
            .expect(format!("Failed to parse as JSON: {}", text).as_str())
}
#[cfg(not(test))]
macro_rules ! client_get {
    ($url:expr) => { {
        use reqwest::{
            header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, DNT, CONNECTION, UPGRADE_INSECURE_REQUESTS},
            redirect,
            Client,
        };
        use brotli::Decompressor;
        use std::io::Read;
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.5"));
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"));
        headers.insert(DNT, HeaderValue::from_static("1"));
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
        headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));

        let resp = Client::builder()
            .default_headers(headers)
            .use_rustls_tls()
            .redirect(redirect::Policy::limited(10))
            .cookie_store(true)
            .build()
            .expect("Failed to build reqwest from reqwest::Client::builder()")
            .get($url)
            .send()
            .await
            .unwrap()
            ;
        debug!("RES: {:?}", resp);
        debug!("RES.status(): {:?}", resp.status());
        debug!("RES.headers(): {:?}", resp.headers());
        // debug!("{:?}", resp.text().await.unwrap());
        
        let content_encoding = resp.headers().get("content-encoding")
            .and_then(|h| Some(Some(h.to_str().unwrap().to_owned())))
            .unwrap_or(None)
            ;
            
        let bytes = resp.bytes().await.unwrap();
        
        // let text = if let Some("br") = content_encoding {
        let text = if content_encoding.is_some() && content_encoding == Some("br".to_owned()) {
            let mut decompressor = Decompressor::new(&bytes[..], bytes.len());
            let mut dec = Vec::new();
            decompressor.read_to_end(&mut dec)
                .unwrap()
                ;
            String::from_utf8(dec)
                .unwrap()
        } else {
            String::from_utf8(bytes.to_vec())
                .unwrap()
        };
        
        use serde_json;
        serde_json::from_str(&text)
            .expect(format!("Failed to parse as JSON: {}", text).as_str())
	
    } }
}

use std::{
    collections::HashMap,
};

#[cfg(test)]
macro_rules! target_html_compressed_s50_call {
    () => {
        std::fs::read_to_string( "tests/dw28/dw28_S5028C2012D_20201223.html" )
            .expect ( "Failed to open file [dw28_S5028C2012D]" )
    };
}

pub struct DW06;

macro_rules! DW_PRICE_TABLE_URL {
    ($symbol:expr, $price:expr) => {
        format ! ( "https://dw06.kkpfg.com/DW/GetCalculator?lang=en&dwCode={symbol}&underlyCalPrice={price}", symbol=$symbol, price=$price )
    };

    ($symbol:expr) => {
        format ! ( "https://dw06.kkpfg.com/DW/GetCalculator?lang=en&dwCode={symbol}&underlyCalPrice=0", symbol=$symbol )
    };
}

#[async_trait(?Send)]
impl DWPriceTable for DW06 {
    type UnderlyingType = i32;
    type DWType = f32;

    //type TableResult = Result<HashMap<i32, Vec<f32>>, ( )>;

    async fn get_underlying_dw_price_table ( dw_info: &DWInfo ) -> Result<HashMap<i32, Vec<f32>>, Error> {
        use log::debug;

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
        
        let url = DW_PRICE_TABLE_URL ! ( dw_info.symbol );

        let table: JsonData =
            client_get(url.as_str())
.await // testing to replace macro with fn to find error line for testing
            ;

        let mut u_dw_price_map = HashMap::<i32,Vec<f32>>::new ( );
        let table = table.data;
        if table.is_none () {
            return Ok ( u_dw_price_map );
        }
        let table = table.unwrap().dw_price_matrix_table;
        for row in table.bid_rows.into_iter ( ) {
            
            let dws = vec![
                row.bid_t1,
                row.bid_t2,
                row.bid_t3,
                row.bid_t4,
                row.bid_t5,
            ];
            
            u_dw_price_map.insert ( ( ( row.underly_bid_offer * 100.0 ).round ( ) ) as i32, dws );
        }
         
        Ok ( u_dw_price_map )
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
    
    // #[cfg(test)]
    // fn mock_next_return(&self) -> String {
    //     unimplemented!();
    //     String::new()
    // }

    // #[cfg(test)]
    // fn mock_push_return(&self, retn: String) {
    //     unimplemented!();
    // }
}

#[derive(Deserialize, Debug)]
struct JsonData {
    #[serde(rename = "ResponseCode")]
    response_code: u32,

    #[serde(rename = "Data", deserialize_with = "deserialize_null_or_none")]
    data: Option<Data>,
}

#[derive(Deserialize, Debug)]
struct Data {
    #[serde(rename = "DWCode")]
    dw_code: String,
    #[serde(rename = "UnderlyDisplay")]
    underly_display: String,
    #[serde(rename = "DwPriceMatrixTable")]
    dw_price_matrix_table: DwPriceMatrixTable,
}

#[derive(Deserialize, Debug)]
struct DwPriceMatrixTable {
    #[serde(rename = "BidRows")]
    bid_rows: Vec<UnderlyingDwRow>,
}

#[derive(Deserialize, Debug)]
struct UnderlyingDwRow {
    #[serde(rename = "UnderlyBidOffer")]
    underly_bid_offer: f32,
    #[serde(rename = "BidT1", deserialize_with = "deserialize_null_or_0f32")]
    bid_t1: f32,
    #[serde(rename = "BidT2", deserialize_with = "deserialize_null_or_0f32")]
    bid_t2: f32,
    #[serde(rename = "BidT3", deserialize_with = "deserialize_null_or_0f32")]
    bid_t3: f32,
    #[serde(rename = "BidT4", deserialize_with = "deserialize_null_or_0f32")]
    bid_t4: f32,
    #[serde(rename = "BidT5", deserialize_with = "deserialize_null_or_0f32")]
    bid_t5: f32,
}

/// Deserialize value from json into Some<Data>. If the original data is "null", then returns None
fn deserialize_null_or_none<'de, D>(de: D) -> Result<Option<Data>, D::Error>
where
    D: Deserializer<'de>,
{
    let k = Option::<Data>::deserialize(de)?;
    Ok(k)
}

/// Deserialize value from json into f32. If the original data is "null", then returns 0.0f32
fn deserialize_null_or_0f32<'de, D>(de: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let k = Option::<f32>::deserialize(de)?;
    Ok(k.unwrap_or(0f32))
}

#[cfg(test)]
use crate::testing::{gen_mock, test_count, test_last_dw_symbol};
#[cfg(test)]
gen_mock!(dw06);

#[cfg(test)]
pub mod dw06_tests {
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
    pub async fn test_get_underlying_dw_price_table_call ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( "https://dw06.kkpfg.com/DW/GetCalculator?lang=en&dwCode=DW06C2408F&underlyCalPrice=0".to_owned ( ).into_boxed_str ( ), target_json!("CALL").to_owned ( ) );
        } );
        
        let out = DW06::get_underlying_dw_price_table(& DWInfo::from_str ( "DW06C2408F" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );

        // check details
        assert_eq ! ( table.keys ( ).len ( ), 41 );
        for underlying_key in ( 1645000i32..=1745000i32 ).step_by ( 2500 ) {
            assert ! ( table.contains_key ( &underlying_key ), "Not found underlying [{}] in table", underlying_key );
            
            let dw_list = table.get ( &underlying_key );
            assert ! ( dw_list.is_some ( ) );
            let dw_list = dw_list.unwrap ( );
            assert_eq ! ( dw_list.len ( ), 5usize );
            
            // debug!("{:?}", dw_list);

            match dw_list [ 0 ] {
                v if v == 0.01 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1655000 ),
                v if v == 0.02 => assert ! ( underlying_key >= 1657500 && underlying_key <= 1675000 ),
                v if v == 0.03 => assert ! ( underlying_key >= 1677500 && underlying_key <= 1687500 ),
                v if v == 0.04 => assert ! ( underlying_key >= 1690000 && underlying_key <= 1700000 ),
                v if v == 0.05 => assert ! ( underlying_key >= 1702500 && underlying_key <= 1707500 ),
                v if v == 0.06 => assert ! ( underlying_key >= 1710000 && underlying_key <= 1717500 ),
                v if v == 0.07 => assert ! ( underlying_key >= 1720000 && underlying_key <= 1725000 ),
                v if v == 0.08 => assert ! ( underlying_key >= 1727500 && underlying_key <= 1730000 ),
                v if v == 0.09 => assert ! ( underlying_key >= 1732500 && underlying_key <= 1737500 ),
                v if v == 0.10 => assert ! ( underlying_key >= 1740000 && underlying_key <= 1742500 ),
                v if v == 0.11 => assert ! ( underlying_key >= 1745000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }

            match dw_list [ 1 ] {                
                v if v == 0.01 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1662500 ),
                v if v == 0.02 => assert ! ( underlying_key >= 1665000 && underlying_key <= 1682500 ),
                v if v == 0.03 => assert ! ( underlying_key >= 1685000 && underlying_key <= 1695000 ),
                v if v == 0.04 => assert ! ( underlying_key >= 1697500 && underlying_key <= 1705000 ),
                v if v == 0.05 => assert ! ( underlying_key >= 1707500 && underlying_key <= 1715000 ),
                v if v == 0.06 => assert ! ( underlying_key >= 1717500 && underlying_key <= 1722500 ),
                v if v == 0.07 => assert ! ( underlying_key >= 1725000 && underlying_key <= 1730000 ),
                v if v == 0.08 => assert ! ( underlying_key >= 1732500 && underlying_key <= 1737500 ),
                v if v == 0.09 => assert ! ( underlying_key >= 1740000 && underlying_key <= 1742500 ),
                v if v == 0.10 => assert ! ( underlying_key >= 1745000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }

            match dw_list [ 2 ] {
                v if v == 0.00 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }

            match dw_list [ 3 ] {
                v if v == 0.00 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }
            
            match dw_list [ 4 ] {
                v if v == 0.00 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }
        }
    }

    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table_put ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( "https://dw06.kkpfg.com/DW/GetCalculator?lang=en&dwCode=DW06P2408A&underlyCalPrice=0".to_owned ( ).into_boxed_str ( ), target_json!("PUT").to_owned ( ) );
        } );
        
        let out = DW06::get_underlying_dw_price_table(& DWInfo::from_str ( "DW06P2408A" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );

        // check details
        assert_eq ! ( table.keys ( ).len ( ), 41 );
        for underlying_key in ( 1645000i32..=1745000i32 ).step_by ( 2500 ) {
            assert ! ( table.contains_key ( &underlying_key ), "Not found underlying [{}] in table", underlying_key );
            
            let dw_list = table.get ( &underlying_key );
            assert ! ( dw_list.is_some ( ) );
            let dw_list = dw_list.unwrap ( );
            assert_eq ! ( dw_list.len ( ), 5usize );
            
            // debug!("{:?}", dw_list);

            match dw_list [ 0 ] {
                v if v == 0.08 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1650000 ),
                v if v == 0.07 => assert ! ( underlying_key >= 1652500 && underlying_key <= 1660000 ),
                v if v == 0.06 => assert ! ( underlying_key >= 1662500 && underlying_key <= 1672500 ),
                v if v == 0.05 => assert ! ( underlying_key >= 1675000 && underlying_key <= 1685000 ),
                v if v == 0.04 => assert ! ( underlying_key >= 1687500 && underlying_key <= 1700000 ),
                v if v == 0.03 => assert ! ( underlying_key >= 1702500 && underlying_key <= 1720000 ),
                v if v == 0.02 => assert ! ( underlying_key >= 1722500 && underlying_key <= 1745000 ),
                v @ _ => panic ! ( "Not found DW: {}", v )
            }

            match dw_list [ 1 ] {                
                v if v == 0.07 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1652500 ),
                v if v == 0.06 => assert ! ( underlying_key >= 1655000 && underlying_key <= 1665000 ),
                v if v == 0.05 => assert ! ( underlying_key >= 1667500 && underlying_key <= 1677500 ),
                v if v == 0.04 => assert ! ( underlying_key >= 1680000 && underlying_key <= 1692500 ),
                v if v == 0.03 => assert ! ( underlying_key >= 1695000 && underlying_key <= 1710000 ),
                v if v == 0.02 => assert ! ( underlying_key >= 1712500 && underlying_key <= 1737500 ),
                v if v == 0.01 => assert ! ( underlying_key >= 1740000 && underlying_key <= 1745000 ),
                v @ _ => panic ! ( "Not found DW: {}", v )
            }

            match dw_list [ 2 ] {
                v if v == 0.00 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }

            match dw_list [ 3 ] {
                v if v == 0.00 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }
            
            match dw_list [ 4 ] {
                v if v == 0.00 => assert ! ( underlying_key >= 1645000 && underlying_key <= 1745000 ),
                _ => panic ! ( )
            }
        }
    }

    #[tokio::test]
    pub async fn test_get_underlying_dw_price_table_not_found ( ) {
        setup ( );
        HTML_MAP.with ( |html_map| {
            let mut result = html_map.borrow_mut ( );
            // let mut result = HTML_MAP
            //     .lock ( )
            //     .unwrap ( );
            result.insert ( "".to_owned ( ).into_boxed_str ( ), target_json!("FAIL").to_owned ( ) );
        } );
        
        let out = DW06::get_underlying_dw_price_table(& DWInfo::from_str ( "XX06C0000X" ).unwrap ( ) )
            .await;
        
        assert ! ( out.is_ok ( ) );
        
        let table = out.unwrap ( );

        // check details
        assert_eq ! ( table.keys ( ).len ( ), 0 );
    }
}