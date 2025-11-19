
#[test]
fn create_clinic() {

    let a = 5;
    let mut b = 0;
    let c = "Clinic A".to_string();
    if a > 3 {
        println!("Value a: {}", a);
        b = test(a);
    }

    println!("Value a: {}", a);
    println!("Value b: {}", b);

    test2(c);

    assert_eq!(2 + 2, 4);
}

fn test(a: i32) -> i32 {
    a + 1
}

fn test2(a: String) {
    println!("Clinic name: {}", a);
}
