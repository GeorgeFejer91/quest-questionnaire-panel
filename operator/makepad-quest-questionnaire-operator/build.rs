fn main() {
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("resources/vr-headset.ico");
        resource.set("FileDescription", "Quest Questionnaire Operator");
        resource.set("ProductName", "Quest Questionnaire Operator");
        resource
            .compile()
            .expect("failed to embed Windows app icon");
    }
}
