#[cfg(windows)]
fn main() {
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("assets/peripersonal-retrofuturistic-vr.ico");
    resource.set("FileDescription", "Peripersonal Quest Questionnaire Operator");
    resource.set("ProductName", "Peripersonal VR Operator");
    resource
        .compile()
        .expect("failed to embed Windows operator icon");
}

#[cfg(not(windows))]
fn main() {}
