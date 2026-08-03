mod tests {
    use jiff::civil::date;
    use regex::Regex;
    use ulid::Ulid;
    use uuid::{Timestamp, Uuid};

    #[test]
    fn ulid_ulid_test() {
        println!("ulid: {}", Ulid::generate());
        println!("ulid: {}", Ulid::generate());
        println!("ulid: {}", Ulid::generate());
        println!("ulid: {}", Ulid::generate());
        println!("ulid: {}", Ulid::generate());
        println!("ulid: {}", Ulid::generate());
    }

    #[test]
    fn uuid_test() {
        let enero = "01941f29-7c00-7b6a-862b-2a1ed79d8fa7";
        println!("1. enero: {}", enero);
        let uuid = Uuid::parse_str(enero).unwrap();
        println!("1. enero uuid7: {}", uuid);
        let seconds = uuid.get_timestamp().unwrap().to_unix();
        println!("1. enero uuid7.timestamp: {:?}", uuid.get_timestamp());
        println!(
            "1. enero uuid7.timestamp: {:?}",
            jiff::Timestamp::from_second(seconds.0 as i64)
        );
        println!("1. enero ulid: {}", Ulid::from_bytes(uuid.into_bytes()));
        println!();

        let enero = "01941f29-7c00-7000-0000-000000000000";
        println!("1. enero: {}", enero);
        let uuid = Uuid::parse_str(enero).unwrap();
        println!("1. enero uuid7: {}", uuid);
        let seconds = uuid.get_timestamp().unwrap().to_unix();
        println!("1. enero uuid7.timestamp: {:?}", uuid.get_timestamp());
        println!(
            "1. enero uuid7.timestamp: {:?}",
            jiff::Timestamp::from_second(seconds.0 as i64)
        );
        println!("1. enero ulid: {}", Ulid::from_bytes(uuid.into_bytes()));
        println!();
    }

    #[test]
    fn uuid_range_test() {
        fn uuid_for_month(year: i16, mes: i8) {
            let re = Regex::new(r"[\p{L}\p{N}]").unwrap();

            let zoned = date(year, mes, 1)
                .at(0, 0, 0, 0)
                .in_tz("America/Lima")
                .unwrap();
            let timestamp_sec = zoned.timestamp().as_second();
            let uuid7 = Uuid::new_v7(Timestamp::from_unix(
                uuid::NoContext,
                timestamp_sec as u64,
                0,
            ));
            let _inicio: String = uuid7.to_string().chars().take(15).collect();
            let _final: String = uuid7.to_string().chars().skip(15).collect();
            let _final2 = re.replace_all(&_final, "0");

            println!("{}: {} {}{}", mes, uuid7, _inicio, _final2);
        }

        uuid_for_month(2025, 1);
        uuid_for_month(2025, 2);
        uuid_for_month(2025, 3);
        uuid_for_month(2025, 4);
        uuid_for_month(2025, 5);
        uuid_for_month(2025, 6);
        uuid_for_month(2025, 7);
        uuid_for_month(2025, 8);
        uuid_for_month(2025, 9);
        uuid_for_month(2025, 10);
        uuid_for_month(2025, 11);
        uuid_for_month(2025, 12);
        uuid_for_month(2026, 1);
    }
}
