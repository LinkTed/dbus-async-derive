# dbus-async-derive
`dbus-async-derive` is a proc derive macro to implement the `dbus_async::Handler`.
This crate should be used to create a DBus service.
[![Latest version](https://img.shields.io/crates/v/dbus-async-derive.svg)](https://crates.io/crates/dbus-derive-derive)
[![License](https://img.shields.io/crates/l/dbus-async-derive.svg)](https://opensource.org/licenses/BSD-3-Clause)

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
dbus-async-derive = "1.0"
async-trait = "0.1"
```

## Example
The following example show how to create a DBus sevice with the interface `org.example.interface`.
This interface has a method `ExampleMethod` and a property `ExampleProperty`.
The object is avaiable at `/org/example/object/path`.
```rust
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
```

