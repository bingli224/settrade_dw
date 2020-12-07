
/// # Settrade Underlying-DW Price Table Scraper
/// 
/// Scrape DW (derivative warrant) price table from official DW websites.
/// 
/// ## Supported DW
/// 
/// | DW # | Website |
/// | ---- | ---- |
/// | DW01 | https://www.blswarrant.com/ |
/// | DW13 | https://www.thaiwarrant.com/ |
/// | DW19 | http://dw19club.com/ |
/// | DW28 | https://www.thaidw.com/ |
/// 

use std::collections::HashMap;
use chrono::{
    Duration,
    Utc,
    DateTime,
    Timelike,
    Weekday,
    Datelike,
    Date,
};

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
pub fn get_latest_working_date_time ( ) -> DateTime<Utc> {
    get_working_date_time_from ( Utc::now ( ) )
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
/// * `datetime` - A DateTime<Utc> object as the base date/time.
pub fn get_working_date_time_from ( mut datetime: DateTime<Utc> ) -> DateTime<Utc> {

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

pub trait DWPriceTable <T> {
    fn get_underlying_dw_price_table ( ) -> HashMap<T, Vec<T>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    /// Test: get current date time
    fn test_get_latest_working_date_time ( ) {
        let now = Utc::now ( );

        let result = get_latest_working_date_time();
        
        let gap = now - result;

        assert_eq! ( gap < Duration::seconds ( 2 ), true );
    }
    
    fn gen_working_day ( ) -> Date<Utc> {
        //let mut datetime = Utc.ymd ( 2000, 1, 1 ).and_hms ( 16, rand.gen_range ( 0, 60 ), rand.gen_range ( 0, 60 ) );
        let mut date = Utc::today ( );

        // make sure it's working day
        match date.weekday() {
            Weekday::Sat => date = date + Duration::days ( 2 ),
            Weekday::Sun => date = date + Duration::days ( 1 ),
            _ => ()
        }
        
        date
    }

    #[test]
    /// Test: in current working day, after 16:30, get current next date
    fn test_get_working_date_time_from_working_day_after_1630 () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms ( 16, rand.gen_range (30, 60), rand.gen_range (0, 60) );

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_ne ! ( datetime, new_datetime );
        assert_eq ! ( datetime.date()+Duration::days(1), new_datetime.date() );
    }

    #[test]
    /// Test: in current working day, 16:00 - 16:29, get current date time
    fn test_get_working_date_time_from_working_day_1600_to_1629 () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms ( 16, rand.gen_range (0, 30), rand.gen_range (0, 60) );

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_eq ! ( datetime, new_datetime );
    }

    #[test]
    /// Test: in current working day, 00:00 - 16:00, get current date time
    fn test_get_working_date_time_from_working_day_before_1600 () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms ( rand.gen_range ( 0, 16 ), rand.gen_range (0, 60), rand.gen_range (0, 60) );

        let new_datetime = get_working_date_time_from( datetime );
        
        assert_eq ! ( datetime, new_datetime );
    }

    #[test]
    /// Test: in current working day, after 16:30, get current next date
    fn test_get_working_date_time_from_saturday () {
        let mut rand = rand::thread_rng();

        let datetime = gen_working_day().and_hms ( rand.gen_range(0, 24), rand.gen_range (0, 60), rand.gen_range (0, 60) );
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

        let datetime = gen_working_day().and_hms ( rand.gen_range(0, 24), rand.gen_range (0, 60), rand.gen_range (0, 60) );
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
