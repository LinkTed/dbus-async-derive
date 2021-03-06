# dbus-async-derive
`dbus-async-derive` is a proc derive macro to implement the `dbus_async::Handler`.
This crate should be used to create a DBus service.
[![Latest version](https://img.shields.io/crates/v/dbus-async-derive.svg)](https://crates.io/crates/dbus-derive-derive)
[![License](https://img.shields.io/crates/l/dbus-async-derive.svg)](https://opensource.org/licenses/BSD-3-Clause)

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
dbus-async-derive = "2.0"
dbus-async = "2.0"
dbus-message-parser = "3.1"
async-trait = "0.1"
```

## Example
The following example show how to create a DBus sevice with the interface `org.example.interface`.
This interface has a method `ExampleMethod` and a property `ExampleProperty`.
The object is avaiable at `/org/example/object/path`.
```rust
use dbus_async::{Binder, DBus};
use dbus_async_derive::Handler;
use dbus_message_parser::{Error, MessageHeader};
use std::convert::TryInto;

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
    ) -> Result<(), (Error, String)> {
        // The code of the method
        println!(
            "The DBus socket where the message came from: {}",
            dbus.get_address()
        );
        // ...
        Ok(())
    }

    async fn get_property(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<String, (Error, String)> {
        Ok(self.property.clone())
    }

    async fn set_property(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        new_value: String,
    ) -> Result<(), (Error, String)> {
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
    let object_path = "/org/example/object/path".try_into().unwrap();
    // Bind the same object to the second object path
    dbus_object
        .bind(dbus, object_path)
        .await
        .expect("Something went wrong");
}
```

## DBus :left_right_arrow:  Rust type
The following table show how the type conversion works:
| Name                                     | DBus       | Rust                       |
|------------------------------------------|------------|----------------------------|
| Byte                                     | `y`        | `u8`                       |
| Boolean                                  | `b`        | `bool`                     |
| Signed 16-bit integer                    | `n`        | `i16`                      |
| Unsigned 16-bit integer                  | `q`        | `u16`                      |
| Signed 32-bit integer                    | `i`        | `i32`                      |
| Unsigned 32-bit integer                  | `u`        | `u32`                      |
| Signed 32-bit integer                    | `x`        | `i64`                      |
| Unsigned 32-bit integer                  | `t`        | `u64`                      |
| IEEE 754 double-precision floating point | `d`        | `f64`                      |
| Unsigned 32-bit integer file descriptor  | `h`        | `std::os::unix::io::RawFd` |
| String                                   | `s`        | `String`                   |
| Object Path                              | `o`        | `String`                   |
| Signature                                | `o`        | `String`                   |
| Array                                    | `aT`       | `Vec<T>`                   |
| Struct                                   | `(T1T2..)` | `(T1, T2, ..)`             |
| Dict-Entry                               | `{T1T2}`   | `(T1, T2)`                 |

### Example
The following table shows how the type conversion works for the type container:
| Name       | DBus    | Rust                 |
|------------|---------|----------------------|
| Array      | `ay`    | `Vec<u8>`            |
| Struct     | `(isi)` | `(i32, String, i32)` |
| Dict-Entry | `{ys}`  | `(u8, String)`       |
