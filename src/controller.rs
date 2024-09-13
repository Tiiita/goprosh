use gopro_controller as gp_ctrl;

pub async fn discover() -> Vec<GoPro> {
    let mut central = gp_ctrl::init(None).await.unwrap();
    let devices = gp_ctrl::scan(&mut central).await.unwrap();

    devices
        .iter()
        .map(|name| GoPro::new(name.clone()))
        .collect()
}

pub struct GoPro {
    pub name: String,
    pub recording: bool,
}

impl GoPro {
    pub fn new(name: String) -> Self {
        GoPro {
            name,
            recording: false,
        }
    }

    pub fn connect(&self) -> &Self {
        println!("Unimplemented");
        &self
    }

    pub fn disconnect(&self) {
        println!("Unimplemented");
    }

    pub fn record(&self) {}
    pub fn stop_record(&self) {}
}
