error_chain! {
    foreign_links {
        Io(::std::io::Error) #[cfg(unix)];
    }

    errors {
        FormatError(t: String) {
            description("invalid file format")
            display("invalid file format: '{}'", t)
        }
        EncodingError {
            description("encoding error")
            display("encoding error")
        }
    }
}
