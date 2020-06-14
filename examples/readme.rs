use dbus_async::{Binder, DBus};
use dbus_async_derive::Handler;
use dbus_message_parser::MessageHeader;

#[derive(Handler)]
#[interface(
    "org.example.interface",
    method("ExampleMethod", method),
    property("ExampleProperty", "s", get_property = "get", set_property = "set")
)]
struct DBusObject {
    property: String,
}

impl DBusObject {
    async fn method(
        &mut self,
        dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<(), (String, String)> {
        // The code of the method
        println!(
            "The DBus socket where the message came from: {}",
            dbus.get_socket_path()
        );
        // ...
        Ok(())
    }

    async fn get_property(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<String, (String, String)> {
        Ok(self.property.clone())
    }

    async fn set_property(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        new_value: String,
    ) -> Result<(), (String, String)> {
        self.property = new_value;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let (dbus, _connection_join_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");
    // Create the object
    let dbus_object = DBusObject {
        property: "".to_string(),
    };
    let object_path = "/org/example/object/path";
    // Bind the same object to the second object path
    dbus_object
        .bind(dbus, object_path)
        .await
        .expect("Something went wrong");
}
