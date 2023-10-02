pub mod public {
    pub const HOME: &str = "/";
    pub const ABOUT: &str = "/about";
    pub const PRIVATES_ROOT: &str = "/p";
    pub const PRIVATES: &str = "/p/*";
    pub const NOT_FOUND: &str = "/*any";
}

pub mod private {
    pub const PRIVATE: &str = "/p";
    pub const HISTORIES: &str = "histories";
    pub const HISTORY_DETAIL: &str = "histories/:id";
    // pub const NOT_FOUND: &str = "/p";
    pub const NOT_FOUND2: &str = "/p/*";
}