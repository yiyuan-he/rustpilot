use std::io::{self, Write};

const MODELS: &[(&str, &str)] = &[
    ("claude-opus-4-1-20250805", "opus 4.1 (best)"),
    ("claude-opus-4-20250514", "opus 4"),
    ("claude-sonnet-4-20250514", "sonnet 4"),
    ("claude-3-7-sonnet-20250219", "sonnet 3.7"),
    ("claude-3-5-haiku-20241022", "haiku 3.5 (fast)"),
    ("claude-3-haiku-20240307", "haiku 3"),
];

pub fn select_model() -> String {
    println!("\nmodel:");
    for (i, (_, name)) in MODELS.iter().enumerate() {
        println!("{}: {}", i + 1, name);
    }
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let idx = input
        .trim()
        .parse::<usize>()
        .ok()
        .and_then(|n| n.checked_sub(1))
        .and_then(|i| MODELS.get(i))
        .unwrap_or(&MODELS[0]);

    println!();
    idx.0.to_string()
}
