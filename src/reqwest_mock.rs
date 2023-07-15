
//#[cfg(test)]
use std::{io::Error, sync::Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;
use log::{
    debug,
};

lazy_static ! {
    /// URL-to-HTML-result map for internet mock
    pub static ref HTML_MAP : Mutex<HashMap<Box<str>, String>> = Mutex::new ( HashMap::<Box<str>, String>::new ( ) );
}


/*
// reqwest currently requires tokio 0.2, so disable futures
use futures::future::{
    self,
    Future,
};
*/
pub struct Client {
}

impl Default for Client {
    fn default ( ) -> Self {
        Client {
        }
    }
}

impl Client {
    pub fn new ( ) -> Self {
        debug ! ( "Client::new()" );
        Client::default ( )
    }
    
    pub fn get ( self, url: &str ) -> RequestBuilder {
        debug ! ( "Client.get({})", url );
        RequestBuilder {
            url: url.to_string(),
        }
    }
}

pub struct RequestBuilder {
    url: String,
}

impl RequestBuilder {
    pub fn header ( self, _key: &str, _value: &str ) -> Self {
        debug ! ( "RequestBuilder.header()" );
        self
    }

    pub async fn send ( self ) -> Result<Response, std::io::Error> {
        debug ! ( "RequestBuilder.send()" );
        let html_map = HTML_MAP.lock ( ).unwrap ( );
        if let Some ( result ) = html_map.get ( self.url.as_str ( ) ) {
            debug ! ( "RequestBuilder.send(): Found URL: {}", self.url );
            Ok ( Response {
                result: result.to_string ( ),
            } )
        } else if let Some ( result ) = html_map.get ( "" ) {
            // default
            debug ! ( "RequestBuilder.send(): Not found URL: {}", self.url );
            Ok ( Response {
                result: result.to_string ( ),
            } )
        } else {
            Err ( Error::new ( std::io::ErrorKind::Other, format ! ( "Mock 404: {}", self.url ) ) )
        }
    }
}
pub struct Response {
    result: String,
}

impl Response {
    pub async fn text ( self ) -> Result<String, std::io::Error> {
        debug ! ( "Response.text()" );
        Ok ( self.result )
    }
}

pub trait MockHtmlResult {
    fn target_html ( &self ) -> String;
}

/*
pub mod blocking {
    use super::*;

    pub struct Client { }
    
    impl Default for Client {
        fn default ( ) -> Self {
            Client { }
        }
    }
    
    impl Client {
        pub fn new ( ) -> Self {
            debug ! ( "Client::new()" );
            Client::default ( )
        }
        
        pub fn get ( self, url: &str ) -> RequestBuilder {
            debug ! ( "Client.get({})", url );
            RequestBuilder {
                url: url.to_string(),
            }
        }
    }
    
    pub struct RequestBuilder {
        url: String,
    }

    impl RequestBuilder {
        pub fn header ( self, _key: &str, _value: &str ) -> Self {
            debug ! ( "RequestBuilder.header()" );
            self
        }

        pub fn send ( self ) -> Result<Response, std::io::Error> {
            debug ! ( "RequestBuilder.send()" );
            let html_map = HTML_MAP.lock ( ).unwrap ( );
            if let Some ( result ) = html_map.get ( self.url.as_str ( ) ) {
                debug ! ( "RequestBuilder.send(): Found URL: {}", self.url );
                Ok ( Response {
                    result: result.to_string ( ),
                } )
            } else if let Some ( result ) = html_map.get ( "" ) {
                // default
                debug ! ( "RequestBuilder.send(): Not found URL: {}", self.url );
                Ok ( Response {
                    result: result.to_string ( ),
                } )
            } else {
                Err ( Error::new ( std::io::ErrorKind::Other, "Mock 404" ) )
            }
        }
    }
    pub struct Response {
        result: String,
    }
    
    impl Response {
        pub fn text ( self ) -> Result<String, std::io::Error> {
            debug ! ( "Response.text()" );
            Ok ( self.result )
        }
    }
    
    pub trait MockHtmlResult {
        fn target_html ( &self ) -> String;
    }
}
*/