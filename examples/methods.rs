use dbus_async::{Binder, DBus};
use dbus_async_derive::Handler;
use dbus_message_parser::{Error, MessageHeader, Value};
use std::convert::TryInto;

#[derive(Handler)]
#[interface(
    "org.example.methods",
    method("Method", method),
    method("MethodWithArgs", method_with_args, "su"),
    method("MethodWithReturnValue", method_with_return_value, "", "i"),
    method("MethodWithArgsReturnValue", method_with_args_return_value, "n", "iv")
)]
struct MethodsObject {}

impl MethodsObject {
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

    async fn method_with_args(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        arg_0: String,
        arg_1: u32,
    ) -> Result<(), (Error, String)> {
        println!("The following arguments are received: {}, {}", arg_0, arg_1);
        if arg_0.is_empty() {
            // If arg_0 is empty then send a error message
            Err((
                "org.example.Error.Name".try_into().unwrap(),
                "This is an error message".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    async fn method_with_return_value(
        &mut self,
        _dbus: &DBus,
        msg_header: &MessageHeader,
    ) -> Result<i32, (Error, String)> {
        println!(
            "The sender who send the message: {:?}",
            msg_header.get_sender()
        );
        // The return value of the method call
        Ok(100)
    }

    async fn method_with_args_return_value(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        arg_0: i16,
    ) -> Result<(i32, Box<Value>), (Error, String)> {
        if arg_0 == 0 {
            let i = Value::Int32(100);
            Ok((10, Box::new(i)))
        } else {
            let o = Value::ObjectPath("/object/path/example".try_into().unwrap());
            Ok((20, Box::new(o)))
        }
    }
}

#[tokio::main]
async fn main() {
    let (dbus, _connection_join_handle) = DBus::session(true)
        .await
        .expect("failed to get the DBus object");

    let method_object = MethodsObject {};
    let object_path = "/org/example/methods".try_into().unwrap();
    method_object
        .bind(dbus, object_path)
        .await
        .expect("Something went wrong");
}
