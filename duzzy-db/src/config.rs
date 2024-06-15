pub struct ConnConfig {
    name: String,
    host: String,
    port: u32,
    db: Option<String>,
    user: String,
    password: Option<String>,
}

#[derive(Default)]
pub struct ConnConfigBuilder {
    name: Option<String>,
    host: Option<String>,
    port: Option<u32>,
    db: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl ConnConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u32) -> Self {
        self.port = Some(port);
        self
    }

    pub fn db(mut self, db: String) -> Self {
        self.db = Some(db);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    pub fn build(self) -> ConnConfig {
        self.into()
    }
}

impl From<ConnConfigBuilder> for ConnConfig {
    fn from(builder: ConnConfigBuilder) -> Self {
        let host = builder.host.unwrap_or("localhost".to_owned());
        let port = builder.port.unwrap_or(5432);
        let user = builder.user.unwrap_or("postgres".to_owned());
        let name = builder
            .name
            .unwrap_or_else(|| format!("{}@{}:{}", &user, &host, &port));

        Self {
            name,
            host,
            port,
            user,
            db: builder.db,
            password: builder.password,
        }
    }
}
