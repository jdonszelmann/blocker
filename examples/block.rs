use blocker::block;

async fn num() -> i64 {
    return 10;
}

pub fn main() {
    let f1 = num();
    let f2 = num();
    let f3 = num();

    assert_eq!(10, block(f2));
    assert_eq!(10, block(f1));
    assert_eq!(10, block(f3));
}
