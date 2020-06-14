use dbus_async::{Binder, DBus};
use dbus_async_derive::Handler;
use dbus_message_parser::MessageHeader;
use futures::lock::Mutex;
use std::sync::Arc;

#[derive(Handler)]
#[interface(
    "org.example.dbus.object",
    method("Method", method),
    property("Property", "s", get_property = "get", set_property = "set")
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
    // Wrap it to a Arc<Mutex>
    let dbus_object_1 = Arc::new(Mutex::new(dbus_object));
    let object_path_1 = "/org/example/object_1";
    // Clone the Arc reference
    let dbus_object_2 = dbus_object_1.clone();
    let object_path_2 = "/org/example/object_2";
    // Bind the object to the first object path
    tokio::spawn(dbus_object_1.bind(dbus.clone(), object_path_1));
    // Bind the same object to the second object path
    dbus_object_2
        .bind(dbus, object_path_2)
        .await
        .expect("Something went wrong");
}
