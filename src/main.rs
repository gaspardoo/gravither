use gravither::{Gravither, UserId};

fn main() {
    let user_a = UserId::new("user-a");
    let user_b = UserId::new("user-b");

    let mut gravither = Gravither::two_users("user-a", "user-b");

    gravither
        .expose_information(&user_b, "Public information from user-b")
        .expect("user-b must exist");

    gravither
        .observe(&user_a, &user_b)
        .expect("user-a must be able to observe user-b");

    println!("Gravither — two-user connection prototype");
    println!("Active connection lines: {}", gravither.connection_lines().len());

    for flow in gravither.information_flows() {
        println!(
            "Information flow: {} -> {}",
            flow.source.as_str(),
            flow.observer.as_str()
        );
        for item in flow.information {
            println!("  exposed: {item}");
        }
    }
}
