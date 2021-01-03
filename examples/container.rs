use dbus_async::{Binder, DBus};
use dbus_async_derive::Handler;
use dbus_message_parser::{Error, MessageHeader};
use std::convert::TryInto;

#[derive(Handler)]
#[interface(
    "org.example.methods",
    method("MethodArray", method_array, "ai"),
    method("MethodArrayReturn", method_array_return, "", "ai"),
    method("MethodStruct", method_struct, "(isi)"),
    method("MethodStructReturn", method_struct_return, "", "(isi)"),
    method("MethodDict", method_dict, "a{yi}"),
    method("MethodDictReturn", method_dict_return, "", "a{yi}")
)]
struct MethodsObject {}

impl MethodsObject {
    async fn method_array(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        arg_0: Vec<i32>,
    ) -> Result<(), (Error, String)> {
        println!("The following arguments are received: {:?}", arg_0);
        // ...
        Ok(())
    }

    async fn method_array_return(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<Vec<i32>, (Error, String)> {
        // ...
        Ok(vec![1, 2, 3])
    }

    async fn method_struct(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        arg_0: (i32, String, i32),
    ) -> Result<(), (Error, String)> {
        println!(
            "The following arguments are received: ({}, {}, {})",
            arg_0.0, arg_0.1, arg_0.2
        );
        // ...
        Ok(())
    }

    async fn method_struct_return(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<(i32, String, i32), (Error, String)> {
        // ...
        Ok((10, "String".to_string(), 20))
    }

    async fn method_dict(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
        arg_0: Vec<(u8, i32)>,
    ) -> Result<(), (Error, String)> {
        println!("The following arguments are received: {:?}", arg_0);
        // ...
        Ok(())
    }

    async fn method_dict_return(
        &mut self,
        _dbus: &DBus,
        _msg_header: &MessageHeader,
    ) -> Result<Vec<(u8, i32)>, (Error, String)> {
        // ...
        Ok(vec![(1, 100)])
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
