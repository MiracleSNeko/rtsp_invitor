/// Self defined I/O macros
#[macro_export]
macro_rules! new_bufio {
    () => {{
        (std::io::stdin(), std::io::stdout(), String::new())
    }};
}

#[macro_export]
macro_rules! init_lockedio {
    ($cin: expr, $cout: expr) => {{
        (
            BufReader::new($cin.lock()).lines(),
            BufWriter::new($cout.lock()),
        )
    }};
}

#[macro_export]
macro_rules! getline {
    ($cin: expr) => {{
        $cin.next().unwrap()
    }};
}