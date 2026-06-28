pub enum Kind {
    Type,
    Function(Box<Kind>, Box<Kind>),
}
