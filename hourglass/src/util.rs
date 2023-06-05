use chrono::{DateTime, Local, Utc};

pub mod date_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // let s = format!("{}", date.format(FORMAT));

        // we convert DateTime into a RFC3339 date because it is universal so other languages like Javascript can parse it
        serializer.serialize_str(&date.to_rfc3339())
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<DateTime<Utc>>().map_err(serde::de::Error::custom)
        // Utc.datetime_from_str(&s, FORMAT)
        //     .map_err(serde::de::Error::custom)
    }
}

pub fn format_time(from_dt: DateTime<Utc>, to_dt: DateTime<Utc>) -> String {
    let sec = (to_dt - from_dt).num_seconds();

    let year = 60 * 60 * 24 * 365;
    let month = 60 * 60 * 24 * 30;
    let week = 60 * 60 * 24 * 7;
    let day = 60 * 60 * 24;
    let hour = 60 * 60;
    let minute = 60;

    if sec >= 60 * 60 * 24 * 365 {
        return format!("{}y", sec / year);
    } else if sec >= 60 * 60 * 24 * 30 {
        return format!("{}y", sec / month);
    } else if sec >= 60 * 60 * 24 * 7 {
        return format!("{}w", sec / week);
    } else if sec >= 60 * 60 * 24 {
        return format!("{}d", sec / day);
    } else if sec >= 60 * 60 {
        return format!("{}h", sec / hour);
    } else if sec >= 60 {
        return format!("{}min", sec / minute);
    }

    format!("{}s", sec)
}

pub fn convert_utc_to_local(utc_time: DateTime<Utc>, time_format: &str) -> String {
    let local_time: DateTime<Local> = DateTime::from(utc_time);

    local_time.format(time_format).to_string()
}
