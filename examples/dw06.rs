use settrade_dw::{
    self,
    instrument::dw,
    instrument::dw::DWPriceTable,
    instrument::dw::dw_price_table,
	dw06,
};

#[tokio::main]
pub async fn main() {
    let mut symbols: Vec<_> = std::env::args().skip(1).collect();
    
    if symbols.is_empty() {
        symbols = vec!["HSI06C2506A".to_string()];
		println!("Default symbol: {:?}", symbols);
    }

    if symbols.contains ( &"-".to_string() ) {
		// TEST cloudflare
		for symbol in symbols.iter() {
			test_cloudflare(symbol).await;
		}
	} else {
		for symbol in symbols.iter() {
			let dw_info = dw::DWInfo::from_str(&symbol).unwrap ( );
			println ! ( "{:?}", dw_info );
			let out = dw06::DW06::get_underlying_dw_price_table( &dw_info )
				.await;
			
			println ! ( "{:?}", out );
			if let Ok(rows) = out {
				let mut keys: Vec<&i32> = rows.keys().collect();
				keys.sort();
				for k in keys.into_iter() {
					println!("{}\t{:?}", k, rows.get(k).unwrap());
				}

				// DWPriceTable::<DWType=f32, UnderlyingType=i32>::get_sorted_dw_underlying_price_pairs::<i32> ( rows )
				println!("=TODAY====================");
				dw_price_table::get_sorted_dw_underlying_price_pairs ( rows.clone(), 0 )
					.await
					.unwrap()
					.iter()
					.for_each ( |(k, v)| {
						println!("{}\t:: {:?}", k, v);
					});
				println!("=DAY + 1==================");
				dw_price_table::get_sorted_dw_underlying_price_pairs ( rows.clone(), 1 )
					.await
					.unwrap()
					.iter()
					.for_each ( |(k, v)| {
						println!("{}\t:: {:?}", k, v);
					});
				println!("=DAY + 2==================");
				dw_price_table::get_sorted_dw_underlying_price_pairs ( rows.clone(), 2 )
					.await
					.unwrap()
					.iter()
					.for_each ( |(k, v)| {
						println!("{}\t:: {:?}", k, v);
					});
			}
		}
	}
}

async fn test_cloudflare(symbol: &str) {
	// reference: claude.ai

	use reqwest;
	use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, ACCEPT_ENCODING, DNT, CONNECTION, UPGRADE_INSECURE_REQUESTS};
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

	let resp = reqwest::Client::builder()
		.default_headers(headers)
		.use_rustls_tls()
		.redirect(reqwest::redirect::Policy::limited(10))
		.cookie_store(true)
		.build()
		.expect("Failed to build reqwest from reqwest::Client::builder()")
		.get(format!("https://dw06.kkpfg.com/en/detail/{}", symbol))
		.send()
		.await
		.unwrap()
		;
	println!("RES: {:?}", resp);
	println!("RES.status(): {:?}", resp.status());
	println!("RES.headers(): {:?}", resp.headers());
	// println!("{:?}", resp.text().await.unwrap());
	
	let content_encoding = resp.headers().get("content-encoding")
		.and_then(|h| Some(Some(h.to_str().unwrap().to_owned())))
		.unwrap_or(None)
		;
		
	let bytes = resp.bytes().await.unwrap();
	
	// let text = if let Some("br") = content_encoding {
	let text = if content_encoding.is_some() {
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
	
	println!("TEXT: {}", text);
}