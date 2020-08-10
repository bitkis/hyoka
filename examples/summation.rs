extern crate hyoka;

pub fn main() {
    let mut repl = hyoka::Repl::new("summation>", 0, |x: &mut i32, y: String| {
        let y = {
            let mut y = y;
            y.pop();
            y
        };
        let y = y
            .parse::<i32>()
            .expect(&format!("Cannot convert '{}' into an integer", y));
        *x += y;
        Some(format!("x:{}", x))
    });
    repl.run();
}
